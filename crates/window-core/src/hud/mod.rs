//! HUD core-shader generation.
//!
//! This module owns shader-backed HUD emission: marker allocation, relocation
//! rules, version-profile selection, and version-specific `rendertype_text`
//! rendering.

use std::collections::{BTreeMap, BTreeSet};

use xxhash_rust::xxh3::xxh3_64;

use crate::ir::{LaidOutHud, Rgb};

mod profile;
mod renderer;

use profile::ShaderProfile;

const MARKER_SPAN: u16 = u16::MAX;

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SegmentMarkers {
    markers: BTreeMap<String, Rgb>,
}

impl SegmentMarkers {
    pub fn static_marker(&self, hud: &str) -> Rgb {
        self.markers[&static_key(hud)]
    }

    pub fn slot_marker(&self, hud: &str, slot: &str) -> Rgb {
        self.markers[&slot_key(hud, slot)]
    }
}

pub fn segment_markers(huds: &[&LaidOutHud]) -> SegmentMarkers {
    let mut keys = BTreeSet::new();
    for hud in huds {
        if hud.shader.is_none() {
            continue;
        }
        keys.insert(static_key(&hud.name));
        for slot in &hud.slots {
            keys.insert(slot_key(&hud.name, &slot.name));
        }
    }
    SegmentMarkers { markers: allocate_marker_colors(&keys) }
}

fn static_key(hud: &str) -> String {
    format!("hud/{hud}/static")
}

fn slot_key(hud: &str, slot: &str) -> String {
    format!("hud/{hud}/slot/{slot}")
}

fn allocate_marker_colors(keys: &BTreeSet<String>) -> BTreeMap<String, Rgb> {
    let mut used = BTreeSet::new();
    let mut out = BTreeMap::new();
    for key in keys {
        let mut id = marker_start_id(key);
        while used.contains(&id) {
            id = if id == u16::MAX { 1 } else { id + 1 };
        }
        used.insert(id);
        out.insert(key.clone(), marker_color(id));
    }
    out
}

fn marker_start_id(key: &str) -> u16 {
    (xxh3_64(key.as_bytes()) % MARKER_SPAN as u64) as u16 + 1
}

fn marker_color(id: u16) -> Rgb {
    Rgb { r: (id >> 8) as u8, g: 0, b: id as u8 }
}

pub fn emit(pack_format: Option<u32>, huds: &[&LaidOutHud]) -> ShaderOutput {
    let mut warnings = Vec::new();
    let markers = segment_markers(huds);
    let rules = rules(huds, &markers);
    if rules.is_empty() {
        return ShaderOutput { files: Vec::new(), warnings };
    }

    let Some(pack_format) = pack_format else {
        warnings.push(
            "hud_shaders was enabled, but Window could not determine pack_format; \
             skipping generated core shader override"
                .into(),
        );
        return ShaderOutput { files: Vec::new(), warnings };
    };

    let Some(profile) = ShaderProfile::for_pack_format(pack_format) else {
        warnings.push(format!(
            "hud_shaders was enabled, but pack_format {pack_format} is outside Window's \
             supported shader bands; using vanilla fallback HUD channels only"
        ));
        return ShaderOutput { files: Vec::new(), warnings };
    };

    warnings.push(format!(
        "emitted generated rendertype_text core shaders for {profile}; core shader overrides are \
         client-version-sensitive, so keep actionbar/bossbar/sidebar fallback enabled"
    ));

    let text = profile.render_text(&rules, false, false);
    let text_see_through = profile.render_text(&rules, false, true);
    let text_intensity = profile.render_text(&rules, true, false);
    let text_intensity_see_through = profile.render_text(&rules, true, true);
    let text_background = profile.render_text_background(&rules, false);
    let text_background_see_through = profile.render_text_background(&rules, true);
    let text_intensity_fragment = profile.render_text_intensity_fragment(false);
    let text_intensity_see_through_fragment = profile.render_text_intensity_fragment(true);
    ShaderOutput {
        files: vec![
            ShaderFile { path: "assets/minecraft/shaders/core/rendertype_text.vsh".into(), contents: text.clone() },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_intensity.vsh".into(),
                contents: text_intensity.clone(),
            },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_intensity.fsh".into(),
                contents: text_intensity_fragment,
            },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_see_through.vsh".into(),
                contents: text_see_through,
            },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_intensity_see_through.vsh".into(),
                contents: text_intensity_see_through,
            },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_intensity_see_through.fsh".into(),
                contents: text_intensity_see_through_fragment,
            },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_background.vsh".into(),
                contents: text_background.clone(),
            },
            ShaderFile {
                path: "assets/minecraft/shaders/core/rendertype_text_background_see_through.vsh".into(),
                contents: text_background_see_through,
            },
        ],
        warnings,
    }
}

fn rules(huds: &[&LaidOutHud], markers: &SegmentMarkers) -> Vec<HudShaderRule> {
    let mut rules: Vec<HudShaderRule> = Vec::new();

    for hud in huds {
        let Some(shader) = hud.shader else {
            continue;
        };
        let rule = HudShaderRule {
            source_bottom: shader.source_bottom,
            width: hud.width,
            height: hud.height,
            origin_x: shader.origin_x,
            origin_y: shader.origin_y,
            anchor_x: shader.anchor_x,
            anchor_y: shader.anchor_y,
            offset_x: shader.offset_x,
            offset_y: shader.offset_y,
            segments: shader_segments(hud, markers),
        };

        if let Some(existing) = rules.iter_mut().find(|existing| same_placement(existing, &rule)) {
            existing.segments.extend(rule.segments);
            existing.segments.sort();
            existing.segments.dedup();
            continue;
        }

        rules.push(rule);
    }

    rules
}

fn same_placement(a: &HudShaderRule, b: &HudShaderRule) -> bool {
    a.source_bottom == b.source_bottom
        && a.width == b.width
        && a.height == b.height
        && a.origin_x == b.origin_x
        && a.origin_y == b.origin_y
        && a.anchor_x == b.anchor_x
        && a.anchor_y == b.anchor_y
        && a.offset_x == b.offset_x
        && a.offset_y == b.offset_y
}

#[derive(Clone, Debug)]
pub(super) struct HudShaderRule {
    source_bottom: i32,
    width: u32,
    height: u32,
    origin_x: f32,
    origin_y: f32,
    anchor_x: f32,
    anchor_y: f32,
    offset_x: i32,
    offset_y: i32,
    segments: Vec<HudShaderSegment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HudShaderSegment {
    marker: Rgb,
    display: Rgb,
    texture: HudShaderTexture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HudShaderTexture {
    Intensity,
    Rgba,
}

fn shader_segments(hud: &LaidOutHud, markers: &SegmentMarkers) -> Vec<HudShaderSegment> {
    let mut segments = BTreeSet::new();
    segments.insert(HudShaderSegment {
        marker: markers.static_marker(&hud.name),
        display: Rgb { r: 0xff, g: 0xff, b: 0xff },
        texture: HudShaderTexture::Rgba,
    });
    for slot in &hud.slots {
        segments.insert(HudShaderSegment {
            marker: markers.slot_marker(&hud.name, &slot.name),
            display: slot.color,
            texture: HudShaderTexture::Intensity,
        });
    }
    segments.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::profile::ShaderProfile;
    use super::*;

    use crate::geometry::Rect;
    use crate::ir::{Align, HudChannel, HudShader, SlotIr};

    fn shader_hud(name: &str, slot_name: &str) -> LaidOutHud {
        LaidOutHud {
            name: name.into(),
            channel: HudChannel::ActionBar,
            width: 120,
            height: 20,
            shader: Some(HudShader {
                source_bottom: 59,
                origin_x: 0.5,
                origin_y: 0.1,
                anchor_x: 0.5,
                anchor_y: 0.0,
                offset_x: 0,
                offset_y: 0,
            }),
            draws: Vec::new(),
            slots: vec![SlotIr {
                name: slot_name.into(),
                text: None,
                rect: Rect::new(4, 2, 40, 8),
                align: Align::Left,
                color: Rgb { r: 0xff, g: 0xff, b: 0 },
                shadow: false,
                bold: false,
                italic: false,
                underlined: false,
                strikethrough: false,
                obfuscated: false,
                repeat: None,
            }],
            warnings: Vec::new(),
        }
    }

    #[test]
    fn maps_pack_formats_to_profiles() {
        assert_eq!(ShaderProfile::for_pack_format(9), Some(ShaderProfile::Pack9To13LegacyFog));
        assert_eq!(ShaderProfile::for_pack_format(42), Some(ShaderProfile::Pack42To62NamespacedFog));
        assert_eq!(ShaderProfile::for_pack_format(84), Some(ShaderProfile::Pack84PlusSampleLightmap));
        assert_eq!(ShaderProfile::for_pack_format(8), None);
    }

    #[test]
    fn marker_color_uses_green_zero_and_red_blue_id() {
        assert_eq!(marker_color(0x1234), Rgb { r: 0x12, g: 0, b: 0x34 });
        assert_eq!(renderer::marker_id(marker_color(0xabcd)), 0xabcd);
    }

    #[test]
    fn segment_markers_are_green_zero_unique_and_stable() {
        let hud_a = shader_hud("status", "coins");
        let hud_b = shader_hud("run", "wave");
        let markers = segment_markers(&[&hud_a, &hud_b]);
        let again = segment_markers(&[&hud_a, &hud_b]);

        assert_eq!(markers, again);
        let colors = [
            markers.static_marker("status"),
            markers.slot_marker("status", "coins"),
            markers.static_marker("run"),
            markers.slot_marker("run", "wave"),
        ];
        let ids = colors.iter().map(|&color| renderer::marker_id(color)).collect::<BTreeSet<_>>();

        assert!(colors.iter().all(|color| color.g == 0));
        assert!(ids.iter().all(|&id| id != 0));
        assert_eq!(ids.len(), colors.len());
    }

    #[test]
    fn generated_shader_matches_only_known_green_zero_ids() {
        let hud = shader_hud("status", "coins");
        let markers = segment_markers(&[&hud]);
        let static_id = renderer::marker_id(markers.static_marker("status"));
        let slot_id = renderer::marker_id(markers.slot_marker("status", "coins"));
        let output = emit(Some(84), &[&hud]);
        let shader = output
            .files
            .iter()
            .find(|file| file.path.ends_with("rendertype_text.vsh"))
            .expect("text shader is emitted");

        assert!(shader.contents.contains("if (rgb.g != 0)"));
        assert!(shader.contents.contains("marker == WINDOW_HUD_MARKER_IDS[i]"));
        assert!(shader.contents.contains("if (WINDOW_HUD_TEXTURE_MODES[segment] == 1)"));
        assert!(shader.contents.contains("i >= 0 && window_hud_rule_index(y, color) >= 0"));
        assert!(shader.contents.contains("if (color.a < 0.999)"));
        assert!(shader.contents.contains("window_hud_apply(pos, Color);"));
        assert!(shader.contents.contains(&static_id.to_string()));
        assert!(shader.contents.contains(&slot_id.to_string()));
    }

    #[test]
    fn generated_shader_uses_height_independent_actionbar_source_top() {
        let hud = shader_hud("status", "coins");
        let output = emit(Some(84), &[&hud]);
        let shader = output
            .files
            .iter()
            .find(|file| file.path.ends_with("rendertype_text.vsh"))
            .expect("text shader is emitted");

        assert!(shader.contents.contains("const int WINDOW_HUD_SOURCE_BOTTOMS[WINDOW_HUD_RULES] = int[](59);"));
        assert!(shader.contents.contains("const int WINDOW_HUD_HEIGHTS[WINDOW_HUD_RULES] = int[](20);"));
        assert!(
            shader
                .contents
                .contains("guiHeight - float(WINDOW_HUD_SOURCE_BOTTOMS[i]) - WINDOW_HUD_ACTIONBAR_SOURCE_INSET")
        );
        assert!(
            shader.contents.contains(
                "vec2 sourceTopLeft = vec2(\n        guiWidth * 0.5,\n        guiHeight - float(WINDOW_HUD_SOURCE_BOTTOMS[i]) - WINDOW_HUD_ACTIONBAR_SOURCE_INSET\n    );"
            )
        );
        assert!(shader.contents.contains("const float WINDOW_HUD_ACTIONBAR_SOURCE_INSET = 13.0;"));
        assert!(!shader.contents.contains("float(WINDOW_HUD_SOURCE_BOTTOMS[i]) - height"));
        assert!(shader.contents.contains("int sourceTop = sourceBottom + int(WINDOW_HUD_ACTIONBAR_SOURCE_INSET);"));
    }
}
