# HLS Parser / Sorter
## Overview
This Rust project is designed for parsing and viewing [HLS playlists](https://en.wikipedia.org/wiki/M3U#Extended_M3U). It's made up of two crates, both in the `crates` directory:
- `hls-parse`: Parse HLS playlists into structured data. Provides types and common functionality.
- `hls-sort`: View and sort an HLS playlist fetched and parsed from an HTTP endpoint.

## How to run it
_Assumes valid Rust toolchain install_.
- Do `cargo run` from `crates/hls-sort`, or this workspace's alias: `cargo run-sorter`
- `cargo run-sorter -- -h` for help (sorting options, fetching a playlist besides the default, etc.)
    - For example, `cargo run-sorter -- -v resolution` to sort video streams by resolution

## Technical Details
### Libraries used
The parser uses [nom](https://docs.rs/nom/latest/nom/), a parser-combinator crate. Its ergonomics lie in the composition of [built-in](https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md) and hand-written parsers - it can end up making your parsing code layout look roughly like the input itself (see `hls_audio` function in [this file](./crates/hls-parse/src/parsers.rs)).

As pretty standard in Rust projects, the sorter uses [reqwest](https://docs.rs/reqwest/latest/reqwest/) for HTTP requests and [clap](https://docs.rs/clap/latest/clap/) for arg parsing.

### Priorities
Making the robustness and correctness of this parser match a production-grade one was not the purpose of this excercise. Instead, I wanted to design the codebase to make it easy for someone else to jump in and:
- understand the code
- fix a bug
- expand a type
- add parsing functionality

### Performance
For the purposes of implementation time, this is _not_ a zero-copy parser. For a simpler implementation, input data is copied _once_ so that types that represent a serialized HLS playlist can _own_ the underlying data. Given more time, a future optimization of this parser could reference the underlying string data where possible and avoid copying.

### Production-readiness
Given lack of strictness on the HLS format, this parser is opinionated towards the sample input. A concrete example of this:
- HLS parameters present in the sample input are viewed as **required**. An error is thrown if any HLS parameters are missing.
    - This strictness is a choice: it's possible we'd want to enforce that all Disney-hosted HLS playlists have all of these parameters.
    - The design of `crates/hls-parse/src/builders.rs` make this easy to change if desired: one would simply make the equivalent field in `types.rs` an `Option<T>` and delete the code that extracts `T` during `build()`.

## Inspecting the code
_Where_ to look, and _what_ to look for:
- `crates/hls-parse/src`:
    - `lib.rs`: Definiton of the overarching `HlsPlaylist` type, and tests
    - `types.rs`: Types that `HlsPlaylist` is composed of, to represent different tag/stream types
    - `builders.rs`: Mirror of types in `types.rs`. Used during parsing, then converted to their mirror types.
    - `parsers.rs`: Parsing logic, including `nom` parser functions
- `crates/hls-sort/src/main.rs`: Command line parsing and sorting logic
