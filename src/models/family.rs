//! Internal models for font family structure.

use camino::{Utf8Path, Utf8PathBuf};
use norad::Font as NoradFont;

use super::info::{FontInfo, InfoOverride};

/// A complete font family with all its members.
#[derive(Debug)]
pub struct FontFamily {
    /// Root directory of the font family.
    pub root: Utf8PathBuf,

    /// Font family name (inferred from directory name).
    pub name: String,

    /// Global font information.
    pub info: FontInfo,

    /// Family members (individual fonts).
    pub members: Vec<FamilyMemberSource>,
}

/// A single font within a family.
#[derive(Debug)]
pub struct FamilyMemberSource {
    /// Style name (e.g., "Regular", "Italic").
    pub style_name: String,

    /// Path to the UFO file.
    pub ufo_path: Utf8PathBuf,

    /// Parsed UFO font data.
    pub font: NoradFont,

    /// Optional info.toml overrides for this member.
    pub overrides: Option<InfoOverride>,

    /// Assets directory for this member.
    pub assets_dir: Option<Utf8PathBuf>,
}

impl FontFamily {
    /// Gets the family name without the .fontfamily extension.
    pub fn family_name(&self) -> &str {
        &self.name
    }

    /// Gets the source directory path.
    pub fn source_dir(&self) -> Utf8PathBuf {
        self.root.join("source")
    }

    /// Gets the global assets directory path.
    pub fn assets_dir(&self) -> Utf8PathBuf {
        self.root.join("Assets")
    }

    /// Gets the LICENSE.md path.
    pub fn license_path(&self) -> Utf8PathBuf {
        self.root.join("LICENSE.md")
    }

    /// Gets the CHANGELOG.md path.
    pub fn changelog_path(&self) -> Utf8PathBuf {
        self.root.join("CHANGELOG.md")
    }

    /// Gets the info.toml path.
    pub fn info_path(&self) -> Utf8PathBuf {
        self.root.join("info.toml")
    }
}

impl FamilyMemberSource {
    /// Creates a new family member source.
    pub fn new(style_name: String, ufo_path: Utf8PathBuf, font: NoradFont) -> Self {
        Self {
            style_name,
            ufo_path,
            font,
            overrides: None,
            assets_dir: None,
        }
    }

    /// Gets the parent directory of the UFO file.
    pub fn member_dir(&self) -> &Utf8Path {
        self.ufo_path
            .parent()
            .expect("UFO path should have a parent directory")
    }
}

/// Output format for compiled fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    /// TrueType Font (.ttf)
    Ttf,
    /// TrueType Collection (.ttc)
    Ttc,
    /// Web Open Font Format 2 (.woff2)
    Woff2,
}

impl OutputFormat {
    /// Gets the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Ttf => "ttf",
            OutputFormat::Ttc => "ttc",
            OutputFormat::Woff2 => "woff2",
        }
    }

    /// Parses an output format from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ttf" => Some(OutputFormat::Ttf),
            "ttc" => Some(OutputFormat::Ttc),
            "woff2" => Some(OutputFormat::Woff2),
            _ => None,
        }
    }

    /// Returns all available formats.
    pub fn all() -> &'static [OutputFormat] {
        &[OutputFormat::Ttf, OutputFormat::Ttc, OutputFormat::Woff2]
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format!("Invalid output format: {}", s))
    }
}
