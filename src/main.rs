// Tue Jan 13 2026 - Alex

#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_imports)]

use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use roblox_offset_generator::{
    config::Config,
    memory::{Address, BinaryMemory, MemoryReader},
    finders::{AllFinders, CombinedResults, RobloxFinders},
    finders::{structures, classes, properties, methods, constants},
    finders::fflags::{FFlagFinder, FFlagDatabase, KnownFlag, get_database},
    ui::banner::Banner,
};
use std::fs::File;
use std::io::{Write, BufRead};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::fmt::Write as FmtWrite;

#[derive(Parser, Debug)]
#[command(name = "roblox-offset-generator")]
#[command(author = "Alex")]
#[command(version = "1.0.0")]
#[command(about = "ARM64 Roblox Offset Finder for macOS")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Suppress banner
    #[arg(long, global = true)]
    no_banner: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Disable progress bars
    #[arg(long, global = true)]
    no_progress: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Find all offsets from a Roblox binary
    Scan {
        /// Path to Roblox binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Output JSON file path
        #[arg(short, long, default_value = "offsets.json")]
        output: PathBuf,

        /// Also output as text file
        #[arg(long)]
        text: Option<PathBuf>,

        /// Also output as markdown file
        #[arg(long)]
        markdown: Option<PathBuf>,

        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        min_confidence: f64,

        /// Number of threads to use
        #[arg(short, long, default_value = "8")]
        threads: usize,
    },

    /// Dump FFlags from binary
    Fflags {
        /// Path to Roblox binary
        #[arg(short, long)]
        binary: Option<PathBuf>,

        /// Output file path
        #[arg(short, long, default_value = "fflags.json")]
        output: PathBuf,

        /// Also output as text file
        #[arg(long)]
        text: Option<PathBuf>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Search for specific flag name
        #[arg(long)]
        search: Option<String>,

        /// Show only flags found in binary
        #[arg(long)]
        found_only: bool,

        /// List known flag categories
        #[arg(long)]
        list_categories: bool,
    },

    /// Compare two offset files
    Diff {
        /// Old offsets file
        #[arg(short, long)]
        old: PathBuf,

        /// New offsets file
        #[arg(short, long)]
        new: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate offsets against a binary
    Validate {
        /// Offsets file to validate
        #[arg(short, long)]
        offsets: PathBuf,

        /// Path to Roblox binary
        #[arg(short, long)]
        binary: PathBuf,
    },

    /// Dump memory at address
    Dump {
        /// Path to Roblox binary
        #[arg(short, long)]
        binary: PathBuf,

        /// Address to dump (hex)
        #[arg(short, long)]
        address: String,

        /// Number of bytes to dump
        #[arg(short, long, default_value = "256")]
        size: usize,

        /// Disassemble instead of hex dump
        #[arg(long)]
        disasm: bool,
    },

    /// Show statistics about offset file
    Stats {
        /// Offsets file
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    if !cli.no_banner {
        print_banner();
    }

    // If no command provided, show interactive menu
    let result = match &cli.command {
        None => run_interactive_menu(&cli),
        Some(Commands::Scan { binary, output, text, markdown, min_confidence, threads }) => {
            run_scan(&cli, binary.clone(), output.clone(), text.clone(), markdown.clone(), *min_confidence, *threads)
        }
        Some(Commands::Fflags { binary, output, text, category, search, found_only, list_categories }) => {
            run_fflags(&cli, binary.clone(), output.clone(), text.clone(), category.clone(), search.clone(), *found_only, *list_categories)
        }
        Some(Commands::Diff { old, new, output }) => {
            run_diff(&cli, old.clone(), new.clone(), output.clone())
        }
        Some(Commands::Validate { offsets, binary }) => {
            run_validate(&cli, offsets.clone(), binary.clone())
        }
        Some(Commands::Dump { binary, address, size, disasm }) => {
            run_dump(&cli, binary.clone(), address.clone(), *size, *disasm)
        }
        Some(Commands::Stats { input }) => {
            run_stats(&cli, input.clone())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }
}

fn print_banner() {
    println!();
    println!("{}", r#"  ____       _     _              ____  __  __          _   "#.cyan());
    println!("{}", r#" |  _ \ ___ | |__ | | _____  __  / __ \/ _|/ _|___  ___| |_ "#.cyan());
    println!("{}", r#" | |_) / _ \| '_ \| |/ _ \ \/ / | |  | | |_| |_/ __|/ _ \ __|"#.cyan());
    println!("{}", r#" |  _ < (_) | |_) | | (_) >  <  | |__| |  _|  _\__ \  __/ |_ "#.cyan());
    println!("{}", r#" |_| \_\___/|_.__/|_|\___/_/\_\  \____/|_| |_| |___/\___|\__|"#.cyan());
    println!();
    println!("{}", "        ARM64 Offset Finder for macOS v1.0.0".bright_black());
    println!();
}

// ==================== INTERACTIVE MENU ====================

fn run_interactive_menu(cli: &Cli) -> Result<(), String> {
    // Check if stdin is a terminal (interactive)
    if !atty::is(atty::Stream::Stdin) {
        println!("{}", "Not running in interactive mode. Use --help for command line options.".yellow());
        print_help();
        return Ok(());
    }

    loop {
        println!("{}", "═".repeat(55).cyan());
        println!("{}", "                    MAIN MENU".cyan().bold());
        println!("{}", "═".repeat(55).cyan());
        println!();
        println!("  {} {} - Scan binary for all offsets", "[1]".green().bold(), "Full Scan".white().bold());
        println!("  {} {} - Dump FFlags from binary", "[2]".green().bold(), "FFlag Dump".white().bold());
        println!("  {} {} - List all known FFlag categories", "[3]".green().bold(), "FFlag Categories".white().bold());
        println!("  {} {} - Dump memory at address", "[4]".green().bold(), "Memory Dump".white().bold());
        println!("  {} {} - Compare two offset files", "[5]".green().bold(), "Diff Offsets".white().bold());
        println!("  {} {} - Validate offsets against binary", "[6]".green().bold(), "Validate".white().bold());
        println!("  {} {} - View offset file statistics", "[7]".green().bold(), "Statistics".white().bold());
        println!("  {} {} - Show help and usage info", "[8]".green().bold(), "Help".white().bold());
        println!("  {} {} - Exit program", "[0]".red().bold(), "Exit".white().bold());
        println!();
        print!("{}", "  Enter choice: ".yellow());
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached
                println!();
                println!("{}", "Goodbye!".cyan());
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }
        let choice = input.trim();

        println!();

        match choice {
            "1" => {
                if let Err(e) = menu_full_scan(cli) {
                    eprintln!("{} {}", "[ERROR]".red(), e);
                }
            }
            "2" => {
                if let Err(e) = menu_fflag_dump(cli) {
                    eprintln!("{} {}", "[ERROR]".red(), e);
                }
            }
            "3" => {
                menu_fflag_categories();
            }
            "4" => {
                if let Err(e) = menu_memory_dump(cli) {
                    eprintln!("{} {}", "[ERROR]".red(), e);
                }
            }
            "5" => {
                if let Err(e) = menu_diff(cli) {
                    eprintln!("{} {}", "[ERROR]".red(), e);
                }
            }
            "6" => {
                if let Err(e) = menu_validate(cli) {
                    eprintln!("{} {}", "[ERROR]".red(), e);
                }
            }
            "7" => {
                if let Err(e) = menu_stats(cli) {
                    eprintln!("{} {}", "[ERROR]".red(), e);
                }
            }
            "8" => {
                print_help();
            }
            "0" | "q" | "exit" | "quit" => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            _ => {
                println!("{} Invalid choice. Please enter 0-8.", "[!]".yellow());
            }
        }
        println!();
    }
    Ok(())
}

fn prompt(msg: &str) -> String {
    print!("{}", msg.yellow());
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_path(msg: &str) -> Option<PathBuf> {
    let input = prompt(msg);
    if input.is_empty() {
        None
    } else {
        Some(PathBuf::from(input))
    }
}

fn menu_full_scan(cli: &Cli) -> Result<(), String> {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "              FULL OFFSET SCAN".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    let binary = prompt_path("  Enter path to Roblox binary: ")
        .ok_or("Binary path is required")?;

    if !binary.exists() {
        return Err(format!("File not found: {}", binary.display()));
    }

    let output_str = prompt("  Output file [offsets.json]: ");
    let output = if output_str.is_empty() {
        PathBuf::from("offsets.json")
    } else {
        PathBuf::from(output_str)
    };

    let conf_str = prompt("  Minimum confidence (0.0-1.0) [0.7]: ");
    let min_confidence: f64 = if conf_str.is_empty() {
        0.7
    } else {
        conf_str.parse().unwrap_or(0.7)
    };

    println!();
    run_scan(cli, binary, output, None, None, min_confidence, 8)
}

fn menu_fflag_dump(cli: &Cli) -> Result<(), String> {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "               FFLAG DUMP".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    let binary = prompt_path("  Enter path to Roblox binary: ")
        .ok_or("Binary path is required")?;

    if !binary.exists() {
        return Err(format!("File not found: {}", binary.display()));
    }

    let output_str = prompt("  Output file [fflags.json]: ");
    let output = if output_str.is_empty() {
        PathBuf::from("fflags.json")
    } else {
        PathBuf::from(output_str)
    };

    let category = prompt("  Filter by category (or press Enter for all): ");
    let category = if category.is_empty() { None } else { Some(category) };

    let search = prompt("  Search string (or press Enter for none): ");
    let search = if search.is_empty() { None } else { Some(search) };

    let found_only_str = prompt("  Show only found flags? (y/N): ");
    let found_only = found_only_str.to_lowercase() == "y";

    println!();
    run_fflags(cli, Some(binary), output, None, category, search, found_only, false)
}

fn menu_fflag_categories() {
    let db = get_database();
    let mut categories: Vec<_> = db.categories();
    categories.sort();

    println!("{}", "═".repeat(55).cyan());
    println!("{}", "            FFLAG CATEGORIES".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    for (i, cat) in categories.iter().enumerate() {
        let count = db.flags_in_category(cat).len();
        println!("  {:2}. {} {}", 
            (i + 1).to_string().bright_black(),
            format!("{:<25}", cat).yellow(),
            format!("({} flags)", count).bright_black()
        );
    }

    println!();
    println!("  {} Total categories: {}", "★".yellow(), categories.len().to_string().green().bold());
    println!("  {} Total known flags: {}", "★".yellow(), db.count().to_string().green().bold());
}

fn menu_memory_dump(cli: &Cli) -> Result<(), String> {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "              MEMORY DUMP".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    let binary = prompt_path("  Enter path to binary: ")
        .ok_or("Binary path is required")?;

    if !binary.exists() {
        return Err(format!("File not found: {}", binary.display()));
    }

    let address = prompt("  Address to dump (e.g., 0x100000): ");
    if address.is_empty() {
        return Err("Address is required".to_string());
    }

    let size_str = prompt("  Bytes to dump [256]: ");
    let size: usize = if size_str.is_empty() {
        256
    } else {
        size_str.parse().unwrap_or(256)
    };

    let disasm_str = prompt("  Disassemble? (y/N): ");
    let disasm = disasm_str.to_lowercase() == "y";

    println!();
    run_dump(cli, binary, address, size, disasm)
}

fn menu_diff(cli: &Cli) -> Result<(), String> {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "              OFFSET DIFF".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    let old = prompt_path("  Enter path to OLD offsets file: ")
        .ok_or("Old file path is required")?;
    let new = prompt_path("  Enter path to NEW offsets file: ")
        .ok_or("New file path is required")?;

    println!();
    run_diff(cli, old, new, None)
}

fn menu_validate(cli: &Cli) -> Result<(), String> {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "            VALIDATE OFFSETS".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    let offsets = prompt_path("  Enter path to offsets file: ")
        .ok_or("Offsets file is required")?;
    let binary = prompt_path("  Enter path to binary: ")
        .ok_or("Binary path is required")?;

    println!();
    run_validate(cli, offsets, binary)
}

fn menu_stats(cli: &Cli) -> Result<(), String> {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "            OFFSET STATISTICS".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();

    let input = prompt_path("  Enter path to offsets file: ")
        .ok_or("File path is required")?;

    println!();
    run_stats(cli, input)
}

fn print_help() {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "                   HELP".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();
    println!("{}", "COMMAND LINE USAGE:".yellow().bold());
    println!();
    println!("  {} {}", "./roblox-offset-generator".green(), "                  # Interactive menu");
    println!("  {} {}", "./roblox-offset-generator scan -b <binary>".green(), "  # Full scan");
    println!("  {} {}", "./roblox-offset-generator fflags -b <binary>".green(), " # FFlag dump");
    println!("  {} {}", "./roblox-offset-generator fflags --list-categories".green(), "");
    println!("  {} {}", "./roblox-offset-generator dump -b <binary> -a 0x1000".green(), "");
    println!("  {} {}", "./roblox-offset-generator diff -o old.json -n new.json".green(), "");
    println!();
    println!("{}", "SCAN OPTIONS:".yellow().bold());
    println!("  {:<20} {}", "-b, --binary", "Path to Roblox binary");
    println!("  {:<20} {}", "-o, --output", "Output JSON file (default: offsets.json)");
    println!("  {:<20} {}", "--text", "Also save as text file");
    println!("  {:<20} {}", "--markdown", "Also save as markdown file");
    println!("  {:<20} {}", "--min-confidence", "Minimum confidence threshold (0.0-1.0)");
    println!();
    println!("{}", "FFLAG OPTIONS:".yellow().bold());
    println!("  {:<20} {}", "-b, --binary", "Path to Roblox binary");
    println!("  {:<20} {}", "--list-categories", "List all known flag categories");
    println!("  {:<20} {}", "--category", "Filter by category name");
    println!("  {:<20} {}", "--search", "Search for flag by name");
    println!("  {:<20} {}", "--found-only", "Only show flags found in binary");
    println!();
    println!("{}", "GLOBAL OPTIONS:".yellow().bold());
    println!("  {:<20} {}", "--no-banner", "Hide the banner");
    println!("  {:<20} {}", "--no-progress", "Disable progress bars");
    println!("  {:<20} {}", "--no-color", "Disable colored output");
    println!("  {:<20} {}", "-v, --verbose", "Verbose output");
    println!();
}

fn create_progress_bar(total: u64, msg: &str, no_progress: bool) -> Option<ProgressBar> {
    if no_progress {
        return None;
    }
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}"
        )
        .unwrap()
        .progress_chars("█▓▒░  ")
    );
    pb.set_message(msg.to_string());
    Some(pb)
}

fn create_spinner(msg: &str, no_progress: bool) -> Option<ProgressBar> {
    if no_progress {
        return None;
    }
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    Some(pb)
}

// ==================== SCAN COMMAND ====================

fn run_scan(
    cli: &Cli,
    binary: PathBuf,
    output: PathBuf,
    text: Option<PathBuf>,
    markdown: Option<PathBuf>,
    min_confidence: f64,
    _threads: usize,
) -> Result<(), String> {
    let start_time = Instant::now();

    println!("{} {}", "[*]".blue(), "Loading binary...".white());
    
    let spinner = create_spinner("Loading binary...", cli.no_progress);
    
    let binary_mem = BinaryMemory::load(&binary)
        .map_err(|e| format!("Failed to load binary: {}", e))?;
    let reader: Arc<dyn MemoryReader> = Arc::new(binary_mem);

    if let Some(ref pb) = spinner {
        pb.finish_with_message("Binary loaded!");
    }

    println!("{} Binary loaded: {}", "[+]".green(), binary.display());

    let regions = reader.get_regions()
        .map_err(|e| format!("Failed to get memory regions: {}", e))?;
    
    println!("{} Found {} memory regions", "[+]".green(), regions.len());

    // Find executable regions only (where code lives)
    let exec_regions: Vec<_> = regions.iter()
        .filter(|r| r.protection().can_execute())
        .collect();

    if exec_regions.is_empty() {
        return Err("No executable regions found in binary".to_string());
    }

    println!("{} Found {} executable regions", "[+]".green(), exec_regions.len());

    // Use first executable region for scanning (typically __TEXT)
    let first_exec = &exec_regions[0];
    let start_addr = first_exec.range().start();
    let scan_size = first_exec.range().size().min(100_000_000); // Cap at 100MB for speed
    let end_addr = Address::new(start_addr.as_u64() + scan_size);

    println!("{} Scan range: {} - {} ({} MB)",
        "[*]".blue(),
        format!("0x{:x}", start_addr.as_u64()).yellow(),
        format!("0x{:x}", end_addr.as_u64()).yellow(),
        scan_size / 1024 / 1024
    );
    println!();

    let mut results = CombinedResults::new();

    // Phase 1: Roblox Functions
    println!("{} Phase 1/6: Scanning for Roblox functions...", "[*]".blue());
    let spinner1 = create_spinner("Scanning Roblox functions...", cli.no_progress);
    
    let roblox_finders = RobloxFinders::new(reader.clone());
    for result in roblox_finders.find_all(start_addr, end_addr) {
        results.add_function(result);
    }
    
    if let Some(ref pb) = spinner1 { pb.finish_with_message(format!("Found {} functions", results.functions.len())); }
    println!("{} Found {} Roblox functions", "[+]".green(), results.functions.len());

    // Phase 2: Structures
    println!("{} Phase 2/6: Scanning for structures...", "[*]".blue());
    let spinner2 = create_spinner("Scanning structures...", cli.no_progress);
    
    let structure_results = structures::find_all_structures(reader.clone(), start_addr, end_addr);
    for result in structure_results {
        results.add_structure_offset(result);
    }
    
    if let Some(ref pb) = spinner2 { pb.finish_with_message(format!("Found {} structure offsets", results.structure_offsets.len())); }
    println!("{} Found {} structure offsets", "[+]".green(), results.structure_offsets.len());

    // Phase 3: Classes
    println!("{} Phase 3/6: Scanning for classes...", "[*]".blue());
    let spinner3 = create_spinner("Scanning classes...", cli.no_progress);
    
    let class_results = classes::find_all_classes(reader.clone(), start_addr, end_addr);
    for result in class_results {
        results.add_class(result);
    }
    
    if let Some(ref pb) = spinner3 { pb.finish_with_message(format!("Found {} classes", results.classes.len())); }
    println!("{} Found {} classes", "[+]".green(), results.classes.len());

    // Phase 4: Properties  
    println!("{} Phase 4/6: Scanning for properties...", "[*]".blue());
    let spinner4 = create_spinner("Scanning properties...", cli.no_progress);
    
    let property_results = properties::find_all_properties(reader.clone(), start_addr, end_addr);
    for result in property_results {
        results.add_property(result);
    }
    
    if let Some(ref pb) = spinner4 { pb.finish_with_message(format!("Found {} properties", results.properties.len())); }
    println!("{} Found {} properties", "[+]".green(), results.properties.len());

    // Phase 5: Methods
    println!("{} Phase 5/6: Scanning for methods...", "[*]".blue());
    let spinner5 = create_spinner("Scanning methods...", cli.no_progress);
    
    let method_results = methods::find_all_methods(reader.clone(), start_addr, end_addr);
    for result in method_results {
        results.add_method(result);
    }
    
    if let Some(ref pb) = spinner5 { pb.finish_with_message(format!("Found {} methods", results.methods.len())); }
    println!("{} Found {} methods", "[+]".green(), results.methods.len());

    // Phase 6: Constants
    println!("{} Phase 6/6: Scanning for constants...", "[*]".blue());
    let spinner6 = create_spinner("Scanning constants...", cli.no_progress);
    
    let constant_results = constants::find_all_constants(reader.clone(), start_addr, end_addr);
    for result in constant_results {
        results.add_constant(result);
    }
    
    if let Some(ref pb) = spinner6 { pb.finish_with_message(format!("Found {} constants", results.constants.len())); }
    println!("{} Found {} constants", "[+]".green(), results.constants.len());

    println!();

    // Filter and save
    let filtered_results = filter_by_confidence(&results, min_confidence);

    save_scan_results(&filtered_results, &output)?;
    println!("{} Results saved to: {}", "[+]".green(), output.display());

    if let Some(text_path) = text {
        save_text_report(&filtered_results, &text_path)
            .map_err(|e| format!("Failed to save text report: {}", e))?;
        println!("{} Text report saved to: {}", "[+]".green(), text_path.display());
    }

    if let Some(md_path) = markdown {
        save_markdown_report(&filtered_results, &md_path)
            .map_err(|e| format!("Failed to save markdown report: {}", e))?;
        println!("{} Markdown report saved to: {}", "[+]".green(), md_path.display());
    }

    println!();
    print_scan_summary(&filtered_results, start_time.elapsed());

    Ok(())
}

fn filter_by_confidence(results: &CombinedResults, min_confidence: f64) -> CombinedResults {
    CombinedResults {
        functions: results.functions.iter()
            .filter(|f| f.confidence >= min_confidence)
            .cloned()
            .collect(),
        structure_offsets: results.structure_offsets.iter()
            .filter(|s| s.confidence >= min_confidence)
            .cloned()
            .collect(),
        classes: results.classes.clone(),
        properties: results.properties.clone(),
        methods: results.methods.clone(),
        constants: results.constants.clone(),
    }
}

fn print_scan_summary(results: &CombinedResults, elapsed: std::time::Duration) {
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "                SCAN COMPLETE".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();
    println!("  {} Functions found:        {}", "•".cyan(), results.functions.len().to_string().green().bold());
    println!("  {} Structure offsets:      {}", "•".cyan(), results.structure_offsets.len().to_string().green().bold());
    println!("  {} Classes found:          {}", "•".cyan(), results.classes.len().to_string().green().bold());
    println!("  {} Properties found:       {}", "•".cyan(), results.properties.len().to_string().green().bold());
    println!("  {} Methods found:          {}", "•".cyan(), results.methods.len().to_string().green().bold());
    println!("  {} Constants found:        {}", "•".cyan(), results.constants.len().to_string().green().bold());
    println!();
    println!("  {} Total offsets:          {}", "★".yellow(), results.total_count().to_string().green().bold());
    println!("  {} High confidence (>85%): {}", "★".yellow(), results.high_confidence_count().to_string().green().bold());
    println!();
    println!("  {} Time elapsed:           {:.2}s", "⏱".bright_black(), elapsed.as_secs_f64());
    println!();
}

// ==================== FFLAGS COMMAND ====================

fn run_fflags(
    cli: &Cli,
    binary: Option<PathBuf>,
    output: PathBuf,
    text: Option<PathBuf>,
    category: Option<String>,
    search: Option<String>,
    found_only: bool,
    list_categories: bool,
) -> Result<(), String> {
    let start_time = Instant::now();

    if list_categories {
        menu_fflag_categories();
        return Ok(());
    }

    let binary = binary.ok_or("Binary path is required for FFlag scanning")?;

    println!("{} {}", "[*]".blue(), "Loading binary...".white());

    let spinner = create_spinner("Loading binary...", cli.no_progress);

    let binary_mem = BinaryMemory::load(&binary)
        .map_err(|e| format!("Failed to load binary: {}", e))?;
    let reader: Arc<dyn MemoryReader> = Arc::new(binary_mem);

    if let Some(ref pb) = spinner {
        pb.finish_with_message("Binary loaded!");
    }

    println!("{} Binary loaded: {}", "[+]".green(), binary.display());

    let regions = reader.get_regions()
        .map_err(|e| format!("Failed to get memory regions: {}", e))?;

    // Find readable regions and collect their data
    println!("{} Found {} memory regions", "[+]".green(), regions.len());

    // Read binary file directly for string searching (more reliable)
    println!("{} Reading binary data...", "[*]".blue());
    let binary_data = std::fs::read(&binary)
        .map_err(|e| format!("Failed to read binary file: {}", e))?;
    
    println!("{} Binary size: {} MB", "[+]".green(), binary_data.len() / 1024 / 1024);
    println!("{} Scanning for FFlags...", "[*]".blue());
    println!();

    let db = get_database();
    let total_flags = db.count();
    
    let pb = create_progress_bar(total_flags as u64, "Searching for FFlags...", cli.no_progress);

    let mut flags_to_check: Vec<_> = db.all_flags().collect();

    if let Some(ref cat) = category {
        flags_to_check.retain(|f| f.category.eq_ignore_ascii_case(cat));
        println!("{} Filtering by category: {}", "[*]".blue(), cat.yellow());
    }

    if let Some(ref s) = search {
        let search_lower = s.to_lowercase();
        flags_to_check.retain(|f| f.name.to_lowercase().contains(&search_lower));
        println!("{} Searching for: {}", "[*]".blue(), s.yellow());
    }

    println!("{} Checking {} flags...", "[*]".blue(), flags_to_check.len());
    println!();

    let mut found_flags: Vec<&KnownFlag> = Vec::new();
    let mut not_found_flags: Vec<&KnownFlag> = Vec::new();

    // Use the raw binary data for string searching
    let data = &binary_data;

    for (i, flag) in flags_to_check.iter().enumerate() {
        if let Some(ref p) = pb {
            p.set_position(i as u64);
            if i % 500 == 0 {
                p.set_message(format!("Checking {}...", flag.name));
            }
        }

        let flag_bytes = flag.name.as_bytes();
        let found = data.windows(flag_bytes.len()).any(|w| w == flag_bytes);

        if found {
            found_flags.push(*flag);
        } else {
            not_found_flags.push(*flag);
        }
    }

    if let Some(ref p) = pb {
        p.finish_with_message("Scan complete!");
    }

    let output_flags: &Vec<&KnownFlag> = if found_only {
        &found_flags
    } else {
        &flags_to_check
    };

    let json_output = serde_json::json!({
        "total_checked": flags_to_check.len(),
        "found_in_binary": found_flags.len(),
        "not_found": not_found_flags.len(),
        "flags": output_flags.iter().map(|f| {
            serde_json::json!({
                "name": f.name,
                "type": format!("{:?}", f.flag_type),
                "category": f.category,
                "found_in_binary": found_flags.iter().any(|ff| ff.name == f.name),
            })
        }).collect::<Vec<_>>()
    });

    let json_str = serde_json::to_string_pretty(&json_output)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    
    std::fs::write(&output, &json_str)
        .map_err(|e| format!("Failed to write output: {}", e))?;

    println!("{} Results saved to: {}", "[+]".green(), output.display());

    if let Some(text_path) = text {
        let mut text_content = String::new();
        writeln!(text_content, "FFlag Report").unwrap();
        writeln!(text_content, "============").unwrap();
        writeln!(text_content).unwrap();
        writeln!(text_content, "Total checked: {}", flags_to_check.len()).unwrap();
        writeln!(text_content, "Found in binary: {}", found_flags.len()).unwrap();
        writeln!(text_content).unwrap();
        
        if !found_flags.is_empty() {
            writeln!(text_content, "Found Flags:").unwrap();
            writeln!(text_content, "------------").unwrap();
            for flag in &found_flags {
                writeln!(text_content, "  {} [{}]", flag.name, flag.category).unwrap();
            }
        }
        
        std::fs::write(&text_path, text_content)
            .map_err(|e| format!("Failed to write text output: {}", e))?;
        println!("{} Text report saved to: {}", "[+]".green(), text_path.display());
    }

    println!();
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "             FFLAG SCAN COMPLETE".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();
    println!("  {} Total flags checked:    {}", "•".cyan(), flags_to_check.len().to_string().white());
    println!("  {} Found in binary:        {}", "•".cyan(), found_flags.len().to_string().green().bold());
    println!("  {} Not found:              {}", "•".cyan(), not_found_flags.len().to_string().yellow());
    println!();
    println!("  {} Time elapsed:           {:.2}s", "⏱".bright_black(), start_time.elapsed().as_secs_f64());
    println!();

    if !found_flags.is_empty() && cli.verbose {
        println!("{}", "Found Flags (first 20):".yellow().bold());
        for flag in found_flags.iter().take(20) {
            println!("  {} {} [{}]", "✓".green(), flag.name.cyan(), flag.category.bright_black());
        }
        if found_flags.len() > 20 {
            println!("  ... and {} more", found_flags.len() - 20);
        }
        println!();
    }

    Ok(())
}

// ==================== OTHER COMMANDS ====================

fn run_diff(cli: &Cli, old: PathBuf, new: PathBuf, output: Option<PathBuf>) -> Result<(), String> {
    println!("{} Comparing offset files...", "[*]".blue());
    println!("  Old: {}", old.display());
    println!("  New: {}", new.display());
    println!();

    if !old.exists() {
        return Err(format!("Old file not found: {}", old.display()));
    }
    if !new.exists() {
        return Err(format!("New file not found: {}", new.display()));
    }

    let old_content = std::fs::read_to_string(&old)
        .map_err(|e| format!("Failed to read old file: {}", e))?;
    let new_content = std::fs::read_to_string(&new)
        .map_err(|e| format!("Failed to read new file: {}", e))?;

    println!("{}", "═".repeat(55).cyan());
    println!("{}", "               DIFF RESULTS".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();
    println!("  {} Comparison complete", "[+]".green());
    println!();

    Ok(())
}

fn run_validate(cli: &Cli, offsets: PathBuf, binary: PathBuf) -> Result<(), String> {
    println!("{} Validating offsets...", "[*]".blue());
    println!("  Offsets: {}", offsets.display());
    println!("  Binary: {}", binary.display());
    println!();

    if !offsets.exists() {
        return Err(format!("Offsets file not found: {}", offsets.display()));
    }
    if !binary.exists() {
        return Err(format!("Binary not found: {}", binary.display()));
    }

    println!("{}", "═".repeat(55).cyan());
    println!("{}", "           VALIDATION RESULTS".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();
    println!("  {} Validation complete", "[+]".green());
    println!();

    Ok(())
}

fn run_dump(cli: &Cli, binary: PathBuf, address: String, size: usize, disasm: bool) -> Result<(), String> {
    let addr = if address.starts_with("0x") || address.starts_with("0X") {
        u64::from_str_radix(&address[2..], 16)
            .map_err(|_| "Invalid hex address")?
    } else {
        address.parse::<u64>()
            .map_err(|_| "Invalid address")?
    };

    println!("{} Loading binary...", "[*]".blue());

    let binary_mem = BinaryMemory::load(&binary)
        .map_err(|e| format!("Failed to load binary: {}", e))?;
    let reader: Arc<dyn MemoryReader> = Arc::new(binary_mem);

    println!("{} Dumping {} bytes at {}", "[*]".blue(), size, format!("0x{:x}", addr).yellow());
    println!();

    let data = reader.read_bytes(Address::new(addr), size)
        .map_err(|e| format!("Failed to read memory: {}", e))?;

    if disasm {
        println!("{}", "Disassembly:".yellow().bold());
        for (i, chunk) in data.chunks(4).enumerate() {
            let offset = i * 4;
            print!("{:08x}:  ", addr + offset as u64);
            for b in chunk {
                print!("{:02x} ", b);
            }
            println!();
        }
    } else {
        println!("{}", "Hex Dump:".yellow().bold());
        for (i, chunk) in data.chunks(16).enumerate() {
            let offset = i * 16;
            print!("{:08x}:  ", addr + offset as u64);
            
            for (j, b) in chunk.iter().enumerate() {
                print!("{:02x} ", b);
                if j == 7 { print!(" "); }
            }
            
            for _ in chunk.len()..16 {
                print!("   ");
            }
            
            print!(" |");
            
            for b in chunk {
                if *b >= 0x20 && *b <= 0x7e {
                    print!("{}", *b as char);
                } else {
                    print!(".");
                }
            }
            
            println!("|");
        }
    }
    println!();

    Ok(())
}

fn run_stats(cli: &Cli, input: PathBuf) -> Result<(), String> {
    println!("{} Loading offsets file...", "[*]".blue());

    if !input.exists() {
        return Err(format!("File not found: {}", input.display()));
    }

    let content = std::fs::read_to_string(&input)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    println!();
    println!("{}", "═".repeat(55).cyan());
    println!("{}", "            OFFSET STATISTICS".cyan().bold());
    println!("{}", "═".repeat(55).cyan());
    println!();
    println!("  File: {}", input.display());
    println!();

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if let Some(arr) = value.as_array() {
                println!("  {} {}: {}", "•".cyan(), key, arr.len().to_string().green());
            } else if let Some(obj) = value.as_object() {
                println!("  {} {}: {}", "•".cyan(), key, obj.len().to_string().green());
            }
        }
    }

    println!();

    Ok(())
}

// ==================== HELPERS ====================

fn calculate_scan_range(regions: &[roblox_offset_generator::memory::MemoryRegion]) -> (Address, Address) {
    let mut min_addr = u64::MAX;
    let mut max_addr = 0u64;

    for region in regions {
        let start = region.range().start().as_u64();
        let end = start + region.range().size();

        if start < min_addr {
            min_addr = start;
        }
        if end > max_addr {
            max_addr = end;
        }
    }

    (Address::new(min_addr), Address::new(max_addr))
}

fn save_scan_results(results: &CombinedResults, path: &PathBuf) -> Result<(), String> {
    let json_map = results.to_json_map();
    let json_string = serde_json::to_string_pretty(&json_map)
        .map_err(|e| format!("Serialization error: {}", e))?;

    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

fn save_text_report(results: &CombinedResults, path: &PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;

    writeln!(file, "Roblox Offset Report")?;
    writeln!(file, "====================")?;
    writeln!(file)?;

    writeln!(file, "Functions ({}):", results.functions.len())?;
    for func in &results.functions {
        writeln!(file, "  {}: 0x{:016x} [{:.0}%]",
            func.name, func.address.as_u64(), func.confidence * 100.0)?;
    }
    writeln!(file)?;

    writeln!(file, "Structure Offsets ({}):", results.structure_offsets.len())?;
    for offset in &results.structure_offsets {
        writeln!(file, "  {}.{}: 0x{:x}",
            offset.structure_name, offset.field_name, offset.offset)?;
    }

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
    writeln!(file)?;

    writeln!(file, "## Functions")?;
    writeln!(file)?;
    writeln!(file, "| Name | Address | Confidence |")?;
    writeln!(file, "|------|---------|------------|")?;
    for func in &results.functions {
        writeln!(file, "| {} | `0x{:016x}` | {:.0}% |",
            func.name, func.address.as_u64(), func.confidence * 100.0)?;
    }

    Ok(())
}
