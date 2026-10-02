//! Shared building blocks for the generated GLSL sources.

use crate::hud::rule::{HudShaderRule, HudShaderTexture};
use crate::ir::Rgb;

#[derive(Clone, Copy, Debug)]
pub(super) enum ScreenSizeSource {
    ProjectionMatrix,
    Globals,
}

pub(super) struct ShaderSegment {
    pub(super) rule: usize,
    pub(super) marker: Rgb,
    pub(super) display: Rgb,
    pub(super) texture: HudShaderTexture,
}

/// Fog declarations of the GLSL 150 shaders that predate the dynamic-transform pack format.
pub(super) struct V150Fog {
    pub(super) import: &'static str,
    pub(super) ivew: &'static str,
    pub(super) fog_shape: &'static str,
    pub(super) vertex_distance_decl: &'static str,
    pub(super) vertex_distance_assign: String,
}

/// Fog declarations of the shaders using split spherical and cylindrical distances.
pub(super) struct DynamicFog {
    pub(super) fog_import: &'static str,
    pub(super) fog_outputs: &'static str,
    pub(super) fog_assign: &'static str,
}

pub(super) fn v150_fog(namespaced_import: bool, legacy_fog: bool, see_through: bool) -> V150Fog {
    let import = if see_through {
        ""
    } else if namespaced_import {
        "#moj_import <minecraft:fog.glsl>"
    } else {
        "#moj_import <fog.glsl>"
    };
    let fog_distance = if legacy_fog {
        "fog_distance(ModelViewMat, IViewRotMat * Position, FogShape)"
    } else {
        "fog_distance(Position, FogShape)"
    };
    V150Fog {
        import,
        ivew: if legacy_fog && !see_through { "\nuniform mat3 IViewRotMat;" } else { "" },
        fog_shape: if see_through { "" } else { "\nuniform int FogShape;" },
        vertex_distance_decl: if see_through { "" } else { "out float vertexDistance;\n" },
        vertex_distance_assign: if see_through {
            String::new()
        } else {
            format!("\n    vertexDistance = {fog_distance};")
        },
    }
}

pub(super) fn dynamic_fog(see_through: bool) -> DynamicFog {
    if see_through {
        return DynamicFog { fog_import: "", fog_outputs: "", fog_assign: "" };
    }
    DynamicFog {
        fog_import: "#moj_import <minecraft:fog.glsl>\n",
        fog_outputs: "out float sphericalVertexDistance;\nout float cylindricalVertexDistance;\n",
        fog_assign: "\n    sphericalVertexDistance = fog_spherical_distance(Position);\n    cylindricalVertexDistance = fog_cylindrical_distance(Position);",
    }
}

/// Returns the vertex-shader output declaration and assignment for the texture mode.
pub(super) fn texture_mode_snippets(texture_mode_output: bool) -> (&'static str, &'static str) {
    if texture_mode_output {
        ("\nflat out int windowHudTextureMode;", "\n    windowHudTextureMode = window_hud_texture_mode(Color);")
    } else {
        ("", "")
    }
}

pub(super) fn join_rule_values<F>(rules: &[HudShaderRule], mut value: F) -> String
where
    F: FnMut(&HudShaderRule) -> String,
{
    rules.iter().map(&mut value).collect::<Vec<_>>().join(", ")
}

pub(super) fn render_gui_size_helper(screen_size: ScreenSizeSource) -> &'static str {
    match screen_size {
        ScreenSizeSource::ProjectionMatrix => {
            r#"
vec2 window_hud_gui_size() {
    return vec2(abs(2.0 / ProjMat[0][0]), abs(2.0 / ProjMat[1][1]));
}
"#
        }
        ScreenSizeSource::Globals => {
            r#"
vec2 window_hud_gui_size() {
    vec2 pixel = abs(vec2(ProjMat[0][0], ProjMat[1][1])) / 2.0;
    float guiScale = max(1.0, round(pixel.x * ScreenSize.x));
    return ScreenSize / guiScale;
}
"#
        }
    }
}

pub(super) fn format_glsl_float(value: f32) -> String {
    let mut s = format!("{value:.6}");
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    s
}

pub(super) fn rgb_vec3(color: Rgb) -> String {
    format!(
        "vec3({}, {}, {})",
        format_glsl_float(color.r as f32 / 255.0),
        format_glsl_float(color.g as f32 / 255.0),
        format_glsl_float(color.b as f32 / 255.0)
    )
}

pub(super) fn segment_table(rules: &[HudShaderRule]) -> Vec<ShaderSegment> {
    rules
        .iter()
        .enumerate()
        .flat_map(|(rule, hud_rule)| {
            hud_rule.segments.iter().copied().map(move |segment| ShaderSegment {
                rule,
                marker: segment.marker,
                display: segment.display,
                texture: segment.texture,
            })
        })
        .collect()
}

pub(super) fn texture_mode(texture: HudShaderTexture) -> i32 {
    match texture {
        HudShaderTexture::Intensity => 0,
        HudShaderTexture::Rgba => 1,
    }
}
