//! Parsing for info.toml files.

use camino::Utf8Path;
use std::fs;

use crate::{
    error::{Error, Result},
    models::{FontInfo, InfoOverride},
};

/// Parses a global info.toml file.
pub fn parse_info_toml(path: &Utf8Path) -> Result<FontInfo> {
    if !path.exists() {
        return Err(Error::MissingFile {
            path: path.to_path_buf(),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| Error::Io(e))?;

    toml::from_str(&content).map_err(|e| Error::TomlParse {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Parses an info.toml override file.
pub fn parse_info_override(path: &Utf8Path) -> Result<InfoOverride> {
    let content = fs::read_to_string(path).map_err(|e| Error::Io(e))?;

    toml::from_str(&content).map_err(|e| Error::TomlParse {
        path: path.to_path_buf(),
        source: e,
    })
}
