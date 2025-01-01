use cask_db::disk_store::DiskStorage;

fn main() {
    let mut store = DiskStorage::new(None);
    store.init();
    store.set("hello".to_string(), "world".to_string());
    store.set("name".to_string(), "jojo".to_string());
    store.set("lol".to_string(), "laugh".to_string());

    println!("name: {} ", if let Some(value) = store.get("name") { value } else { "not found".to_string() });
}
