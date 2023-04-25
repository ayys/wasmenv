use clap::{command, Parser, Subcommand};
use cmd::{install::install, list::list, current::current, shell::shell};
use semver::VersionReq;

mod cmd;
mod utils;
use anyhow::Result;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display the currently active version of wasmer
    Current {},

    /// Configure wasmerenv for a specific shell (bash, zsh, fish)
    Shell {
        /// Specify a shell name, gives output for current shell if not specified
        name: Option<String>,
    },

    /// Install wasmer
    Use {
        /// Use a specific version. Install the latest version if not specified
        version: Option<VersionReq>,
    },

    /// List all the available versions of wasmer
    List {
        /// Filter versions based on semver
        version: Option<VersionReq>,

        /// Limit the number of versions to show
        #[arg(long, short, default_value = "5")]
        count: Option<usize>,

        #[arg(long, short, default_value = "false")]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = cli.command;
    match command {
        Commands::Use { version } => install(version),
        Commands::List {
            version,
            count,
            all,
        } => list(version, count, all),
        Commands::Current {} => current(),
        Commands::Shell { name } => shell(name),
    }
}
