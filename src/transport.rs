//! Transport Layer interface

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Serialize, Deserialize)]
pub enum ResponseGet {
    Ok(Option<String>),
    Err(String),
}

#[derive(Serialize, Deserialize)]
pub enum ResponseSet {
    Ok(()),
    Err(String),
}

#[derive(Serialize, Deserialize)]
pub enum ResponseRemove {
    Ok(()),
    Err(String),
}
