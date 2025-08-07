# Spotify Release List CLI

CLI that connects to the Spotify API via your Spotify Web-App and fetches artists you follow and releases for the current release week or older once.

## Pre-requisites

- rustc
- cargo
- [Spotify Web-App](https://developer.spotify.com/documentation/web-api/tutorials/getting-started)

## Limitations

- Fetches only albums as release type
- Depends on Spotify Web-App restrictions like rate limits and authentication
- No crate for installation

## Building

- Clone the repository
- Run `cargo build --release`

## Running

- Run `cargo run --release`

## Installation

- Run `cargo install --path .`

## Commands

TBA

## ToDo

- [x] Create function to authenticate to Spotify API via Spotify Web-App
- [x] Fetch following artists from Spotify-API
- [x] Cache fetched artists to local file cache
- [x] Fetch releases for cached artists
- [x] Cache artist-releases to local file cache
- [x] Create playlist for fetched releases
- [ ] Expose necessary information in readme
- [ ] Add possibility to fetch Singles and others from Spotify API
- [ ] Automatically remove artist-release-cache after fetching all releases
- [ ] Automatically remove artist state cache after fetching all releases
- [x] Remove static public configuration
- [ ] Integrate authentication with more than one Spotiy App
- [ ] Add to crates.io
