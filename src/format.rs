use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct KeyEntry {
    timestamp: usize,
    pub position: usize,
    total_size: usize,
}

impl KeyEntry {
    pub fn init(timestamp: usize, position: usize, total_size: usize) -> Self {
        KeyEntry {
            timestamp,
            position,
            total_size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValue {
    timestamp: usize,
    key: String,
    value: String,
}

impl KeyValue {
    pub fn new(timestamp: usize, key: String, value: String) -> Self {
        KeyValue {
            timestamp,
            key,
            value,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        let bytes = bincode::serialize(self)?;

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        let kv = bincode::deserialize(bytes)?;

        Ok(kv)
    }

    pub fn encode_header(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let timestamp_bytes = usize::to_be_bytes(self.timestamp);
        let key_size_bytes = usize::to_be_bytes(self.key.len());
        let value_size_bytes = usize::to_be_bytes(self.value.len());

        bytes.extend(timestamp_bytes);
        bytes.extend(key_size_bytes);
        bytes.extend(value_size_bytes);

        bytes
    }

    pub fn decode_header(bytes: &[u8]) -> (usize, usize, usize) {
        let timestamp = usize::from_be_bytes(bytes[..8].try_into().unwrap());
        let key_size = usize::from_be_bytes(bytes[8..16].try_into().unwrap());
        let value_size = usize::from_be_bytes(bytes[16..24].try_into().unwrap());

        (timestamp, key_size, value_size)
    }
}