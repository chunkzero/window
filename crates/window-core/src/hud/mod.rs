//! HUD core-shader generation.
//!
//! This module owns shader-backed HUD emission: marker allocation, relocation
//! rules, version-profile selection, and version-specific core text shader
//! rendering.

use crate::ir::LaidOutHud;
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

/// Emits core text shaders that relocate shader-placed HUDs.
///
/// Fails when shader-placed HUDs exist but `pack_format` is unknown or has no shader profile,
/// since their baked marker colors and placement only render correctly through these shaders.
pub fn emit(pack_format: Option<u32>, huds: &[&LaidOutHud]) -> Result<ShaderOutput> {
    let markers = segment_markers(huds);
    let rules = rules(huds, &markers);
    if rules.is_empty() {
        return Ok(ShaderOutput::default());
    }

    let Some(pack_format) = pack_format else {
        return Err(Error::Validation(
            "hud_shaders is enabled with shader-placed HUDs, but Window could not determine pack_format; \
             set the pack format or remove the HUD `shader` placement"
                .into(),
        ));
    };

    let Some(profile) = ShaderProfile::for_pack_format(pack_format) else {
        return Err(Error::Validation(format!(
            "hud_shaders is enabled with shader-placed HUDs, but pack_format {pack_format} has no core shader \
             profile; target a supported pack format or remove the HUD `shader` placement"
        )));
    };

    Ok(ShaderOutput {
        files: shader_files(profile, &rules),
        warnings: vec![format!(
            "emitted generated core text shaders for {profile}; core shader overrides are \
             client-version-sensitive, so keep actionbar/bossbar/sidebar fallback enabled"
        )],
    })
}

fn shader_files(profile: ShaderProfile, rules: &[HudShaderRule]) -> Vec<ShaderFile> {
    profile.sources(rules).into_iter().map(|(name, contents)| shader_file(name, contents)).collect()
}

fn shader_file(name: &str, contents: String) -> ShaderFile {
    ShaderFile { path: format!("assets/minecraft/shaders/core/{name}"), contents }
}
