pub mod api;
pub mod cli;
pub mod config;
pub mod management;
pub mod server;
pub mod spotify;
pub mod types;
pub mod utils;

pub type Res<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "o".blue().bold(), std::format_args!($($arg)*));
  })
}

#[macro_export]
macro_rules! success {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "✓".green().bold(), std::format_args!($($arg)*));
  })
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "!".red().bold(), std::format_args!($($arg)*));
    std::process::exit(1);
  })
}

#[macro_export]
macro_rules! warning {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "!".yellow().bold(), std::format_args!($($arg)*));
  })
}
