//! HUD core-shader generation.
//!
//! This module owns shader-backed HUD emission: marker allocation, relocation
//! rules, version-profile selection, and version-specific core text shader
//! rendering.

use std::ops::RangeInclusive;

use serde_json::{Value, json};

use crate::authoring::{FormatRange, FormatVersion};
use crate::ir::LaidOutHud;
use crate::{Error, Result};

mod markers;
mod profile;
mod renderer;
mod rule;
#[cfg(test)]
mod tests;

pub use markers::{SegmentMarkers, segment_markers};
use profile::{SUPPORTED_FORMATS, ShaderProfile};
use rule::{HudShaderRule, rules};

/// The first resource pack format that loads `pack.mcmeta` overlays.
const FIRST_OVERLAY_FORMAT: u32 = 18;
/// Overlay entries reaching below this format also need the legacy `formats` field.
const FIRST_MIN_MAX_FORMAT: u32 = 65;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderFile {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShaderOutput {
    pub files: Vec<ShaderFile>,
    /// `pack.mcmeta` overlay entries for the profiles placed in overlay directories.
    pub overlays: Vec<Value>,
    pub warnings: Vec<String>,
}

/// Emits core text shaders that relocate shader-placed HUDs.
///
/// The profile for the lowest declared format goes in the pack root; every later profile the
/// range reaches goes in its own overlay directory. Fails when shader-placed HUDs exist but the
/// pack formats are unknown, any declared format has no shader profile, or a profile change
/// falls below the first format that supports overlays, since baked marker colors and placement
/// only render correctly through these shaders.
pub fn emit(formats: Option<FormatRange>, huds: &[&LaidOutHud]) -> Result<ShaderOutput> {
    let markers = segment_markers(huds);
    let rules = rules(huds, &markers);
    if rules.is_empty() {
        return Ok(ShaderOutput::default());
    }

    let Some(formats) = formats else {
        return Err(Error::Validation(
            "hud_shaders is enabled with shader-placed HUDs, but Window could not determine the pack format; \
             set the pack format or remove the HUD `shader` placement"
                .into(),
        ));
    };
    let (min, max) = (formats.min.major(), formats.max.major());
    if min > max {
        return Err(Error::Validation(format!(
            "pack declares min_format {min} above max_format {max}; fix the pack.mcmeta format range"
        )));
    }
    let unsupported: Vec<String> = [
        (min < *SUPPORTED_FORMATS.start()).then(|| format_span(min..=max.min(SUPPORTED_FORMATS.start() - 1))),
        (max > *SUPPORTED_FORMATS.end()).then(|| format_span(min.max(SUPPORTED_FORMATS.end() + 1)..=max)),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !unsupported.is_empty() {
        return Err(Error::Validation(format!(
            "hud_shaders is enabled with shader-placed HUDs, but declared pack formats {} have no core shader profile \
             (supported: {}); narrow the pack's declared formats or remove the HUD `shader` placement",
            unsupported.join(" and "),
            format_span(SUPPORTED_FORMATS),
        )));
    }

    let mut output = ShaderOutput::default();
    let mut labels = Vec::new();
    for (i, (profile, majors)) in ShaderProfile::covering(min, max).enumerate() {
        labels.push(profile.to_string());
        if i == 0 {
            output.files.extend(shader_files("", profile, &rules));
            continue;
        }
        let (start, end) = (*majors.start(), *majors.end());
        if start < FIRST_OVERLAY_FORMAT {
            return Err(Error::Validation(format!(
                "pack formats {} need the {profile}, but formats before {FIRST_OVERLAY_FORMAT} cannot load \
                 overlays; raise the pack's min_format to at least {start} or remove the HUD `shader` placement",
                format_span(start..=end.min(FIRST_OVERLAY_FORMAT - 1)),
            )));
        }
        let directory = format!("window_hud_{start}_{end}");
        output.files.extend(shader_files(&format!("{directory}/"), profile, &rules));
        output.overlays.push(overlay_entry(&directory, majors, formats.max));
    }
    output.warnings.push(format!(
        "emitted generated core text shaders for {}; core shader overrides are client-version-sensitive, \
         so keep actionbar/bossbar/sidebar fallback enabled",
        labels.join(", ")
    ));
    Ok(output)
}

/// An overlay entry for `majors`, ending at the declared `max` when it reaches it.
fn overlay_entry(directory: &str, majors: RangeInclusive<u32>, max: FormatVersion) -> Value {
    let (start, end) = (*majors.start(), *majors.end());
    let max_format = if end == max.major() { max } else { FormatVersion::Major(end) };
    let mut entry = json!({ "directory": directory, "min_format": start, "max_format": max_format });
    if start < FIRST_MIN_MAX_FORMAT {
        entry["formats"] = json!([start, end]);
    }
    entry
}

fn format_span(formats: RangeInclusive<u32>) -> String {
    if formats.start() == formats.end() {
        formats.start().to_string()
    } else {
        format!("{}-{}", formats.start(), formats.end())
    }
}

fn shader_files(prefix: &str, profile: ShaderProfile, rules: &[HudShaderRule]) -> Vec<ShaderFile> {
    profile
        .sources(rules)
        .into_iter()
        .map(|(name, contents)| ShaderFile { path: format!("{prefix}assets/minecraft/shaders/core/{name}"), contents })
        .collect()
}
