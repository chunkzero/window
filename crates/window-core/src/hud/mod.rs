//! Core-shader generation.
//!
//! This module owns shader-backed HUD emission: marker allocation, relocation
//! rules, version-profile selection, and version-specific core text shader
//! rendering. The pack_format 85-88 text shaders also draw hover outlines and
//! full-screen windows.

use crate::ir::{LaidOutHud, Rgb};
use crate::{Error, Result};

mod markers;
mod profile;
mod renderer;
mod rule;
#[cfg(test)]
mod tests;

pub use markers::{SegmentMarkers, TintedSegment, segment_markers, tinted_segments};
use profile::ShaderProfile;
use rule::{HudShaderRule, rules};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderFile {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShaderOutput {
    pub files: Vec<ShaderFile>,
    pub warnings: Vec<String>,
}

/// Color of the corner pixels of a hover outline glyph, by which the text shader finds hover glyphs.
pub const HOVER_GLYPH_ID: Rgb = Rgb { r: 0x57, g: 0x48, b: 0x56 };

/// Blue channel of the pixel beside each corner of a hover outline glyph, whose red and green hold the glyph's
/// width and height minus one. A glyph is a hover glyph only with both this and [`HOVER_GLYPH_ID`].
pub const HOVER_SIZE_MARK: u8 = 0xa9;

/// Emits core text shaders that relocate shader-placed HUDs, where `row_offsets` holds the rows each of `huds` is
/// baked below its source origin, that with `hover` move hover outline glyphs out of tooltips onto their regions, and
/// that with `fullscreen` draw full-screen windows on their canvas and hide the HUD strip while one is open.
///
/// Fails when shader-placed HUDs exist, `hover` is set, or `fullscreen` is set but `pack_format` is unknown or has
/// no shader profile, since baked marker colors, HUD placement, hover glyphs, and full-screen canvases only render
/// correctly through these shaders. `hover` and `fullscreen` need the pack_format 85-88 profile.
pub fn emit(
    pack_format: Option<u32>,
    huds: &[&LaidOutHud],
    row_offsets: &[i32],
    hover: bool,
    fullscreen: bool,
) -> Result<ShaderOutput> {
    let markers = segment_markers(huds)?;
    let rules = rules(huds, row_offsets, &markers);
    if rules.is_empty() && !hover && !fullscreen {
        return Ok(ShaderOutput::default());
    }
    let (option, remedy) = if fullscreen {
        ("the project has full-screen windows", "remove the full-screen windows")
    } else if rules.is_empty() {
        ("`coreShaders.hover` is enabled", "turn `coreShaders.hover` off")
    } else {
        ("`coreShaders.hud` is enabled with shader-placed HUDs", "remove the HUD `shader` placement")
    };

    let Some(pack_format) = pack_format else {
        return Err(Error::Validation(format!(
            "{option}, but Window could not determine pack_format; set the pack format or {remedy}"
        )));
    };

    let Some(profile) = ShaderProfile::for_pack_format(pack_format) else {
        return Err(Error::Validation(format!(
            "{option}, but pack_format {pack_format} has no core shader profile; target a supported pack format or \
             {remedy}"
        )));
    };

    if fullscreen && profile != ShaderProfile::Pack85To88DefineVariants {
        return Err(Error::Validation(format!(
            "full-screen windows are drawn by a core text shader Window generates only for pack_format 85 through \
             88, but the pack targets pack_format {pack_format}; target pack_format 85 through 88 or remove the \
             full-screen windows"
        )));
    }
    if hover && profile != ShaderProfile::Pack85To88DefineVariants {
        return Err(Error::Validation(format!(
            "`coreShaders.hover` needs pack_format 85 through 88, but the pack targets pack_format {pack_format}; \
             target a supported pack format or turn `coreShaders.hover` off"
        )));
    }

    let fallback = if rules.is_empty() { "" } else { ", so keep actionbar/bossbar/sidebar fallback enabled" };
    Ok(ShaderOutput {
        files: shader_files(profile, &rules, hover, fullscreen),
        warnings: vec![format!(
            "emitted generated core text shaders for {profile}; core shader overrides are \
             client-version-sensitive{fallback}"
        )],
    })
}

fn shader_files(profile: ShaderProfile, rules: &[HudShaderRule], hover: bool, fullscreen: bool) -> Vec<ShaderFile> {
    profile.sources(rules, hover, fullscreen).into_iter().map(|(name, contents)| shader_file(name, contents)).collect()
}

fn shader_file(name: &str, contents: String) -> ShaderFile {
    ShaderFile { path: format!("assets/minecraft/shaders/core/{name}"), contents }
}
