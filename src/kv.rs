use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::fs::{create_dir, read_dir, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};
use uuid::Uuid;

use crate::{KvsError, Result};

/// The KvStore store the key-value database.
///
/// The log-structed database system is implemented.
///
/// Example:
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key1".to_owned(), "value1".to_owned());
/// //let value = store.get("key1".to_owned());
/// //assert_eq!(value, Some("value1".to_owned()));
///
pub struct KvStore {
    // log writer
    logger: Logger,
    /// Key -> the index of latest serialized Cmd
    index: HashMap<String, CmdIdx>,
}

struct CmdIdx {
    start: usize,
    len: usize,
}
impl CmdIdx {
    fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// The Command struct will represent an entry in the log
#[derive(Serialize, Deserialize, Debug)]
enum Cmd {
    Set { key: String, value: String },
    Remove { key: String },
}

impl KvStore {
    /// Set key `k` to value `v`
    pub fn set(&mut self, k: String, v: String) -> Result<()> {
        let cmd = Cmd::Set {
            key: k.to_owned(),
            value: v,
        };
        let pos = self.logger.pos;
        serde_json::to_writer(&mut self.logger, &cmd)?;
        self.logger.flush()?;
        self.index
            .insert(k.clone(), CmdIdx::new(pos, self.logger.pos - pos));
        Ok(())
    }

    /// Get the value of key `k`
    pub fn get(&mut self, k: String) -> Result<Option<String>> {
        match self.index.get(&k) {
            Some(cmd_idx) => {
                // read from log
                self.logger
                    .seek(SeekFrom::Start(cmd_idx.start.try_into().unwrap()))?;
                let reader = self
                    .logger
                    .reader
                    .get_mut()
                    .take(cmd_idx.len.try_into().unwrap());
                if let Cmd::Set { value, .. } = serde_json::from_reader(reader)? {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            // None => Ok(Some(KvsError::KeyNotFound.to_string())),
            None => Ok(None),
        }
    }

    /// Remove the key `k`
    pub fn remove(&mut self, k: String) -> Result<()> {
        // Check whether key is exist.
        self.index.get(&k).ok_or(KvsError::KeyNotFound)?;

        // Construct a remove command
        let cmd = Cmd::Remove { key: k.to_owned() };
        let pos = self.logger.pos;
        serde_json::to_writer(&mut self.logger, &cmd)?;
        self.logger.flush()?;
        self.index
            .insert(k, CmdIdx::new(pos, self.logger.pos - pos));

        Ok(())
    }
    /// Open KvStore
    /// `path` is the directory of the log
    pub fn open(path: impl Into<PathBuf> + std::marker::Copy) -> Result<Self> {
        let mut ret = KvStore {
            logger: Logger::new(path)?,
            index: HashMap::new(),
        };

        // reconstruct the index
        let mut reader =
            Deserializer::from_reader(std::io::Read::by_ref(&mut ret.logger)).into_iter::<Cmd>();

        let mut read_pos = 0;
        // for cmd in reader {
        while let Some(cmd) = reader.next() {
            let len = reader.byte_offset() - read_pos;
            let cmd_idx = CmdIdx::new(read_pos, len);
            match cmd? {
                Cmd::Set { key, .. } => {
                    ret.index.insert(key.clone(), cmd_idx);
                }
                Cmd::Remove { key } => {
                    ret.index.insert(key.clone(), cmd_idx);
                }
            }
            read_pos += len;
        }

        // NOTE: we will ONLY append the log!!
        ret.logger.pos = read_pos;

        Ok(ret)
    }
}

fn get_file_handler(path: impl Into<PathBuf>) -> Result<File> {
    // If there was no log file in the `path` directory, create one with uuid file name.
    // Else we reuse the existing file.
    let pathbuf = path.into();
    let default_pathbuf = pathbuf.join(format!("{}.log", Uuid::new_v4().to_string()));
    let file_name = match read_dir(pathbuf.as_path()).into_iter().next() {
        Some(dir) => {
            let mut ret = default_pathbuf.clone();
            for file in dir.into_iter() {
                match file {
                    Ok(file_path) => {
                        if file_path.path().as_path().extension() == Some("log".as_ref())
                            && ret == default_pathbuf
                        {
                            ret = file_path.path();
                        }
                    }
                    _ => continue,
                };
            }
            ret
        }
        None => default_pathbuf,
    };

    if read_dir(&pathbuf).is_err() {
        create_dir(&pathbuf)?
    }

    Ok(OpenOptions::new()
        .write(true)
        .read(true)
        .append(true)
        .create(true)
        .open(file_name)?)
}

struct Logger {
    writer: BufWriter<File>,
    reader: BufReader<File>,
    pos: usize,      // current curson in the file
    read_pos: usize, // for construct the cache
}
impl Logger {
    fn new(path: impl Into<PathBuf> + std::marker::Copy) -> Result<Self> {
        Ok(Logger {
            writer: BufWriter::new(get_file_handler(path)?),
            reader: BufReader::new(get_file_handler(path)?),
            pos: 0,
            read_pos: 0,
        })
    }
}
// NOTE: Why we need to implement Write trait?
// Since the serde_json will NOT return the number of bytes it write,
// we need to hijack the process in the middle to collect the length of data.
impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let nbytes = self.writer.write(buf)?;
        self.pos += nbytes;
        Ok(nbytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
impl Seek for Logger {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}
impl Read for Logger {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let nbytes = self.reader.read(buf)?;
        self.read_pos += nbytes;
        Ok(nbytes)
    }
}
