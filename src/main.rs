use clap::{Arg, Command};
use std::path::{Path, PathBuf};
use std::process;
use std::fs;

mod config;
mod processor;
mod walker;
mod watcher;

use config::GhostScrubConfig;
use walker::FileWalker;
use watcher::FileWatcher;

#[derive(Debug)]
struct CliConfig {
    paths: Vec<PathBuf>,
    dry_run: bool,
    watch: bool,
    config_file: Option<PathBuf>,
    verbose: bool,
}

fn main() {
    let matches = Command::new("ghost-scrub")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Strip invisible characters from text and code files")
        .subcommand(
            Command::new("init")
                .about("Create a default .ghostscrub configuration file")
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .help("Overwrite existing configuration file")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .arg(
            Arg::new("paths")
                .help("Files or directories to process (defaults to current directory)")
                .value_name("PATH")
                .num_args(0..)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .short('n')
                .help("Show what would be changed without modifying files")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("watch")
                .long("watch")
                .short('w')
                .help("Watch directories for changes and process files automatically")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Path to configuration file (defaults to .ghostscrub)")
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Show detailed output including diffs of changes")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Handle init subcommand
    if let Some(init_matches) = matches.subcommand_matches("init") {
        let force = init_matches.get_flag("force");
        if let Err(e) = run_init(force) {
            eprintln!("Init error: {}", e);
            process::exit(1);
        }
        return;
    }

    let cli_config = CliConfig {
        paths: matches
            .get_many::<PathBuf>("paths")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        dry_run: matches.get_flag("dry-run"),
        watch: matches.get_flag("watch"),
        config_file: matches.get_one::<PathBuf>("config").cloned(),
        verbose: matches.get_flag("verbose"),
    };

    // Load configuration
    let ghost_config = if let Some(config_path) = &cli_config.config_file {
        match GhostScrubConfig::load_from_file(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error loading config file {}: {}", config_path.display(), e);
                process::exit(1);
            }
        }
    } else {
        GhostScrubConfig::load_default()
    };

    if cli_config.watch {
        if let Err(e) = run_watch_mode(&cli_config, ghost_config) {
            eprintln!("Watch mode error: {}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = run_single_pass(&cli_config, ghost_config) {
            eprintln!("Processing error: {}", e);
            process::exit(1);
        }
    }
}

fn run_single_pass(cli_config: &CliConfig, ghost_config: GhostScrubConfig) -> Result<(), Box<dyn std::error::Error>> {
    let walker = FileWalker::new(ghost_config);
    let result = walker.process_paths(&cli_config.paths, cli_config.dry_run, cli_config.verbose)?;
    result.print_summary(cli_config.dry_run);
    Ok(())
}

fn run_init(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Path::new(".ghostscrub");

    if config_path.exists() && !force {
        eprintln!("Configuration file .ghostscrub already exists. Use --force to overwrite.");
        process::exit(1);
    }

    let default_config = include_str!("../template.ghostscrub");
    fs::write(config_path, default_config)?;

    println!("Created .ghostscrub configuration file with default settings.");
    Ok(())
}

fn run_watch_mode(cli_config: &CliConfig, ghost_config: GhostScrubConfig) -> Result<(), Box<dyn std::error::Error>> {
    let watcher = FileWatcher::new(ghost_config);
    watcher.watch_paths(&cli_config.paths)?;
    Ok(())
}