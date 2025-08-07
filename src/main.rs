use std::sync::Arc;

use clap::{
    CommandFactory, Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_complete::{Shell, generate};

use sporlcli::{cli, config, types::PkceToken};
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

#[derive(Parser, Debug, Clone)]
enum Command {
    #[clap(about = "Authorize with Spotify API")]
    Auth,
    #[clap(about = "Handle followed artists")]
    Artists(ArtistsOptions),
    #[clap(about = "Handle releases")]
    Releases(ReleaseOptions),
    #[clap(about = "Create playlist for called weeks")]
    Playlist(PlaylistOptions),
    #[clap(about = "Some helper information about releases and artists")]
    Info(InfoOptions),
    #[clap(about = "Get shell completions")]
    Completions(CompletionsOption),
}

#[derive(Parser, Debug, Clone)]
struct ArtistsOptions {
    #[clap(long)]
    update: bool,
    #[clap(long)]
    search: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct ReleaseOptions {
    #[clap(long)]
    previous_weeks: Option<u32>,
    #[clap(long)]
    release_date: Option<String>,
    #[clap(long)]
    update: bool,
    #[clap(long)]
    force: bool,
}

#[derive(Parser, Debug, Clone)]
struct PlaylistOptions {
    #[clap(long)]
    previous_weeks: Option<u32>,
    #[clap(long)]
    release_date: Option<String>,
}

#[derive(Parser, Debug, Clone)]
struct InfoOptions {
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
struct CompletionsOption {
    shell: Shell,
}

#[tokio::main]
async fn main() {
    config::load_env();

    let cli = Cli::parse();

    match cli.command {
        Command::Auth => {
            let oauth_result: Arc<Mutex<Option<PkceToken>>> = Arc::new(Mutex::new(None));
            cli::auth(Arc::clone(&oauth_result)).await;
        }
        Command::Artists(opt) => cli::artists(opt.update, opt.search).await,
        Command::Releases(opt) => {
            cli::releases(opt.update, opt.force, opt.previous_weeks, opt.release_date).await
        }
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
