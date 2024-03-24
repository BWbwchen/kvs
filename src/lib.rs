#![deny(missing_docs)]
//! A simple Key-Value database

pub use error::{KvsError, Result};
pub use kv::KvStore;

/// default log file path
pub static DEFAULT_LOG_FILE: &str = "./";

mod error;
mod kv;
