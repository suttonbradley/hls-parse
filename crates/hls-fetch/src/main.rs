use std::str::FromStr;

use anyhow::Context;
use hls_parse::HlsPlaylist;

fn main() -> anyhow::Result<()> {
    let hls_raw_data = reqwest::blocking::get(
        "https://lw.bamgrid.com/2.0/hls/vod/bam/ms02/hls/dplus/bao/master_unenc_hdr10_all.m3u8",
    )
    .context("failed to GET from requested http endpoint")?
    .text()?;

    let playlist = HlsPlaylist::from_str(hls_raw_data.as_str())?;
    println!("{}", playlist);

    Ok(())
}
