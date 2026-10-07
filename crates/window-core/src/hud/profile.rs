use std::fmt;

use super::renderer::{
    render_dynamic_text_intensity_fragment, render_v150_dynamic_text_background_shader,
    render_v150_dynamic_text_shader, render_v150_text_background_shader, render_v150_text_intensity_fragment,
    render_v150_text_shader, render_v330_define_variant_text_background_shader,
    render_v330_define_variant_text_fragment, render_v330_define_variant_text_shader,
    render_v330_text_background_shader, render_v330_text_shader,
};
use super::rule::HudShaderRule;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShaderProfile {
    Pack9To13LegacyFog,
    Pack14To41Fog,
    Pack42To62NamespacedFog,
    Pack63To83DynamicTransforms,
    Pack84SampleLightmap,
    Pack85To88DefineVariants,
}

impl ShaderProfile {
    pub(super) fn for_pack_format(pack_format: u32) -> Option<Self> {
        match pack_format {
            9..=13 => Some(Self::Pack9To13LegacyFog),
            14..=41 => Some(Self::Pack14To41Fog),
            42..=62 => Some(Self::Pack42To62NamespacedFog),
            63..=83 => Some(Self::Pack63To83DynamicTransforms),
            84 => Some(Self::Pack84SampleLightmap),
            85..=88 => Some(Self::Pack85To88DefineVariants),
            _ => None,
        }
    }

    /// Returns `(file name, source)` pairs for `assets/minecraft/shaders/core`.
    /// Hover outlines are only drawn by the pack_format 85-88 sources; other profiles ignore `hover_outlines`.
    pub(super) fn sources(self, rules: &[HudShaderRule], hover_outlines: bool) -> Vec<(&'static str, String)> {
        match self {
            Self::Pack9To13LegacyFog => rendertype_sources(
                |mode, see| render_v150_text_shader(rules, false, true, mode, see),
                |see| render_v150_text_background_shader(rules, false, true, see),
                |see| render_v150_text_intensity_fragment(false, false, see),
            ),
            Self::Pack14To41Fog => rendertype_sources(
                |mode, see| render_v150_text_shader(rules, false, false, mode, see),
                |see| render_v150_text_background_shader(rules, false, false, see),
                |see| render_v150_text_intensity_fragment(false, false, see),
            ),
            Self::Pack42To62NamespacedFog => rendertype_sources(
                |mode, see| render_v150_text_shader(rules, true, false, mode, see),
                |see| render_v150_text_background_shader(rules, true, false, see),
                |see| render_v150_text_intensity_fragment(true, false, see),
            ),
            Self::Pack63To83DynamicTransforms => rendertype_sources(
                |mode, see| render_v150_dynamic_text_shader(rules, mode, see),
                |see| render_v150_dynamic_text_background_shader(rules, see),
                |see| render_dynamic_text_intensity_fragment(150, see),
            ),
            Self::Pack84SampleLightmap => rendertype_sources(
                |mode, see| render_v330_text_shader(rules, mode, see),
                |see| render_v330_text_background_shader(rules, see),
                |see| render_dynamic_text_intensity_fragment(330, see),
            ),
            Self::Pack85To88DefineVariants => {
                let mut sources = vec![
                    ("text.vsh", render_v330_define_variant_text_shader(rules, hover_outlines)),
                    ("text.fsh", render_v330_define_variant_text_fragment(hover_outlines)),
                ];
                if !rules.is_empty() {
                    sources.push(("text_background.vsh", render_v330_define_variant_text_background_shader(rules)));
                }
                sources
            }
        }
    }
}

/// Lays out the per-pipeline `rendertype_text*` programs used through pack_format 84.
fn rendertype_sources(
    text: impl Fn(bool, bool) -> String,
    background: impl Fn(bool) -> String,
    intensity_fragment: impl Fn(bool) -> String,
) -> Vec<(&'static str, String)> {
    vec![
        ("rendertype_text.vsh", text(false, false)),
        ("rendertype_text_intensity.vsh", text(true, false)),
        ("rendertype_text_intensity.fsh", intensity_fragment(false)),
        ("rendertype_text_see_through.vsh", text(false, true)),
        ("rendertype_text_intensity_see_through.vsh", text(true, true)),
        ("rendertype_text_intensity_see_through.fsh", intensity_fragment(true)),
        ("rendertype_text_background.vsh", background(false)),
        ("rendertype_text_background_see_through.vsh", background(true)),
    ]
}

impl fmt::Display for ShaderProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Pack9To13LegacyFog => "pack_format 9-13 GLSL 150 legacy-fog text shader",
            Self::Pack14To41Fog => "pack_format 14-41 GLSL 150 text shader",
            Self::Pack42To62NamespacedFog => "pack_format 42-62 GLSL 150 namespaced-import text shader",
            Self::Pack63To83DynamicTransforms => "pack_format 63-83 GLSL 150 dynamic-transform text shader",
            Self::Pack84SampleLightmap => "pack_format 84 GLSL 330 sample-lightmap text shader",
            Self::Pack85To88DefineVariants => "pack_format 85-88 GLSL 330 define-variant text shader",
        };
        f.write_str(label)
    }
}
