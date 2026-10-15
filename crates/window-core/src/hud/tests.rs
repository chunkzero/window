use std::collections::BTreeSet;

use super::markers::marker_color;
use super::profile::ShaderProfile;
use super::rule::{HudShaderSegment, HudShaderTexture, rules};
use super::{TintedSegment, emit, renderer, segment_markers};

use crate::geometry::Rect;
use crate::ir::{Align, ArtIr, Handle, HandleKind, HudChannel, HudShader, LaidOutHud, Rgb, SlotIr};

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
            fit: Default::default(),
            tint: None,
            source: None,
        }],
        switches: Vec::new(),
        art: Vec::new(),
        layers: Vec::new(),
        handles: Default::default(),
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

    assert!(emit(None, &[&hud], &[0], false, false).is_err());
    assert!(emit(Some(89), &[&hud], &[0], false, false).is_err());
}

#[test]
fn define_variant_profile_overrides_core_text_programs() {
    let hud = shader_hud("status", "coins");
    let output = emit(Some(88), &[&hud], &[0], false, false).unwrap();
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
fn hover_outlines_join_the_define_variant_shaders_with_or_without_huds() {
    let hud = shader_hud("status", "coins");
    let names = |output: &super::ShaderOutput| {
        output.files.iter().map(|file| file.path.rsplit('/').next().unwrap().to_string()).collect::<Vec<_>>()
    };

    let both = emit(Some(88), &[&hud], &[0], true, false).unwrap();
    assert_eq!(names(&both), ["text.vsh", "text.fsh", "text_background.vsh"]);
    assert!(both.files[0].contents.contains("const int WINDOW_HUD_RULES = 1;"));
    assert!(both.files[0].contents.contains("#endif\n#ifdef IS_GUI\n    window_hover_apply();\n#endif\n}"));
    assert!(both.files[1].contents.contains("if (windowHudTextureMode != 1) {"));
    assert!(both.files[1].contents.contains("if (windowHover == 1) {"));

    // Without a shader-placed HUD there are no rule tables and no text background to relocate.
    let alone = emit(Some(88), &[], &[], true, false).unwrap();
    assert_eq!(names(&alone), ["text.vsh", "text.fsh"]);
    assert!(!alone.files[0].contents.contains("WINDOW_HUD_RULES"));
    assert!(alone.files[0].contents.contains("window_hover_apply();"));
    assert_eq!(alone.files[1].contents, both.files[1].contents);

    assert!(emit(Some(88), &[], &[], false, false).unwrap().files.is_empty());
    let err = emit(Some(84), &[&hud], &[0], true, false).unwrap_err().to_string();
    assert!(err.contains("`coreShaders.hover` needs pack_format 85 through 88"), "{err}");
}

#[test]
fn fullscreen_branch_joins_the_text_shader_and_hides_the_hud_strip() {
    let hud = shader_hud("status", "coins");
    let names = |output: &super::ShaderOutput| {
        output.files.iter().map(|file| file.path.rsplit('/').next().unwrap().to_string()).collect::<Vec<_>>()
    };

    let all = emit(Some(88), &[&hud], &[0], true, true).unwrap();
    assert_eq!(
        names(&all),
        ["text.vsh", "text.fsh", "text_background.vsh", "position_tex_color.vsh", "position_tex_color.fsh"]
    );
    let vertex = &all.files[0].contents;
    assert!(vertex.contains("#moj_import <minecraft:projection.glsl>\n#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)\n#moj_import <minecraft:window_fullscreen.glsl>\n#endif"));
    // Marked glyphs leave before the HUD and hover branches, which still follow.
    let main = &vertex[vertex.find("void main()").unwrap()..];
    assert!(main.starts_with("void main() {\n#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)\n    if (window_fs_is_marker(Color)) {\n        window_fs_apply();\n        return;\n    }\n#endif\n    vec3 pos = Position;\n    window_hud_apply(pos, Color);"), "{main}");
    assert!(main.contains("window_hover_apply();"));
    let without = emit(Some(88), &[&hud], &[0], true, false).unwrap();
    assert_eq!(all.files[1], without.files[1]);
    assert_eq!(all.files[2], without.files[2]);

    let alone = emit(Some(88), &[], &[], false, true).unwrap();
    assert_eq!(names(&alone), ["text.vsh", "text.fsh", "position_tex_color.vsh", "position_tex_color.fsh"]);
    assert!(alone.files[0].contents.contains("void window_fs_apply()"));
    let fragment = &alone.files[3].contents;
    assert!(fragment.contains("const vec2 WINDOW_FS_HUD_STRIP = vec2(96.0, 56.0);"), "{fragment}");
    assert!(alone.files[2].contents.contains("float GameTime;\n    int MenuBlurRadius;\n    int UseRgss;\n};"));

    for pack_format in [None, Some(84), Some(89)] {
        let err = emit(pack_format, &[], &[], false, true).unwrap_err().to_string();
        assert!(err.contains("full-screen windows"), "{err}");
    }
}

#[test]
fn fullscreen_depth_ignores_the_camera_projection() {
    let alone = emit(Some(88), &[], &[], false, true).unwrap();
    let vertex = &alone.files[0].contents;
    let depth = &vertex[vertex.find("float window_fs_depth(int layer)").unwrap()..];
    let depth = &depth[..depth.find("\n}").unwrap()];
    assert!(!depth.contains("ProjMat"), "{depth}");
    let step = format!("const float WINDOW_FS_LAYER_DEPTH_STEP = {:?};", crate::fullscreen::LAYER_DEPTH_STEP);
    assert!(vertex.contains(&step), "{vertex}");

    // Reversed-Z depth is about near / distance, so the layers cover only geometry within a sliver of the near plane:
    // a particle a millimetre past it must be covered by the backdrop. Every layer survives the 32-bit float
    // depth buffer as a distinct value below the near plane.
    let near = 0.05_f64;
    let depths: Vec<f64> = (0..=255).map(crate::fullscreen::layer_depth).collect();
    let backdrop_distance = near / depths[0];
    assert!(backdrop_distance - near < 2e-5, "{backdrop_distance}");
    assert!(near / 0.051 < depths[0], "a nearby particle is covered by the backdrop");
    assert!(depths[255] < 1.0, "{depths:?}");
    let stored: Vec<f32> = depths.iter().map(|&depth| depth as f32).collect();
    assert!(stored.windows(2).all(|pair| pair[0] < pair[1]), "layers are strictly ordered, nearer when higher");
    assert!(stored.iter().zip(&depths).all(|(&stored, &depth)| f64::from(stored) == depth), "exact in f32");
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
    let markers = segment_markers(&[&hud_a, &hud_b]).unwrap();
    let again = segment_markers(&[&hud_a, &hud_b]).unwrap();

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
fn tinted_segments_get_a_marker_per_palette_entry_drawn_in_its_color() {
    let mut hud = shader_hud("status", "coins");
    hud.slots[0].tint = Some("tone".into());
    hud.art.push(ArtIr { name: "art/0".into(), draws: Vec::new(), tint: Some("tone".into()), bar: None });
    let tone = Handle {
        kind: HandleKind::Tint,
        values: vec!["ok".into(), "warn".into()],
        colors: vec!["#3ddc84".into(), "#ffb020".into()],
        shape: Vec::new(),
        initial: None,
        selectable: false,
        only: Vec::new(),
        uses: Vec::new(),
    };
    hud.handles.insert("tone".into(), tone);
    let markers = segment_markers(&[&hud]).unwrap();
    let segments = &rules(&[&hud], &[0], &markers)[0].segments;

    // The static segment, the slot's own marker, and one marker per palette entry of the art and of the slot.
    assert_eq!(segments.len(), 6);
    assert_eq!(segments.iter().map(|segment| segment.marker).collect::<BTreeSet<_>>().len(), 6);
    let palette = [("ok", Rgb::new(0x3d, 0xdc, 0x84)), ("warn", Rgb::new(0xff, 0xb0, 0x20))];
    let tinted = [
        (TintedSegment::Art("art/0"), HudShaderTexture::Rgba),
        (TintedSegment::Slot("coins"), HudShaderTexture::Intensity),
    ];
    for (segment, texture) in tinted {
        for (value, display) in palette {
            let marker = markers.tint_marker("status", &segment, value);
            assert!(segments.contains(&HudShaderSegment { marker, display, texture }), "{segment:?} {value}");
        }
    }
}

#[test]
fn generated_shader_matches_only_known_green_zero_ids() {
    let hud = shader_hud("status", "coins");
    let markers = segment_markers(&[&hud]).unwrap();
    let static_id = renderer::marker_id(markers.static_marker("status"));
    let slot_id = renderer::marker_id(markers.slot_marker("status", "coins"));
    let output = emit(Some(84), &[&hud], &[0], false, false).unwrap();
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
    let output = emit(Some(84), &[&hud], &[0], false, false).unwrap();
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
