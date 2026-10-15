use std::collections::BTreeMap;

use serde_json::json;

use crate::fullscreen::{self, INCLUDE_PATH};
use crate::ir::FullscreenMode;
use crate::manifest::Manifest;
use crate::pipeline::{CompileInput, CompileOutput, compile_project_json};

fn compile(fullscreen: serde_json::Value) -> crate::Result<CompileOutput> {
    let project = json!({ "fullscreens": [fullscreen] }).to_string();
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
    assert_eq!(color(&canvas.overlay_marker, 0), (254, "255, 255, 255"));

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
fn packs_without_fullscreens_emit_no_contract() {
    let project = json!({ "huds": [{ "name": "h", "children": [] }] }).to_string();
    let out = compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    assert!(out.manifest.fullscreen_canvas.is_none());
    assert!(out.files.iter().all(|file| !file.path.contains("fullscreen")));
}

#[test]
fn rejects_inventory_elements_and_hitbox_items() {
    let item = json!({ "type": "item", "handle": { "kind": "items", "id": "loot" }, "slots": [0] });
    let error = compile(json!({ "name": "menu", "children": [item] })).unwrap_err().to_string();
    assert!(error.contains("fullscreen `menu` has no inventory slots for inventory element `loot`"), "{error}");

    let region = json!({ "type": "region", "on_click": { "kind": "action", "id": "play" }, "width": 8, "height": 8,
                         "tooltip": "Play" });
    let error = compile(json!({ "name": "menu", "children": [region] })).unwrap_err().to_string();
    assert!(error.contains("sets `tooltip` or `item_model`, which need inventory slots"), "{error}");
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
