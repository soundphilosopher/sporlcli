use std::sync::Arc;

use clap::{
    ArgAction, CommandFactory, Parser, Subcommand,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_complete::{Shell, generate};

use sporlcli::{cli, config, error, types::PkceToken, utils};
use tokio::sync::Mutex;

/// Creates custom styling for the CLI interface.
///
/// Defines the color scheme and styling effects for the command-line interface
/// using ANSI colors. This enhances the visual appearance of help text, usage
/// information, and other CLI output elements.
///
/// # Returns
///
/// A `Styles` configuration with custom colors for different CLI elements:
/// - Headers and usage text: White and bold
/// - Literals: Bright blue
/// - Placeholders: Bright green
///
/// # Example
///
/// ```
/// let cli_styles = styles();
/// // Used internally by clap for styling CLI output
/// ```
fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::White.on_default() | Effects::BOLD)
        .usage(AnsiColor::White.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightBlue.on_default())
        .placeholder(AnsiColor::BrightGreen.on_default())
}

/// Main CLI structure for the Spotify Release Tracker application.
///
/// Defines the root command-line interface using clap, including version information,
/// application metadata, and custom styling. This serves as the entry point for
/// all CLI operations and contains the top-level command structure.
///
/// The CLI supports various subcommands for different operations:
/// - Authentication with Spotify
/// - Artist management
/// - Release tracking
/// - Playlist creation
/// - Information queries
/// - Shell completion generation
#[derive(Parser, Debug, Clone)]
#[clap(
  version = env!("CARGO_PKG_VERSION"),
  name=env!("CARGO_PKG_NAME"),
  bin_name=env!("CARGO_PKG_NAME"),
  author=env!("CARGO_PKG_AUTHORS"),
  about=env!("CARGO_PKG_DESCRIPTION"),
  styles=styles(),
)]
struct Cli {
    /// The subcommand to execute
    #[clap(subcommand)]
    command: Command,
}

/// Enumeration of all available CLI subcommands.
///
/// Represents the main functionality areas of the application, each corresponding
/// to a different aspect of music release tracking and Spotify integration.
/// Each variant maps to specific CLI operations and their associated options.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Authorize with Spotify API
    Auth,

    /// Handle followed artists
    Artists(ArtistsOptions),

    /// Handle releases
    Releases(ReleasesOptions),

    /// Create playlist for called weeks
    #[clap(about = "Create playlist for called weeks")]
    Playlist(PlaylistOptions),

    /// Some helper information about releases and artists
    Info(InfoOptions),

    /// Get shell completions
    Completions(CompletionsOption),
}

/// Configuration options for artist-related commands.
///
/// Provides options for both searching existing artists and managing artist data
/// through subcommands. The structure prevents conflicts between search operations
/// and subcommand execution to ensure clear command semantics.
///
/// # Usage
///
/// - `sporlcli artists --search "artist name"` - Search for artists
/// - `sporlcli artists update` - Update artist data
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Handle followed artists",
    args_conflicts_with_subcommands = true // disallow mixing --search with subcommands
)]
pub struct ArtistsOptions {
    /// Search for artists by name
    #[clap(long)]
    pub search: Option<String>,

    /// Subcommands for artist management operations
    #[command(subcommand)]
    pub command: Option<ArtistsSubcommand>,
}

/// Subcommands available under the artists command.
///
/// Defines the specific operations that can be performed on artist data,
/// currently supporting update operations with various configuration options.
#[derive(Subcommand, Debug, Clone)]
pub enum ArtistsSubcommand {
    /// Update artists data from Spotify
    Update(ArtistsUpdateOpts),
}

/// Options for updating artist information.
///
/// Controls how the artist update process behaves, including whether to
/// bypass caching mechanisms and update guards for forced refreshes.
#[derive(Parser, Debug, Clone)]
pub struct ArtistsUpdateOpts {
    /// Force update, bypassing caches and update guards
    #[clap(long)]
    pub force: bool,
}

/// Configuration options for release-related commands.
///
/// Provides options for querying releases with time-based filters and managing
/// release data through subcommands. Prevents conflicts between query operations
/// and data management operations to maintain command clarity.
///
/// # Usage
///
/// - `sporlcli releases --previous-weeks 4` - Show releases from last 4 weeks
/// - `sporlcli releases --release-date 2023-10-17` - Show releases for specific date
/// - `sporlcli releases update` - Update release data
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Handle releases",
    args_conflicts_with_subcommands = true // disallow mixing query flags with `update`
)]
pub struct ReleasesOptions {
    /// Number of previous weeks to include in the query
    #[clap(long)]
    pub previous_weeks: Option<u32>,

    /// Filter releases by a specific date (YYYY-MM-DD format)
    #[clap(long)]
    pub release_date: Option<String>,

    /// Subcommands for release management operations
    #[command(subcommand)]
    pub command: Option<ReleasesSubcommand>,
}

/// Subcommands available under the releases command.
///
/// Defines the specific operations that can be performed on release data,
/// currently supporting update operations with configurable release type filtering.
#[derive(Subcommand, Debug, Clone)]
pub enum ReleasesSubcommand {
    /// Update releases data from Spotify
    Update(ReleasesUpdateOpts),
}

/// Options for updating release information.
///
/// Controls the release update process, including force update capabilities
/// and filtering by release types. Supports specifying which types of releases
/// to include during the update process (albums, singles, etc.).
#[derive(Parser, Debug, Clone)]
pub struct ReleasesUpdateOpts {
    /// Force update, bypassing caches and update guards
    #[clap(long)]
    pub force: bool,

    /// Release type(s) to include during update (can be repeated)
    ///
    /// Accepts values like "album", "single", "compilation", "appears_on", or "all".
    /// Multiple values can be specified by repeating the flag or using comma separation.
    #[clap(
        long = "type",
        default_value = "album,single",
        value_parser = utils::parse_release_kinds,
        action = ArgAction::Append,
        num_args = 1
    )]
    pub release_types: utils::ReleaseKinds,
}

/// Options for playlist creation commands.
///
/// Configures the time range and filtering criteria for creating playlists
/// based on music releases. Supports both relative time ranges (previous weeks)
/// and specific date targeting.
///
/// # Usage
///
/// - `sporlcli playlist --previous-weeks 2` - Create playlist for last 2 weeks
/// - `sporlcli playlist --release-date 2023-10-17` - Create playlist for specific date
/// - `sporlcli playlist --type album` - Create playlist for specific release type/kind
#[derive(Parser, Debug, Clone)]
pub struct PlaylistOptions {
    /// Number of previous weeks to include in the playlist
    #[clap(long)]
    previous_weeks: Option<u32>,

    /// Target a specific release date for the playlist (YYYY-MM-DD format)
    #[clap(long)]
    release_date: Option<String>,

    /// Release type(s) to include during update (can be repeated)
    ///
    /// Accepts values like "album", "single", "compilation", "appears_on", or "all".
    /// Multiple values can be specified by repeating the flag or using comma separation.
    #[clap(
        long = "type",
        default_value = "album,single",
        value_parser = utils::parse_release_kinds,
        action = ArgAction::Append,
        num_args = 1
    )]
    pub release_kinds: utils::ReleaseKinds,
}

/// Options for information and statistics commands.
///
/// Provides various flags for displaying different types of information about
/// releases, artists, and time periods. Supports both boolean flags for enabling
/// specific information displays and time-based filtering options.
///
/// # Usage
///
/// - `sporlcli info --release-week --previous-weeks 1` - Show current week info
/// - `sporlcli info --artists` - Show artist statistics
#[derive(Parser, Debug, Clone)]
pub struct InfoOptions {
    /// Display information about the current release week
    #[clap(long)]
    release_week: bool,

    /// Display information about followed artists
    #[clap(long)]
    artists: bool,

    /// Number of previous weeks to include in the information display
    #[clap(long)]
    previous_weeks: Option<u32>,

    /// Filter information by a specific release date (YYYY-MM-DD format)
    #[clap(long)]
    release_date: Option<String>,
}

/// Options for shell completion generation.
///
/// Configures the shell completion generator to produce completion scripts
/// for different shell environments. Supports various shells including
/// bash, zsh, fish, and PowerShell.
///
/// # Usage
///
/// - `sporlcli completions bash` - Generate bash completions
/// - `sporlcli completions zsh` - Generate zsh completions
#[derive(Parser, Debug, Clone)]
pub struct CompletionsOption {
    /// The shell to generate completions for
    shell: Shell,
}

/// Main entry point for the Spotify Release Tracker CLI application.
///
/// Initializes the application environment, parses command-line arguments,
/// and dispatches to the appropriate command handlers. This function coordinates
/// the entire application flow from startup through command execution.
///
/// The main function performs the following operations:
/// 1. Loads environment configuration from files and environment variables
/// 2. Parses command-line arguments using clap
/// 3. Dispatches to appropriate command handlers based on the subcommand
/// 4. Manages shared state for OAuth operations when needed
///
/// # Command Routing
///
/// - `auth` - Initiates OAuth authentication flow with Spotify
/// - `artists` - Manages followed artists (list, search, update)
/// - `releases` - Handles music release tracking (list, update, filter)
/// - `playlist` - Creates playlists based on release data
/// - `info` - Displays statistics and information
/// - `completions` - Generates shell completion scripts
///
/// # Error Handling
///
/// The function handles configuration loading errors gracefully and continues
/// execution. Individual command handlers are responsible for their own error
/// management and user feedback.
///
/// # Async Context
///
/// Runs in a Tokio async runtime to support asynchronous operations throughout
/// the application, particularly for HTTP requests to the Spotify API and
/// local web server operations during OAuth flows.
#[tokio::main]
async fn main() {
    if let Err(e) = config::load_env().await {
        error!("Cannot load environment. Err: {}", e);
    }

    let cli = Cli::parse();

    match cli.command {
        Command::Auth => {
            let oauth_result: Arc<Mutex<Option<PkceToken>>> = Arc::new(Mutex::new(None));
            cli::auth(Arc::clone(&oauth_result)).await;
        }
        Command::Artists(opt) => match opt.command {
            Some(ArtistsSubcommand::Update(u)) => cli::update_artists(u.force).await,
            None => cli::list_artists(opt.search).await,
        },

        Command::Releases(opt) => match opt.command {
            Some(ReleasesSubcommand::Update(u)) => {
                cli::update_releases(u.force, &u.release_types).await
            }
            None => cli::list_releases(opt.previous_weeks, opt.release_date).await,
        },

        Command::Playlist(opt) => {
            cli::playlist(opt.previous_weeks, opt.release_date, &opt.release_kinds).await
        }
        Command::Info(opt) => {
            cli::info(
                opt.release_week,
                opt.artists,
                opt.previous_weeks,
                opt.release_date,
            )
            .await
        }
        Command::Completions(opt) => {
            let mut cmd = Cli::command_for_update();
            let name = cmd.get_name().to_string();
            generate(opt.shell, &mut cmd, name, &mut std::io::stdout())
        }
    }
}
