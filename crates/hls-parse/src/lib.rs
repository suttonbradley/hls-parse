//! Parser for HLS playlists of M3U8 format.
//!
//! Provides data types that reflect HLS data (streams, media, etc.),
//! and functions to parse raw data into those types.

mod builders;
mod constants;
mod parsers;
pub mod types;

use std::{fmt::Display, str::FromStr};

/// Represents a parsed HLS playlist, supporting various `#EXT-X-*` extensions.
#[derive(Default, Debug)]
pub struct HlsPlaylist {
    // FIXME: These fields contain `Vec`s wrapped in other types, in order to impl std::fmt::Display.
    //        Unwrap these, for ergonomics (avoid `.inner`), and implement display of these types another way.
    pub audio_streams: types::media::AudioStreams,
    pub streams: types::stream_info::Streams,
    pub iframe_streams: types::stream_info::IframeStreams,
    /// Playlist protocol version
    pub version: usize,
}

impl Display for HlsPlaylist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}\n{}",
            self.audio_streams, self.streams, self.iframe_streams
        )
    }
}

impl FromStr for HlsPlaylist {
    // Make the return type of from_str equivalent to
    // anyhow::Result to avoid conversion.
    type Err = anyhow::Error;

    fn from_str(data: &str) -> std::result::Result<Self, Self::Err> {
        parsers::parse_hls_playlist(data)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::types::media::{Audio, AudioChannelInfo};
    use crate::types::stream_info::{IframeStreamInfo, Resolution, StreamInfo, StreamInfoCommon};

    use super::*;

    /// Just parse the sample input from a file, without checking parsed values for correctness.
    #[test]
    fn test_parse_sample_input() {
        // Get contents of sample input file
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR environment variable is not set");
        let file_path =
            Path::new(manifest_dir.as_str()).join(Path::new("test-fixtures/sample-input.txt"));
        let data = std::fs::read_to_string(file_path).expect("failed to read sample input file");

        let playlist = HlsPlaylist::from_str(data.as_str()).unwrap();
        assert_eq!(playlist.audio_streams.inner.len(), 4);
        assert_eq!(playlist.streams.inner.len(), 36);
        assert_eq!(playlist.iframe_streams.inner.len(), 2);
    }

    /// Parse basic elements that don't return structured data.
    #[test]
    fn test_parse_no_data() {
        let data = "#EXTM3U
#EXT-X-INDEPENDENT-SEGMENTS
# other comment
";
        let _ = HlsPlaylist::from_str(data).unwrap();
    }

    /// Parse basic elements that don't return structured data.
    #[test]
    fn test_parse_version() {
        let data = "#EXT-X-VERSION:5
";
        let playlist = HlsPlaylist::from_str(data).unwrap();
        assert_eq!(playlist.version, 5);
    }

    /// Parse audio media data only.
    #[test]
    fn test_parse_audio() {
        let data = "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aac-128k\",NAME=\"English\",LANGUAGE=\"en\",DEFAULT=YES,AUTOSELECT=YES,CHANNELS=\"2\",URI=\"audio/unenc/aac_128k/vod.m3u8\"

#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aac-64k\",NAME=\"English\",LANGUAGE=\"en\",DEFAULT=YES,AUTOSELECT=YES,CHANNELS=\"2\",URI=\"audio/unenc/aac_64k/vod.m3u8\"

#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"atmos\",NAME=\"English\",LANGUAGE=\"en\",DEFAULT=YES,AUTOSELECT=YES,CHANNELS=\"16/JOC\",URI=\"audio/unenc/atmos_1024k/vod.m3u8\"
";
        let playlist = HlsPlaylist::from_str(data).unwrap();
        println!("{playlist:?}");
        assert_eq!(
            playlist.audio_streams.inner[0],
            Audio {
                group_id: "aac-128k".to_owned(),
                name: "English".to_owned(),
                language: "en".to_owned(),
                default: true,
                auto_select: true,
                channel_info: AudioChannelInfo {
                    channels: 2,
                    joc: false,
                },
                uri: "audio/unenc/aac_128k/vod.m3u8".to_owned(),
            }
        );
        assert!(playlist.audio_streams.inner[2].channel_info.joc);
    }

    /// Parse stream data only.
    #[test]
    fn test_parse_stream() {
        let data = "#EXT-X-STREAM-INF:BANDWIDTH=2483789,AVERAGE-BANDWIDTH=1762745,CODECS=\"mp4a.40.2,hvc1.2.4.L90.90\",RESOLUTION=960x540,FRAME-RATE=23.97,VIDEO-RANGE=PQ,AUDIO=\"aac-128k\",CLOSED-CAPTIONS=NONE
hdr10/unenc/1650k/vod.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=15811232,AVERAGE-BANDWIDTH=10058085,CODECS=\"mp4a.40.2,hvc1.2.4.L150.90\",RESOLUTION=2560x1440,FRAME-RATE=23.97,VIDEO-RANGE=PQ,AUDIO=\"aac-128k\",CLOSED-CAPTIONS=NONE
hdr10/unenc/10000k/vod.m3u8
";

        let playlist = HlsPlaylist::from_str(data).unwrap();
        println!("{playlist:?}");
        assert_eq!(
            playlist.streams.inner[0],
            StreamInfo {
                common: StreamInfoCommon {
                    bandwidth: 2483789,
                    codecs: vec!["mp4a.40.2".to_owned(), "hvc1.2.4.L90.90".to_owned()],
                    resolution: Resolution {
                        width: 960,
                        height: 540,
                    },
                    video_range: "PQ".to_owned(),
                    uri: "hdr10/unenc/1650k/vod.m3u8".to_owned(),
                },
                average_bandwidth: 1762745,
                frame_rate: 23.97,
                audio_codec: "aac-128k".to_owned(),
                closed_captions: "NONE".to_owned(),
            }
        );
    }

    /// Parse iframe stream data only.
    #[test]
    fn test_parse_iframe() {
        let data = "#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=222552,CODECS=\"hvc1.2.4.L93.90\",RESOLUTION=1280x720,VIDEO-RANGE=PQ,URI=\"hdr10/unenc/3300k/vod-iframe.m3u8\"

#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=77758,CODECS=\"hvc1.2.4.L63.90\",RESOLUTION=640x360,VIDEO-RANGE=PQ,URI=\"hdr10/unenc/900k/vod-iframe.m3u8\"
";

        let playlist = HlsPlaylist::from_str(data).unwrap();
        println!("{playlist:?}");
        assert_eq!(
            playlist.iframe_streams.inner[1],
            IframeStreamInfo {
                common: StreamInfoCommon {
                    bandwidth: 77758,
                    codecs: vec!["hvc1.2.4.L63.90".to_owned()],
                    resolution: Resolution {
                        width: 640,
                        height: 360,
                    },
                    video_range: "PQ".to_owned(),
                    uri: "hdr10/unenc/900k/vod-iframe.m3u8".to_owned(),
                },
            }
        );
    }

    /// Expect failure on invalid m3u8 input.
    #[test]
    fn test_parse_fail() {
        let data = "this line should never exist in an HLS playlist!";
        assert!(HlsPlaylist::from_str(data).is_err());
    }
}
