use cask_db::args;
use cask_db::{commands, Error};
use clap::Parser;

fn main() -> Result<(), Error> {
    match args::Cli::parse().command {
        args::Commands::Create(create_args) => commands::create(create_args),
        args::Commands::Init(init_args) => commands::init(init_args),
        args::Commands::Get(get_args) => commands::get(get_args),
        args::Commands::Set(set_args) => commands::set(set_args),
        args::Commands::Delete(delete_args) => commands::delete(delete_args),
        args::Commands::Merge(merge_args) => commands::merge(merge_args),
    }
}
