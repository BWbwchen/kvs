use std::{ env::current_dir, net::SocketAddr, process::exit};

use clap::{Args, Parser, Subcommand, ValueEnum};
use kvs::{KvStore, KvsEngine, KvsServer, Result, SledKvsEngine};
use log::{info, error, LevelFilter};

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    /// Start the server and begin listening for incoming connections.
    #[arg(
        long, 
        value_name = "Server Address",
        default_value = DEFAULT_LISTENING_ADDRESS,
    )]
    addr: SocketAddr,

    /// The engine that kvs used.
    #[arg(
        long,
        value_name = "Engine",
        value_enum,
        default_value_t = EngineEnum::Kvs,
    )]
    engine: EngineEnum,
}

#[derive(ValueEnum, Clone, Default, Debug, PartialEq, Eq)]
enum EngineEnum {
    /// kvs
    #[default]
    Kvs,

    /// sled
    Sled,
}

impl ToString for EngineEnum {
    fn to_string(&self) -> String {
        match self {
            EngineEnum::Kvs => "kvs".to_owned(), 
            EngineEnum::Sled => "sled".to_owned(), 
        }
    }
}

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let cli = Cli::parse();

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Listening on:  {}", cli.addr);
    info!("Storage engine:  {:?}", cli.engine);

    let full_path = current_dir()?.join(cli.engine.to_string());
    let other_engine = if cli.engine == EngineEnum::Kvs {
        EngineEnum::Sled
    } else {
        EngineEnum::Kvs
    };
    let other_path = current_dir()?.join(other_engine.to_string());
    if !full_path.exists() && other_path.exists() {
        // error
        error!("Wrong engine!");
        exit(1);
    }

    match cli.engine {
        EngineEnum::Kvs => {
            info!("Start kvs server");
            KvsServer::new(KvStore::open(full_path)?).run(cli.addr)?;
        },
        EngineEnum::Sled => {
            KvsServer::new(SledKvsEngine::new(sled::open(full_path)?)).run(cli.addr)?;
        }
    }

    Ok(())
}
