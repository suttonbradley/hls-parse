//! Types to represent HLS data, including descriptive tags with associated data.

// Types of media under tag #EXT-X-MEDIA
pub mod media {
    #[derive(Debug, PartialEq)]
    pub struct Audio {
        pub(crate) group_id: String,
        pub(crate) name: String,
        pub(crate) language: String,
        pub(crate) default: bool,
        pub(crate) auto_select: bool,
        pub(crate) channels: usize,
        pub(crate) uri: String,
    }

    // TODO: implement subtitles
}

// Types for parsing #EXT-X-STREAM-INF
pub mod stream_info {
    use std::str::FromStr;

    use anyhow::Context;

    #[derive(Debug, Default, PartialEq)]
    pub struct StreamInfo {
        pub(crate) bandwidth: usize,
        pub(crate) average_bandwidth: usize,
        // TODO: represent as struct
        pub(crate) codecs: Vec<String>,
        pub(crate) resolution: Resolution,
        pub(crate) frame_rate: f32,
        pub(crate) video_range: String,
        // TODO: use enum of common audio formats?
        pub(crate) audio_codec: String,
        pub(crate) closed_captions: bool,
        /// URI of the media playlist this struct holds metadata for
        pub(crate) uri: String,
    }

    #[derive(Debug, Default, PartialEq)]
    pub(crate) struct Resolution {
        // TODO: could represent as enum of common resolutions
        // TODO: could store as u16, as max reasonable value is ~8k
        pub(crate) width: usize,
        pub(crate) height: usize,
    }

    impl FromStr for Resolution {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Expects format WxH. Split on 'x' and parse each surrounding string to int.
            let split = s.split('x').collect::<Vec<_>>();
            Ok(Self {
                width: split[0]
                    .parse::<usize>()
                    .with_context(|| format!("failed to parse pixed width: {}", split[0]))?
                    .to_owned(),
                height: split[1]
                    .parse::<usize>()
                    .with_context(|| format!("failed to parse pixed height: {}", split[1]))?
                    .to_owned(),
            })
        }
    }
}
