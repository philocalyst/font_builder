//! Font compilation from UFO to various output formats.

use camino::{Utf8Path, Utf8PathBuf};
use std::{collections::HashMap, fs};
use write_fonts::{tables as write_tables, types::NameId, FontBuilder};

use crate::{
    error::{Error, Result},
    models::{FamilyMemberSource, OutputFormat},
};

/// Compiles UFO fonts to various output formats.
pub struct FontCompiler {
    /// Output directory for compiled fonts.
    output_dir: Utf8PathBuf,

    /// Formats to compile to.
    formats: Vec<OutputFormat>,
}

impl FontCompiler {
    /// Creates a new font compiler.
    pub fn new(output_dir: Utf8PathBuf, formats: Vec<OutputFormat>) -> Self {
        Self {
            output_dir,
            formats,
        }
    }

    /// Compiles a single family member to all configured formats.
    pub fn compile_member(
        &self,
        member: &FamilyMemberSource,
        family_name: &str,
    ) -> Result<HashMap<String, String>> {
        let mut output_files = HashMap::new();

        for format in &self.formats {
            let file_name = self.generate_filename(family_name, &member.style_name, *format);
            let output_path = self.output_dir.join(&file_name);

            self.compile_to_format(member, &output_path, *format)?;

            output_files.insert(format.extension().to_string(), file_name);
        }

        Ok(output_files)
    }

    /// Generates a filename for a compiled font.
    fn generate_filename(
        &self,
        family_name: &str,
        style_name: &str,
        format: OutputFormat,
    ) -> String {
        format!("{}-{}.{}", family_name, style_name, format.extension())
    }

    /// Compiles a UFO to a specific format.
    fn compile_to_format(
        &self,
        member: &FamilyMemberSource,
        output_path: &Utf8Path,
        format: OutputFormat,
    ) -> Result<()> {
        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::Io(e))?;
        }

        match format {
            OutputFormat::Ttf => self.compile_to_ttf(member, output_path),
            OutputFormat::Woff => self.compile_to_woff(member, output_path),
            OutputFormat::Woff2 => self.compile_to_woff2(member, output_path),
            OutputFormat::Ttc => Err(Error::Compilation {
                style: member.style_name.clone(),
                reason: "TTC format requires multiple fonts (not yet implemented)".to_string(),
            }),
        }
    }

    /// Compiles a UFO to TTF format using write-fonts.
    fn compile_to_ttf(&self, member: &FamilyMemberSource, output_path: &Utf8Path) -> Result<()> {
        // This is a simplified implementation. In a production system,
        // you would need to:
        // 1. Convert UFO glyphs to TrueType outlines
        // 2. Generate all required OpenType tables
        // 3. Handle hinting, kerning, features, etc.

        // For now, we'll create a basic structure that demonstrates the approach
        let font_info = &member.font.font_info;

        let family_name = font_info.family_name.as_deref().unwrap_or("Unknown");

        let style_name = font_info
            .style_name
            .as_deref()
            .unwrap_or(&member.style_name);

        // Note: This is a placeholder. A full implementation would require
        // converting UFO data structures to write-fonts format, which is
        // complex and beyond a simple example. In production, you might:
        // 1. Use fontmake via FFI
        // 2. Use ufo2ft Python library via PyO3
        // 3. Implement full UFO->TTF conversion (very complex)

        // For demonstration, we'll create a minimal valid error
        Err(Error::Compilation {
            style: member.style_name.clone(),
            reason: format!(
                "TTF compilation not fully implemented. \
                 Would compile {} {} to {}",
                family_name, style_name, output_path
            ),
        })
    }

    /// Compiles a UFO to WOFF format.
    fn compile_to_woff(&self, member: &FamilyMemberSource, output_path: &Utf8Path) -> Result<()> {
        // WOFF is a compressed TTF, so we would:
        // 1. Generate TTF
        // 2. Convert to WOFF format

        Err(Error::Compilation {
            style: member.style_name.clone(),
            reason: "WOFF compilation not yet implemented".to_string(),
        })
    }

    /// Compiles a UFO to WOFF2 format.
    fn compile_to_woff2(&self, member: &FamilyMemberSource, output_path: &Utf8Path) -> Result<()> {
        Err(Error::Compilation {
            style: member.style_name.clone(),
            reason: "WOFF2 compilation not yet implemented".to_string(),
        })
    }
}

/// Extracts weight value from font info.
pub fn extract_weight(member: &FamilyMemberSource) -> u16 {
    member
        .font
        .font_info
        .open_type_os2_weight_class
        .map(|w| w as u16)
        .unwrap_or(400)
}

/// Extracts width value from font info.
pub fn extract_width(member: &FamilyMemberSource) -> u16 {
    member
        .font
        .font_info
        .open_type_os2_width_class
        .map(|w| w as u16 * 10) // Convert 1-9 scale to percentage
        .unwrap_or(100)
}

/// Extracts italic angle from font info.
pub fn extract_italic_angle(member: &FamilyMemberSource) -> f32 {
    member
        .font
        .font_info
        .italic_angle
        .map(|a| a.abs() as f32)
        .unwrap_or(0.0)
}

/// Maps weight value to CSS name.
pub fn weight_to_css_name(weight: u16) -> &'static str {
    match weight {
        100 => "100",
        200 => "200",
        300 => "300",
        400 => "normal",
        500 => "500",
        600 => "600",
        700 => "bold",
        800 => "800",
        900 => "900",
        _ => "normal",
    }
}

/// Maps weight value to OpenType name.
pub fn weight_to_ot_name(weight: u16) -> &'static str {
    match weight {
        100 => "Thin",
        200 => "ExtraLight",
        300 => "Light",
        400 => "Regular",
        500 => "Medium",
        600 => "SemiBold",
        700 => "Bold",
        800 => "ExtraBold",
        900 => "Black",
        _ => "Regular",
    }
}

/// Maps italic angle to CSS style name.
pub fn angle_to_css_name(angle: f32) -> &'static str {
    if angle > 0.0 {
        "italic"
    } else {
        "normal"
    }
}

/// Maps italic angle to OpenType style name.
pub fn angle_to_ot_name(angle: f32) -> &'static str {
    if angle > 0.0 {
        "Italic"
    } else {
        "Roman"
    }
}

/// Maps width value to CSS name.
pub fn width_to_css_name(width: u16) -> &'static str {
    match width {
        50 => "ultra-condensed",
        62 => "extra-condensed",
        75 => "condensed",
        87 => "semi-condensed",
        100 => "normal",
        112 => "semi-expanded",
        125 => "expanded",
        150 => "extra-expanded",
        200 => "ultra-expanded",
        _ => "normal",
    }
}

/// Maps width value to OpenType name.
pub fn width_to_ot_name(width: u16) -> &'static str {
    match width {
        50 => "UltraCondensed",
        62 => "ExtraCondensed",
        75 => "Condensed",
        87 => "SemiCondensed",
        100 => "Medium",
        112 => "SemiExpanded",
        125 => "Expanded",
        150 => "ExtraExpanded",
        200 => "UltraExpanded",
        _ => "Medium",
    }
}
