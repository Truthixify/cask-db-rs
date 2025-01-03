#[derive(Debug)]
pub struct KeyEntry {
    pub file_id: u32,
    timestamp: usize,
    pub position: usize,
    pub total_size: usize,
}

impl KeyEntry {
    pub fn init(file_id: u32, timestamp: usize, position: usize, total_size: usize) -> Self {
        KeyEntry {
            file_id,
            timestamp,
            position,
            total_size,
        }
    }
}

pub struct KeyValue {
    pub crc: u32,
    pub timestamp: usize,
    pub key: String,
    pub value: String,
}

impl KeyValue {
    pub fn new(timestamp: usize, key: String, value: String) -> Self {
        let mut bytes = vec![];

        let timestamp_bytes = timestamp.to_be_bytes();
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();

        bytes.extend(&timestamp_bytes);
        bytes.extend(key_bytes);
        bytes.extend(value_bytes);

        let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(&bytes);

        KeyValue {
            crc,
            timestamp,
            key,
            value,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        let mut bytes = vec![];
        bytes.extend(self.crc.to_be_bytes());
        bytes.extend(self.timestamp.to_be_bytes());
        bytes.extend(self.key.len().to_be_bytes());
        bytes.extend(self.value.len().to_be_bytes());
        bytes.extend(self.key.as_bytes());
        bytes.extend(self.value.as_bytes());

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        let crc = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let timestamp = usize::from_be_bytes(bytes[4..12].try_into().unwrap());
        let key_size = usize::from_be_bytes(bytes[12..20].try_into().unwrap());
        let value_size = usize::from_be_bytes(bytes[20..28].try_into().unwrap());
        let key = String::from_utf8(bytes[28..28 + key_size].to_vec()).unwrap();
        let value =
            String::from_utf8(bytes[28 + key_size..28 + key_size + value_size].to_vec()).unwrap();

        Ok(KeyValue {
            crc,
            timestamp,
            key,
            value,
        })
    }

    pub fn encode_header(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let crc_bytes = u32::to_be_bytes(self.crc);
        let timestamp_bytes = usize::to_be_bytes(self.timestamp);
        let key_size_bytes = usize::to_be_bytes(self.key.len());
        let value_size_bytes = usize::to_be_bytes(self.value.len());

        bytes.extend(crc_bytes);
        bytes.extend(timestamp_bytes);
        bytes.extend(key_size_bytes);
        bytes.extend(value_size_bytes);

        bytes
    }

    pub fn decode_header(bytes: &[u8]) -> (u32, usize, usize, usize) {
        let crc = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let timestamp = usize::from_be_bytes(bytes[4..12].try_into().unwrap());
        let key_size = usize::from_be_bytes(bytes[12..20].try_into().unwrap());
        let value_size = usize::from_be_bytes(bytes[20..28].try_into().unwrap());

        (crc, timestamp, key_size, value_size)
    }
}
