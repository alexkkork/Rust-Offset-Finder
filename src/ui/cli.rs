// Tue Jan 13 2026 - Alex

use crate::config::Config;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "roblox-offset-generator")]
#[command(author = "Alex")]
#[command(version = "1.0.0")]
#[command(about = "ARM64 Roblox Offset Generator - Finds all offsets using hybrid analysis")]
#[command(long_about = None)]
pub struct CliInterface {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, help = "Target Roblox binary file")]
    pub binary: Option<PathBuf>,

    #[arg(short, long, help = "Target Roblox process name")]
    pub process: Option<String>,

    #[arg(short, long, default_value = "offsets.json", help = "Output file path")]
    pub output: PathBuf,

    #[arg(short, long, default_value = "json", help = "Output format")]
    pub format: OutputFormat,

    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(short, long, help = "Quiet mode - minimal output")]
    pub quiet: bool,

    #[arg(long, help = "Number of threads to use")]
    pub threads: Option<usize>,

    #[arg(long, help = "Disable pattern scanning")]
    pub no_patterns: bool,

    #[arg(long, help = "Disable symbol matching")]
    pub no_symbols: bool,

    #[arg(long, help = "Disable XRef analysis")]
    pub no_xrefs: bool,

    #[arg(long, help = "Disable heuristic analysis")]
    pub no_heuristics: bool,

    #[arg(long, help = "Skip offset validation")]
    pub skip_validation: bool,

    #[arg(long, default_value = "0.85", help = "Minimum confidence threshold (0.0-1.0)")]
    pub confidence: f64,

    #[arg(long, help = "Export format for code generation")]
    pub export: Option<ExportFormat>,

    #[arg(long, help = "Generate HTML report")]
    pub html_report: bool,

    #[arg(long, help = "Generate Markdown report")]
    pub md_report: bool,

    #[arg(long, help = "Compare with existing offsets file")]
    pub diff: Option<PathBuf>,

    #[arg(long, help = "Show progress bars")]
    pub progress: bool,

    #[arg(long, help = "Disable colored output")]
    pub no_color: bool,

    #[arg(long, help = "Config file path")]
    pub config: Option<PathBuf>,

    #[arg(long, help = "Timeout in seconds")]
    pub timeout: Option<u64>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Scan a binary file for offsets")]
    Scan {
        #[arg(help = "Path to the binary file")]
        path: PathBuf,

        #[arg(short, long, help = "Scan specific sections only")]
        sections: Option<Vec<String>>,
    },

    #[command(about = "Attach to a running process")]
    Attach {
        #[arg(help = "Process name or PID")]
        target: String,

        #[arg(short, long, help = "Scan memory regions")]
        regions: bool,
    },

    #[command(about = "Dump all found offsets")]
    Dump {
        #[arg(short, long, help = "Output file")]
        output: PathBuf,

        #[arg(short, long, default_value = "all", help = "What to dump")]
        what: DumpTarget,
    },

    #[command(about = "Compare two offset files")]
    Diff {
        #[arg(help = "Old offsets file")]
        old: PathBuf,

        #[arg(help = "New offsets file")]
        new: PathBuf,

        #[arg(short, long, help = "Show only breaking changes")]
        breaking_only: bool,
    },

    #[command(about = "Export offsets to code")]
    Export {
        #[arg(help = "Input offsets file")]
        input: PathBuf,

        #[arg(help = "Output file")]
        output: PathBuf,

        #[arg(short, long, help = "Export format")]
        format: ExportFormat,
    },

    #[command(about = "Validate offset file")]
    Validate {
        #[arg(help = "Offsets file to validate")]
        file: PathBuf,

        #[arg(short, long, help = "Binary to validate against")]
        binary: Option<PathBuf>,
    },

    #[command(about = "Show information about a binary")]
    Info {
        #[arg(help = "Path to the binary")]
        path: PathBuf,

        #[arg(short, long, help = "Show detailed info")]
        detailed: bool,
    },

    #[command(about = "Search for specific patterns")]
    Search {
        #[arg(help = "Pattern to search for (hex string)")]
        pattern: String,

        #[arg(short, long, help = "Target binary")]
        binary: PathBuf,

        #[arg(short, long, help = "Maximum results")]
        limit: Option<usize>,
    },

    #[command(about = "List available finders")]
    ListFinders {
        #[arg(short, long, help = "Filter by category")]
        category: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
    Csv,
    Binary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ExportFormat {
    Cpp,
    Rust,
    Lua,
    Python,
    JavaScript,
    Frida,
    Ida,
    Ghidra,
    CheatEngine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DumpTarget {
    All,
    Functions,
    Structures,
    Classes,
    Properties,
    Methods,
    Constants,
}

impl CliInterface {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn to_config(&self) -> Config {
        let mut config = Config::default();

        if let Some(binary) = &self.binary {
            config.target_binary = Some(binary.clone());
        }

        if let Some(process) = &self.process {
            config.target_process = Some(process.clone());
        }

        config.output_file = self.output.clone();
        config.enable_verbose_output = self.verbose;
        config.enable_progress_bars = self.progress || !self.quiet;

        if let Some(threads) = self.threads {
            config.max_threads = threads;
        }

        config.enable_pattern_scanning = !self.no_patterns;
        config.enable_symbol_matching = !self.no_symbols;
        config.enable_xref_analysis = !self.no_xrefs;
        config.enable_heuristic_analysis = !self.no_heuristics;
        config.skip_validation = self.skip_validation;
        config.pattern_confidence_threshold = self.confidence;

        if let Some(timeout) = self.timeout {
            config.timeout_seconds = timeout;
        }

        config
    }

    pub fn has_target(&self) -> bool {
        self.binary.is_some() || self.process.is_some()
    }

    pub fn print_help() {
        use clap::CommandFactory;
        let mut cmd = Self::command();
        cmd.print_help().unwrap();
    }

    pub fn print_version() {
        println!("roblox-offset-generator v1.0.0");
        println!("Architecture: ARM64");
        println!("Platform: macOS");
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(Commands::Scan { path, .. }) = &self.command {
            if !path.exists() {
                return Err(format!("Binary file not found: {:?}", path));
            }
        }

        if let Some(binary) = &self.binary {
            if !binary.exists() {
                return Err(format!("Binary file not found: {:?}", binary));
            }
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err("Confidence must be between 0.0 and 1.0".to_string());
        }

        if let Some(threads) = self.threads {
            if threads == 0 {
                return Err("Thread count must be at least 1".to_string());
            }
        }

        Ok(())
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    pub fn should_show_progress(&self) -> bool {
        self.progress || (!self.quiet && !self.verbose)
    }
}

impl Default for CliInterface {
    fn default() -> Self {
        Self {
            command: None,
            binary: None,
            process: None,
            output: PathBuf::from("offsets.json"),
            format: OutputFormat::Json,
            verbose: false,
            quiet: false,
            threads: None,
            no_patterns: false,
            no_symbols: false,
            no_xrefs: false,
            no_heuristics: false,
            skip_validation: false,
            confidence: 0.85,
            export: None,
            html_report: false,
            md_report: false,
            diff: None,
            progress: false,
            no_color: false,
            config: None,
            timeout: None,
        }
    }
}

pub fn parse_cli() -> CliInterface {
    CliInterface::parse_args()
}

pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_cli();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    match &cli.command {
        Some(Commands::ListFinders { category }) => {
            list_finders(category.as_deref());
        }
        Some(Commands::Info { path, detailed }) => {
            show_binary_info(path, *detailed)?;
        }
        _ => {
            if cli.has_target() {
                run_scan(&cli)?;
            } else {
                CliInterface::print_help();
            }
        }
    }

    Ok(())
}

fn list_finders(category: Option<&str>) {
    println!("Available Finders:\n");

    let finders = vec![
        ("lua_api", "Lua API Functions", vec![
            "lua_gettop", "lua_settop", "lua_pushvalue", "lua_type",
            "lua_tonumber", "lua_toboolean", "lua_tostring", "lua_touserdata",
            "lua_rawget", "lua_rawgeti", "lua_rawset", "lua_rawseti",
            "lua_getfield", "lua_createtable", "lua_newthread", "lua_resume", "lua_pcall",
        ]),
        ("roblox", "Roblox Functions", vec![
            "LuauLoad", "NewThread", "PushInstance", "GetTypename",
            "IdentityPropagator", "taskDefer", "taskSpawn", "sctxResume",
            "PushCClosure", "CreateJob", "RequireCheck", "rbxCrash", "TaskScheduler",
        ]),
        ("bytecode", "Bytecode Functions", vec![
            "OpcodeLookup", "OpcodeLookup_",
        ]),
        ("structures", "Structure Offsets", vec![
            "luastate_base", "luastate_top", "luastate_stack",
            "extraspace_identity", "sctx_identity",
            "extraspace_capabilities", "sctx_capabilities",
            "closure_proto", "proto_capabilities",
        ]),
    ];

    for (cat, name, items) in finders {
        if category.is_none() || category == Some(cat) {
            println!("{}:", name);
            for item in items {
                println!("  - {}", item);
            }
            println!();
        }
    }
}

fn show_binary_info(path: &PathBuf, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 4096];
    file.read(&mut buffer)?;

    println!("Binary Information:");
    println!("  Path: {:?}", path);
    println!("  Size: {} bytes", std::fs::metadata(path)?.len());

    if buffer.len() >= 4 {
        let magic = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let format = match magic {
            0xFEEDFACF => "Mach-O 64-bit",
            0xFEEDFACE => "Mach-O 32-bit",
            0xCAFEBABE => "Mach-O Fat Binary",
            0xBEBAFECA => "Mach-O Fat Binary (reversed)",
            _ => "Unknown",
        };
        println!("  Format: {}", format);
    }

    if detailed {
        println!("\nDetailed information would be shown here...");
    }

    Ok(())
}

fn run_scan(cli: &CliInterface) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting offset scan...");
    println!("This would run the full scan with the provided configuration.");
    Ok(())
}
