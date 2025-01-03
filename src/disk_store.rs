use crate::format::{KeyEntry, KeyValue};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct DiskStorage {
    file_id_counter: u32,
    file: File,
    write_position: usize,
    key_dir: HashMap<String, KeyEntry>,
    base_dir: String,
}

impl DiskStorage {
    const HEADER_SIZE: usize = 28;
    const TOMBSTONE: &str = "TOMBSTONE";

    pub fn new(base_dir: Option<String>) -> Self {
        let base_dir = base_dir.unwrap_or("db".to_string());
        if !Path::new(&base_dir).exists() {
            std::fs::create_dir(&base_dir).unwrap();
        }

        let file_path = Path::new(&base_dir).join("0.db");
        let write_position = 0;
        let key_dir: HashMap<String, KeyEntry> = HashMap::new();

        DiskStorage {
            file_id_counter: 0,
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open(&file_path)
                .unwrap(),
            write_position,
            key_dir,
            base_dir,
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

        let file_path = Path::new(&self.base_dir).join(format!("{}.db", self.file_id_counter));

        if fs::metadata(file_path).unwrap().len() > 100 {
            self.file_id_counter += 1;
            self.write_position = 0;

            let file_path = Path::new(&self.base_dir).join(format!("{}.db", self.file_id_counter));

            self.file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open(&file_path)
                .unwrap();
        }

        self.file.write(&bytes).unwrap();

        let key_entry = KeyEntry::init(
            self.file_id_counter,
            timestamp,
            self.write_position,
            total_size,
        );
        self.key_dir.insert(key.to_string(), key_entry);
        self.write_position += total_size;
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match self.key_dir.get(key) {
            Some(key_entry) => {
                let file_path = Path::new(&self.base_dir).join(format!("{}.db", key_entry.file_id));
                let mut file = File::open(file_path).unwrap();
                file.seek(SeekFrom::Start(key_entry.position as u64))
                    .unwrap();

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

    pub fn delete(&mut self, key: &str) {
        self.set(key, Self::TOMBSTONE);
    }

    pub fn merge(&mut self) {}

    fn init_key_dir(&mut self) {
        println!("****----------initialising the database----------****");
        let mut file_paths: Vec<String> = fs::read_dir(&self.base_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .map(|entry| entry.path().display().to_string())
            .collect();

        file_paths.sort();

        for file in file_paths {
            let file_path = Path::new(&file).file_name().unwrap().to_str().unwrap();
            self.write_position = 0;
            self.file_id_counter = file_path.replace(".db", "").parse::<u32>().unwrap();
            self.file = File::open(&file).unwrap();
            self.load_file();
        }

        println!("****----------initialisation complete----------****")
    }

    fn load_file(&mut self) {
        loop {
            let mut header_buf = [0u8; Self::HEADER_SIZE];
            self.file.read(&mut header_buf).unwrap();
            if header_buf[0] == 0 {
                break;
            }

            let (_, _, key_size, value_size) = KeyValue::decode_header(&header_buf);

            let data_size = key_size + value_size;
            let total_size = Self::HEADER_SIZE + data_size;
            let mut data_buf = vec![0u8; data_size];
            self.file.read(&mut data_buf).unwrap();

            let full_data = [header_buf.to_vec(), data_buf].concat();

            let kv = KeyValue::from_bytes(&full_data).unwrap();

            let key_entry = KeyEntry::init(
                self.file_id_counter,
                kv.timestamp,
                self.write_position,
                total_size,
            );
            self.key_dir.insert(kv.key.clone(), key_entry);
            self.write_position += total_size;
            println!("loaded key: {}, value: {}", kv.key, kv.value);
        }
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

fn list_files_in_directory(dir_path: &str) -> std::io::Result<Vec<String>> {
    let mut file_paths = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            file_paths.push(entry.path().display().to_string());
        }
    }
    Ok(file_paths)
}
