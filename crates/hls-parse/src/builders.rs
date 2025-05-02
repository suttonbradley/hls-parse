//! Builders that are 1:1 with types in the `types` module,
//! with optional fields for parsing compatibility.

use std::str::FromStr;

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
    /// URI of the audio stream the other metadata fields describe
    // TODO: represent as http::uri::Uri ?
    uri: Option<String>,
}

impl AudioBuilder {
    /// Consume self, producing Ok(`Audio`) if required fields are present.
    pub(crate) fn build(self) -> anyhow::Result<Audio> {
        // TODO: use .context, or just leave as Option
        Ok(Audio {
            group_id: self.group_id.unwrap(),
            name: self.name.unwrap(),
            language: self.language.unwrap(),
            default: self.default.unwrap(),
            auto_select: self.auto_select.unwrap(),
            channel_info: self.channel_info.unwrap(),
            uri: self.uri.unwrap(),
        })
    }

    /// Incorporates the given parameter (name, value) into the builder,
    /// failing if the name doesn't match or necessary conversion of a parameter value fails.
    pub(crate) fn incorporate(mut self, param_tuple: (&str, &str)) -> Self {
        let (param_name, param_value) = param_tuple;
        match param_name {
            // TODO: consts for these strings
            "GROUP-ID" => self.group_id = Some(param_value.to_owned()),
            "NAME" => self.name = Some(param_value.to_owned()),
            "LANGUAGE" => self.language = Some(param_value.to_owned()),
            "DEFAULT" => {
                self.default = Some(
                    bool_from_param_str(param_value)
                        .expect("failed to parse DEFAULT param from YES/NO value"),
                )
            }
            "AUTOSELECT" => {
                self.auto_select = Some(
                    bool_from_param_str(param_value)
                        .expect("failed to parse AUTO-SELECT param from YES/NO value"),
                )
            }
            "CHANNELS" => {
                self.channel_info = Some(
                    AudioChannelInfo::from_str(param_value)
                        .expect("failed to parse CHANNEL-INFO param value"),
                )
            }
            "URI" => self.uri = Some(param_value.to_owned()),
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
        // TODO: use .context, or just leave as Option
        Ok(StreamInfoCommon {
            bandwidth: self.bandwidth.unwrap(),
            codecs: self.codecs.unwrap(),
            resolution: self.resolution.unwrap(),
            video_range: self.video_range.unwrap(),
            uri: self.uri.unwrap(),
        })
    }

    /// Incorporates the given parameter, returning an Err if the name doesn't match, and failing if param value conversion fails.
    // NOTE: different from other `incorporate` calls, this call can fail as it's nested in other types. Errors are handled on the caller side.
    fn incorporate(&mut self, param_tuple: (&str, &str)) -> anyhow::Result<()> {
        let (param_name, param_value) = param_tuple;
        match param_name {
            "BANDWIDTH" => {
                self.bandwidth = Some(
                    usize::from_str(param_value).expect("failed to parse BANDWIDTH param as int"),
                )
            }
            "CODECS" => self.codecs = Some(param_value.split(',').map(|x| x.to_owned()).collect()),
            "RESOLUTION" => {
                self.resolution = Some(
                    Resolution::from_str(param_value).expect("failed to parse RESOLUTION param"),
                )
            }
            "VIDEO-RANGE" => self.video_range = Some(param_value.to_owned()),
            "URI" => self.uri = Some(param_value.to_owned()),
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
        // TODO: use .context, or just leave as Option
        Ok(StreamInfo {
            common: self.common.build()?,
            average_bandwidth: self.average_bandwidth.unwrap(),
            frame_rate: self.frame_rate.unwrap(),
            audio_codec: self.audio_codec.unwrap(),
            closed_captions: self.closed_captions.unwrap(),
        })
    }

    /// Incorporates the given parameter (name, value) into the builder,
    /// failing if the name doesn't match or necessary conversion of a parameter value fails.
    pub(crate) fn incorporate(mut self, param_tuple: (&str, &str)) -> Self {
        let (param_name, param_value) = param_tuple;
        if let Err(_) = self.common.incorporate(param_tuple) {
            match param_name {
                "AVERAGE-BANDWIDTH" => {
                    self.average_bandwidth = Some(
                        usize::from_str(param_value)
                            .expect("failed to parse AVERAGE-BANDWIDTH param as int"),
                    )
                }
                "FRAME-RATE" => {
                    self.frame_rate = Some(
                        f32::from_str(param_value)
                            .expect("failed to parse FRAME-RATE param as int"),
                    )
                }
                "AUDIO" => self.audio_codec = Some(param_value.to_owned()),
                "CLOSED-CAPTIONS" => self.closed_captions = Some(param_value.to_owned()),
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
        // TODO: use .context, or just leave as Option
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
