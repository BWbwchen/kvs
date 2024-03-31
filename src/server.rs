//! Use `KvsEngine` object to perform the server functionality

use serde_json::Deserializer;
use std::{
    io::{BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use log::{error, info};

use crate::transport::{Request, ResponseGet, ResponseRemove, ResponseSet};
use crate::KvsEngine;
use crate::Result;

/// Struct for server object
pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    /// Create a server object with `KvsEngine`
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    /// Run this server object
    pub fn run<A: ToSocketAddrs>(mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(&addr)?;
        info!(
            "Start server and listen on: {}",
            addr.to_socket_addrs().unwrap().next().unwrap()
        );

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        error!("Serving client error: {e}");
                    }
                }
                Err(e) => error!("Connection Failed. {e}"),
            }
        }
        Ok(())
    }

    /// Private server functionality
    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        // Get/Parse the request
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let serde_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        macro_rules! send_response {
            ($response:expr) => {{
                let response = $response;
                serde_json::to_writer(&mut writer, &response)?;
                writer.flush()?;
            }};
        }

        for request in serde_reader {
            // Execute the command and Send the response
            let request = request?;
            info!("Got request from {}", peer_addr);
            match request {
                Request::Get { key } => send_response!(match self.engine.get(key) {
                    Ok(value) => ResponseGet::Ok(value),
                    Err(e) => ResponseGet::Err(format!("{e}")),
                }),
                Request::Set { key, value } => send_response!(match self.engine.set(key, value) {
                    Ok(()) => ResponseSet::Ok(()),
                    Err(e) => ResponseSet::Err(format!("{e}")),
                }),
                Request::Remove { key } => send_response!(match self.engine.remove(key) {
                    Ok(()) => ResponseRemove::Ok(()),
                    Err(e) => ResponseRemove::Err(format!("{e}")),
                }),
            }
        }
        Ok(())
    }
}
