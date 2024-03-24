use clap::{Parser, Subcommand};
use kvs::{KvStore, KvsError, Result};
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut store = KvStore::open(kvs::DEFAULT_LOG_FILE)?;

    match &cli.command {
        Some(Commands::Set { k, v }) => store.set(k.to_owned(), v.to_owned())?,
        Some(Commands::Get { k }) => {
            if let Some(v) = store.get(k.to_owned())? {
                println!("{}", v);
            } else {
                println!("Key not found");
            }
        }
        Some(Commands::Rm { k }) => store.remove(k.to_owned())?,
        _ => unreachable!(),
    };
    Ok(())
}
