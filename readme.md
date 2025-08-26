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

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ installed
- Spotify account (Premium recommended for playlist features)
- Spotify Developer App (for API credentials)

### Installation

```bash
# Clone the repository
git clone https://github.com/soundphilosopher/sporl
cd sporl

# Build and install
cargo install --path .
```

### Setup

1. **Create a Spotify App**:
   - Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
   - Create a new app
   - Note your Client ID and Client Secret
   - Add `http://localhost:8080/callback` to Redirect URIs

2. **Configure Environment**:
   ```bash
   # Copy the example configuration
   cp ~/.local/share/sporlcli/.env.example ~/.local/share/sporlcli/.env

   # Edit with your Spotify app credentials
   nano ~/.local/share/sporlcli/.env
   ```

3. **Authenticate**:
   ```bash
   sporlcli auth
   ```

4. **Initial Setup**:
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

# List all followed artists
sporlcli artists

# Search for specific artists
sporlcli artists --search "arctic monkeys"
```

### Tracking Releases
```bash
# Update release data for all followed artists
sporlcli releases update

# Force complete refresh
sporlcli releases update --force

# Update specific release types
sporlcli releases update --type album,single

# List current week's releases
sporlcli releases

# Show last 4 weeks of releases
sporlcli releases --previous-weeks 4

# Show releases for a specific date
sporlcli releases --release-date 2023-12-25
```

### Creating Playlists
```bash
# Create playlist for current week
sporlcli playlist

# Create playlists for last 3 weeks
sporlcli playlist --previous-weeks 3

# Create playlist for specific date
sporlcli playlist --release-date 2023-12-01
```

### Information & Statistics
```bash
# Show current release week info
sporlcli info --release-week

# Show artist statistics
sporlcli info --artists

# Show previous weeks information
sporlcli info --previous-weeks 5

# Look up release week for specific date
sporlcli info --release-date 2023-12-25
```

### Shell Completions
```bash
# Generate completions for your shell
sporlcli completions bash > ~/.bash_completions/sporlcli
sporlcli completions zsh > ~/.zsh/completions/_sporlcli
sporlcli completions fish > ~/.config/fish/completions/sporlcli.fish
```

## ⚠️ Limitations & Considerations

### Spotify API Rate Limits

SporlCLI respects Spotify's API rate limits to ensure reliable operation:

- **Rate Limit**: ~100 requests per minute per application
- **Burst Limit**: Temporary higher rates allowed, but sustained high usage is throttled
- **Retry-After**: The tool automatically respects `Retry-After` headers (up to 2 minutes)
- **Exponential Backoff**: Built-in delays between batch operations (30 seconds between artist chunks)

**Impact on Usage**:
- Initial setup for users with many followed artists (500+) may take 15-30 minutes
- Force updates (`--force`) will always take longer than incremental updates
- Large playlist creation operations may have delays between batches

### Account Requirements

#### Basic Functionality
- **Free Spotify Account**: Required for authentication and artist following
- **Followed Artists**: You must follow artists on Spotify for the tool to track their releases
- **API Access**: Requires creating a Spotify Developer App (free)

#### Premium Features
- **Playlist Creation**: Requires Spotify Premium subscription
- **Playlist Modification**: Premium account needed to create/modify playlists
- **Track Previews**: Full track access requires Premium

### Data Limitations

#### Release Data Quality
- **Date Precision**: Only releases with day-precision dates are processed
  - ✅ Included: "2023-10-15" (exact date)
  - ❌ Excluded: "2023-10" (month only) or "2023" (year only)
- **Regional Availability**: Some releases may not be available in your market
- **Release Types**: Limited to Spotify's classification (album, single, compilation, appears_on)

#### Historical Data
- **Spotify Limitations**: Cannot fetch releases older than what Spotify provides in artist discography
- **Cache Dependency**: Historical data depends on when you first ran updates
- **No Backfill**: Cannot retroactively get releases from before you followed an artist

### Performance Considerations

#### Initial Setup Time
For reference, typical setup times based on followed artists:
- **< 50 artists**: 2-5 minutes
- **50-200 artists**: 5-15 minutes
- **200-500 artists**: 15-30 minutes
- **500+ artists**: 30+ minutes

#### Storage Requirements
Approximate local storage usage:
- **Artist Cache**: ~1-5 MB (depending on number of followed artists)
- **Release Cache**: ~10-50 MB per year of data
- **Total**: Usually under 100 MB for typical usage

#### Network Usage
- **Initial Update**: ~1-10 MB download (depends on artist count and release volume)
- **Incremental Updates**: ~100 KB - 2 MB per update
- **API Calls**: ~2-5 calls per artist during updates

### Technical Limitations

#### Authentication
- **Token Expiry**: Access tokens expire after 1 hour (automatically refreshed)
- **Refresh Token Rotation**: Refresh tokens may rotate and require re-authentication
- **Scope Requirements**: Specific OAuth scopes needed for different features
- **Browser Dependency**: Initial auth requires browser access for OAuth flow

#### Concurrent Usage
- **Single Instance**: Designed for single-user, single-instance usage
- **File Locking**: No protection against concurrent access to cache files
- **State Conflicts**: Running multiple updates simultaneously may cause issues

#### Platform Support
- **Tested Platforms**: Linux, macOS, Windows 10+
- **ARM Support**: Should work but not extensively tested
- **Container Usage**: Requires special setup for OAuth browser flow

### API Dependencies

#### Spotify Web API
- **Service Availability**: Dependent on Spotify API uptime
- **API Changes**: Breaking changes in Spotify API may require tool updates
- **Feature Deprecation**: Some features may become unavailable if Spotify removes API endpoints

#### Third-Party Dependencies
- **OAuth Flow**: Depends on system browser availability
- **Local Server**: Requires ability to bind to localhost:8080 (configurable)
- **Network Access**: Requires unrestricted HTTPS access to Spotify's APIs

### Known Issues & Workarounds

#### Common Problems

1. **"Too Many Requests" Errors**
   - **Cause**: Hitting rate limits during large updates
   - **Solution**: Tool automatically retries with delays
   - **Workaround**: Use `--force` sparingly, prefer incremental updates

2. **Missing Recent Releases**
   - **Cause**: Spotify may have delays in making new releases available via API
   - **Solution**: Run `sporlcli releases update` periodically
   - **Timing**: New releases typically appear within 24-48 hours

3. **Playlist Creation Failures**
   - **Cause**: Insufficient permissions or non-Premium account
   - **Solution**: Ensure Premium account and proper OAuth scopes
   - **Check**: Verify `playlist-modify-public` and `playlist-modify-private` scopes

4. **Large Artist Collections**
   - **Issue**: Very long initial update times (1000+ artists)
   - **Workaround**: Run initial update during off-peak hours
   - **Alternative**: Consider unfollowing inactive artists to improve performance

#### Browser-Related Issues

1. **OAuth Browser Won't Open**
   - **Manual Solution**: Copy URL from terminal and open manually
   - **SSH/Remote**: Use port forwarding or run auth on local machine

2. **Corporate Networks**
   - **Firewall Issues**: May block Spotify API access
   - **Proxy Problems**: May interfere with OAuth flow
   - **Solution**: Contact IT team for API access allowlist

### Best Practices

#### For Optimal Performance
- Run incremental updates regularly rather than forcing full refreshes
- Update artists monthly, releases weekly
- Use specific release types (`--type album`) instead of `all` when possible
- Set up automated updates during low-usage periods

#### For Reliability
- Monitor cache directory disk space (~100 MB buffer recommended)
- Re-authenticate if experiencing persistent auth errors
- Keep tool updated for latest Spotify API compatibility
- Backup cache files before major version updates

#### For Large Collections
- Consider using release type filters to reduce API calls
- Run updates during off-peak hours to avoid rate limiting
- Use `--previous-weeks` parameter judiciously to limit data fetched
- Monitor network usage if on metered connections

## 🏗️ Architecture

### Data Organization
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
└── .env                        # Configuration
```

### Release Week System
- Weeks start on Saturday and end on Friday
- Week 1 begins on the Saturday before/on January 1st
- Consistent numbering system for reliable organization

## ⚙️ Configuration

The `.env` file supports these variables:

```bash
# Local Server Configuration (Web App Redirect URI)
SERVER_ADDRESS=127.0.0.1:8080

# Spotify API Configuration
SPOTIFY_API_AUTH_CLIENT_ID=your_client_id
SPOTIFY_API_AUTH_CLIENT_SECRET=your_client_secret
SPOTIFY_USER_ID=your_spotify_username

# Spotify API Configuration and Endpoints (usually don't need to change)
SPOTIFY_API_REDIRECT_URI=http://${SERVER_ADDRESS}/callback
SPOTIFY_API_AUTH_SCOPE="user-library-read user-follow-read user-read-email user-read-private playlist-modify-private playlist-modify-public playlist-read-private"
SPOTIFY_API_AUTH_URL=https://accounts.spotify.com/authorize
SPOTIFY_API_TOKEN_URL=https://accounts.spotify.com/api/token
SPOTIFY_API_URL=https://api.spotify.com/v1
```

## 🔧 Advanced Usage

### Release Types
Filter updates by release type:
- `album` - Full-length albums
- `single` - Singles and EPs
- `appears_on` - Compilations and features
- `compilation` - Greatest hits, etc.
- `all` - All types

### Batch Operations
```bash
# Update everything in sequence
sporlcli artists update && sporlcli releases update --type all

# Create playlists for multiple weeks
for week in {1..4}; do
  sporlcli playlist --previous-weeks $week
done
```

### Automation
Set up cron jobs for regular updates:
```bash
# Daily artist check at 9 AM
0 9 * * * /usr/local/bin/sporlcli artists update

# Weekly release update on Fridays at 10 AM
0 10 * * 5 /usr/local/bin/sporlcli releases update
```

## 🛠️ Development

### Building from Source
```bash
git clone https://github.com/yourusername/sporlcli.git
cd sporlcli
cargo build --release
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
├── main.rs             # CLI entry point
├── config.rs           # Configuration management
├── server.rs           # OAuth callback server
├── types.rs            # Data structures
├── utils.rs            # Utility functions
├── api/                # HTTP API endpoints
├── cli/                # CLI command implementations
├── management/         # Data management and caching
└── spotify/            # Spotify API integration
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Spotify Web API](https://developer.spotify.com/documentation/web-api/) for providing the music data
- [clap](https://clap.rs/) for the excellent CLI framework
- [tokio](https://tokio.rs/) for async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client functionality

## 📞 Support

If you encounter any issues or have questions:

1. Search existing [issues](https://github.com/soundphilosopher/sporl/issues)
2. Create a new issue with detailed information

## 🗺️ Roadmap

- [ ] Support for multiple Spotify accounts
- [ ] Advanced playlist customization options
- [ ] Release notifications and alerts
- [ ] Statistics and analytics dashboard
- [ ] Export functionality for release data

---

**Made with ❤️ for music lovers who want to stay up-to-date with their favorite artists!
