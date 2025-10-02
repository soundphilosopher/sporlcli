# SporlCLI - Spotify Release Tracker

A powerful command-line tool for tracking music releases from your followed artists on Spotify. Never miss a new album, single, or EP from your favorite musicians again!

## ✨ Features

- **🎵 Release Tracking**: Automatically track new releases from all your followed Spotify artists
- **📅 Weekly Organization**: Organize releases by weekly periods for easy browsing
- **🎧 Smart Playlists**: Auto-generate playlists with the latest releases
- **⚡ Intelligent Caching**: Fast performance with smart local caching
- **🔍 Flexible Filtering**: Filter by release types (albums, singles, EPs, compilations)
- **📊 Statistics & Info**: Get insights about your followed artists and releases
- **🔐 Secure Authentication**: OAuth 2.0 PKCE flow for secure Spotify integration
- **🌐 Cross-Platform**: Works on Linux, macOS, and Windows
- **📱 Resume Capability**: Interrupted operations can be resumed automatically

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ installed ([Install Rust](https://rustup.rs/))
- Spotify account (Premium recommended for playlist features)
- Spotify Developer App (for API credentials)

### Installation

```bash
# Clone the repository
git clone https://github.com/soundphilosopher/sporlcli
cd sporlcli

# Build and install
cargo install --path .
```

### Setup

#### 1. Create a Spotify App

- Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
- Create a new app with these settings:
  - **App Name**: SporlCLI (or your preferred name)
  - **App Description**: Personal release tracker
  - **Website**: Not required
  - **Redirect URIs**: `http://127.0.0.1:8080/callback`
- Note your **Client ID** (you won't need the Client Secret for PKCE flow)

#### 2. Configure Environment

The configuration file location depends on your operating system:

**Linux:**
```bash
# Configuration directory
~/.local/share/sporlcli/

# Copy and edit the example configuration
cp ~/.local/share/sporlcli/.env.example ~/.local/share/sporlcli/.env
nano ~/.local/share/sporlcli/.env
```

**macOS:**
```bash
# Configuration directory
~/Library/Application Support/sporlcli/

# Copy and edit the example configuration
cp "~/Library/Application Support/sporlcli/.env.example" "~/Library/Application Support/sporlcli/.env"
nano "~/Library/Application Support/sporlcli/.env"
```

**Windows:**
```powershell
# Configuration directory
%LOCALAPPDATA%\sporlcli\

# Copy and edit the example configuration (PowerShell)
Copy-Item "$env:LOCALAPPDATA\sporlcli\.env.example" "$env:LOCALAPPDATA\sporlcli\.env"
notepad "$env:LOCALAPPDATA\sporlcli\.env"
```

**Required Configuration:**
```bash
# Local Server Configuration (OAuth callback)
SERVER_ADDRESS="127.0.0.1:8080"

# Spotify API Configuration
SPOTIFY_API_AUTH_CLIENT_ID=your_client_id_here
SPOTIFY_USER_ID=your_spotify_username

# These usually don't need to be changed
SPOTIFY_API_REDIRECT_URI="http://${SERVER_ADDRESS}/callback"
SPOTIFY_API_AUTH_SCOPE="user-library-read user-follow-read user-read-email user-read-private playlist-modify-private playlist-modify-public playlist-read-private"
SPOTIFY_API_AUTH_URL="https://accounts.spotify.com/authorize"
SPOTIFY_API_TOKEN_URL="https://accounts.spotify.com/api/token"
SPOTIFY_API_URL="https://api.spotify.com/v1"
```

**Note:** You do NOT need `SPOTIFY_API_AUTH_CLIENT_SECRET` as this application uses the PKCE flow for enhanced security.

#### 3. Authenticate

```bash
# Authenticate with Spotify (this will open your browser)
sporlcli auth
```

#### 4. Initial Setup

```bash
# Fetch your followed artists
sporlcli artists update

# Fetch release data
sporlcli releases update
```

## 📖 Usage

### Authentication
```bash
# Authenticate with Spotify (run this first)
sporlcli auth
```

### Managing Artists
```bash
# Update your followed artists cache
sporlcli artists update

# Force complete refresh of artists
sporlcli artists update --force

# List all followed artists
sporlcli artists

# Search for specific artists
sporlcli artists --search "arctic monkeys"
```

### Tracking Releases
```bash
# Update release data for all followed artists
sporlcli releases update

# Force complete refresh of all release data
sporlcli releases update --force

# Update specific release types
sporlcli releases update --type album,single

# List current week's releases
sporlcli releases

# Show last 4 weeks of releases
sporlcli releases --previous-weeks 4

# Show releases for a specific date's week
sporlcli releases --release-date 2023-12-25

# Show 2 weeks before a specific date
sporlcli releases --release-date 2023-12-25 --previous-weeks 2
```

### Creating Playlists
```bash
# Create playlist for current week
sporlcli playlist

# Create playlists for last 3 weeks
sporlcli playlist --previous-weeks 3

# Create playlist for specific date's week
sporlcli playlist --release-date 2023-12-01

# Create playlists for 2 weeks before specific date
sporlcli playlist --release-date 2023-12-01 --previous-weeks 2
```

### Information & Statistics
```bash
# Show current release week info
sporlcli info --release-week

# Show artist statistics (cache vs remote count)
sporlcli info --artists

# Show previous weeks information
sporlcli info --previous-weeks 5

# Look up release week for specific date
sporlcli info --release-date 2023-12-25
```

### Shell Completions
```bash
# Bash
sporlcli completions bash > ~/.bash_completions/sporlcli

# Zsh
sporlcli completions zsh > ~/.zsh/completions/_sporlcli

# Fish
sporlcli completions fish > ~/.config/fish/completions/sporlcli.fish

# PowerShell (Windows)
sporlcli completions powershell > sporlcli.ps1
```

## 🏗️ Architecture

### Data Organization

**Linux:**
```
~/.local/share/sporlcli/
├── cache/
│   ├── artist-releases.json    # Artist-to-releases mapping
│   └── token.json              # OAuth tokens
├── releases/
│   └── {year}/
│       └── {week}/
│           └── releases.json   # Weekly release data
├── state/
│   ├── state_artists.json     # Update progress tracking
│   └── state_releases.json    # Release update state
├── .env                        # Configuration
└── .env.example               # Configuration template
```

**macOS:**
```
~/Library/Application Support/sporlcli/
├── cache/
├── releases/
├── state/
├── .env
└── .env.example
```

**Windows:**
```
%LOCALAPPDATA%\sporlcli\
├── cache\
├── releases\
├── state\
├── .env
└── .env.example
```

### Release Week System
- Weeks start on Saturday and end on Friday
- Week 1 begins on the Saturday after or on January 1st
- Consistent numbering system for reliable organization
- Release years start with week 1 and end with week 52
- Examples:
  - Week 1 of 2024 is January 6-12, 2024
  - Week 52 of 2024 is December 28, 2024 - January 3, 2025
  - Week 1 of 2025 is January 4-10, 2025

## ⚙️ Configuration Reference

All configuration is managed through environment variables, typically set in your `.env` file:

### Required Settings
```bash
# Your Spotify application's client ID (from Spotify Developer Dashboard)
SPOTIFY_API_AUTH_CLIENT_ID=your_client_id

# Your Spotify username (for playlist creation)
SPOTIFY_USER_ID=your_username

# Local server address for OAuth callback
SERVER_ADDRESS=127.0.0.1:8080
```

### Optional Settings (usually don't need changes)
```bash
# OAuth redirect URI (must match Spotify app settings)
SPOTIFY_API_REDIRECT_URI=http://${SERVER_ADDRESS}/callback

# OAuth scope permissions
SPOTIFY_API_AUTH_SCOPE=user-library-read user-follow-read user-read-email user-read-private playlist-modify-private playlist-modify-public playlist-read-private

# Spotify API endpoints
SPOTIFY_API_AUTH_URL=https://accounts.spotify.com/authorize
SPOTIFY_API_TOKEN_URL=https://accounts.spotify.com/api/token
SPOTIFY_API_URL=https://api.spotify.com/v1
```

## 🔧 Advanced Usage

### Release Types
Filter updates by release type:
- `album` - Full-length albums
- `single` - Singles and EPs
- `appears_on` - Albums the artist appears on but doesn't own
- `compilation` - Greatest hits, compilations
- `all` - All of the above types

Example:
```bash
# Only track albums and singles
sporlcli releases update --type album,single
```

### Batch Operations

**Linux/macOS:**
```bash
# Update everything in sequence
sporlcli artists update && sporlcli releases update --type all

# Create playlists for multiple weeks
for week in {1..4}; do
  sporlcli playlist --previous-weeks $week
done
```

**Windows (PowerShell):**
```powershell
# Update everything in sequence
sporlcli artists update; sporlcli releases update --type all

# Create playlists for multiple weeks
for ($i=1; $i -le 4; $i++) {
    sporlcli playlist --previous-weeks $i
}
```

### Automation

**Linux/macOS (cron):**
```bash
# Edit crontab
crontab -e

# Add these lines:
# Daily artist check at 9 AM
0 9 * * * /usr/local/bin/sporlcli artists update

# Weekly release update on Fridays at 10 AM
0 10 * * 5 /usr/local/bin/sporlcli releases update
```

**Windows (Task Scheduler):**
```powershell
# Create a scheduled task for daily artist updates
schtasks /create /tn "SporlCLI Artist Update" /tr "sporlcli.exe artists update" /sc daily /st 09:00

# Create a scheduled task for weekly release updates (Fridays at 10 AM)
schtasks /create /tn "SporlCLI Release Update" /tr "sporlcli.exe releases update" /sc weekly /d FRI /st 10:00
```

## ⚠️ Important Limitations & Considerations

### Spotify API Rate Limits

SporlCLI respects Spotify's API rate limits to ensure reliable operation:

- **Rate Limit**: ~100 requests per minute per application
- **Automatic Retry**: Respects `Retry-After` headers (up to 2 minutes)
- **Batch Delays**: Built-in 30-second delays between artist processing chunks
- **Progressive Timeouts**: Longer delays for repeated rate limit hits

**Performance Expectations:**
- **< 50 artists**: 2-5 minutes for initial setup
- **50-200 artists**: 5-15 minutes for initial setup
- **200-500 artists**: 15-30 minutes for initial setup
- **500+ artists**: 30+ minutes for initial setup

### Account Requirements

#### Basic Functionality (Free Spotify Account)
✅ Authentication and artist following
✅ Release tracking and caching
✅ Information queries and statistics
✅ Viewing release data

#### Premium Features (Spotify Premium Required)
🎵 **Playlist Creation**: Create new playlists
🎵 **Playlist Modification**: Add tracks to playlists
🎵 **Full Track Access**: Complete track information

### Data Limitations

#### Release Data Quality
- ✅ **Precise Dates**: Only releases with exact dates (e.g., "2023-10-15")
- ❌ **Imprecise Dates**: Excludes releases with only month/year (e.g., "2023-10")
- 🌍 **Regional Availability**: Some releases may not be available in your market
- 📝 **Classification**: Limited to Spotify's release type categories

#### Technical Considerations
- **Storage**: ~50-200 MB typical usage for cache and data
- **Network**: ~1-10 MB initial download, ~100KB-2MB for updates
- **Authentication**: Tokens expire after 1 hour (automatically refreshed)
- **Concurrent Usage**: Designed for single-user, single-instance operation

### Known Issues & Troubleshooting

#### Common Issues

**1. Authentication Problems**
```bash
# If auth fails, try:
sporlcli auth

# If browser doesn't open, copy the URL manually from the terminal
```

**2. Missing Recent Releases**
```bash
# Spotify may delay API availability for new releases (24-48 hours)
sporlcli releases update
```

**3. Rate Limiting Messages**
```bash
# Normal during large updates - the tool will wait and retry automatically
# Just let it run, or try again during off-peak hours
```

**4. Playlist Creation Fails**
- Ensure you have Spotify Premium
- Check that `SPOTIFY_USER_ID` matches your exact Spotify username
- Verify OAuth scopes include playlist permissions

#### Platform-Specific Issues

**Windows:**
- If PowerShell execution policy blocks scripts, run:
  ```powershell
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
  ```

**macOS:**
- If you get permission errors, ensure terminal has full disk access
- Path with spaces: Use quotes around paths in commands

**Linux:**
- Ensure `~/.local/bin` is in your PATH for global access
- Some distributions may require additional dependencies for the browser opening

## 🛠️ Development

### Building from Source
```bash
git clone https://github.com/soundphilosopher/sporl.git
cd sporl
cargo build --release

# The binary will be at target/release/sporlcli
```

### Running Tests
```bash
cargo test
```

### Documentation
```bash
# Generate and view documentation
cargo doc --open
```

### Project Structure
```
src/
├── lib.rs              # Library root and common utilities
├── main.rs             # CLI entry point and argument parsing
├── config.rs           # Configuration management (.env loading)
├── server.rs           # OAuth callback HTTP server
├── types.rs            # Data structures and type definitions
├── utils.rs            # Utility functions (dates, PKCE, etc.)
├── api/                # HTTP API endpoints for callback server
│   ├── mod.rs          # API module exports
│   ├── callback.rs     # OAuth callback handler
│   └── health.rs       # Health check endpoint
├── cli/                # CLI command implementations
│   ├── mod.rs          # CLI module exports
│   ├── artists.rs      # Artist management commands
│   ├── auth.rs         # Authentication command
│   ├── info.rs         # Information and statistics commands
│   ├── playlist.rs     # Playlist creation commands
│   └── releases.rs     # Release tracking commands
├── management/         # Data management and caching layer
│   ├── mod.rs          # Management module exports
│   ├── artist.rs       # Artist data management
│   ├── auth.rs         # Token lifecycle management
│   ├── release.rs      # Release data organization
│   └── state.rs        # Operation state tracking
└── spotify/            # Spotify API integration
    ├── mod.rs          # Spotify module exports
    ├── artists.rs      # Artist API operations
    ├── auth.rs         # OAuth flow implementation
    ├── playlist.rs     # Playlist API operations
    └── releases.rs     # Release API operations
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run tests (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Coding Guidelines
- Follow Rust naming conventions
- Add documentation for public functions
- Include error handling for external API calls
- Write tests for new functionality

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Spotify Web API](https://developer.spotify.com/documentation/web-api/) for providing comprehensive music data
- [clap](https://clap.rs/) for the excellent CLI framework with derive macros
- [tokio](https://tokio.rs/) for async runtime and utilities
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client functionality
- [serde](https://serde.rs/) for JSON serialization/deserialization

## 📞 Support

If you encounter any issues or have questions:

1. **Check the troubleshooting section** above for common issues
2. **Search existing [issues](https://github.com/soundphilosopher/sporl/issues)** for similar problems
3. **Create a new issue** with:
   - Your operating system (Windows/macOS/Linux)
   - Rust version (`rustc --version`)
   - Complete error message
   - Steps to reproduce the problem
   - Your configuration (remove sensitive values)

## 🗺️ Roadmap

- [ ] **Multi-Account Support**: Handle multiple Spotify accounts
- [ ] **Advanced Playlist Options**: Custom playlist descriptions, artwork
- [ ] **Release Notifications**: Desktop/email notifications for new releases
- [ ] **Export Functionality**: Export release data to CSV/JSON
- [ ] **Statistics Dashboard**: Web-based analytics view
- [ ] **Custom Week Definitions**: Alternative week numbering systems
- [ ] **Release Filters**: Filter by genre, label, or custom criteria
- [ ] **Playlist Templates**: Customizable playlist creation rules

---

**Made with ❤️ for music lovers who want to stay up-to-date with their favorite artists!**

*Compatible with Linux, macOS, and Windows • Powered by Spotify Web API
