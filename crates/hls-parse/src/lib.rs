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

    use super::*;

    #[test]
    fn test_parse() {
        let data = "#EXTM3U
#EXT-X-INDEPENDENT-SEGMENTS

#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aac-128k\",NAME=\"English\",LANGUAGE=\"en\",DEFAULT=YES,AUTOSELECT=YES,CHANNELS=\"2\",URI=\"audio/unenc/aac_128k/vod.m3u8\"

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
    fn test_parse_fail() {
        let data = "this line should never exist in an HLS playlist!";
        assert!(HlsPlaylist::from_str(data).is_err());
    }
}
