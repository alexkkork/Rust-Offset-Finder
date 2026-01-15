// Wed Jan 15 2026 - Alex

pub use super::args::Command;

use super::args::{GenerateArgs, DiffArgs, StatsArgs, ValidateArgs, DumpArgs};
use std::path::PathBuf;

pub trait CommandExecutor {
    fn execute(&self) -> anyhow::Result<()>;
    fn name(&self) -> &'static str;
}

impl CommandExecutor for GenerateArgs {
    fn execute(&self) -> anyhow::Result<()> {
        log::info!("Executing generate command");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "generate"
    }
}

impl CommandExecutor for DiffArgs {
    fn execute(&self) -> anyhow::Result<()> {
        log::info!("Executing diff command");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "diff"
    }
}

impl CommandExecutor for StatsArgs {
    fn execute(&self) -> anyhow::Result<()> {
        log::info!("Executing stats command");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "stats"
    }
}

impl CommandExecutor for ValidateArgs {
    fn execute(&self) -> anyhow::Result<()> {
        log::info!("Executing validate command");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "validate"
    }
}

impl CommandExecutor for DumpArgs {
    fn execute(&self) -> anyhow::Result<()> {
        log::info!("Executing dump command");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "dump"
    }
}
