use cask_db::disk_store::DiskStorage;

fn main() {
    let mut store = DiskStorage::new(None);
    store.init();
    store.set("hello", "world");
    store.set("name", "jojo");
    store.set("lol", "laugh");

    println!("name: {} ", if let Some(value) = store.get("name") { value } else { "not found".to_string() });
    println!("store: {:?}", store);
}
