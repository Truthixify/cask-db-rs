pub mod disk_store;
mod format;
mod rb_trees;
pub mod args;
pub mod commands;

pub type Error = Box<dyn std::error::Error>;