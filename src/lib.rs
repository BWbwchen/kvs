#![deny(missing_docs)]
//! A simple Key-Value database

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
pub use server::KvsServer;

/// default log file path
pub static DEFAULT_LOG_FILE: &str = "./";

mod client;
mod engines;
mod error;
mod server;
mod transport;
