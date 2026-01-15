// Wed Jan 15 2026 - Alex

pub mod args;
pub mod commands;
pub mod handler;

pub use args::{Args, GenerateArgs, DiffArgs, StatsArgs};
pub use commands::Command;
pub use handler::CommandHandler;

use clap::Parser;

pub fn parse_args() -> Args {
    Args::parse()
}

pub fn run() -> anyhow::Result<()> {
    let args = parse_args();
    let handler = CommandHandler::new();
    handler.execute(args)
}
