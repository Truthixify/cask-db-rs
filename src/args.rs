use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Init(InitArgs),
    Get(GetArgs),
    Set(SetArgs),
    Delete(DeleteArgs),
    Merge(MergeArgs),
}

#[derive(Parser)]
pub struct CreateArgs {
    pub base_dir: Option<String>,
}

#[derive(Parser)]
pub struct InitArgs {
    pub base_dir: Option<String>,
}

#[derive(Parser)]
pub struct GetArgs {
    pub key: String,
    pub base_dir: Option<String>,
}

#[derive(Parser)]
pub struct SetArgs {
    pub key: String,
    pub value: String,
    pub base_dir: Option<String>,
}

#[derive(Parser)]
pub struct DeleteArgs {
    pub key: String,
    pub base_dir: Option<String>,
}

#[derive(Parser)]
pub struct MergeArgs {
    pub base_dir: Option<String>,
}
