//! Parser for HLS playlists of M3U8 format.
//!
//! Provides data types that reflect HLS data (streams, media, etc.).
//! TODO:

mod parsers;
mod types;

use std::str::FromStr;

#[derive(Default, Debug)]
/// Represents a parsed HLS playlist, supporting various `#EXT-X-*` tags.
pub struct HlsPlaylist {
    audio_tracks: Vec<types::media::Audio>,
    streams: Vec<types::stream_info::StreamInfo>,
    iframe_streams: Vec<types::stream_info::IframeStreamInfo>,
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
    use crate::types::media::Audio;
    use crate::types::stream_info::{IframeStreamInfo, Resolution, StreamInfo, StreamInfoCommon};

    use super::*;

    #[test]
    /// Parse basic elements that don't return structured data.
    fn test_parse_basic() {
        let data = "#EXTM3U
#EXT-X-INDEPENDENT-SEGMENTS
";
        let _ = HlsPlaylist::from_str(data).unwrap();
    }

    #[test]
    fn test_parse_audio() {
        let data = "#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aac-128k\",NAME=\"English\",LANGUAGE=\"en\",DEFAULT=YES,AUTOSELECT=YES,CHANNELS=\"2\",URI=\"audio/unenc/aac_128k/vod.m3u8\"

#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aac-64k\",NAME=\"English\",LANGUAGE=\"en\",DEFAULT=YES,AUTOSELECT=YES,CHANNELS=\"2\",URI=\"audio/unenc/aac_64k/vod.m3u8\"
";
        let playlist = HlsPlaylist::from_str(data).unwrap();
        println!("{playlist:?}");
        assert_eq!(
            playlist.audio_tracks[0],
            Audio {
                group_id: "aac-128k".to_owned(),
                name: "English".to_owned(),
                language: "en".to_owned(),
                default: true,
                auto_select: true,
                channels: 2,
                uri: "audio/unenc/aac_128k/vod.m3u8".to_owned(),
            }
        );
    }

    #[test]
    fn test_parse_stream() {
        let data = "#EXT-X-STREAM-INF:BANDWIDTH=2483789,AVERAGE-BANDWIDTH=1762745,CODECS=\"mp4a.40.2,hvc1.2.4.L90.90\",RESOLUTION=960x540,FRAME-RATE=23.97,VIDEO-RANGE=PQ,AUDIO=\"aac-128k\",CLOSED-CAPTIONS=NONE
hdr10/unenc/1650k/vod.m3u8
";

        let playlist = HlsPlaylist::from_str(data).unwrap();
        println!("{playlist:?}");
        assert_eq!(
            playlist.streams[0],
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
                closed_captions: false,
            }
        );
    }

    #[test]
    fn test_parse_iframe() {
        let data = "#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=222552,CODECS=\"hvc1.2.4.L93.90\",RESOLUTION=1280x720,VIDEO-RANGE=PQ,URI=\"hdr10/unenc/3300k/vod-iframe.m3u8\"

#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=77758,CODECS=\"hvc1.2.4.L63.90\",RESOLUTION=640x360,VIDEO-RANGE=PQ,URI=\"hdr10/unenc/900k/vod-iframe.m3u8\"
";

        let playlist = HlsPlaylist::from_str(data).unwrap();
        println!("{playlist:?}");
        assert_eq!(
            playlist.iframe_streams[1],
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

    #[test]
    fn test_parse_fail() {
        let data = "this line should never exist in an HLS playlist!";
        assert!(HlsPlaylist::from_str(data).is_err());
    }
}
