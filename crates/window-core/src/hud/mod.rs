//! HUD core-shader generation.
//!
//! This module owns shader-backed HUD emission: marker allocation, relocation
//! rules, version-profile selection, and version-specific core text shader
//! rendering. The same text shaders also draw window hover outlines.

use crate::ir::{LaidOutHud, Rgb};
use crate::{Error, Result};

mod markers;
mod profile;
mod renderer;
mod rule;
#[cfg(test)]
mod tests;

pub use markers::{SegmentMarkers, segment_markers};
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

/// Color of the first data pixel of a hover outline glyph, which the text shader matches to find hover glyphs.
pub const HOVER_GLYPH_ID: Rgb = Rgb { r: 0x57, g: 0x48, b: 0x56 };

/// Text color of a hover glyph's foreground copy, which the text shader hides; only its shadow copy is moved and drawn.
pub const HOVER_HIDDEN_COLOR: Rgb = Rgb { r: 0x57, g: 0x48, b: 0x01 };

/// Emits core text shaders that relocate shader-placed HUDs and, when `hover_outlines` is set, move hover glyphs
/// out of tooltips onto their buttons.
///
/// Fails when shader-placed HUDs exist but `pack_format` is unknown or has no shader profile,
/// since their baked marker colors and placement only render correctly through these shaders.
/// Hover outlines need the pack_format 85-88 profile.
pub fn emit(pack_format: Option<u32>, huds: &[&LaidOutHud], hover_outlines: bool) -> Result<ShaderOutput> {
    let markers = segment_markers(huds);
    let rules = rules(huds, &markers);
    if rules.is_empty() && !hover_outlines {
        return Ok(ShaderOutput::default());
    }

    let Some(pack_format) = pack_format else {
        return Err(Error::Validation(
            "core text shaders are enabled, but Window could not determine pack_format; set the pack format, \
             remove the HUD `shader` placement, or disable hoverOutlines"
                .into(),
        ));
    };

    let Some(profile) = ShaderProfile::for_pack_format(pack_format) else {
        return Err(Error::Validation(format!(
            "core text shaders are enabled, but pack_format {pack_format} has no core shader profile; target a \
             supported pack format, remove the HUD `shader` placement, or disable hoverOutlines"
        )));
    };

    if hover_outlines && profile != ShaderProfile::Pack85To88DefineVariants {
        return Err(Error::Validation(format!(
            "hoverOutlines needs the pack_format 85-88 core shader profile, but the pack targets pack_format \
             {pack_format}; target a supported pack format or disable hoverOutlines"
        )));
    }

    Ok(ShaderOutput {
        files: shader_files(profile, &rules, hover_outlines),
        warnings: vec![format!(
            "emitted generated core text shaders for {profile}; core shader overrides are \
             client-version-sensitive, so keep actionbar/bossbar/sidebar fallback enabled"
        )],
    })
}

fn shader_files(profile: ShaderProfile, rules: &[HudShaderRule], hover_outlines: bool) -> Vec<ShaderFile> {
    profile.sources(rules, hover_outlines).into_iter().map(|(name, contents)| shader_file(name, contents)).collect()
}

fn shader_file(name: &str, contents: String) -> ShaderFile {
    ShaderFile { path: format!("assets/minecraft/shaders/core/{name}"), contents }
}
