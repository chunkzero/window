use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

use crate::compose::Texture;
use crate::fullscreen::{self, INCLUDE_PATH};
use crate::ir::FullscreenMode;
use crate::manifest::Manifest;
use crate::pipeline::{CompileInput, CompileOutput, compile_project_json};

fn compile(fullscreen: serde_json::Value) -> crate::Result<CompileOutput> {
    compile_at(fullscreen, json!(88))
}

fn compile_at(fullscreen: serde_json::Value, pack_format: serde_json::Value) -> crate::Result<CompileOutput> {
    let project = json!({ "fullscreens": [fullscreen], "target": { "pack_format": pack_format } }).to_string();
    compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new()))
}

/// The layer and color index a marker color encodes, after checking its signature and kind.
fn decode(marker: &str, kind: u32) -> (u32, usize) {
    let bits = u32::from_str_radix(marker.trim_start_matches('#'), 16).unwrap();
    assert_eq!(bits >> 20, fullscreen::MARKER_SIGNATURE, "{marker}");
    assert_eq!(bits >> 18 & 3, kind, "{marker}");
    (bits >> 10 & 0xff, (bits & 0x3ff) as usize)
}

fn include(out: &CompileOutput) -> &str {
    let file = out.files.iter().find(|file| file.path == INCLUDE_PATH).expect("the include is emitted");
    std::str::from_utf8(file.contents.as_bytes()).unwrap()
}

/// The include's color table, as `r, g, b` strings.
fn color_table(include: &str) -> Vec<String> {
    let line = include.lines().find(|line| line.starts_with("const vec3 WINDOW_FS_COLORS")).unwrap();
    let list = line.split_once("vec3[](").unwrap().1.trim_end_matches(");");
    list.split("vec3(")
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.trim_end_matches([')', ',', ' ']).replace(".0", ""))
        .collect()
}

fn panel() -> serde_json::Value {
    json!({ "art": "shape", "kind": "panel", "fill": "#123456", "border_width": 0, "radius": 0, "inset_depth": 0,
            "width": 40, "height": 20 })
}

#[test]
fn compiles_layers_controls_and_the_contract() {
    let tone = json!({ "kind": "tint", "id": "tone", "values": ["ok", "low"], "colors": ["#3ddc84", "#ff4d4d"] });
    let out = compile(json!({
        "name": "menu", "mode": "horse", "backdrop": "#101010",
        "children": [
            { "type": "sprite", "art": panel(), "x": 220, "y": 125 },
            { "type": "slot", "handle": { "kind": "text", "id": "title" }, "width": 60, "x": 10, "y": 200,
              "color": "#ffd700" },
            { "type": "slot", "handle": { "kind": "text", "id": "status" }, "width": 60, "x": 10, "y": 220,
              "color": tone },
            { "type": "region", "on_click": { "kind": "action", "id": "play" }, "width": 40, "height": 20 }
        ]
    }))
    .unwrap();
    let manifest = &out.manifest;
    let menu = &manifest.fullscreens["menu"];
    assert_eq!(menu.surface.mode, FullscreenMode::Horse);
    assert_eq!(menu.surface.size, [480, 270]);
    assert!(!menu.static_text.is_empty());

    let canvas = manifest.fullscreen_canvas.as_ref().expect("the canvas entry is present");
    let table = color_table(include(&out));
    let color = |marker: &str, kind: u32| {
        let (layer, index) = decode(marker, kind);
        (layer, table[index].as_str())
    };
    assert_eq!(color(&menu.static_marker, 0), (1, "255, 255, 255"));
    assert_eq!(color(&menu.markers["title"], 0), (2, "255, 215, 0"));
    assert_eq!(color(&menu.markers["status"], 0).0, 3);
    let status = &menu.slots["status"].tint_markers;
    assert_eq!(color(&status["ok"], 0), (3, "61, 220, 132"));
    assert_eq!(color(&status["low"], 0), (3, "255, 77, 77"));
    assert_eq!(color(menu.backdrop_marker.as_deref().unwrap(), 2), (0, "16, 16, 16"));
    assert_eq!(color(&canvas.cursor.marker, 1), (255, "255, 255, 255"));
    assert_eq!(color(&canvas.overlay_marker, 0), (fullscreen::OVERLAY_LAYER.into(), "255, 255, 255"));

    let play = &menu.controls["play"];
    assert_eq!((play.index, play.x, play.y, play.width, play.height), (0, 0, 0, 40, 20));
    assert!(play.hover && play.action.as_deref() == Some("play"));
    let include = include(&out);
    assert!(include.contains("WINDOW_FS_CONTROL_COUNT = 1;"), "{include}");
    assert!(include.contains("vec4[](vec4(0.0, 0.0, 40.0, 20.0))"), "{include}");

    let parsed = Manifest::from_json(&manifest.to_json_bytes().unwrap()).unwrap();
    assert_eq!(&parsed, manifest);
}

#[test]
fn packs_without_fullscreens_emit_no_contract_or_shaders() {
    let project = json!({ "huds": [{ "name": "h", "children": [] }], "target": { "pack_format": 88 } }).to_string();
    let out = compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    assert!(out.manifest.fullscreen_canvas.is_none());
    assert!(out.files.iter().all(|file| !file.path.contains("fullscreen") && !file.path.contains("/shaders/core/")));
    assert!(out.files.iter().all(|file| file.path != fullscreen::CROSSHAIR_SPRITE_PATH));
}

#[test]
fn full_screen_windows_emit_their_shaders_and_need_the_define_variant_profile() {
    let menu = json!({ "name": "menu", "children": [] });
    let out = compile(menu.clone()).unwrap();
    let shaders: Vec<&str> =
        out.files.iter().filter_map(|file| file.path.strip_prefix("assets/minecraft/shaders/core/")).collect();
    assert_eq!(shaders, ["position_tex_color.fsh", "position_tex_color.vsh", "text.fsh", "text.vsh"]);

    let crosshair = out.files.iter().find(|file| file.path == fullscreen::CROSSHAIR_SPRITE_PATH).unwrap();
    let crosshair = Texture::decode_png(crosshair.contents.as_bytes()).unwrap();
    let alphas: BTreeSet<u8> = crosshair.rgba.chunks(4).map(|texel| texel[3]).collect();
    assert_eq!((crosshair.width, crosshair.height), (15, 15));
    assert_eq!(alphas, BTreeSet::from([0, fullscreen::CROSSHAIR_MARKER_ALPHA]));

    let blanked = json!({ "options": { "vanilla": { "hud": { "crosshair": false } } }, "fullscreens": [menu.clone()],
                          "target": { "pack_format": 88 } });
    let out = compile_project_json(blanked.to_string().as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    let sprites: Vec<_> = out.files.iter().filter(|file| file.path == fullscreen::CROSSHAIR_SPRITE_PATH).collect();
    assert_eq!(sprites.len(), 1);
    assert_eq!(Texture::decode_png(sprites[0].contents.as_bytes()).unwrap().rgba, Texture::transparent(15, 15).rgba);

    let error = compile_at(menu.clone(), json!(84)).unwrap_err().to_string();
    assert!(error.contains("full-screen windows are drawn by a core text shader"), "{error}");
    let error = compile_at(menu, json!(null)).unwrap_err().to_string();
    assert!(error.contains("the project has full-screen windows, but Window could not determine"), "{error}");
}

#[test]
fn rejects_items_runtime_sprites_and_hitbox_models() {
    let reject = |child: serde_json::Value| compile(json!({ "name": "menu", "children": [child] })).unwrap_err();
    let item = json!({ "type": "item", "handle": { "kind": "items", "id": "loot" }, "slots": [0] });
    let error = reject(item).to_string();
    assert!(error.contains("cannot contain inventory element `loot`: real items need inventory slots"), "{error}");

    let sprite = json!({ "type": "sprite_slot", "handle": { "kind": "sprite", "id": "icon" }, "width": 16,
                         "height": 16 });
    let error = reject(sprite).to_string();
    assert!(
        error.contains("cannot contain runtime sprite slot `icon`: full-screen windows draw no runtime"),
        "{error}"
    );

    let region = json!({ "type": "region", "width": 8, "height": 8, "item_model": "window:button" });
    let error = reject(region).to_string();
    assert!(error.contains("sets `item_model`, which draws the hitbox item of inventory slots"), "{error}");

    let region = json!({ "type": "region", "width": 8, "height": 8, "tooltip": "<red>Hot" });
    let error = reject(region).to_string();
    assert!(error.contains("full-screen tooltips draw plain text, without MiniMessage tags"), "{error}");

    let region =
        json!({ "type": "region", "width": 8, "height": 8, "tooltip": { "title": "Hot", "lines": ["§cDanger"] } });
    let error = reject(region).to_string();
    assert!(error.contains("full-screen tooltips draw plain text, without formatting codes"), "{error}");
}

#[test]
fn splits_embedded_newlines_into_tooltip_lines() {
    let region = json!({ "type": "flex", "x": 100, "y": 50, "style": { "width": 40, "height": 20 }, "children": [
        { "type": "region", "on_click": { "kind": "action", "id": "play" },
          "tooltip": { "title": "First\nSecond", "lines": ["Third\n\nFifth"] } }
    ] });
    let out = compile(json!({ "name": "menu", "children": [region] })).unwrap();
    let tooltip = out.manifest.fullscreens["menu"].controls["play"].tooltip.as_ref().unwrap();
    let rows: Vec<(&str, i32)> = tooltip.lines.iter().map(|line| (line.text.as_deref().unwrap(), line.y)).collect();
    assert_eq!(rows, [("First", 76), ("Second", 88), ("Third", 98), ("Fifth", 118)]);
    assert!(tooltip.lines.iter().all(|line| !line.text.as_deref().unwrap().contains('\n')));
}

#[test]
fn lays_tooltips_out_beside_their_controls_above_every_window_layer() {
    let region = |id: &str, x: u32, y: u32, width: u32, tooltip: serde_json::Value| {
        let region = json!({ "type": "region", "on_click": { "kind": "action", "id": id }, "tooltip": tooltip });
        json!({ "type": "flex", "x": x, "y": y, "style": { "width": width, "height": 20 }, "children": [region] })
    };
    let out = compile(json!({ "name": "menu", "children": [
        region("top", 0, 10, 40, json!("Top")),
        region("bottom", 470, 240, 10, json!({ "title": "Bottom", "lines": ["One", "Two"] })),
        region("plain", 200, 100, 40, json!(null)),
    ] }))
    .unwrap();
    let menu = &out.manifest.fullscreens["menu"];
    assert_eq!(menu.controls["plain"].tooltip, None);
    let table = color_table(include(&out));

    // Centered below its control: a 4px inset around one 8px line.
    let top = menu.controls["top"].tooltip.as_ref().unwrap();
    assert_eq!(decode(&top.marker, 0).0, u32::from(fullscreen::TOOLTIP_LAYER));
    let [title] = top.lines.as_slice() else { panic!("{:?}", top.lines) };
    assert_eq!((title.x, title.y, title.text.as_deref()), ((40 - (title.width as i32 + 8)) / 2 + 4, 36, Some("Top")));
    let (layer, color) = decode(title.shader_marker.as_deref().unwrap(), 0);
    assert_eq!((layer, table[color].as_str()), (u32::from(fullscreen::TOOLTIP_TEXT_LAYER), "255, 255, 255"));

    // Above its control where the canvas ends below it, and kept on the canvas; the title is 2px further from its
    // lines than they are apart.
    let bottom = menu.controls["bottom"].tooltip.as_ref().unwrap();
    let rows: Vec<(i32, i32)> = bottom.lines.iter().map(|line| (line.x, line.y)).collect();
    let panel_height = 8 + 2 + 2 * 10 + 8;
    let top_y = 240 - 2 - panel_height + 4;
    let left = 480 - (bottom.lines.iter().map(|line| line.width).max().unwrap() as i32 + 8) + 4;
    assert_eq!(rows, [(left, top_y), (left, top_y + 12), (left, top_y + 22)]);
    let gray = decode(bottom.lines[1].shader_marker.as_deref().unwrap(), 0).1;
    assert_eq!(table[gray], "170, 170, 170");

    let file = out
        .files
        .iter()
        .find(|file| file.path.ends_with(&format!("menu_tooltip_{}_32_{}x16_0_0.png", title.x - 4, title.width + 8)))
        .unwrap();
    let panel = Texture::decode_png(file.contents.as_bytes()).unwrap();
    assert_eq!((panel.width, panel.height), (title.width + 8, 16));
    assert_eq!(panel.get(0, 0)[3], 0, "the corners are cut");
    assert_eq!(panel.get(4, 4), [0x10, 0x00, 0x10, 0xf0]);
}

#[test]
fn a_region_filling_a_box_becomes_a_control_without_slots() {
    let region = json!({ "type": "region", "on_click": { "kind": "action", "id": "play" } });
    let row = json!({ "type": "flex", "x": 100, "y": 50, "style": { "width": 64, "height": 24 },
                      "children": [region] });
    let out = compile(json!({ "name": "menu", "children": [row] })).unwrap();
    let play = &out.manifest.fullscreens["menu"].controls["play"];
    assert_eq!((play.x, play.y, play.width, play.height), (100, 50, 64, 24));
}

fn shape(width: u32, height: u32) -> serde_json::Value {
    json!({ "art": "shape", "kind": "panel", "fill": "#123456", "border_width": 0, "radius": 0, "inset_depth": 0,
            "width": width, "height": height })
}

/// The sizes of the full-screen texture PNGs under `directory`, by file name.
fn texture_sizes(out: &CompileOutput, directory: &str) -> BTreeMap<String, (u32, u32)> {
    let prefix = format!("assets/window/textures/font/{directory}/");
    out.files
        .iter()
        .filter_map(|file| {
            let name = file.path.strip_prefix(&prefix)?;
            let texture = Texture::decode_png(file.contents.as_bytes()).unwrap();
            Some((name.to_string(), (texture.width, texture.height)))
        })
        .collect()
}

#[test]
fn tiles_a_canvas_sized_frame_into_glyphs_within_the_atlas_page() {
    let frame = json!({ "type": "sprite", "art": shape(480, 270) });
    let out = compile(json!({ "name": "menu", "children": [frame] })).unwrap();
    let sizes = texture_sizes(&out, "fullscreen/window");
    let tiles = [("menu_0_0.png", (256, 256)), ("menu_1_0.png", (224, 256)), ("menu_0_1.png", (256, 14))];
    assert_eq!(sizes.len(), 4, "{sizes:?}");
    assert!(tiles.iter().all(|(name, size)| sizes[*name] == *size), "{sizes:?}");
    assert_eq!(sizes["menu_1_1.png"], (224, 14));
}

#[test]
fn tiles_dispersed_art_and_skips_empty_tiles() {
    let tone = json!({ "kind": "tint", "id": "tone", "values": ["a", "b"], "colors": ["#3ddc84", "#ff4d4d"] });
    let sprite = |x: u32, y: u32| json!({ "type": "sprite", "art": shape(8, 8), "tint": tone, "x": x, "y": y });
    let out = compile(json!({ "name": "menu", "children": [sprite(0, 0), sprite(470, 250)] })).unwrap();
    let sizes = texture_sizes(&out, "fullscreen/window");
    let names: Vec<&str> = sizes.keys().map(String::as_str).collect();
    assert_eq!(names, ["menu_art/0_0_0.png", "menu_art/0_1_0.png", "menu_art/0_1_1.png"], "{sizes:?}");
    assert!(sizes.values().all(|&(width, height)| width <= 256 && height <= 256), "{sizes:?}");
}

#[test]
fn full_screen_textures_do_not_collide_with_window_textures() {
    let art = json!({ "type": "sprite", "art": shape(8, 8) });
    let project = json!({
        "windows": [{ "name": "fullscreen_menu", "container": "generic_9x3", "children": [art.clone()] }],
        "fullscreens": [{ "name": "menu", "children": [art] }],
        "target": { "pack_format": 88 },
    })
    .to_string();
    let out = compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    let paths: Vec<&str> = out.files.iter().map(|file| file.path.as_str()).collect();
    let font = "assets/window/textures/font";
    assert!(paths.contains(&format!("{font}/fullscreen_menu.png").as_str()), "{paths:?}");
    assert!(paths.contains(&format!("{font}/fullscreen/window/menu_0_0.png").as_str()), "{paths:?}");
    assert!(paths.contains(&format!("{font}/fullscreen/builtin/cursor.png").as_str()), "{paths:?}");
}

#[test]
fn rejects_shadowed_text() {
    let label = json!({ "type": "slot", "handle": { "kind": "text", "id": "title" }, "width": 60, "shadow": true });
    let error = compile(json!({ "name": "menu", "children": [label] })).unwrap_err().to_string();
    assert!(error.contains("fullscreen `menu` slot `title` sets `shadow`"), "{error}");
}

#[test]
fn outlines_hovered_controls_and_emits_the_blank_item_model() {
    let region = |id: &str, hover: bool, y: u32| {
        let region = json!({ "type": "region", "on_click": { "kind": "action", "id": id }, "hover": hover });
        json!({ "type": "flex", "x": 100, "y": y, "style": { "width": 40, "height": 20 }, "children": [region] })
    };
    let out =
        compile(json!({ "name": "menu", "children": [region("play", true, 50), region("quit", false, 80)] })).unwrap();
    let controls = &out.manifest.fullscreens["menu"].controls;
    assert!(controls["play"].outline.as_ref().is_some_and(|text| !text.is_empty()));
    assert_eq!(controls["quit"].outline, None);

    let file = |path: &str| out.files.iter().find(|file| file.path == path).unwrap_or_else(|| panic!("{path}"));
    let outline = file("assets/window/textures/font/fullscreen/window/menu_hover_0_0_0.png");
    let outline = Texture::decode_png(outline.contents.as_bytes()).unwrap();
    assert_eq!((outline.width, outline.height), (40, 20));
    assert_eq!(outline.get(0, 0), [0xff; 4]);
    assert_eq!(outline.get(39, 10), [0xff; 4]);
    assert_eq!(outline.get(1, 1)[3], 0, "the interior is transparent");
    let blank = file("assets/window/items/fullscreen/blank.json");
    assert_eq!(blank.contents.as_bytes(), b"{\"model\":{\"type\":\"minecraft:empty\"}}\n");
}
