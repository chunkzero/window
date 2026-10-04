use std::fmt;
use std::ops::RangeInclusive;

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

/// Profiles in ascending, contiguous pack-format order.
const PROFILES: [(ShaderProfile, RangeInclusive<u32>); 6] = [
    (ShaderProfile::Pack9To13LegacyFog, 9..=13),
    (ShaderProfile::Pack14To41Fog, 14..=41),
    (ShaderProfile::Pack42To62NamespacedFog, 42..=62),
    (ShaderProfile::Pack63To83DynamicTransforms, 63..=83),
    (ShaderProfile::Pack84SampleLightmap, 84..=84),
    (ShaderProfile::Pack85To88DefineVariants, 85..=88),
];

/// Pack formats covered by a shader profile.
pub(super) const SUPPORTED_FORMATS: RangeInclusive<u32> = 9..=88;

impl ShaderProfile {
    #[cfg(test)]
    pub(super) fn for_pack_format(pack_format: u32) -> Option<Self> {
        PROFILES.iter().find(|(_, formats)| formats.contains(&pack_format)).map(|(profile, _)| *profile)
    }

    /// The profiles serving `min..=max`, each with the major formats it serves there, in ascending order.
    pub(super) fn covering(min: u32, max: u32) -> impl Iterator<Item = (Self, RangeInclusive<u32>)> {
        PROFILES.into_iter().filter_map(move |(profile, formats)| {
            let start = (*formats.start()).max(min);
            let end = (*formats.end()).min(max);
            (start <= end).then_some((profile, start..=end))
        })
    }

    /// Returns `(file name, source)` pairs for `assets/minecraft/shaders/core`.
    pub(super) fn sources(self, rules: &[HudShaderRule]) -> Vec<(&'static str, String)> {
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
            Self::Pack85To88DefineVariants => vec![
                ("text.vsh", render_v330_define_variant_text_shader(rules)),
                ("text.fsh", render_v330_define_variant_text_fragment()),
                ("text_background.vsh", render_v330_define_variant_text_background_shader(rules)),
            ],
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
