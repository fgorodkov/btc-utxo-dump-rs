use db_key::Key;
use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::options::Options;
use leveldb::options::ReadOptions;
use std::path::Path;

pub struct ChainStateDB(Database<ChainStateKey>);

impl ChainStateDB {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let mut options = Options::new();
        options.create_if_missing = false;

        let db = Database::open(path, options)
            .map_err(|e| anyhow::anyhow!("Failed to open database at {}: {}", path.display(), e))?;

        Ok(Self(db))
    }
    pub fn iter(&self) -> impl Iterator<Item = (ChainStateKey, Vec<u8>)> + '_ {
        let read_options = ReadOptions::new();
        self.0.iter(read_options)
    }
}

#[derive(Debug)]
pub struct ChainStateKey(Vec<u8>);

impl ChainStateKey {
    pub fn first_byte(&self) -> u8 {
        self.0[0]
    }
}
impl Key for ChainStateKey {
    fn from_u8(key: &[u8]) -> Self {
        Self(key.to_vec())
    }
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.0)
    }
}

#[derive(Debug)]
pub struct ChainStateValue(Vec<u8>);

impl ChainStateValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn deobfuscate(&self, key: &[u8]) -> Vec<u8> {
        self.0
            .iter()
            .zip(key.iter().cycle())
            .map(|(a, b)| a ^ b)
            .collect()
    }
}
