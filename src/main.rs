use cask_db::disk_store::DiskStorage;

fn main() {
    let mut store = DiskStorage::new(None);
    store.init();
    store.set("hello", "world");
    store.set("name", "jojo");
    store.set("lol", "laugh");
    store.set("eren", "jaegar");

    println!(
        "name: {} ",
        if let Some(value) = store.get("eren") {
            value
        } else {
            "not found".to_string()
        }
    );
    println!("store: {:?}", store);
}
