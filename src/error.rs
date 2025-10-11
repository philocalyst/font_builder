//! Error types for the font-builder library.

use camino::Utf8PathBuf;
use norad::FormatVersion;
use std::fmt;
use thiserror::Error;

/// Result type alias for font-builder operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for font-builder operations.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error.
    #[error("Failed to parse TOML file at {path}: {source}")]
    TomlParse {
        path: Utf8PathBuf,
        source: toml::de::Error,
    },

    /// TOML serialization error.
    #[error("Failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// JSON serialization error.
    #[error("Failed to serialize JSON: {0}")]
    JsonSerialize(#[from] serde_json::Error),

    /// UFO parsing error.
    #[error("Failed to parse UFO at {path}: {source}")]
    UfoParse {
        path: Utf8PathBuf,
        source: norad::error::FontLoadError,
    },

    /// Missing required file.
    #[error("Missing required file: {path}")]
    MissingFile { path: Utf8PathBuf },

    /// Missing required directory.
    #[error("Missing required directory: {path}")]
    MissingDirectory { path: Utf8PathBuf },

    /// Invalid font family structure.
    #[error("Invalid font family structure at {path}: {reason}")]
    InvalidStructure { path: Utf8PathBuf, reason: String },

    /// Validation error.
    #[error("Validation error in {context}: {reason}")]
    Validation { context: String, reason: String },

    /// Missing required field.
    #[error("Missing required field '{field}' in {context}")]
    MissingField { field: String, context: String },

    /// Invalid date format.
    #[error("Invalid date format for '{field}': {value}")]
    InvalidDate { field: String, value: String },

    /// Invalid SPDX license identifier.
    #[error("Invalid SPDX license identifier: {0}")]
    InvalidLicense(String),

    /// Font compilation error.
    #[error("Font compilation error for {style}: {reason}")]
    Compilation { style: String, reason: String },

    // TODO: FIX ERROR reporting
    /// Inconsistent UFO versions.
    #[error(
        "Inconsistent UFO versions in font family: expected , \
         found  in {path}"
    )]
    InconsistentUfoVersion {
        expected: FormatVersion,
        found: FormatVersion,
        path: Utf8PathBuf,
    },

    /// No family members found.
    #[error("No family members found in source directory: {0}")]
    NoFamilyMembers(Utf8PathBuf),

    /// Invalid font form.
    #[error("Invalid font form: {0}")]
    InvalidFontForm(String),

    /// Multiple errors occurred.
    #[error("Multiple errors occurred:\n{}", format_errors(.0))]
    Multiple(Vec<Error>),
}

/// UFO format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfoVersion {
    /// UFO version 2.
    V2,
    /// UFO version 3.
    V3,
}

impl fmt::Display for UfoVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UfoVersion::V2 => write!(f, "UFO 2"),
            UfoVersion::V3 => write!(f, "UFO 3"),
        }
    }
}

fn format_errors(errors: &[Error]) -> String {
    errors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  {}. {}", i + 1, e))
        .collect::<Vec<_>>()
        .join("\n")
}

impl Error {
    /// Creates a validation error.
    pub fn validation(context: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::Validation {
            context: context.into(),
            reason: reason.into(),
        }
    }

    /// Creates a missing field error.
    pub fn missing_field(field: impl Into<String>, context: impl Into<String>) -> Self {
        Error::MissingField {
            field: field.into(),
            context: context.into(),
        }
    }
}
