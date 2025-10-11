//! Generates manifest JSON from font family data.

use std::collections::HashMap;

use crate::{
    builder::font_compiler::{
        angle_to_css_name, angle_to_ot_name, extract_italic_angle, extract_weight, extract_width,
        weight_to_css_name, weight_to_ot_name, width_to_css_name, width_to_ot_name,
    },
    error::Result,
    models::{
        FamilyMember, FamilyMemberSource, FontFamily, Manifest, SlopeInfo, WeightInfo, WidthInfo,
    },
};

/// Generates manifest data from a font family.
pub struct ManifestGenerator;

impl ManifestGenerator {
    /// Generates a complete manifest for a font family.
    pub fn generate(
        family: &FontFamily,
        compiled_files: &HashMap<String, HashMap<String, String>>,
    ) -> Result<Manifest> {
        let mut manifest = Manifest::new(family.family_name().to_string());

        // Copy global information
        manifest.publication_date = family.info.publication_date.clone();
        manifest.spdx_license_identifier = family.info.license.clone();
        manifest.summary = family.info.summary.clone();
        manifest.fontform = family.info.font_form.clone();
        manifest.copyright = family.info.copyright.clone();
        manifest.trademark = family.info.trademark.clone();
        manifest.description = family.info.description.clone();
        manifest.foundry = family.info.foundry.clone();
        manifest.contributors = family.info.contributors.clone();
        manifest.designers = family.info.designers.clone();

        // Generate family member entries
        for member in &family.members {
            let member_manifest = Self::generate_member_manifest(family, member, compiled_files)?;
            manifest.family_members.push(member_manifest);
        }

        Ok(manifest)
    }

    /// Generates manifest data for a single family member.
    fn generate_member_manifest(
        family: &FontFamily,
        member: &FamilyMemberSource,
        compiled_files: &HashMap<String, HashMap<String, String>>,
    ) -> Result<FamilyMember> {
        let font_info = &member.font.font_info;

        // Extract names
        let family_name = font_info
            .family_name
            .as_deref()
            .unwrap_or(family.family_name());

        let style_name = font_info
            .style_name
            .as_deref()
            .unwrap_or(&member.style_name);

        let postscript_fallback = format!("{}-{}", family_name, style_name);
        let postscript_name = font_info
            .postscript_font_name
            .as_deref()
            .unwrap_or(&postscript_fallback);

        let full_font_name = format!("{} {}", family_name, style_name);

        // Extract version
        let version = font_info
            .version_major
            .zip(font_info.version_minor)
            .map(|(major, minor)| format!("{}.{}", major, minor))
            .unwrap_or_else(|| "1.0".to_string());

        // Get compiled font files
        let font_files = compiled_files
            .get(&member.style_name)
            .cloned()
            .unwrap_or_default();

        // Extract unicode ranges
        let unicode_ranges = extract_unicode_ranges(member);

        // Extract available glyphs (sample)
        let available_glyphs = extract_glyph_names(member);

        // Extract weight information
        let weight = extract_weight(member);
        let weights = WeightInfo {
            value: weight,
            css_name: weight_to_css_name(weight).to_string(),
            open_type_name: weight_to_ot_name(weight).to_string(),
        };

        // Extract slope information
        let angle = extract_italic_angle(member);
        let slopes = SlopeInfo {
            value: angle,
            css_name: angle_to_css_name(angle).to_string(),
            open_type_name: angle_to_ot_name(angle).to_string(),
        };

        // Extract width information
        let width = extract_width(member);
        let width_info = WidthInfo {
            value: width,
            css_name: width_to_css_name(width).to_string(),
            open_type_name: width_to_ot_name(width).to_string(),
        };

        // Extract OpenType features (placeholder)
        let opentype_features = extract_opentype_features(member);

        // Build overrides map
        let mut overrides = HashMap::new();
        if let Some(override_info) = &member.overrides {
            if let Some(desc) = &override_info.description {
                overrides.insert(
                    "description".to_string(),
                    serde_json::Value::String(desc.clone()),
                );
            }
            if let Some(summary) = &override_info.summary {
                overrides.insert(
                    "summary".to_string(),
                    serde_json::Value::String(summary.clone()),
                );
            }
            if let Some(pub_date) = &override_info.publication_date {
                overrides.insert(
                    "publication_date".to_string(),
                    serde_json::Value::String(pub_date.clone().to_string()),
                );
            }
        }

        Ok(FamilyMember {
            style_name: style_name.to_string(),
            postscript_name: postscript_name.to_string(),
            full_font_name,
            version,
            font_files,
            unicode_ranges,
            available_glyphs,
            weights,
            slopes,
            width: width_info,
            opentype_features,
            overrides,
        })
    }
}

/// Extracts unicode ranges from a font.
fn extract_unicode_ranges(member: &FamilyMemberSource) -> Vec<String> {
    let mut ranges = Vec::new();
    let mut codepoints: Vec<u32> = member
        .font
        .default_layer()
        .iter()
        .map(|c| c.codepoints.len() as u32)
        .collect();

    if codepoints.is_empty() {
        return ranges;
    }

    codepoints.sort_unstable();
    codepoints.dedup();

    // Group into ranges
    let mut start = codepoints[0];
    let mut end = start;

    for &cp in &codepoints[1..] {
        if cp == end + 1 {
            end = cp;
        } else {
            ranges.push(format_unicode_range(start, end));
            start = cp;
            end = cp;
        }
    }
    ranges.push(format_unicode_range(start, end));

    ranges
}

/// Formats a unicode range.
fn format_unicode_range(start: u32, end: u32) -> String {
    if start == end {
        format!("U+{:04X}", start)
    } else {
        format!("U+{:04X}-{:04X}", start, end)
    }
}

/// Extracts glyph names from a font (limited sample).
fn extract_glyph_names(member: &FamilyMemberSource) -> Vec<String> {
    member
        .font
        .default_layer()
        .iter()
        .take(100) // Limit to first 100 glyphs
        .map(|glyph| glyph.name().to_string())
        .collect()
}

/// Extracts OpenType features (placeholder).
fn extract_opentype_features(_member: &FamilyMemberSource) -> HashMap<String, bool> {
    // This would require parsing the features.fea file or
    // checking UFO lib for feature definitions
    let mut features = HashMap::new();
    features.insert("kern".to_string(), true);
    features.insert("liga".to_string(), false);
    features.insert("smcp".to_string(), false);
    features
}
