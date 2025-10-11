//! Font Builder Library
//!
//! A strongly-typed library for parsing and building font families
//! according to the Font Distribution Specification.

pub mod builder;
pub mod error;
pub mod models;
pub mod parser;
pub mod validator;

pub use error::{Error, Result};
pub use models::{FontFamily, Manifest, OutputFormat};

use camino::{Utf8Path, Utf8PathBuf};
use std::{collections::HashMap, fs};

use builder::{FontCompiler, ManifestGenerator};
use parser::parse_font_family;

/// Configuration for building a font family.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Output directory for compiled fonts and manifest.
    pub output_dir: Utf8PathBuf,

    /// Output formats to generate.
    pub formats: Vec<OutputFormat>,

    /// Whether to validate the font family structure.
    pub validate: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: Utf8PathBuf::from("dist"),
            formats: vec![OutputFormat::Ttf, OutputFormat::Woff2],
            validate: true,
        }
    }
}

/// Main entry point for building a font family.
pub fn build_font_family(family_dir: &Utf8Path, config: BuildConfig) -> Result<Utf8PathBuf> {
    // Parse the font family
    let family = parse_font_family(family_dir)?;

    // Create output directory
    fs::create_dir_all(&config.output_dir).map_err(Error::Io)?;

    // Compile fonts
    let compiler = FontCompiler::new(config.output_dir.clone(), config.formats);
    let mut compiled_files = HashMap::new();

    for member in &family.members {
        let files = compiler.compile_member(member, family.family_name())?;
        compiled_files.insert(member.style_name.clone(), files);
    }

    // Generate manifest
    let manifest = ManifestGenerator::generate(&family, &compiled_files)?;

    // Write manifest to JSON
    let manifest_path = config.output_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json).map_err(Error::Io)?;

    Ok(manifest_path)
}
