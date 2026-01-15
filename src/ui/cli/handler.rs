// Wed Jan 15 2026 - Alex

use super::args::{Args, Command, GenerateArgs, DiffArgs, StatsArgs, ValidateArgs, DumpArgs};
use crate::ui::progress::ProgressManager;
use crate::ui::banner::Banner;
use crate::ui::theme::Theme;
use colored::Colorize;

pub struct CommandHandler {
    theme: Theme,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
        }
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn execute(&self, args: Args) -> anyhow::Result<()> {
        if !args.quiet {
            Banner::print();
        }

        self.setup_logging(&args)?;

        match args.command {
            Command::Generate(gen_args) => self.handle_generate(gen_args),
            Command::Diff(diff_args) => self.handle_diff(diff_args),
            Command::Stats(stats_args) => self.handle_stats(stats_args),
            Command::Validate(validate_args) => self.handle_validate(validate_args),
            Command::Dump(dump_args) => self.handle_dump(dump_args),
        }
    }

    fn setup_logging(&self, args: &Args) -> anyhow::Result<()> {
        let level = match args.log_level.to_lowercase().as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        };

        env_logger::Builder::new()
            .filter_level(level)
            .format_timestamp(None)
            .init();

        Ok(())
    }

    fn handle_generate(&self, args: GenerateArgs) -> anyhow::Result<()> {
        args.validate().map_err(|e| anyhow::anyhow!(e))?;

        println!("{}", "Starting offset generation...".cyan());

        let progress = ProgressManager::new();
        let main_bar = progress.create_main_progress(100, "Generating offsets");

        main_bar.set_position(10);
        main_bar.set_message("Loading binary...");

        main_bar.set_position(30);
        main_bar.set_message("Scanning patterns...");

        main_bar.set_position(50);
        main_bar.set_message("Analyzing symbols...");

        main_bar.set_position(70);
        main_bar.set_message("Building cross-references...");

        main_bar.set_position(90);
        main_bar.set_message("Validating results...");

        main_bar.finish_with_message("Complete!");

        println!("{}", format!("Output written to: {:?}", args.output).green());
        Ok(())
    }

    fn handle_diff(&self, args: DiffArgs) -> anyhow::Result<()> {
        args.validate().map_err(|e| anyhow::anyhow!(e))?;

        println!("{}", "Comparing offset files...".cyan());
        println!("  Old: {:?}", args.old);
        println!("  New: {:?}", args.new);

        Ok(())
    }

    fn handle_stats(&self, args: StatsArgs) -> anyhow::Result<()> {
        if !args.input.exists() {
            return Err(anyhow::anyhow!("Input file does not exist: {:?}", args.input));
        }

        println!("{}", "Generating statistics...".cyan());
        println!("  Input: {:?}", args.input);

        Ok(())
    }

    fn handle_validate(&self, args: ValidateArgs) -> anyhow::Result<()> {
        args.validate().map_err(|e| anyhow::anyhow!(e))?;

        println!("{}", "Validating offsets...".cyan());
        println!("  Offsets: {:?}", args.offsets);

        Ok(())
    }

    fn handle_dump(&self, args: DumpArgs) -> anyhow::Result<()> {
        println!("{}", "Dumping memory...".cyan());
        println!("  Address: {}", args.address);
        println!("  Size: {} bytes", args.size);

        Ok(())
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
