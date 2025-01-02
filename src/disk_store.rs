use crate::format::{KeyEntry, KeyValue};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct DiskStorage {
    file_name: String,
    file: File,
    write_position: usize,
    key_dir: HashMap<String, KeyEntry>,
}

impl DiskStorage {
    const HEADER_SIZE: usize = 28;
    pub fn new(file_name: Option<String>) -> Self {
        let file_name = file_name.unwrap_or("data.db".to_string());
        let write_position = 0;
        let key_dir: HashMap<String, KeyEntry> = HashMap::new();

        DiskStorage {
            file_name: file_name.clone(),
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open(&file_name)
                .unwrap(),
            write_position,
            key_dir,
        }
    }

    fn is_open_file_empty(&self) -> bool {
        self.file
            .metadata()
            .map(|metadata| metadata.len() == 0)
            .unwrap_or(false)
    }

    pub fn init(&mut self) {
        if !self.is_open_file_empty() {
            self.init_key_dir();
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let kv = KeyValue::new(timestamp, key.to_string(), value.to_string());
        let bytes = kv.to_bytes().unwrap();
        let header = kv.encode_header();
        let total_size = header.len() + bytes.len();

        self.file.write(&bytes).unwrap();

        let key_entry = KeyEntry::init(timestamp, self.write_position, total_size);
        self.key_dir.insert(key.to_string(), key_entry);
        self.write_position += total_size;
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match self.key_dir.get(key) {
            Some(key_entry) => {
                let mut file = File::open(&self.file_name).unwrap();
                file.seek(SeekFrom::Start(key_entry.position as u64))
                    .unwrap();

                // let mut v = vec![0u8; 400];
                // file.read(&mut v).unwrap();
                // println!("v: {:?}", v);

                let mut data_buf = vec![0u8; key_entry.total_size];
                file.read(&mut data_buf).unwrap();

                let kv = KeyValue::from_bytes(&data_buf).unwrap();

                let mut bytes = vec![];

                let timestamp_bytes = kv.timestamp.to_be_bytes();
                let key_bytes = kv.key.as_bytes();
                let value_bytes = kv.value.as_bytes();

                bytes.extend(&timestamp_bytes);
                bytes.extend(key_bytes);
                bytes.extend(value_bytes);

                let checksum = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(&bytes);

                if kv.crc == checksum {
                    Some(kv.value)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn init_key_dir(&mut self) {
        let mut file = File::open(&self.file_name).unwrap();

        println!("****----------initialising the database----------****");
        loop {
            let mut header_buf = [0u8; Self::HEADER_SIZE];
            file.read(&mut header_buf).unwrap();
            if header_buf[0] == 0 {
                break;
            }

            let (_, _, key_size, value_size) = KeyValue::decode_header(&header_buf);

            let data_size = key_size + value_size;
            let total_size = Self::HEADER_SIZE + data_size;
            let mut data_buf = vec![0u8; data_size];
            file.read(&mut data_buf).unwrap();

            let full_data = [header_buf.to_vec(), data_buf].concat();

            let kv = KeyValue::from_bytes(&full_data).unwrap();

            let key_entry = KeyEntry::init(kv.timestamp, self.write_position, total_size);
            self.key_dir.insert(kv.key.clone(), key_entry);
            self.write_position += total_size;
            println!("loaded key: {}, value: {}", kv.key, kv.value);
        }
        println!("****----------initialisation complete----------****")
    }
}

impl Write for DiskStorage {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
