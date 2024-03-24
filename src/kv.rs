use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::fs::{create_dir, read_dir, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};
use uuid::Uuid;

use crate::{KvsError, Result, DEFAULT_LOG_FILE};

const MAX_LOG_UNCOMPACTED_BYTES: u64 = 1024 * 1024;

/// The KvStore store the key-value database.
///
/// The log-structed database system is implemented.
///
/// Example:
/// ```rust
/// # use kvs::{KvStore, Result};
///
/// # fn main() -> Result<()> {
/// let mut store = KvStore::open(kvs::DEFAULT_LOG_FILE)?;
/// store.set("key1".to_owned(), "value1".to_owned());
/// let value = store.get("key1".to_owned())?;
/// assert_eq!(value, Some("value1".to_owned()));
/// # Ok(())
/// # }
/// ```
pub struct KvStore {
    // log writer
    logger: Logger,
    /// Key -> the index of latest serialized Cmd
    index: HashMap<String, CmdIdx>,
    /// uncompacted size
    uncompacted: u64,
}

struct CmdIdx {
    start: usize,
    len: usize,
    cmd: Cmd, // cache the value here, If it is empty, it represend remove.
}
impl CmdIdx {
    fn new(start: usize, len: usize, cmd: Cmd) -> Self {
        Self { start, len, cmd }
    }
    fn default() -> Self {
        Self::new(0, 0, Cmd::Empty)
    }
}

/// The Command struct will represent an entry in the log
#[derive(Serialize, Deserialize, Debug, Clone)]
enum Cmd {
    Set { key: String, value: String },
    Remove { key: String },
    Empty,
}

impl KvStore {
    /// Set key `k` to value `v`
    pub fn set(&mut self, k: String, v: String) -> Result<()> {
        let cmd = Cmd::Set {
            key: k.to_owned(),
            value: v.clone(),
        };
        let pos = self.logger.pos;
        serde_json::to_writer(&mut self.logger, &cmd)?;
        self.logger.flush()?;
        let len = self.logger.pos - pos;
        self.index.insert(k.clone(), CmdIdx::new(pos, len, cmd));
        self.uncompacted += len as u64;

        if self.uncompacted > MAX_LOG_UNCOMPACTED_BYTES {
            self.compact()?;
        }
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
        let len = self.logger.pos - pos;
        self.index.insert(k, CmdIdx::new(pos, len, cmd));
        self.uncompacted += len as u64;

        if self.uncompacted > MAX_LOG_UNCOMPACTED_BYTES {
            self.compact()?;
        }
        Ok(())
    }
    /// Open KvStore
    /// `path` is the directory of the log
    pub fn open(path: impl Into<PathBuf> + std::marker::Copy) -> Result<Self> {
        let mut ret = KvStore {
            logger: Logger::new(path)?,
            index: HashMap::new(),
            uncompacted: 0,
        };

        // reconstruct the index
        let mut reader =
            Deserializer::from_reader(std::io::Read::by_ref(&mut ret.logger)).into_iter::<Cmd>();

        let mut read_pos = 0;
        // for cmd in reader {
        while let Some(cmd) = reader.next() {
            let len = reader.byte_offset() - read_pos;
            let mut cmd_idx = CmdIdx::new(read_pos, len, Cmd::Empty);
            match cmd? {
                Cmd::Set { key, value } => {
                    cmd_idx.cmd = Cmd::Set {
                        key: key.clone(),
                        value,
                    };
                    ret.index.insert(key.clone(), cmd_idx);
                }
                Cmd::Remove { key } => {
                    cmd_idx.cmd = Cmd::Remove { key: key.clone() };
                    ret.index.insert(key.clone(), cmd_idx);
                }
                Cmd::Empty => return Err(KvsError::UnexpectedCommandType.into()),
            }
            read_pos += len;
        }

        // NOTE: we will ONLY append the log!!
        ret.logger.pos = read_pos;

        Ok(ret)
    }

    fn compact(&mut self) -> Result<()> {
        // dump self.index into new file and reset the writer
        let old_log = self.logger.filename.clone();
        let new_log = old_log
            .parent()
            .ok_or(KvsError::UnexpectedCommandType)?
            .join(format!("{}.log", Uuid::new_v4().to_string()));
        self.logger.writer = BufWriter::new(get_file_handler(&new_log)?);

        let mut new_index: HashMap<String, CmdIdx> = HashMap::new();
        let mut pos = 0;
        for (k, cmdidx) in self.index.iter() {
            let cmd = &cmdidx.cmd;
            serde_json::to_writer(&mut self.logger, &cmd)?;
            self.logger.flush()?;
            let len = self.logger.pos - pos;
            new_index.insert(k.to_string(), CmdIdx::new(pos, len, cmd.clone()));
            pos += len;
        }
        // reset the reader to this new file
        self.logger.reader = BufReader::new(get_file_handler(&new_log)?);
        self.index = new_index;

        self.logger.pos = pos;
        self.uncompacted = 0;

        // remember to remove the file
        println!("About to remove: {}", old_log.to_str().unwrap());
        std::fs::remove_file(&old_log)?;
        Ok(())
    }
}

fn get_file_handler(path: impl Into<PathBuf>) -> Result<File> {
    Ok(OpenOptions::new()
        .write(true)
        .read(true)
        .append(true)
        .create(true)
        .open(path.into())?)
}

fn get_log_name(path: impl Into<PathBuf>) -> Result<PathBuf> {
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
        create_dir(&pathbuf)?;
    }
    Ok(file_name)
}

struct Logger {
    filename: PathBuf,
    writer: BufWriter<File>,
    reader: BufReader<File>,
    pos: usize,      // current curson in the file
    read_pos: usize, // for construct the cache
}
impl Logger {
    fn new(path: impl Into<PathBuf> + std::marker::Copy) -> Result<Self> {
        let filename = get_log_name(path)?;
        Ok(Logger {
            filename: filename.clone(),
            writer: BufWriter::new(get_file_handler(&filename)?),
            reader: BufReader::new(get_file_handler(&filename)?),
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
