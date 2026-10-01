use std::fmt;

use super::HudShaderRule;
use super::renderer::{
    render_dynamic_text_intensity_fragment, render_v150_dynamic_text_background_shader,
    render_v150_dynamic_text_shader, render_v150_text_background_shader, render_v150_text_intensity_fragment,
    render_v150_text_shader, render_v330_text_background_shader, render_v330_text_shader,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShaderProfile {
    Pack9To13LegacyFog,
    Pack14To41Fog,
    Pack42To62NamespacedFog,
    Pack63To83DynamicTransforms,
    Pack84PlusSampleLightmap,
}

impl ShaderProfile {
    pub(super) fn for_pack_format(pack_format: u32) -> Option<Self> {
        match pack_format {
            9..=13 => Some(Self::Pack9To13LegacyFog),
            14..=41 => Some(Self::Pack14To41Fog),
            42..=62 => Some(Self::Pack42To62NamespacedFog),
            63..=83 => Some(Self::Pack63To83DynamicTransforms),
            84.. => Some(Self::Pack84PlusSampleLightmap),
            _ => None,
        }
    }

    pub(super) fn render_text(self, rules: &[HudShaderRule], texture_mode_output: bool, see_through: bool) -> String {
        match self {
            Self::Pack9To13LegacyFog => render_v150_text_shader(rules, false, true, texture_mode_output, see_through),
            Self::Pack14To41Fog => render_v150_text_shader(rules, false, false, texture_mode_output, see_through),
            Self::Pack42To62NamespacedFog => {
                render_v150_text_shader(rules, true, false, texture_mode_output, see_through)
            }
            Self::Pack63To83DynamicTransforms => {
                render_v150_dynamic_text_shader(rules, texture_mode_output, see_through)
            }
            Self::Pack84PlusSampleLightmap => render_v330_text_shader(rules, texture_mode_output, see_through),
        }
    }

    pub(super) fn render_text_background(self, rules: &[HudShaderRule], see_through: bool) -> String {
        match self {
            Self::Pack9To13LegacyFog => render_v150_text_background_shader(rules, false, true, see_through),
            Self::Pack14To41Fog => render_v150_text_background_shader(rules, false, false, see_through),
            Self::Pack42To62NamespacedFog => render_v150_text_background_shader(rules, true, false, see_through),
            Self::Pack63To83DynamicTransforms => render_v150_dynamic_text_background_shader(rules, see_through),
            Self::Pack84PlusSampleLightmap => render_v330_text_background_shader(rules, see_through),
        }
    }

    pub(super) fn render_text_intensity_fragment(self, see_through: bool) -> String {
        match self {
            Self::Pack9To13LegacyFog | Self::Pack14To41Fog => {
                render_v150_text_intensity_fragment(false, false, see_through)
            }
            Self::Pack42To62NamespacedFog => render_v150_text_intensity_fragment(true, false, see_through),
            Self::Pack63To83DynamicTransforms => render_dynamic_text_intensity_fragment(150, see_through),
            Self::Pack84PlusSampleLightmap => render_dynamic_text_intensity_fragment(330, see_through),
        }
    }
}

impl fmt::Display for ShaderProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Pack9To13LegacyFog => "pack_format 9-13 GLSL 150 legacy-fog text shader",
            Self::Pack14To41Fog => "pack_format 14-41 GLSL 150 text shader",
            Self::Pack42To62NamespacedFog => "pack_format 42-62 GLSL 150 namespaced-import text shader",
            Self::Pack63To83DynamicTransforms => "pack_format 63-83 GLSL 150 dynamic-transform text shader",
            Self::Pack84PlusSampleLightmap => "pack_format 84+ GLSL 330 sample-lightmap text shader",
        };
        f.write_str(label)
    }
}
