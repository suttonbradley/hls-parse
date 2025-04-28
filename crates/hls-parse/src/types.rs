//! Types to represent HLS data, including descriptive tags with associated data.

// Types of media under tag #EXT-X-MEDIA
pub(crate) mod media {
    #[derive(Default, Debug, PartialEq)]
    pub(crate) struct Audio {
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

mod stream_info {
    struct StreamInfo {
        bandwidth: usize,
        average_bandwidth: usize,
        // TODO: represent as struct
        codecs: Vec<String>,
        resolution: Resolution,
        frame_rate: f32,
        video_range: String,
        // TODO: use enum of common audio formats?
        audio_codec: String,
        closed_captions: String,
    }

    struct Resolution {
        // TODO: could represent as enum of common resolutions
        // TODO: could store as u16, as max reasonable value is ~8k
        width: usize,
        height: usize,
    }
}
