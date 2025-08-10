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

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::White.on_default() | Effects::BOLD)
        .usage(AnsiColor::White.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightBlue.on_default())
        .placeholder(AnsiColor::BrightGreen.on_default())
}

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
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Authorize with Spotify API
    Auth,

    /// Handle followed artists
    Artists(ArtistsOptions),

    /// Handle releases
    Releases(ReleasesOptions),

    #[clap(about = "Create playlist for called weeks")]
    Playlist(PlaylistOptions),

    /// Some helper information about releases and artists
    Info(InfoOptions),

    /// Get shell completions
    Completions(CompletionsOption),
}

#[derive(Parser, Debug, Clone)]
#[command(
    about = "Handle followed artists",
    args_conflicts_with_subcommands = true // disallow mixing --search with subcommands
)]
pub struct ArtistsOptions {
    /// Search for artists
    #[clap(long)]
    pub search: Option<String>,

    /// Subcommands under `artists` (e.g., `update`)
    #[command(subcommand)]
    pub command: Option<ArtistsSubcommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ArtistsSubcommand {
    /// Update artists
    Update(ArtistsUpdateOpts),
}

#[derive(Parser, Debug, Clone)]
pub struct ArtistsUpdateOpts {
    /// Force update (skip caches/guards)
    #[clap(long)]
    pub force: bool,
}

#[derive(Parser, Debug, Clone)]
#[command(
    about = "Handle releases",
    args_conflicts_with_subcommands = true // disallow mixing query flags with `update`
)]
pub struct ReleasesOptions {
    /// Number of previous weeks to include
    #[clap(long)]
    pub previous_weeks: Option<u32>,

    /// Filter by a specific release date (YYYY-MM-DD)
    #[clap(long)]
    pub release_date: Option<String>,

    /// Subcommands under `releases` (e.g., `update`)
    #[command(subcommand)]
    pub command: Option<ReleasesSubcommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ReleasesSubcommand {
    /// Update releases
    Update(ReleasesUpdateOpts),
}

#[derive(Parser, Debug, Clone)]
pub struct ReleasesUpdateOpts {
    /// Force update (skip caches/guards)
    #[clap(long)]
    pub force: bool,

    /// Release type(s) to include during update; can be repeated
    #[clap(
        long = "type",
        default_value = "album",
        value_parser = utils::parse_release_kinds,
        action = ArgAction::Append,
        num_args = 1
    )]
    pub release_types: utils::ReleaseKinds,
}

#[derive(Parser, Debug, Clone)]
pub struct PlaylistOptions {
    #[clap(long)]
    previous_weeks: Option<u32>,
    #[clap(long)]
    release_date: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct InfoOptions {
    #[clap(long)]
    release_week: bool,
    #[clap(long)]
    artists: bool,
    #[clap(long)]
    previous_weeks: Option<u32>,
    #[clap(long)]
    release_date: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct CompletionsOption {
    shell: Shell,
}

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

        Command::Playlist(opt) => cli::playlist(opt.previous_weeks, opt.release_date).await,
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
