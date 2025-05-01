//! Types to represent HLS data, including descriptive tags with associated data.

// Types of media under tag #EXT-X-MEDIA
pub mod media {
    use std::{fmt::Display, str::FromStr};

    use anyhow::Context;

    #[derive(Debug, Default)]
    pub struct AudioStreams {
        pub inner: Vec<Audio>,
    }

    impl Display for AudioStreams {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "Audio Streams")?;
            writeln!(f, "-------------")?;
            writeln!(
                f,
                "| {:^10} | {:^10} | {:^10} | {:^7} | {:^10} | {:^8} | {:^35} |",
                "GROUP-ID", "NAME", "LANGUAGE", "DEFAULT", "AUTOSELECT", "CHANNELS", "URI"
            )?;
            for i in self.inner.iter() {
                writeln!(f, "{i}")?;
            }
            Ok(())
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Audio {
        pub group_id: String,
        pub name: String,
        pub language: String,
        pub default: bool,
        pub auto_select: bool,
        pub channel_info: AudioChannelInfo,
        /// URI of the audio stream the other metadata fields describe
        // TODO: represent as http::uri::Uri ?
        pub uri: String,
    }

    impl FromStr for AudioChannelInfo {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Detect optional "/JOC" on channels string
            let split = s.split('/').collect::<Vec<_>>();
            Ok(Self {
                channels: split[0]
                    .parse::<usize>()
                    .with_context(|| format!("failed to parse channel count: {}", split[0]))?,
                joc: split.len() > 1 && split[1] == "JOC",
            })
        }
    }

    impl Display for Audio {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "| {:^10} | {:^10} | {:^10} | {:^7} | {:^10} | {} | {:^35} |",
                self.group_id,
                self.name,
                self.language,
                self.default,
                self.auto_select,
                self.channel_info,
                self.uri
            )
        }
    }

    #[derive(Debug, Eq, PartialEq, PartialOrd)]
    pub struct AudioChannelInfo {
        pub channels: usize,
        pub joc: bool,
    }

    impl Display for AudioChannelInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{:^8}",
                format!("{}{}", self.channels, if self.joc { "/JOC" } else { "" })
            )
        }
    }

    impl Ord for AudioChannelInfo {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // Sort by channels first, breaking tie on joc
            if self.channels < other.channels {
                std::cmp::Ordering::Less
            } else if self.channels > other.channels {
                std::cmp::Ordering::Greater
            } else {
                self.joc.cmp(&other.joc)
            }
        }
    }

    // TODO: implement subtitles
}

// Types for parsing #EXT-X-STREAM-INF
pub mod stream_info {
    use std::{fmt::Display, str::FromStr};

    use anyhow::Context;

    /// Data related to all stream types (regular and iframe streams).
    #[derive(Debug, Default, PartialEq)]
    pub struct StreamInfoCommon {
        pub bandwidth: usize,
        pub codecs: Vec<String>,
        pub resolution: Resolution,
        pub video_range: String,
        /// URI of the media playlist that other metadata fields describe
        // TODO: represent as http::uri::Uri ?
        pub uri: String,
    }

    #[derive(Debug, Default)]
    pub struct Streams {
        pub inner: Vec<StreamInfo>,
    }

    impl Display for Streams {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "Video Streams")?;
            writeln!(f, "-------------")?;
            writeln!(
                f,
                "| {:^10} | {:^17} | {:^30} | {:^11} | {:^10} | {:^11} | {:^10} | {:^15} | {:^30} |",
                "BANDWIDTH",
                "AVERAGE-BANDWIDTH",
                "CODECS",
                "RESOLUTION",
                "FRAME-RATE",
                "VIDEO-RANGE",
                "AUDIO",
                "CLOSED-CAPTIONS",
                "URI"
            )?;
            for i in self.inner.iter() {
                writeln!(f, "{i}")?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct StreamInfo {
        pub common: StreamInfoCommon,
        pub average_bandwidth: usize,
        pub frame_rate: f32,
        // TODO: use enum of common audio formats?
        pub audio_codec: String,
        pub closed_captions: bool,
    }

    impl Display for StreamInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "| {:^10} | {:^17} | {:^30} | {} | {:^10} | {:^11} | {:^10} | {:^15} | {:^30} |",
                self.common.bandwidth,
                self.average_bandwidth,
                self.common.codecs.join(", "),
                self.common.resolution,
                self.frame_rate,
                self.common.video_range,
                self.audio_codec,
                self.closed_captions,
                self.common.uri
            )
        }
    }

    #[derive(Debug, Default)]
    pub struct IframeStreams {
        pub inner: Vec<IframeStreamInfo>,
    }

    impl Display for IframeStreams {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "IFrame Streams")?;
            writeln!(f, "--------------")?;
            writeln!(
                f,
                "| {:^10} | {:^30} | {:^11} | {:^11} | {:^35} |",
                "BANDWIDTH", "CODECS", "RESOLUTION", "VIDEO-RANGE", "URI"
            )?;
            for i in self.inner.iter() {
                writeln!(f, "{i}")?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct IframeStreamInfo {
        pub common: StreamInfoCommon,
    }

    impl Display for IframeStreamInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "| {:^10} | {:^30} | {} | {:^11} | {:^35} |",
                self.common.bandwidth,
                self.common.codecs.join(", "),
                self.common.resolution,
                self.common.video_range,
                self.common.uri
            )
        }
    }

    #[derive(Debug, Default, Eq, PartialEq, PartialOrd)]
    pub struct Resolution {
        // TODO: could represent as enum of common resolutions
        // TODO: could store as u16, as max reasonable value is ~8k
        pub width: usize,
        pub height: usize,
    }

    impl FromStr for Resolution {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Expects format WxH. Split on 'x' and parse each surrounding string to int.
            let split = s.split('x').collect::<Vec<_>>();
            Ok(Self {
                width: split[0]
                    .parse::<usize>()
                    .with_context(|| format!("failed to parse pixed width: {}", split[0]))?,
                height: split[1]
                    .parse::<usize>()
                    .with_context(|| format!("failed to parse pixed height: {}", split[1]))?,
            })
        }
    }

    impl Display for Resolution {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:>5}x{:<5}", self.width, self.height)
        }
    }

    impl Ord for Resolution {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            let width_cmp = self.width.cmp(&other.width);
            match width_cmp {
                std::cmp::Ordering::Equal => width_cmp,
                _ => self.height.cmp(&other.height),
            }
        }
    }
}
