use clap::{command, Parser, Subcommand};
use cmd::{
    clear_cache::clear_cache, current::current, exec::exec, install::install, list::list,
    shell::shell,
};
use semver_eq::VersionReq;
use std::{
    io::{self, Read},
    str::FromStr,
};
mod cmd;
use std::env;

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
    Current {
        #[arg(long, default_value = "false")]
        verbose: bool,
    },

    /// Clears the cache
    ClearCache {},

    /// Configure wasmenv for a specific shell (bash, zsh, fish)
    Shell {
        /// Specify a shell name, gives output for current shell if not specified
        name: Option<String>,
    },

    /// Install wasmer
    Use {
        /// Use a specific version. Install the latest version if not specified
        version: Option<VersionReq>,

        /// install pre-release
        #[arg(long, default_value = "false")]
        prerelease: bool,
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

    /// Run command with wasmer
    Exec {
        /// Filter versions based on semver
        #[arg(long, short)]
        use_version: Option<VersionReq>,

        /// wasmer command to run
        command: Vec<String>,

        /// install pre-release
        #[arg(long, default_value = "false")]
        prerelease: bool,
    },
}

fn get_version_from_stdin() -> Option<VersionReq> {
    if atty::is(atty::Stream::Stdin) {
        return None;
    }
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        return None;
    }
    let stripped_buffer = buffer.strip_suffix('\n')?;

    buffer = stripped_buffer.to_string();
    Some(VersionReq::from_str(&buffer).unwrap())
}

fn get_version_from_env() -> Option<VersionReq> {
    match env::var("WASMER_VERSION") {
        Ok(val) => Some(VersionReq::from_str(&val).unwrap()),
        Err(_) => None,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let stdin_version = get_version_from_stdin();
    let user_specified_version = if stdin_version.is_some() {
        stdin_version
    } else {
        get_version_from_env()
    };

    let command = cli.command;
    match command {
        Commands::Use {
            version,
            prerelease,
        } => {
            let version_to_use = if version.is_some() {
                version
            } else {
                user_specified_version
            };
            install(version_to_use, prerelease)
        }
        Commands::List {
            version,
            count,
            all,
        } => {
            let version_to_use = if version.is_some() {
                version
            } else {
                user_specified_version
            };
            list(version_to_use, count, all)
        }
        Commands::Current { verbose } => current(verbose),
        Commands::Shell { name } => shell(name),
        Commands::Exec {
            use_version,
            command,
            prerelease,
        } => exec(use_version, command, prerelease),
        Commands::ClearCache {} => clear_cache(),
    }
}
