//! Models for the output manifest JSON.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete font family manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest format version.
    pub manifest_version: String,

    /// Font family name.
    pub font_family_name: String,

    /// Publication date (YYYY-MM-DD).
    pub publication_date: String,

    /// SPDX license identifier.
    pub spdx_license_identifier: String,

    /// Summary description.
    pub summary: String,

    /// Font form classification.
    pub fontform: String,

    /// Copyright notice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    /// Trademark notice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trademark: Option<String>,

    /// Detailed description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Foundry information.
    pub foundry: super::info::Foundry,

    /// Contributors.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<super::info::Person>,

    /// Designers.
    pub designers: Vec<super::info::Person>,

    /// Family members.
    pub family_members: Vec<FamilyMember>,
}

/// Individual font family member in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Style name (e.g., "Regular", "Italic", "Bold").
    pub style_name: String,

    /// PostScript name.
    pub postscript_name: String,

    /// Full font name.
    pub full_font_name: String,

    /// Font version.
    pub version: String,

    /// Generated font files.
    pub font_files: HashMap<String, String>,

    /// Unicode ranges covered.
    pub unicode_ranges: Vec<String>,

    /// Available glyphs (sample or full list).
    pub available_glyphs: Vec<String>,

    /// Weight information.
    pub weights: WeightInfo,

    /// Slope/style information.
    pub slopes: SlopeInfo,

    /// Width information.
    pub width: WidthInfo,

    /// OpenType features.
    pub opentype_features: HashMap<String, bool>,

    /// Any overrides from family member info.toml.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub overrides: HashMap<String, serde_json::Value>,
}

/// Weight information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightInfo {
    /// Numeric weight value (100-900).
    pub value: u16,

    /// CSS weight name.
    pub css_name: String,

    /// OpenType weight name.
    pub open_type_name: String,
}

/// Slope/style information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlopeInfo {
    /// Slope angle in degrees.
    pub value: f32,

    /// CSS style name.
    pub css_name: String,

    /// OpenType style name.
    pub open_type_name: String,
}

/// Width information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidthInfo {
    /// Width percentage (50-200).
    pub value: u16,

    /// CSS width name.
    pub css_name: String,

    /// OpenType width name.
    pub open_type_name: String,
}

impl Manifest {
    /// Current manifest format version.
    pub const VERSION: &'static str = "1.0";

    /// Creates a new manifest with default version.
    pub fn new(font_family_name: String) -> Self {
        Self {
            manifest_version: Self::VERSION.to_string(),
            font_family_name,
            publication_date: String::new(),
            spdx_license_identifier: String::new(),
            summary: String::new(),
            fontform: String::new(),
            copyright: None,
            trademark: None,
            description: None,
            foundry: super::info::Foundry {
                name: String::new(),
                website: None,
                email: None,
            },
            contributors: Vec::new(),
            designers: Vec::new(),
            family_members: Vec::new(),
        }
    }
}
