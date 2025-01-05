use crate::format::{KeyEntry, KeyValue};
use crate::rb_trees::{RBNode, RBTree};
use std::{
    collections::VecDeque,
    error::Error,
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
    key_dir: RBTree<String, KeyEntry>,
    base_dir: String,
    tombstone: VecDeque<String>,
}

impl DiskStorage {
    const HEADER_SIZE: usize = 28;

    pub fn new(base_dir: Option<String>) -> Self {
        let base_dir = base_dir.unwrap_or("db".to_string());

        if !Path::new(&base_dir).exists() {
            std::fs::create_dir(&base_dir).unwrap();
        }

        let file_path = Path::new(&base_dir).join("0.db");
        let write_position = 0;
        let key_dir = RBTree::new();
        let tombstone = VecDeque::new();

        DiskStorage {
            file_id_counter: 1,
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
            tombstone,
        }
    }

    fn is_directory_empty(&self) -> std::io::Result<bool> {
        let mut entries = fs::read_dir(&self.base_dir)?;
        Ok(entries.next().is_none())
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_directory_empty()? {
            self.init_key_dir()?;

            let file_path =
                Path::new(&self.base_dir).join(format!("{}.db", self.file_id_counter - 1));

            self.file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open(&file_path)
                .unwrap();
        }
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let kv = KeyValue::new(timestamp, key.to_string(), value.to_string());
        let bytes = kv.to_bytes().unwrap();
        let total_size = bytes.len();

        if self.file.metadata().unwrap().len() > 100 {
            self.file_id_counter += 1;
            self.write_position = 0;

            let file_path =
                Path::new(&self.base_dir).join(format!("{}.db", self.file_id_counter - 1));

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
            self.file_id_counter - 1,
            timestamp,
            self.write_position,
            total_size,
        );
        self.key_dir.insert(key.to_string(), key_entry);
        self.write_position += total_size;
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match self.key_dir.find(&key.to_string()) {
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
        self.key_dir.delete(&key.to_string());
        self.tombstone.push_back(key.to_string());
    }

    pub fn merge(&mut self) {
        let mut file_paths = self.files_in_dir();
        if let Some(active_file) = file_paths.pop() {
            let active_file_path = Path::new(&active_file);
            self.file = OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
                .open(&active_file_path)
                .unwrap();
        }

        for path in file_paths {
            let file_path = Path::new(&path);
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&file_path)
                .unwrap();
            self.write_position = 0;

            loop {
                file.seek(SeekFrom::Start(self.write_position as u64))
                    .unwrap();
                let mut header_buf = [0u8; Self::HEADER_SIZE];
                file.read(&mut header_buf).unwrap();

                if header_buf == [0u8; Self::HEADER_SIZE] {
                    break;
                }

                let (_, _, key_size, value_size) = KeyValue::decode_header(&header_buf).unwrap();

                let mut key_buf = vec![0u8; key_size];
                file.read(&mut key_buf).unwrap();
                let key = String::from_utf8(key_buf).unwrap();

                let total_size = Self::HEADER_SIZE + key_size + value_size;

                if self.tombstone.contains(&key) {
                    file.seek(SeekFrom::Start(self.write_position as u64))
                        .unwrap();
                    let empty_buf = vec![0u8; total_size];
                    file.write(&empty_buf).unwrap();
                }

                self.write_position += total_size;
            }
        }
    }

    fn files_in_dir(&mut self) -> Vec<String> {
        let mut file_paths: Vec<String> = fs::read_dir(&self.base_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .map(|entry| entry.path().display().to_string())
            .collect();

        file_paths.sort();

        file_paths
    }

    fn init_key_dir(&mut self) -> Result<(), Box<dyn Error>> {
        println!("****----------initialising the database----------****");

        let file_paths = self.files_in_dir();
        self.file_id_counter = file_paths.len() as u32;

        for (id, file) in file_paths.iter().enumerate() {
            self.write_position = 0;
            self.file = File::open(&file)?;
            self.load_file(id as u32)?;
        }

        println!("****----------initialisation complete----------****");

        Ok(())
    }

    fn load_file(&mut self, id: u32) -> Result<(), Box<dyn Error>> {
        loop {
            let mut header_buf = [0u8; Self::HEADER_SIZE];
            self.file.read(&mut header_buf)?;
            if header_buf == [0u8; Self::HEADER_SIZE] {
                break;
            }

            let (_, _, key_size, value_size) = KeyValue::decode_header(&header_buf)?;

            let data_size = key_size + value_size;
            let total_size = Self::HEADER_SIZE + data_size;
            let mut data_buf = vec![0u8; data_size];
            self.file.read(&mut data_buf)?;

            let full_data = [header_buf.to_vec(), data_buf].concat();

            let kv = KeyValue::from_bytes(&full_data)?;

            let key_entry = KeyEntry::init(id, kv.timestamp, self.write_position, total_size);
            self.key_dir.insert(kv.key.clone(), key_entry);
            self.write_position += total_size;
            println!("loaded key: {}, value: {}", kv.key, kv.value);
        }

        Ok(())
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
