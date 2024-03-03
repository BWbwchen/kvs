//! A simple Key-Value database

#![deny(missing_docs)]
use std::collections::HashMap;

/// The KvStore store the key-value database.
///
/// Currently, KvStore store key-value pair in memory and doesn't persistent the content to the disk.
///
/// Example:
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key1".to_owned(), "value1".to_owned());
/// let value = store.get("key1".to_owned());
/// assert_eq!(value, Some("value1".to_owned()));
/// ```
pub struct KvStore {
    storage: HashMap<String, String>,
}

impl KvStore {
    /// Create a KvStore
    pub fn new() -> Self {
        KvStore {
            storage: HashMap::new(),
        }
    }
    /// Set key `k` to value `v`
    pub fn set(&mut self, k: String, v: String) {
        self.storage.insert(k, v);
    }
    /// Get the value of key `k`
    pub fn get(&mut self, k: String) -> Option<String> {
        self.storage.get(&k).cloned()
    }
    /// Remove the key `k`
    pub fn remove(&mut self, k: String) {
        self.storage.remove(&k);
    }
}
