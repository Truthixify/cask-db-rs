use std::{collections::HashMap, fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::Path};
use crate::format::{KeyEntry, KeyValue};


#[derive(Debug)]
pub struct DiskStorage {
    file_name: String,
    file: File,
    write_position: usize,
    key_dir: HashMap<String, KeyEntry>,
}

impl DiskStorage {
    const HEADER_SIZE: usize = 24;
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
                .open(&file_name)
                .unwrap(),
            write_position,
            key_dir,
        }
    }
    pub fn init(&mut self) {
        let file_name = self.file_name.clone();

        if Path::new(&file_name).exists() {
            self.init_key_dir();
        } else {
            self.file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&file_name)
                .unwrap();
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        let timestamp = 0;
        let kv = KeyValue::new(timestamp, key.clone(), value);
        let bytes = kv.to_bytes().unwrap();
        let header = kv.encode_header();
        let total_size = header.len() + bytes.len();

        self.file.write(&header).unwrap();
        self.file.write(&bytes).unwrap();

        let key_entry = KeyEntry::init(timestamp, self.write_position, total_size);
        self.key_dir.insert(key, key_entry);
        self.write_position += total_size;
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match self.key_dir.get(key) {
            Some(key_entry) => {
                let mut file = File::open(&self.file_name).unwrap();
                file.seek(SeekFrom::Start(key_entry.position as u64)).unwrap();

                let mut header_buf = [0u8; Self::HEADER_SIZE];
                file.read(&mut header_buf).unwrap();
                let (_, _, value_size) = KeyValue::decode_header(&header_buf);

                let mut value_buf = vec![0u8; value_size];
                file.read(&mut value_buf).unwrap();
                let value = String::from_utf8(value_buf).unwrap();

                Some(value)
            }
            None => None,
        }
    }

    fn init_key_dir(&mut self) {
        let mut file = File::open(&self.file_name).unwrap();
        let position = 0;
        
        loop {
            let mut header_buf = [0u8; Self::HEADER_SIZE];
            file.read(&mut header_buf).unwrap();
            if header_buf[0] == 0 {
                break;
            }

            let (timestamp, key_size, value_size) = KeyValue::decode_header(&header_buf);

            let mut key_buf = vec![0u8; key_size];
            file.read(&mut key_buf).unwrap();
            let key = String::from_utf8(key_buf).unwrap();

            let mut value_buf = vec![0u8; value_size];
            file.read(&mut value_buf).unwrap();
            let value = String::from_utf8(value_buf).unwrap();

            let total_size = Self::HEADER_SIZE + key_size + value_size;

            let key_entry = KeyEntry::init(timestamp, position, total_size);
            self.key_dir.insert(key.clone(), key_entry);
            self.write_position += total_size;  
            println!("loaded key: {}, value: {}", key, value);
        }
    }

    pub fn close(&mut self) {
        self.file.flush().unwrap();
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