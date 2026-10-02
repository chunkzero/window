//! HUD core-shader generation.
//!
//! This module owns shader-backed HUD emission: marker allocation, relocation
//! rules, version-profile selection, and version-specific `rendertype_text`
//! rendering.

use crate::ir::LaidOutHud;

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

pub fn emit(pack_format: Option<u32>, huds: &[&LaidOutHud]) -> ShaderOutput {
    let markers = segment_markers(huds);
    let rules = rules(huds, &markers);
    if rules.is_empty() {
        return ShaderOutput::default();
    }

    let Some(pack_format) = pack_format else {
        return skipped(
            "hud_shaders was enabled, but Window could not determine pack_format; \
             skipping generated core shader override"
                .into(),
        );
    };

    let Some(profile) = ShaderProfile::for_pack_format(pack_format) else {
        return skipped(format!(
            "hud_shaders was enabled, but pack_format {pack_format} is outside Window's \
             supported shader bands; using vanilla fallback HUD channels only"
        ));
    };

    ShaderOutput {
        files: shader_files(profile, &rules),
        warnings: vec![format!(
            "emitted generated rendertype_text core shaders for {profile}; core shader overrides are \
             client-version-sensitive, so keep actionbar/bossbar/sidebar fallback enabled"
        )],
    }
}

fn skipped(warning: String) -> ShaderOutput {
    ShaderOutput { files: Vec::new(), warnings: vec![warning] }
}

fn shader_files(profile: ShaderProfile, rules: &[HudShaderRule]) -> Vec<ShaderFile> {
    vec![
        shader_file("rendertype_text.vsh", profile.render_text(rules, false, false)),
        shader_file("rendertype_text_intensity.vsh", profile.render_text(rules, true, false)),
        shader_file("rendertype_text_intensity.fsh", profile.render_text_intensity_fragment(false)),
        shader_file("rendertype_text_see_through.vsh", profile.render_text(rules, false, true)),
        shader_file("rendertype_text_intensity_see_through.vsh", profile.render_text(rules, true, true)),
        shader_file("rendertype_text_intensity_see_through.fsh", profile.render_text_intensity_fragment(true)),
        shader_file("rendertype_text_background.vsh", profile.render_text_background(rules, false)),
        shader_file("rendertype_text_background_see_through.vsh", profile.render_text_background(rules, true)),
    ]
}

fn shader_file(name: &str, contents: String) -> ShaderFile {
    ShaderFile { path: format!("assets/minecraft/shaders/core/{name}"), contents }
}
