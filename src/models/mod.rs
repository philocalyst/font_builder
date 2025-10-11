//! Data models for font-builder.

pub mod family;
pub mod info;
pub mod manifest;

pub use family::{FamilyMemberSource, FontFamily, OutputFormat};
pub use info::{FontInfo, Foundry, InfoOverride, Person};
pub use manifest::{FamilyMember, Manifest, SlopeInfo, WeightInfo, WidthInfo};
