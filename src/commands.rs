use crate::args::{CreateArgs, DeleteArgs, GetArgs, InitArgs, MergeArgs, SetArgs};
use crate::{disk_store::DiskStorage, Error};

pub fn create(args: CreateArgs) -> Result<(), Error> {
    DiskStorage::new(args.base_dir);

    Ok(())
}

pub fn init(args: InitArgs) -> Result<(), Error> {
    let mut store = DiskStorage::new(args.base_dir);
    store.init()?;

    Ok(())
}

pub fn get(args: GetArgs) -> Result<(), Error> {
    let mut store = DiskStorage::new(args.base_dir);
    store.init()?;
    store.get(&args.key);

    Ok(())
}

pub fn set(args: SetArgs) -> Result<(), Error> {
    let mut store = DiskStorage::new(args.base_dir);
    store.init()?;
    store.set(&args.key, &args.value);

    Ok(())
}

pub fn delete(args: DeleteArgs) -> Result<(), Error> {
    let mut store = DiskStorage::new(args.base_dir);
    store.init()?;
    store.delete(&args.key);

    Ok(())
}

pub fn merge(args: MergeArgs) -> Result<(), Error> {
    let mut store = DiskStorage::new(args.base_dir);
    store.init()?;
    store.merge()?;

    Ok(())
}