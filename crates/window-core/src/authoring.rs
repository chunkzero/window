//! TypeScript-authored project model.
//!
//! The rpp plugin loads `window/**/*.ts` definitions, merges their exported values,
//! JSON-encodes the result, and hands it to this module. The accepted JSON is
//! intentionally close to the public model: frames and sprites live under
//! `theme`, and windows carry a tree of typed elements.

mod button;
mod element;
mod hud;
mod insets;
mod parse;
mod patterns;
mod project;
mod theme;

#[cfg(test)]
mod tests;

use self::project::ProjectDto;
use crate::model::{Hud, Theme, Window};
use crate::{Error, Result};

/// Everything parsed from a TypeScript-authored Window project.
#[derive(Clone, Debug, Default)]
pub struct ParsedProject {
    /// Merged theme definitions (global names; duplicates are errors).
    pub theme: Theme,
    /// All windows (duplicate window names are errors).
    pub windows: Vec<Window>,
    /// All HUDs (duplicate HUD names are errors).
    pub huds: Vec<Hud>,
    /// Build/runtime options supplied by the rpp host.
    pub options: BuildOptions,
    /// Target Minecraft/resource-pack version metadata supplied by the rpp host.
    pub target: PackTarget,
}

/// Build options supplied by the rpp plugin host.
#[derive(Clone, Debug, Default)]
pub struct BuildOptions {
    /// Whether to emit generated core shader overrides for HUD relocation.
    pub hud_shaders: bool,
}

/// Target pack metadata supplied by the rpp plugin host.
#[derive(Clone, Debug, Default)]
pub struct PackTarget {
    /// Resource pack format, when known.
    pub pack_format: Option<u32>,
}

/// Parse the plugin's JSON project payload into the authored model.
pub fn project_from_json(bytes: &[u8]) -> Result<ParsedProject> {
    let dto: ProjectDto = serde_json::from_slice(bytes)
        .map_err(|error| Error::Parse { path: "window lua project".into(), message: error.to_string() })?;
    dto.into_project()
}
