//! `nom`-compatible parser functions and accompanying datatypes for HLS tags and data.
//!
//! Parsers take a reference to the data stream,
//! moving the head of the stream forward to account for parsed data.
//! As a rule of thumb, parsers in this module strip extra whitespace
//! newlines to set up input for subsequent parsers.

use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{take_till, take_until};
use nom::character::complete::{digit1, newline, not_line_ending, space0};
use nom::combinator::{all_consuming, eof, map_res, opt};
use nom::multi::{fold_many1, many1};
use nom::{IResult, Parser};
use nom::{bytes::complete::tag, character::complete::multispace0};

use crate::HlsPlaylist;
use crate::builders::{AudioBuilder, IframeStreamInfoBuilder, StreamInfoBuilder};
use crate::constants::*;

type NomStrError<'a> = nom::error::Error<&'a str>;

/// Holds possible HLS playlist elements for flexibility in parser return types.
/// Outside of this module, use types from the `types` module directly instead.
// OPTIMIZATION: Box contained types to reduce the size of this enum?
#[derive(Debug)]
enum HlsElement {
    NoData,
    Audio(AudioBuilder),
    StreamInfo(StreamInfoBuilder),
    IframeStreamInfo(IframeStreamInfoBuilder),
    Version(usize),
}

impl HlsElement {
    /// Consumes self, moving it into the HLS playlist matching its variant.
    fn add_to_playlist(self, playlist: &mut HlsPlaylist) -> anyhow::Result<()> {
        match self {
            HlsElement::NoData => (),
            HlsElement::Audio(x) => playlist.audio_streams.inner.push(x.build()?),
            HlsElement::StreamInfo(x) => playlist.streams.inner.push(x.build()?),
            HlsElement::IframeStreamInfo(x) => playlist.iframe_streams.inner.push(x.build()?),
            HlsElement::Version(v) => playlist.version = v,
        }
        Ok(())
    }
}

// Parse the entire input stream, incorporating all components into the returned `HlsPlaylist`.
// Returns an error if any line or component fails to parse.
pub(crate) fn parse_hls_playlist<'a>(data: &'a str) -> anyhow::Result<HlsPlaylist> {
    let mut res = HlsPlaylist::default();

    // TODO: split `data` into lines for easier error identification

    // Try using all available parsing functions below, collecting the `HlsElement`s returned by successful parsers.
    // By design of the parsing functions, at most one will succeed.
    let components = match all_consuming(many1(alt((
        // Small optimization: roughly ordered by expected frequency (descending)
        hls_stream_info,
        hls_iframe_stream_info,
        hls_audio,
        hls_version,
        hls_independent_segments,
        hls_header,
        // NOTE: must be last, as HLS extensions (#EXT-X-*) are technically comments
        hls_comment,
    ))))
    .parse(data)
    {
        Ok((_, components)) => components,
        Err(e) => anyhow::bail!("{e}"),
    };
    for elt in components {
        elt.add_to_playlist(&mut res)?;
    }

    Ok(res)
}

/// Return a function that can be used to parse the `#EXT-X-` prefix of a line in the HLS playlist.
/// Does perform any parsing - solely meant for composition with other parsers.
// NOTE: This impl is constrained to &str but could be made generic.
fn extension_prefix<'a>() -> impl Parser<&'a str, Error = NomStrError<'a>> {
    tag("#EXT-X-")
}

/// Parse an HLS comment. Anything that starts with `#`.
/// **Try other `hls_*` functions first**, as this matches on `#EXT-X-*` lines.
fn hls_comment<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    map_res((tag("#"), not_line_ending, newline), |_| {
        Ok::<_, NomStrError<'a>>(HlsElement::NoData)
    })
    .parse(data)
}

/// Parse a `#EXTM3U` header.
/// Returns `HlsElement::NoData` on success. Modifies the input to move past the tag.
fn hls_header<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    // Toss parser results, converting to `HlsElement::NoData` instead.
    map_res((tag("#EXTM3U"), multispace0), |_| {
        Ok::<_, NomStrError>(HlsElement::NoData)
    })
    .parse(data)
}

/// Parse an HLS independent segments param from the given string.
/// Returns `HlsElement::NoData` on success. Modifies the input to "move past" the tag.
// TODO: return and store this parameter?
fn hls_independent_segments<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    // Toss parser results, converting to `HlsElement::NoData` instead.
    map_res(
        (
            // Parse #EXT-X-INDEPENDENT-SEGMENTS
            extension_prefix(),
            tag("INDEPENDENT-SEGMENTS"),
            // Clear subsequent whitespace/newlines/eof
            multispace0,
        ),
        |_| Ok::<_, NomStrError>(HlsElement::NoData),
    )
    .parse(data)
}

/// Parse an HLS `#EXT-X-VERSION` param, returning the value as a `str` to be parsed to int later.
fn hls_version<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    // Toss parser results, converting to `HlsElement::NoData` instead.
    map_res(
        (
            // Parse #EXT-X-VERSION:<num>
            extension_prefix(),
            tag("VERSION:"),
            map_res(digit1, usize::from_str),
            // Clear subsequent whitespace/newlines/eof
            multispace0,
        ),
        |(_, _, v, _)| Ok::<_, NomStrError>(HlsElement::Version(v)),
    )
    .parse(data)
}

/// Parse HLS audio media (starts with #EXT-X-MEDIA, contains TYPE=AUDIO param).
/// Return a `HlsElement::Audio` that represents the parsed data.
// TODO: support subtitle variants
fn hls_audio<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    // Parse the beginning of an audio stream tag
    let (rest, _) = (
        extension_prefix(),
        tag("MEDIA:"),
        space0,
        tag("TYPE=AUDIO"),
        space0,
        tag(","),
    )
        .parse(data)?;

    // Try any of the following parameter parsers, folding the result into a builer struct for the desired type.
    // Some params are enclosed by quotes and/or need conversion from the returned str value into another type.
    let (rest, builder) = fold_many1(
        alt((
            // TODO: repr GROUP-ID with enum given known-good set
            comma_terminated_param(P_GROUP_ID, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_NAME, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_LANGUAGE, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_DEFAULT, ParamEnclose::None),
            comma_terminated_param(P_AUTOSELECT, ParamEnclose::None),
            comma_terminated_param(P_CHANNELS, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_URI, ParamEnclose::DoubleQuotes),
        )),
        AudioBuilder::default,
        |builder, param_tuple| builder.incorporate(param_tuple),
    )
    .parse(rest)?;

    // Strip newline expected before next tag, or recognize end of input
    let (rest, _) = alt((multispace0, eof)).parse(rest)?;

    Ok((rest, HlsElement::Audio(builder)))
}

/// Parse an HLS stream (starts with #EXT-X-STREAM-INF).
/// Return a `HlsElement::StreamInfo` that represents the parsed data.
fn hls_stream_info<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    // Parse the beginning of a video stream tag
    let (rest, _) = (extension_prefix(), tag("STREAM-INF:"), space0).parse(data)?;

    // Try any of the following parameter parsers, folding the result into a builer struct for the desired type.
    // Some params are enclosed by quotes and/or need conversion from the returned str value into another type.
    let (rest, mut builder) = fold_many1(
        alt((
            comma_terminated_param(P_BANDWIDTH, ParamEnclose::None),
            comma_terminated_param(P_AVERAGE_BANDWIDTH, ParamEnclose::None),
            comma_terminated_param(P_CODECS, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_RESOLUTION, ParamEnclose::None),
            comma_terminated_param(P_FRAME_RATE, ParamEnclose::None),
            comma_terminated_param(P_VIDEO_RANGE, ParamEnclose::None),
            comma_terminated_param(P_AUDIO, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_CLOSED_CAPTIONS, ParamEnclose::None),
        )),
        StreamInfoBuilder::default,
        |builder, param_tuple| builder.incorporate(param_tuple),
    )
    .parse(rest)?;

    // Parse resource URI expected on the next line, then newlines or end of input
    let (rest, uri) = map_res(
        (space0, newline, not_line_ending, alt((multispace0, eof))),
        |tuple| Ok::<_, NomStrError<'a>>(tuple.2),
    )
    .parse(rest)?;

    builder.common.uri = Some(uri.to_owned());

    Ok((rest, HlsElement::StreamInfo(builder)))
}

/// Parse an HLS iframe stream (starts with #EXT-X-I-FRAME-STREAM-INF).
/// Return a `HlsElement::IframeStreamInfo` that represents the parsed data.
fn hls_iframe_stream_info<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    // Parse the beginning of an ifram video stream tag
    let (rest, _) = (extension_prefix(), tag("I-FRAME-STREAM-INF:"), space0).parse(data)?;

    // Try any of the following parameter parsers, folding the result into a builer struct for the desired type.
    // Some params are enclosed by quotes and/or need conversion from the returned str value into another type.
    let (rest, builder) = fold_many1(
        alt((
            comma_terminated_param(P_BANDWIDTH, ParamEnclose::None),
            comma_terminated_param(P_CODECS, ParamEnclose::DoubleQuotes),
            comma_terminated_param(P_RESOLUTION, ParamEnclose::None),
            comma_terminated_param(P_VIDEO_RANGE, ParamEnclose::None),
            comma_terminated_param(P_URI, ParamEnclose::DoubleQuotes),
        )),
        IframeStreamInfoBuilder::default,
        |builder, param_tuple| builder.incorporate(param_tuple),
    )
    .parse(rest)?;

    // Strip newline expected before next tag, or recognize end of input
    let (rest, _) = alt((multispace0, eof)).parse(rest)?;

    Ok((rest, HlsElement::IframeStreamInfo(builder)))
}

// ---------- Functions and utilities for parsing HLS parameters ----------

/// Represents the chars surrounding an HLS param, for flexibility parsing
/// params of the form 'PARAM_NAME=<value>' that may be wrapped in quotes.
#[derive(Debug)]
enum ParamEnclose {
    // NOTE: other param value wrappers may be added here
    None,
    DoubleQuotes,
}

/// Given a param_name, returns a parser function that matches on '<param_name>=<value>,'
/// and returns a tuple containing the parameter name and value. Tolerates spaces.
/// Uses `enclosed_by` to parse delimiters surrounding the parameter value.
fn comma_terminated_param<'a>(
    param_name: &'a str,
    enclosed_by: ParamEnclose,
) -> impl Parser<&'a str, Output = (&'a str, &'a str), Error = NomStrError<'a>> {
    // Map result of the combined parser to just the parameter value, returned from a param_value_* function
    map_res(
        (
            tag(param_name),
            space0,
            tag("="),
            space0,
            match enclosed_by {
                ParamEnclose::None => param_value_no_enclosure,
                ParamEnclose::DoubleQuotes => param_value_double_quoted,
            },
            space0,
            // Take comma if present - friendly towards last param in a given line
            opt(tag(",")),
        ),
        move |tuple| Ok::<_, NomStrError<'a>>((param_name, tuple.4)),
    )
}

/// Parse and return a parameter value with no enclosing quotes. Terminated at whitespace or comma.
fn param_value_no_enclosure<'a>(data: &'a str) -> IResult<&'a str, &'a str, NomStrError<'a>> {
    // Try whitespace- and comma-terminated parsers, using what works
    alt((take_till(|c: char| c == ',' || c.is_whitespace()),)).parse(data)
}

/// Parse and return a parameter value enclosed in double quotes.
fn param_value_double_quoted<'a>(data: &'a str) -> IResult<&'a str, &'a str, NomStrError<'a>> {
    // Map result to the parameter value returned by the middle parser.
    map_res(
        (
            tag("\""),
            take_until::<&'a str, &'a str, _>("\""),
            tag("\""),
        ),
        |tuple| Ok::<_, NomStrError<'a>>(tuple.1),
    )
    .parse(data)
}
