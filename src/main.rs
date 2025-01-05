use cask_db::disk_store::DiskStorage;

fn main() {
    let mut store = DiskStorage::new(None);
    store.init().unwrap();
    // store.set("hello", "world");
    // store.set("name", "jojo");
    // store.set("lol", "laugh");
    // store.set("marco", "palito");
    // store.set("k", "jaegar");
    // store.set("kub", "boy");
    // store.set("kas", "omo");
    // store.set("firi", "baby");
    // store.set("hmm", "hmm");
    // store.set("key", "value");

    // store.set("mama", "awon boys");
    // store.set("bombo", "bimbo");
    // store.set("mum", "mums");
    // store.set("mum", "mums");
    // store.set("mum", "mums");
    store.delete("lol");
    // store.delete("marco");
    store.merge();

    println!(
        "name: {} ",
        if let Some(value) = store.get("name") {
            value
        } else {
            "not found".to_string()
        }
    );
    println!("store: {:?}", store);
}
