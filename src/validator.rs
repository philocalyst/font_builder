//! Validation logic for font families.

use regex::Regex;

use crate::{
    error::{Error, Result, UfoVersion},
    models::{FamilyMemberSource, FontInfo},
};

static SPDX_LICENSES: &[&str] = &[
    "OFL-1.1",
    "OFL-1.1-RFN",
    "OFL-1.1-no-RFN",
    "MIT",
    "Apache-2.0",
    "CC0-1.0",
];

static VALID_FONT_FORMS: &[&str] = &[
    "serif",
    "sans-serif",
    "script",
    "display",
    "monospace",
    "handwriting",
    "decorative",
];

/// Validates global font information.
pub fn validate_font_info(info: &FontInfo, context: &str) -> Result<()> {
    let mut errors = Vec::new();

    // Validate license
    if !SPDX_LICENSES.contains(&info.license.as_str()) {
        errors.push(Error::InvalidLicense(info.license.clone()));
    }

    // Validate summary is not empty
    if info.summary.trim().is_empty() {
        errors.push(Error::missing_field("summary", context));
    }

    // Validate fontform
    if !VALID_FONT_FORMS.contains(&info.font_form.as_str()) {
        errors.push(Error::InvalidFontForm(info.font_form.clone()));
    }

    // Validate foundry
    if info.foundry.name.trim().is_empty() {
        errors.push(Error::missing_field("foundry.name", context));
    }

    // Validate designers (at least one required)
    if info.designers.is_none() && info.contributors.is_none() {
        errors.push(Error::missing_field("designers or contribitors", context));
    }

    if errors.is_empty() {
        Ok(())
    } else if errors.len() == 1 {
        Err(errors.into_iter().next().unwrap())
    } else {
        Err(Error::Multiple(errors))
    }
}

/// Validates that all family members use the same UFO version.
pub fn validate_ufo_version_consistency(members: &[FamilyMemberSource]) -> Result<()> {
    if members.is_empty() {
        return Ok(());
    }

    let first_version = &members[0].font.meta.format_version;

    for member in &members[1..] {
        let version = member.font.meta.format_version;
        if &version != first_version {
            return Err(Error::InconsistentUfoVersion {
                expected: first_version.to_owned(),
                found: version,
                path: member.ufo_path.clone(),
            });
        }
    }

    Ok(())
}
