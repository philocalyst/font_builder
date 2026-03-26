//! Parsing functionality for font families.

pub mod info_parser;

use crate::{
    error::{Error, Result},
    models::{FamilyMemberSource, FontFamily},
    validator,
};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

/// Parses a complete font family from a directory.
pub fn parse_font_family(root: &Utf8Path) -> Result<FontFamily> {
    // Validate root directory
    if !root.exists() {
        return Err(Error::MissingDirectory {
            path: root.to_path_buf(),
        });
    }

    if !root.is_dir() {
        return Err(Error::InvalidStructure {
            path: root.to_path_buf(),
            reason: "Path is not a directory".to_string(),
        });
    }

    // Extract family name
    let name = extract_family_name(root)?;

    // Parse global info.toml
    let info_path = root.join("info.toml");
    let info = info_parser::parse_info_toml(&info_path)?;

    // Validate global info
    validator::validate_font_info(&info, "global info.toml")?;

    // Parse family members
    let source_dir = root.join("source");
    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(Error::MissingDirectory {
            path: source_dir.clone(),
        });
    }

    let members = parse_family_members(&source_dir)?;

    if members.is_empty() {
        return Err(Error::NoFamilyMembers(source_dir));
    }

    // Validate UFO version consistency
    validator::validate_ufo_version_consistency(&members)?;

    Ok(FontFamily {
        root: root.to_path_buf(),
        name,
        info,
        members,
    })
}

/// Extracts the family name from the directory path.
fn extract_family_name(root: &Utf8Path) -> Result<String> {
    let file_name = root.file_name().ok_or_else(|| Error::InvalidStructure {
        path: root.to_path_buf(),
        reason: "Cannot extract directory name".to_string(),
    })?;

    // Remove .fontfamily extension if present
    let name = file_name
        .strip_suffix(".fontfamily")
        .unwrap_or(file_name)
        .to_string();

    if name.is_empty() {
        return Err(Error::InvalidStructure {
            path: root.to_path_buf(),
            reason: "Empty font family name".to_string(),
        });
    }

    Ok(name)
}

/// Parses all family members from the source directory.
fn parse_family_members(source_dir: &Utf8Path) -> Result<Vec<FamilyMemberSource>> {
    let mut members = Vec::new();
    let mut errors = Vec::new();

    // Iterate through subdirectories in source/
    for entry in fs::read_dir(source_dir).map_err(|e| Error::Io(e))? {
        let entry = entry.map_err(|e| Error::Io(e))?;
        let path = entry.path();

        // Convert to Utf8PathBuf
        let path = Utf8PathBuf::try_from(path).map_err(|_| Error::InvalidStructure {
            path: Utf8PathBuf::from("non-UTF8 path"),
            reason: "Path contains non-UTF-8 characters".to_string(),
        })?;

        if !path.is_dir() {
            continue;
        }

        match parse_family_member(&path) {
            Ok(Some(member)) => members.push(member),
            Ok(None) => {} // No UFO files in this directory
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        if members.is_empty() {
            return Err(Error::Multiple(errors));
        }
        // Log warnings but continue if we have at least some members
        eprintln!("Warnings encountered while parsing family members:");
        for error in errors {
            eprintln!("  - {}", error);
        }
    }

    Ok(members)
}

/// Parses a single family member from a directory.
fn parse_family_member(member_dir: &Utf8Path) -> Result<Option<FamilyMemberSource>> {
    // Find UFO files in this directory
    let ufo_files = find_ufo_files(member_dir)?;

    if ufo_files.is_empty() {
        return Ok(None);
    }

    if ufo_files.len() > 1 {
        return Err(Error::InvalidStructure {
            path: member_dir.to_path_buf(),
            reason: format!(
                "Multiple UFO files found. Expected exactly one. Found: {}",
                ufo_files.len()
            ),
        });
    }

    let ufo_path = &ufo_files[0];
    let style_name = extract_style_name(ufo_path)?;

    // Parse UFO file
    let font = norad::Font::load(ufo_path).map_err(|e| Error::UfoParse {
        path: ufo_path.clone(),
        source: e,
    })?;

    // Check for info.toml override
    let override_path = member_dir.join("info.toml");
    let overrides = if override_path.exists() {
        Some(info_parser::parse_info_override(&override_path)?)
    } else {
        None
    };

    // Check for Assets directory
    let assets_dir = member_dir.join("Assets");
    let assets_dir = if assets_dir.exists() && assets_dir.is_dir() {
        Some(assets_dir)
    } else {
        None
    };

    let mut member = FamilyMemberSource::new(style_name, ufo_path.clone(), font);
    member.overrides = overrides;
    member.assets_dir = assets_dir;

    Ok(Some(member))
}

/// Finds all UFO files in a directory.
fn find_ufo_files(dir: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    let mut ufo_files = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| Error::Io(e))? {
        let entry = entry.map_err(|e| Error::Io(e))?;
        let path = entry.path();

        // Convert to Utf8PathBuf
        let path = Utf8PathBuf::try_from(path).map_err(|_| Error::InvalidStructure {
            path: Utf8PathBuf::from("non-UTF8 path"),
            reason: "Path contains non-UTF-8 characters".to_string(),
        })?;

        if path.is_dir()
            && path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("ufo"))
                .unwrap_or(false)
        {
            ufo_files.push(path);
        }
    }

    Ok(ufo_files)
}

/// Extracts the style name from a UFO file path.
///
/// Examples:
/// - "Regular.ufo" -> "Regular"
/// - "Regular+Italic.ufo" -> "Regular+Italic"
fn extract_style_name(ufo_path: &Utf8Path) -> Result<String> {
    let file_name = ufo_path
        .file_name()
        .ok_or_else(|| Error::InvalidStructure {
            path: ufo_path.to_path_buf(),
            reason: "Cannot extract UFO file name".to_string(),
        })?;

    let style_name = file_name
        .strip_suffix(".ufo")
        .ok_or_else(|| Error::InvalidStructure {
            path: ufo_path.to_path_buf(),
            reason: "UFO file does not have .ufo extension".to_string(),
        })?;

    if style_name.is_empty() {
        return Err(Error::InvalidStructure {
            path: ufo_path.to_path_buf(),
            reason: "Empty style name in UFO file".to_string(),
        });
    }

    Ok(style_name.to_string())
}
