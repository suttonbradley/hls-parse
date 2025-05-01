use std::str::FromStr;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use hls_parse::{
    HlsPlaylist,
    types::{
        media::Audio,
        stream_info::{IframeStreamInfo, StreamInfo},
    },
};

const DEFAULT_HLS_URL: &'static str =
    "https://lw.bamgrid.com/2.0/hls/vod/bam/ms02/hls/dplus/bao/master_unenc_hdr10_all.m3u8";
const CLAP_HELP: &'static str =
    "A simple viewing/sorting tool for HLS playlists fetched from a URL.
When no sort is selected for a given tag type, order appears as-parsed from the playlist.";

#[derive(Parser)]
#[command(about = CLAP_HELP)]
struct Args {
    /// (override) URL to fetch HLS playlist from
    #[arg(short = 'u')]
    url: Option<String>,
    /// Sort HLS audio streams by a parameter value
    #[arg(short = 'a')]
    sort_audio: Option<AudioSort>,
    /// Sort HLS video streams by a parameter value
    #[arg(short = 'v')]
    sort_video: Option<VideoSort>,
    /// Sort HLS iframe streams by a parameter value
    #[arg(short = 'i')]
    sort_iframe: Option<VideoSort>,
}

/// Enables sorting audio streams by HLS parameters.
// NOTE: Variants limited by request. Add a variant to increase sorting capability.
#[derive(Clone, ValueEnum)]
enum AudioSort {
    Channels,
    GroupId,
}

/// Enables sorting video stream types by HLS parameters.
// NOTE: Variants limited by request. Add a variant to increase sorting capability.
#[derive(Clone, ValueEnum)]
enum VideoSort {
    Bandwidth,
    Resolution,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Fetch data from URL and store body for parsing
    let hls_raw_data = reqwest::blocking::get(args.url.unwrap_or(DEFAULT_HLS_URL.to_owned()))
        .context("failed to GET from requested http endpoint")?
        .text()?;

    // Parse HLS playlist to structured data
    let mut playlist = HlsPlaylist::from_str(hls_raw_data.as_str())?;

    // Perform sorts, if requested
    if let Some(sorter) = args.sort_audio {
        let sort_fn = match sorter {
            AudioSort::Channels => |x: &Audio, y: &Audio| x.channel_info.cmp(&y.channel_info),
            AudioSort::GroupId => |x: &Audio, y: &Audio| x.group_id.cmp(&y.group_id),
        };
        playlist.audio_streams.inner.sort_by(sort_fn);
    }

    // TODO: Could reduce code duplication below by implementing a trait that returns &StreamInfoCommon for various
    //       video stream types, then converting `VideoSort` to a matching sorting function that takes &StreamInfoCommon.
    if let Some(sorter) = args.sort_video {
        let sort_fn = match sorter {
            VideoSort::Bandwidth => {
                |x: &StreamInfo, y: &StreamInfo| x.common.bandwidth.cmp(&y.common.bandwidth)
            }
            VideoSort::Resolution => {
                |x: &StreamInfo, y: &StreamInfo| x.common.resolution.cmp(&y.common.resolution)
            }
        };
        playlist.streams.inner.sort_by(sort_fn);
    }

    if let Some(sorter) = args.sort_iframe {
        let sort_fn = match sorter {
            VideoSort::Bandwidth => |x: &IframeStreamInfo, y: &IframeStreamInfo| {
                x.common.bandwidth.cmp(&y.common.bandwidth)
            },
            VideoSort::Resolution => |x: &IframeStreamInfo, y: &IframeStreamInfo| {
                x.common.resolution.cmp(&y.common.resolution)
            },
        };
        playlist.iframe_streams.inner.sort_by(sort_fn);
    }

    // Display HLS playlist and exit
    println!("{}", playlist);
    Ok(())
}
