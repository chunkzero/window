use std::collections::BTreeSet;

use super::markers::marker_color;
use super::profile::ShaderProfile;
use super::{emit, renderer, segment_markers};

use crate::geometry::Rect;
use crate::ir::{Align, HudChannel, HudShader, LaidOutHud, Rgb, SlotIr};

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
            font: None,
            repeat: None,
        }],
        warnings: Vec::new(),
    }
}

#[test]
fn maps_pack_formats_to_profiles() {
    assert_eq!(ShaderProfile::for_pack_format(9), Some(ShaderProfile::Pack9To13LegacyFog));
    assert_eq!(ShaderProfile::for_pack_format(42), Some(ShaderProfile::Pack42To62NamespacedFog));
    assert_eq!(ShaderProfile::for_pack_format(84), Some(ShaderProfile::Pack84SampleLightmap));
    assert_eq!(ShaderProfile::for_pack_format(88), Some(ShaderProfile::Pack85To88DefineVariants));
    assert_eq!(ShaderProfile::for_pack_format(8), None);
    assert_eq!(ShaderProfile::for_pack_format(89), None);
}

#[test]
fn rejects_shader_huds_without_a_shader_profile() {
    let hud = shader_hud("status", "coins");

    assert!(emit(None, &[&hud]).is_err());
    assert!(emit(Some(89), &[&hud]).is_err());
}

#[test]
fn define_variant_profile_overrides_core_text_programs() {
    let hud = shader_hud("status", "coins");
    let output = emit(Some(88), &[&hud]).unwrap();
    let paths = output.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();

    assert_eq!(
        paths,
        [
            "assets/minecraft/shaders/core/text.vsh",
            "assets/minecraft/shaders/core/text.fsh",
            "assets/minecraft/shaders/core/text_background.vsh",
        ]
    );
    let vertex = &output.files[0].contents;
    assert!(vertex.contains("#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)\nin ivec2 UV2;"));
    assert!(vertex.contains("window_hud_apply(pos, Color);"));
    assert!(vertex.contains("vertexColor = window_hud_vertex_color(Color, vec4(1.0), Position.y);"));
    assert!(output.files[1].contents.contains("if (windowHudTextureMode != 1) {\n        texColor = texColor.rrrr;"));
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
    let output = emit(Some(84), &[&hud]).unwrap();
    let shader =
        output.files.iter().find(|file| file.path.ends_with("rendertype_text.vsh")).expect("text shader is emitted");

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
    let output = emit(Some(84), &[&hud]).unwrap();
    let shader =
        output.files.iter().find(|file| file.path.ends_with("rendertype_text.vsh")).expect("text shader is emitted");

    assert!(shader.contents.contains("const int WINDOW_HUD_SOURCE_BOTTOMS[WINDOW_HUD_RULES] = int[](59);"));
    assert!(shader.contents.contains("const int WINDOW_HUD_HEIGHTS[WINDOW_HUD_RULES] = int[](20);"));
    assert!(
        shader.contents.contains("guiHeight - float(WINDOW_HUD_SOURCE_BOTTOMS[i]) - WINDOW_HUD_ACTIONBAR_SOURCE_INSET")
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
