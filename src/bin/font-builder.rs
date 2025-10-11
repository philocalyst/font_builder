//! Font Builder CLI
//!
//! Command-line interface for building font families.

#![cfg(feature = "cli")]

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};
use eyre::{Context, Result};
use font_builder::{BuildConfig, OutputFormat};
use std::process;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Font Builder - Build font families from UFO sources
#[derive(Parser, Debug)]
#[command(
    name = "font-builder",
    version,
    about,
    long_about = None,
    propagate_version = true
)]
struct Cli {
    /// Verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build a font family from UFO sources
    Build(BuildArgs),

    /// Validate a font family structure without building
    Validate(ValidateArgs),

    /// Show information about a font family
    Info(InfoArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Path to the font family directory
    #[arg(value_name = "FAMILY_DIR")]
    family_dir: Utf8PathBuf,

    /// Output directory for compiled fonts
    #[arg(short, long, default_value = "dist")]
    output: Utf8PathBuf,

    /// Output formats (comma-separated: ttf,woff,woff2)
    #[arg(short, long, value_delimiter = ',', default_values_t = vec!["ttf".to_string(), "woff2".to_string()])]
    formats: Vec<String>,

    /// Skip validation
    #[arg(long)]
    no_validate: bool,
}

#[derive(Args, Debug)]
struct ValidateArgs {
    /// Path to the font family directory
    #[arg(value_name = "FAMILY_DIR")]
    family_dir: Utf8PathBuf,
}

#[derive(Args, Debug)]
struct InfoArgs {
    /// Path to the font family directory
    #[arg(value_name = "FAMILY_DIR")]
    family_dir: Utf8PathBuf,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
}

fn main() {
    if let Err(e) = run() {
        error!("{:#}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Build(args) => build_command(args),
        Commands::Validate(args) => validate_command(args),
        Commands::Info(args) => info_command(args),
    }
}

fn build_command(args: BuildArgs) -> Result<()> {
    info!("Building font family from: {}", args.family_dir);

    // Parse formats
    let formats: Vec<OutputFormat> = args
        .formats
        .iter()
        .map(|f| OutputFormat::from_str(f).ok_or_else(|| eyre::eyre!("Invalid format: {}", f)))
        .collect::<Result<Vec<_>>>()?;

    let config = BuildConfig {
        output_dir: args.output,
        formats,
        validate: !args.no_validate,
    };

    let manifest_path = font_builder::build_font_family(&args.family_dir, config)
        .wrap_err("Failed to build font family")?;

    info!("Build complete!");
    info!("Manifest written to: {}", manifest_path);

    Ok(())
}

fn validate_command(args: ValidateArgs) -> Result<()> {
    info!("Validating font family: {}", args.family_dir);

    let family =
        font_builder::parser::parse_font_family(&args.family_dir).wrap_err("Validation failed")?;

    info!("✓ Font family structure is valid");
    info!("  Family name: {}", family.name);
    info!("  Members: {}", family.members.len());

    for member in &family.members {
        info!("    - {}", member.style_name);
    }

    Ok(())
}

fn info_command(args: InfoArgs) -> Result<()> {
    let family = font_builder::parser::parse_font_family(&args.family_dir)
        .wrap_err("Failed to parse font family")?;

    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&family.info)?;
            println!("{}", json);
        }
        "text" | _ => {
            println!("Font Family: {}", family.name);
            println!("License: {}", family.info.license);
            println!("Font Form: {}", family.info.font_form);
            println!("Publication Date: {}", family.info.publication_date);
            println!("\nFoundry: {}", family.info.foundry.name);

            if let Some(website) = &family.info.foundry.website {
                println!("  Website: {}", website);
            }

            println!("\nDesigners:");
            for designer in &family.info.designers {
                print!("  - {}", designer.name);
                if let Some(role) = &designer.role {
                    print!(" ({})", role);
                }
                println!();
            }

            if !family.info.contributors.is_empty() {
                println!("\nContributors:");
                for contributor in &family.info.contributors {
                    print!("  - {}", contributor.name);
                    if let Some(role) = &contributor.role {
                        print!(" ({})", role);
                    }
                    println!();
                }
            }

            println!("\nFamily Members ({}):", family.members.len());
            for member in &family.members {
                println!("  - {}", member.style_name);
                if let Some(overrides) = &member.overrides {
                    if !overrides.is_empty() {
                        println!("    (has overrides)");
                    }
                }
            }

            println!("\nSummary:");
            println!("{}", family.info.summary);

            if let Some(desc) = &family.info.description {
                println!("\nDescription:");
                println!("{}", desc);
            }
        }
    }

    Ok(())
}
