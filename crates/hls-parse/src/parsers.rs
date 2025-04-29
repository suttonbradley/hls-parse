//! `nom`-compatible parser functions and accompanying datatypes for HLS tags and data.
//!
//! Parsers take a reference to the data stream,
//! moving the head of the stream forward to account for parsed data.
//! As a rule of thumb, parsers in this module strip extra whitespace
//! newlines to set up input for subsequent parsers.

use nom::branch::alt;
use nom::bytes::complete::{take_till, take_until};
use nom::character::complete::space0;
use nom::combinator::{all_consuming, map_res, opt};
use nom::multi::many1;
use nom::{IResult, Parser};
use nom::{bytes::complete::tag, character::complete::multispace0};

use crate::HlsPlaylist;
use crate::types::media::Audio;

type NomStrError<'a> = nom::error::Error<&'a str>;

/// Holds possible HLS playlist elements for flexibility in parser return types.
/// Outside of this module, use types from the `types` module directly instead.
// OPTIMIZATION: Box contained types to reduce the size of this enum.
#[derive(Debug)]
enum HlsElement {
    NoData,
    Audio(Audio),
}

impl HlsElement {
    /// Consumes self to contribute to the given HLS playlist.
    fn add_to_playlist(self, playlist: &mut HlsPlaylist) {
        match self {
            HlsElement::NoData => (),
            HlsElement::Audio(a) => playlist.audio_tracks.push(a),
        }
    }
}

// Parse the entire input stream, incorporating all components into the returned `HlsPlaylist`.
// Returns an error if any line or component fails to parse.
pub(crate) fn parse_hls_playlist<'a>(data: &'a str) -> anyhow::Result<HlsPlaylist> {
    let mut res = HlsPlaylist::default();

    // Try using all available parsing functions below, collecting the `HlsElement`s returned by successful parsers.
    // By design of the parsing functions, at most one will succeed.
    let components = match all_consuming(many1(alt((
        hls_header,
        hls_independent_segments,
        hls_audio,
    ))))
    .parse(data)
    {
        Ok((_, components)) => components,
        // TODO: ensure this works
        Err(e) => anyhow::bail!("{e:?}"),
    };
    for elt in components {
        elt.add_to_playlist(&mut res);
    }

    Ok(res)
}

/// Return a function that can be used to parse the `#EXT-X-` prefix of a line in the HLS playlist.
/// Does perform any parsing - solely meant for composition with other parsers.
// NOTE: This impl is constrained to &str but could be made generic.
fn extension_prefix<'a>() -> impl Parser<&'a str, Error = NomStrError<'a>> {
    tag("#EXT-X-")
}

/// Parse a `#EXTM3U` header.
/// Returns `HlsElement::NoData` on success. Modifies the input to "move past" the tag.
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
            // Clear subsequent whitespace and newlines
            multispace0,
        ),
        |_| Ok::<_, NomStrError>(HlsElement::NoData),
    )
    .parse(data)
}

/// Parse a  line starting with #EXT-X-MEDIA with a TYPE=AUDIO tag.
/// Return `Audio` that represents the parsed data.
// TODO: support subtitle variants
// TODO: support varied tag ordering using `alt` parser
fn hls_audio<'a>(data: &'a str) -> IResult<&'a str, HlsElement> {
    map_res(
        (
            // Parse "#EXT-X-MEDIA:"
            extension_prefix(),
            tag("MEDIA:"),
            (space0, tag("TYPE=AUDIO"), space0, tag(",")),
            // Parse HLS parameters (TYPE=AUDIO, GROUP-ID=foo, etc.). Some params will be enclosed by quotes
            // and/or need conversion from the returned str value into another type.
            // TODO: some of the following, like GROUP-ID, could be converted to an
            //       enum given a known-good set of values, like audio codecs.
            comma_terminated_param("GROUP-ID", ParamEnclose::DoubleQuotes),
            comma_terminated_param("NAME", ParamEnclose::DoubleQuotes),
            comma_terminated_param("LANGUAGE", ParamEnclose::DoubleQuotes),
            map_res(
                comma_terminated_param("DEFAULT", ParamEnclose::None),
                bool_from_param_str,
            ),
            map_res(
                comma_terminated_param("AUTOSELECT", ParamEnclose::None),
                bool_from_param_str,
            ),
            map_res(
                comma_terminated_param("CHANNELS", ParamEnclose::DoubleQuotes),
                |s| s.parse::<usize>(),
            ),
            comma_terminated_param("URI", ParamEnclose::DoubleQuotes),
            // Clear subsequent whitespace and newlines
            multispace0,
        ),
        |tuple| {
            Ok::<_, NomStrError<'a>>(HlsElement::Audio(Audio {
                group_id: tuple.3.to_owned(),
                name: tuple.4.to_owned(),
                language: tuple.5.to_owned(),
                default: tuple.6,
                auto_select: tuple.7,
                channels: tuple.8,
                uri: tuple.9.to_owned(),
            }))
        },
    )
    .parse(data)
}

/// Represents the chars surrounding an HLS param, for flexibility parsing
/// params of the form 'PARAM_NAME=<value>' that may be wrapped in quotes.
#[derive(Debug)]
enum ParamEnclose {
    // NOTE: other param value wrappers may be added here
    None,
    DoubleQuotes,
}

/// Given a param_name, returns a parser function that matches on
/// '<param_name>=<value>,' and returns the parameter value. Tolerates spaces.
/// Uses `enclosed_by` to parse delimiters surrounding the parameter value.
fn comma_terminated_param<'a>(
    param_name: &'a str,
    enclosed_by: ParamEnclose,
) -> impl Parser<&'a str, Output = &'a str, Error = NomStrError<'a>> {
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
        |tuple| Ok::<_, NomStrError<'a>>(tuple.4),
    )
}

/// Parse and return a parameter value with no enclosing quotes. Terminated at whitespace or comma.
fn param_value_no_enclosure<'a>(data: &'a str) -> IResult<&'a str, &'a str, NomStrError<'a>> {
    alt((
        take_until::<&'a str, &'a str, _>(","),
        take_till(|c: char| c.is_whitespace()),
    ))
    .parse(data)
}

/// Parse and return a parameter value enclosed in double quotes.
fn param_value_double_quoted<'a>(data: &'a str) -> IResult<&'a str, &'a str, NomStrError<'a>> {
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

/// Matches an HLS boolean parameter. Throws an error if not exactly YES or NO.
fn bool_from_param_str(s: &str) -> anyhow::Result<bool> {
    if s == "YES" {
        Ok(true)
    } else if s == "NO" {
        Ok(false)
    } else {
        anyhow::bail!("could not match {s} to str repr of boolean value (YES/NO)")
    }
}
