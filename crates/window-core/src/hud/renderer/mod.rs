mod background;
mod entity;
mod fragment;
mod fullscreen;
mod glsl;
mod hover;
mod text;

use crate::ir::Rgb;

pub(super) use background::{
    render_v150_dynamic_text_background_shader, render_v150_text_background_shader,
    render_v330_define_variant_text_background_shader, render_v330_text_background_shader,
};
pub(super) use entity::render_horse_entity_vertex;
pub(super) use fragment::{
    render_dynamic_text_intensity_fragment, render_v150_text_intensity_fragment,
    render_v330_define_variant_text_fragment,
};
pub(super) use fullscreen::{render_position_tex_color_fragment, render_position_tex_color_vertex};
pub(super) use text::{
    render_v150_dynamic_text_shader, render_v150_text_shader, render_v330_define_variant_text_shader,
    render_v330_text_shader,
};

pub(super) fn marker_id(marker: Rgb) -> u16 {
    debug_assert_eq!(marker.g, 0);
    ((marker.r as u16) << 8) | marker.b as u16
}
