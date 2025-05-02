//! Builders that are 1:1 with types in the `types` module,
//! with optional fields for parsing compatibility.

use std::str::FromStr;

use anyhow::Context;

use crate::constants::*;
use crate::types::media::Audio;
use crate::types::media::AudioChannelInfo;
use crate::types::stream_info::IframeStreamInfo;
use crate::types::stream_info::Resolution;
use crate::types::stream_info::StreamInfo;
use crate::types::stream_info::StreamInfoCommon;

#[derive(Default, Debug)]
pub(crate) struct AudioBuilder {
    group_id: Option<String>,
    name: Option<String>,
    language: Option<String>,
    default: Option<bool>,
    auto_select: Option<bool>,
    channel_info: Option<AudioChannelInfo>,
    uri: Option<String>,
}

impl AudioBuilder {
    /// Consume self, producing Ok(`Audio`) if required fields are present.
    pub(crate) fn build(self) -> anyhow::Result<Audio> {
        let error_prefix = "missing HLS audio param ";
        Ok(Audio {
            group_id: self.group_id.with_context(|| format!("{error_prefix}{P_GROUP_ID}"))?,
            name: self.name.with_context(|| format!("{error_prefix}{P_NAME}"))?,
            language: self.language.with_context(|| format!("{error_prefix}{P_LANGUAGE}"))?,
            default: self.default.with_context(|| format!("{error_prefix}{P_DEFAULT}"))?,
            auto_select: self.auto_select.with_context(|| format!("{error_prefix}{P_AUTOSELECT}"))?,
            channel_info: self.channel_info.with_context(|| format!("{error_prefix}{P_CHANNELS}"))?,
            uri: self.uri.with_context(|| format!("{error_prefix}{P_URI}"))?,
        })
    }

    /// Incorporates the given parameter (name, value) into the builder,
    /// failing if the name doesn't match or necessary conversion of a parameter value fails.
    pub(crate) fn incorporate(mut self, param_tuple: (&str, &str)) -> Self {
        let (param_name, param_value) = param_tuple;
        match param_name {
            P_GROUP_ID => self.group_id = Some(param_value.to_owned()),
            P_NAME => self.name = Some(param_value.to_owned()),
            P_LANGUAGE => self.language = Some(param_value.to_owned()),
            P_DEFAULT => {
                self.default = Some(bool_from_param_str(param_value).expect(
                    format!("failed to parse {P_DEFAULT} param from YES/NO value").as_str(),
                ))
            }
            P_AUTOSELECT => {
                self.auto_select = Some(bool_from_param_str(param_value).expect(
                    format!("failed to parse {P_AUTOSELECT} param from YES/NO value").as_str(),
                ))
            }
            P_CHANNELS => {
                self.channel_info = Some(
                    AudioChannelInfo::from_str(param_value)
                        .expect(format!("failed to parse {P_CHANNELS} param value").as_str()),
                )
            }
            P_URI => self.uri = Some(param_value.to_owned()),
            _ => unreachable!("unhandled param {param_name} passed from parser"),
        }
        self
    }
}

#[derive(Debug, Default)]
pub(crate) struct StreamInfoCommonBuilder {
    bandwidth: Option<usize>,
    codecs: Option<Vec<String>>,
    resolution: Option<Resolution>,
    video_range: Option<String>,
    pub(crate) uri: Option<String>,
}

impl StreamInfoCommonBuilder {
    fn build(self) -> anyhow::Result<StreamInfoCommon> {
        let error_prefix = "missing HLS video param ";
        Ok(StreamInfoCommon {
            bandwidth: self.bandwidth.with_context(|| format!("{error_prefix}{P_BANDWIDTH}"))?,
            codecs: self.codecs.with_context(|| format!("{error_prefix}{P_CODECS}"))?,
            resolution: self.resolution.with_context(|| format!("{error_prefix}{P_RESOLUTION}"))?,
            video_range: self.video_range.with_context(|| format!("{error_prefix}{P_VIDEO_RANGE}"))?,
            uri: self.uri.with_context(|| format!("{error_prefix}{P_URI}"))?,
        })
    }

    /// Incorporates the given parameter, returning an Err if the name doesn't match, and failing if param value conversion fails.
    // NOTE: different from other `incorporate` calls, this call can fail as it's nested in other types. Errors are handled on the caller side.
    fn incorporate(&mut self, param_tuple: (&str, &str)) -> anyhow::Result<()> {
        let (param_name, param_value) = param_tuple;
        match param_name {
            P_BANDWIDTH => {
                self.bandwidth = Some(
                    usize::from_str(param_value)
                        .expect(format!("failed to parse {P_BANDWIDTH} param as int").as_str()),
                )
            }
            P_CODECS => self.codecs = Some(param_value.split(',').map(|x| x.to_owned()).collect()),
            P_RESOLUTION => {
                self.resolution = Some(
                    Resolution::from_str(param_value)
                        .expect(format!("failed to parse {P_RESOLUTION} param").as_str()),
                )
            }
            P_VIDEO_RANGE => self.video_range = Some(param_value.to_owned()),
            P_URI => self.uri = Some(param_value.to_owned()),
            _ => anyhow::bail!("param not covered by common stream info"),
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct StreamInfoBuilder {
    pub(crate) common: StreamInfoCommonBuilder,
    average_bandwidth: Option<usize>,
    frame_rate: Option<f32>,
    audio_codec: Option<String>,
    closed_captions: Option<String>,
}

impl StreamInfoBuilder {
    /// Consume self, producing Ok(`StreamInfo`) if required fields are present.
    pub(crate) fn build(self) -> anyhow::Result<StreamInfo> {
        let error_prefix = "missing HLS video param ";
        Ok(StreamInfo {
            common: self.common.build()?,
            average_bandwidth: self.average_bandwidth.with_context(|| format!("{error_prefix}{P_AVERAGE_BANDWIDTH}"))?,
            frame_rate: self.frame_rate.with_context(|| format!("{error_prefix}{P_FRAME_RATE}"))?,
            audio_codec: self.audio_codec.with_context(|| format!("{error_prefix}{P_AUDIO}"))?,
            closed_captions: self.closed_captions.with_context(|| format!("{error_prefix}{P_CLOSED_CAPTIONS}"))?,
        })
    }

    /// Incorporates the given parameter (name, value) into the builder,
    /// failing if the name doesn't match or necessary conversion of a parameter value fails.
    pub(crate) fn incorporate(mut self, param_tuple: (&str, &str)) -> Self {
        let (param_name, param_value) = param_tuple;
        if let Err(_) = self.common.incorporate(param_tuple) {
            match param_name {
                P_AVERAGE_BANDWIDTH => {
                    self.average_bandwidth = Some(usize::from_str(param_value).expect(
                        format!("failed to parse {P_AVERAGE_BANDWIDTH} param as int").as_str(),
                    ))
                }
                P_FRAME_RATE => {
                    self.frame_rate =
                        Some(f32::from_str(param_value).expect(
                            format!("failed to parse {P_FRAME_RATE} param as int").as_str(),
                        ))
                }
                P_AUDIO => self.audio_codec = Some(param_value.to_owned()),
                P_CLOSED_CAPTIONS => self.closed_captions = Some(param_value.to_owned()),
                _ => unreachable!("unhandled param {param_name} passed from parser"),
            }
        }
        self
    }
}

#[derive(Debug, Default)]
pub(crate) struct IframeStreamInfoBuilder {
    pub(crate) common: StreamInfoCommonBuilder,
}

impl IframeStreamInfoBuilder {
    /// Consume self, producing Ok`IframeStreamInfo`) if required fields are present.
    pub(crate) fn build(self) -> anyhow::Result<IframeStreamInfo> {
        Ok(IframeStreamInfo {
            common: self.common.build()?,
        })
    }

    /// Incorporates the given parameter (name, value) into the builder,
    /// failing if the name doesn't match or necessary conversion of a parameter value fails.
    pub(crate) fn incorporate(mut self, param_tuple: (&str, &str)) -> Self {
        if let Err(_) = self.common.incorporate(param_tuple) {
            unreachable!("unhandled param {} passed from parser", param_tuple.0);
        }
        self
    }
}

/// Matches an HLS boolean parameter value. Throws an error if not exactly YES or NO.
fn bool_from_param_str(s: &str) -> anyhow::Result<bool> {
    if s == "YES" {
        Ok(true)
    } else if s == "NO" {
        Ok(false)
    } else {
        anyhow::bail!("could not match {s} to str repr of boolean value (YES/NO)")
    }
}
