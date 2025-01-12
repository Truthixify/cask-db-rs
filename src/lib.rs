pub mod args;
pub mod commands;
pub mod disk_store;
mod format;
mod rb_trees;

pub type Error = Box<dyn std::error::Error>;
