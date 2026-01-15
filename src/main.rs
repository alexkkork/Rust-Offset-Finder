// Tue Jan 13 2026 - Alex

use clap::Parser;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use roblox_offset_generator::{
    config::Config,
    memory::{Address, BinaryMemory, MemoryReader},
    finders::{AllFinders, CombinedResults},
    ui::banner::Banner,
};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author = "Alex")]
#[command(version = "1.0.0")]
#[command(about = "Roblox Offset Generator for ARM64", long_about = None)]
struct Args {
    #[arg(short, long)]
    binary: PathBuf,

    #[arg(short, long, default_value = "offsets.json")]
    output: PathBuf,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long)]
    no_progress: bool,

    #[arg(long)]
    no_banner: bool,

    #[arg(long)]
    text_output: Option<PathBuf>,

    #[arg(long)]
    markdown_output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if !args.no_banner {
        Banner::print();
    }

    println!("{}", "Roblox Offset Generator for ARM64".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    println!();

    let start_time = Instant::now();

    println!("{} Loading binary: {}", "[*]".blue(), args.binary.display());

    let binary = match BinaryMemory::load(&args.binary) {
        Ok(b) => Arc::new(b) as Arc<dyn MemoryReader>,
        Err(e) => {
            eprintln!("{} Failed to load binary: {}", "[!]".red(), e);
            std::process::exit(1);
        }
    };

    println!("{} Binary loaded successfully", "[+]".green());

    let regions = match binary.get_regions() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} Failed to get memory regions: {}", "[!]".red(), e);
            std::process::exit(1);
        }
    };

    println!("{} Found {} memory regions", "[+]".green(), regions.len());

    let (start_addr, end_addr) = calculate_scan_range(&regions);

    println!("{} Scan range: 0x{:x} - 0x{:x}", "[*]".blue(), start_addr.as_u64(), end_addr.as_u64());
    println!();

    let multi_progress = MultiProgress::new();

    let main_progress = if !args.no_progress {
        let pb = multi_progress.add(ProgressBar::new(100));
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% {msg}")
            .unwrap()
            .progress_chars("#>-"));
        pb.set_message("Initializing...");
        Some(pb)
    } else {
        None
    };

    println!("{} Starting offset discovery...", "[*]".blue());
    println!();

    if let Some(ref pb) = main_progress {
        pb.set_message("Finding Roblox functions...");
        pb.set_position(10);
    }

    let finders = AllFinders::new(binary.clone());

    if let Some(ref pb) = main_progress {
        pb.set_message("Scanning for patterns...");
        pb.set_position(30);
    }

    let results = finders.find_all(start_addr, end_addr);

    if let Some(ref pb) = main_progress {
        pb.set_message("Analyzing results...");
        pb.set_position(80);
    }

    print_results_summary(&results);

    if let Some(ref pb) = main_progress {
        pb.set_message("Saving results...");
        pb.set_position(90);
    }

    if let Err(e) = save_results(&results, &args.output) {
        eprintln!("{} Failed to save results: {}", "[!]".red(), e);
        std::process::exit(1);
    }

    println!("{} Results saved to: {}", "[+]".green(), args.output.display());

    if let Some(text_path) = &args.text_output {
        if let Err(e) = save_text_report(&results, text_path) {
            eprintln!("{} Failed to save text report: {}", "[!]".red(), e);
        } else {
            println!("{} Text report saved to: {}", "[+]".green(), text_path.display());
        }
    }

    if let Some(md_path) = &args.markdown_output {
        if let Err(e) = save_markdown_report(&results, md_path) {
            eprintln!("{} Failed to save markdown report: {}", "[!]".red(), e);
        } else {
            println!("{} Markdown report saved to: {}", "[+]".green(), md_path.display());
        }
    }

    if let Some(ref pb) = main_progress {
        pb.set_position(100);
        pb.finish_with_message("Complete!");
    }

    let elapsed = start_time.elapsed();

    println!();
    println!("{}", "=".repeat(50).cyan());
    println!("{} Offset discovery complete in {:.2}s", "[+]".green(), elapsed.as_secs_f64());
    println!("{} Total offsets found: {}", "[+]".green(), results.total_count());
    println!("{} High confidence: {}", "[+]".green(), results.high_confidence_count());
}

fn calculate_scan_range(regions: &[roblox_offset_generator::memory::MemoryRegion]) -> (Address, Address) {
    let mut min_addr = u64::MAX;
    let mut max_addr = 0u64;

    for region in regions {
        let start = region.range.start.as_u64();
        let end = start + region.range.size;

        if start < min_addr {
            min_addr = start;
        }
        if end > max_addr {
            max_addr = end;
        }
    }

    (Address::new(min_addr), Address::new(max_addr))
}

fn print_results_summary(results: &CombinedResults) {
    println!("{}", "Results Summary".cyan().bold());
    println!("{}", "-".repeat(40).cyan());

    println!("  Functions found: {}", results.functions.len().to_string().green());
    println!("  Structure offsets found: {}", results.structure_offsets.len().to_string().green());
    println!("  Classes found: {}", results.classes.len().to_string().green());
    println!("  Properties found: {}", results.properties.len().to_string().green());
    println!("  Methods found: {}", results.methods.len().to_string().green());
    println!("  Constants found: {}", results.constants.len().to_string().green());

    println!();

    if !results.functions.is_empty() {
        println!("{}", "Functions:".yellow().bold());
        for func in &results.functions {
            let confidence_color = if func.confidence >= 0.85 {
                "green"
            } else if func.confidence >= 0.65 {
                "yellow"
            } else {
                "red"
            };

            let confidence_str = format!("{:.0}%", func.confidence * 100.0);
            println!("  {} 0x{:016x} [{}]",
                func.name.cyan(),
                func.address.as_u64(),
                match confidence_color {
                    "green" => confidence_str.green(),
                    "yellow" => confidence_str.yellow(),
                    _ => confidence_str.red(),
                }
            );
        }
        println!();
    }

    if !results.structure_offsets.is_empty() {
        println!("{}", "Structure Offsets:".yellow().bold());

        let mut by_struct: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
        for offset in &results.structure_offsets {
            by_struct.entry(&offset.structure_name).or_default().push(offset);
        }

        for (struct_name, offsets) in by_struct {
            println!("  {}:", struct_name.cyan());
            for offset in offsets {
                println!("    {}: 0x{:x}", offset.field_name, offset.offset);
            }
        }
        println!();
    }
}

fn save_results(results: &CombinedResults, path: &PathBuf) -> Result<(), std::io::Error> {
    let json_map = results.to_json_map();
    let json_string = serde_json::to_string_pretty(&json_map)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut file = File::create(path)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}

fn save_text_report(results: &CombinedResults, path: &PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;

    writeln!(file, "Roblox Offset Report")?;
    writeln!(file, "====================")?;
    writeln!(file)?;

    writeln!(file, "Functions ({}):", results.functions.len())?;
    writeln!(file, "-----------------")?;
    for func in &results.functions {
        writeln!(file, "  {}: 0x{:016x} [{:.0}%] ({})",
            func.name, func.address.as_u64(), func.confidence * 100.0, func.method)?;
    }
    writeln!(file)?;

    writeln!(file, "Structure Offsets ({}):", results.structure_offsets.len())?;
    writeln!(file, "-----------------------")?;
    for offset in &results.structure_offsets {
        writeln!(file, "  {}.{}: 0x{:x} [{:.0}%]",
            offset.structure_name, offset.field_name, offset.offset, offset.confidence * 100.0)?;
    }
    writeln!(file)?;

    writeln!(file, "Classes ({}):", results.classes.len())?;
    writeln!(file, "-------------")?;
    for class in &results.classes {
        writeln!(file, "  {}: 0x{:016x}", class.name, class.address.as_u64())?;
    }
    writeln!(file)?;

    Ok(())
}

fn save_markdown_report(results: &CombinedResults, path: &PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;

    writeln!(file, "# Roblox Offset Report")?;
    writeln!(file)?;
    writeln!(file, "## Summary")?;
    writeln!(file)?;
    writeln!(file, "| Category | Count |")?;
    writeln!(file, "|----------|-------|")?;
    writeln!(file, "| Functions | {} |", results.functions.len())?;
    writeln!(file, "| Structure Offsets | {} |", results.structure_offsets.len())?;
    writeln!(file, "| Classes | {} |", results.classes.len())?;
    writeln!(file, "| Properties | {} |", results.properties.len())?;
    writeln!(file, "| Methods | {} |", results.methods.len())?;
    writeln!(file, "| Constants | {} |", results.constants.len())?;
    writeln!(file)?;

    writeln!(file, "## Functions")?;
    writeln!(file)?;
    writeln!(file, "| Name | Address | Confidence | Method |")?;
    writeln!(file, "|------|---------|------------|--------|")?;
    for func in &results.functions {
        writeln!(file, "| {} | `0x{:016x}` | {:.0}% | {} |",
            func.name, func.address.as_u64(), func.confidence * 100.0, func.method)?;
    }
    writeln!(file)?;

    writeln!(file, "## Structure Offsets")?;
    writeln!(file)?;
    writeln!(file, "| Structure | Field | Offset | Confidence |")?;
    writeln!(file, "|-----------|-------|--------|------------|")?;
    for offset in &results.structure_offsets {
        writeln!(file, "| {} | {} | `0x{:x}` | {:.0}% |",
            offset.structure_name, offset.field_name, offset.offset, offset.confidence * 100.0)?;
    }
    writeln!(file)?;

    writeln!(file, "## Classes")?;
    writeln!(file)?;
    writeln!(file, "| Name | Address | Parent |")?;
    writeln!(file, "|------|---------|--------|")?;
    for class in &results.classes {
        writeln!(file, "| {} | `0x{:016x}` | {} |",
            class.name, class.address.as_u64(),
            class.parent_class.as_deref().unwrap_or("-"))?;
    }
    writeln!(file)?;

    Ok(())
}
