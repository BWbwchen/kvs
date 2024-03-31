//! Client object aims to
//! 1. Deserialize the response from server.
//! 2. Send serialized request to server.

use crate::{
    transport::{Request, ResponseGet, ResponseRemove, ResponseSet},
    KvsError, Result,
};
use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use std::{
    io::{BufReader, BufWriter, Write},
    net::{TcpStream, ToSocketAddrs},
};

/// Struct for client
pub struct KvsClient {
    sender: BufWriter<TcpStream>,
    receiver: Deserializer<IoRead<BufReader<TcpStream>>>,
}

impl KvsClient {
    /// Establish the connection to server and return a `KvsClient` object.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let tcp_sender = TcpStream::connect(addr)?;
        let tcp_receiver = tcp_sender.try_clone()?;
        Ok(Self {
            sender: BufWriter::new(tcp_sender),
            receiver: Deserializer::from_reader(BufReader::new(tcp_receiver)),
        })
    }
    /// request `get`
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.sender, &Request::Get { key })?;
        self.sender.flush()?;
        let response = ResponseGet::deserialize(&mut self.receiver)?;
        match response {
            ResponseGet::Ok(value) => Ok(value),
            ResponseGet::Err(e) => Err(KvsError::StringError(e).into()),
        }
    }
    /// request `set`
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.sender, &Request::Set { key, value })?;
        self.sender.flush()?;
        let response = ResponseSet::deserialize(&mut self.receiver)?;
        match response {
            ResponseSet::Ok(()) => Ok(()),
            ResponseSet::Err(e) => Err(KvsError::StringError(e).into()),
        }
    }
    /// request `remove`
    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self.sender, &Request::Remove { key })?;
        self.sender.flush()?;
        let response = ResponseRemove::deserialize(&mut self.receiver)?;
        match response {
            ResponseRemove::Ok(()) => Ok(()),
            ResponseRemove::Err(e) => Err(KvsError::StringError(e).into()),
        }
    }
}
