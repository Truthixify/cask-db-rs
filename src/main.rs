use cask_db::disk_store::DiskStorage;

fn main() {
    let mut store = DiskStorage::new(None);
    store.init();
    // store.set("hello", "world");
    // store.set("name", "jojo");
    // store.set("lol", "laugh");
    // store.set("marco", "palito");
    // store.set("k", "jaegar");
    // store.delete("eren");
    // store.set("bastard", "boy");
    // store.set("calm", "omo");
    // store.set("firi", "baby");
    // store.set("hmm", "hmm");

    println!(
        "name: {} ",
        if let Some(value) = store.get("firi") {
            value
        } else {
            "not found".to_string()
        }
    );
    println!("store: {:?}", store);
}
