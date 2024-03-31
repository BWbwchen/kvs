use std::{ env::current_dir, net::SocketAddr};

use clap::{Args, Parser, Subcommand, ValueEnum};
use kvs::{KvStore, KvsClient, KvsEngine, KvsServer, Result};
use log::{info, LevelFilter};

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        /// Start the server and begin listening for incoming connections.
        #[arg(
            long, 
            value_name = "Server Address",
            default_value = DEFAULT_LISTENING_ADDRESS,
        )]
        addr: SocketAddr,
    },
    /// Get the string value of a given string key
    Get {
        /// key
        #[arg(value_name = "KEY")]
        k: String,
        /// Start the server and begin listening for incoming connections.
        #[arg(
            long, 
            value_name = "Server Address",
            default_value = DEFAULT_LISTENING_ADDRESS,
        )]
        addr: SocketAddr,
    },
    /// Remove a given key
    Rm {
        /// key
        #[arg(value_name = "KEY")]
        k: String,
        /// Start the server and begin listening for incoming connections.
        #[arg(
            long, 
            value_name = "Server Address",
            default_value = DEFAULT_LISTENING_ADDRESS,
        )]
        addr: SocketAddr,
    },
}

#[derive(ValueEnum, Clone, Default, Debug)]
enum EngineEnum {
    /// kvs
    #[default]
    Kvs,

    /// sled
    Sled,
}

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let cli = Cli::parse();

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    // info!("Storage engine:  {}", cli.addr);

    // let mut client = KvsClient::connect(cli.addr)?;

    match &cli.command {
        Commands::Set { k, v , addr} => KvsClient::connect(addr)?.set(k.to_owned(), v.to_owned())?,
        Commands::Get { k , addr} => {
            if let Some(v) = KvsClient::connect(addr)?.get(k.to_owned())? {
                println!("{}", v);
            } else {
                println!("Key not found");
            }
        }
        Commands::Rm { k, addr } => KvsClient::connect(addr)?.remove(k.to_owned())?,
    };
    Ok(())
}
