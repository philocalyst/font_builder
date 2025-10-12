//! Models for info.toml configuration.

use chrono::NaiveDate;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Global font family information from info.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FontInfo {
    /// Date of initial publication (YYYY-MM-DD).
    pub publication_date: Date,

    /// SPDX license identifier (e.g., "OFL-1.1").
    pub license: String,

    /// Concise 1-2 sentence description.
    pub summary: String,

    /// General classification (serif, sans-serif, script, display, monospace).
    pub font_form: String,

    /// Website of the font
    pub website: Url,

    /// Full copyright notice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    /// Trademark notice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trademark: Option<String>,

    /// Detailed description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Foundry information.
    pub foundry: Foundry,

    /// Name of the family it belongs to
    pub family: Option<String>,

    /// Contributors (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributors: Option<Vec<Person>>,

    /// Designers (This or a contributor is required, at least one).
    pub designers: Option<Vec<Person>>,
}

/// Foundry/studio information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Foundry {
    /// Foundry name.
    pub name: String,

    /// Foundry website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Foundry email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Person information (designer or contributor).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Person {
    /// Person's name.
    pub name: String,

    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Role description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Override information for specific family members.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoOverride {
    /// Override description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Override summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Override publication date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<Date>,

    /// Additional metadata.
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl InfoOverride {
    /// Checks if the override is empty.
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.summary.is_none()
            && self.publication_date.is_none()
            && self.extra.is_empty()
    }
}
