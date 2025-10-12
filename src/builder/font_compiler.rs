//! Font compilation from UFO to various output formats.

use camino::{Utf8Path, Utf8PathBuf};
use fontc::{generate_font, Flags, Input};
use pollster::FutureExt as _;
use std::{collections::HashMap, fs, path::PathBuf};
use ttf2woff2::Converter;
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
    /// Compiles a UFO to a specific format.
    fn compile_to_format(
        &self,
        member: &FamilyMemberSource,
        output_path: &Utf8Path,
        format: OutputFormat,
    ) -> Result<()> {
        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }

        let bytes = self.compile_to_ttf(member, output_path)?;

        let out = match format {
            OutputFormat::Ttf => bytes,
            OutputFormat::Woff => {
                return Err(Error::Compilation {
                    style: member.style_name.clone(),
                    reason: "WOFF compilation not yet implemented".to_string(),
                })
            }
            OutputFormat::Woff2 => self.compile_to_woff(bytes, output_path)?,
            OutputFormat::Ttc => {
                return Err(Error::Compilation {
                    style: member.style_name.clone(),
                    reason: "TTC compilation not yet implemented".to_string(),
                })
            }
        };

        fs::write(output_path, out).map_err(Error::Io)?;

        Ok(())
    }

    /// Compiles a UFO to TTF format using write-fonts.
    fn compile_to_ttf(&self, family: &FamilyMemberSource, out_path: &Utf8Path) -> Result<Vec<u8>> {
        // 1) Make a temp build dir
        let build_dir = out_path
            .parent()
            .map(|p| p.join("fontc-build"))
            .unwrap_or_else(|| camino::Utf8PathBuf::from("fontc-build"));
        fs::create_dir_all(&build_dir).map_err(crate::error::Error::Io)?;

        let family_name = "ok";
        let style_name = &family.style_name;
        let ufo_basename = family.ufo_path.file_name().unwrap();

        // Then write the designspace...
        let ds_path =
            write_single_ufo_designspace(&build_dir, ufo_basename, family_name, style_name, 0u16)
                .map_err(crate::error::Error::Io)?;

        let flags = Flags::default();
        let bytes = generate_font(
            &Input::new(ds_path.as_std_path()).map_err(|e| crate::error::Error::Compilation {
                style: "unknown".into(),
                reason: format!("fontc input error: {e}"),
            })?,
            build_dir.as_std_path(),
            Some(&PathBuf::from(out_path.as_str())),
            flags,
            /* skip_features */ false,
        )
        .map_err(|e| crate::error::Error::Compilation {
            style: "unknown".into(),
            reason: format!("fontc generate_font error: {e}"),
        })?;

        Ok(bytes)
    }

    /// Compiles a UFO to WOFF format.
    fn compile_to_woff(&self, ttf_bytes: Vec<u8>, output_path: &Utf8Path) -> Result<Vec<u8>> {
        let convertor =
            Converter::from_data(ttf_bytes, None, ttf2woff2::BrotliQuality { value: 255 })
                .block_on()
                .unwrap();

        Ok(convertor.to_woff2().unwrap())
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

fn copy_dir_recursively(src: &camino::Utf8Path, dst: &camino::Utf8Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let to = dst.join(std::ffi::OsStr::to_string_lossy(&file_name).as_ref());
        if path.is_dir() {
            copy_dir_recursively(&camino::Utf8PathBuf::try_from(path).unwrap(), &to)?;
        } else {
            std::fs::copy(path, to)?;
        }
    }
    Ok(())
}

// The helper function:
fn write_single_ufo_designspace(
    build_dir: &camino::Utf8Path,
    ufo_basename: &str,
    family: &str,
    style: &str,
    weight: u16,
) -> std::io::Result<camino::Utf8PathBuf> {
    let ds_path = build_dir.join("temp.designspace");
    // This XML structure mimics a minimal variable font with only one master.
    // This is what fontc's parser expects.
    let ds_xml = format!(
        r#"<?xml version='1.0' encoding='UTF-8'?>
<designspace format="4.1">
  <axes>
    <axis tag="wght" name="Weight" minimum="{weight}" maximum="{weight}" default="{weight}"/>
  </axes>
  <sources>
    <source filename="{ufo}" familyname="{family}" stylename="{style}">
      <location>
        <dimension name="Weight" xvalue="{weight}"/>
      </location>
    </source>
  </sources>
  <instances>
    <instance filename="instance.ufo" familyname="{family}" stylename="{style}">
      <location>
        <dimension name="Weight" xvalue="{weight}"/>
      </location>
    </instance>
  </instances>
</designspace>"#,
        ufo = ufo_basename,
        family = xml_escape(family),
        style = xml_escape(style),
        weight = weight,
    );
    std::fs::write(ds_path.as_std_path(), ds_xml)?;
    Ok(ds_path)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
