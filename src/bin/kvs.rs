use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Set the value of a string key to a string
    Set {
        /// key
        #[arg(value_name = "KEY")]
        k: String,
        /// value
        #[arg(value_name = "VALUE")]
        v: String,
    },
    /// Get the string value of a given string key
    Get {
        /// key
        #[arg(value_name = "KEY")]
        k: String,
    },
    /// Remove a given key
    Rm {
        /// key
        #[arg(value_name = "KEY")]
        k: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Set { k, v }) => {
            unimplemented!("unimplemented")
        }
        Some(Commands::Get { k }) => {
            unimplemented!("unimplemented")
        }
        Some(Commands::Rm { k }) => {
            unimplemented!("unimplemented")
        }
        None => {
            panic!("Only accept set, get, rm")
        }
    }
}
