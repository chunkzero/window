//! Tints: palette-tinted art is its own runtime layer, and tinted art and text take a palette color at runtime.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use window_core::codegen::{KotlinTarget, generate_kotlin};
use window_core::compose::Texture;
use window_core::ir::Layer;
use window_core::pipeline::{CompileInput, CompileOutput, compile_project_json};

fn compile(project: &Value) -> window_core::Result<CompileOutput> {
    compile_project_json(project.to_string().as_bytes(), &CompileInput::new(BTreeMap::new()))
}

fn error(children: Value) -> String {
    let window = json!({ "name": "w", "container": "generic_9x3", "children": children });
    compile(&json!({ "windows": [window] })).expect_err("project should be rejected").to_string()
}

fn tone() -> Value {
    json!({ "kind": "tint", "id": "tone", "values": ["ok", "warn"], "colors": ["#3ddc84", "#ffb020"] })
}

/// An 8x8 box at `x` framed with a flat `fill`.
fn framed(x: i32, fill: &str) -> Value {
    let frame =
        json!({ "art": "shape", "kind": "panel", "fill": fill, "border_width": 0, "radius": 0, "inset_depth": 0 });
    json!({ "type": "flex", "x": x, "y": 20, "style": { "width": 8, "height": 8 }, "frame": frame })
}

fn tinted(x: i32, fill: &str, tint: Value) -> Value {
    let mut element = framed(x, fill);
    element["tint"] = tint;
    element
}

/// The top-left pixel of the emitted texture at `path`.
fn first_pixel(output: &CompileOutput, path: &str) -> [u8; 4] {
    let file = output.files.iter().find(|file| file.path == path).unwrap_or_else(|| panic!("no file {path}"));
    Texture::decode_png(file.contents.as_bytes()).unwrap().rgba[..4].try_into().unwrap()
}

#[test]
fn palette_tinted_art_is_its_own_layer_in_authored_order() {
    let children = json!([
        tinted(8, "#808080", json!("#ff0000")),
        { "type": "slot", "handle": { "kind": "text", "id": "title" }, "x": 30, "y": 20, "width": 40 },
        tinted(80, "#ffffff", tone()),
        { "type": "label", "text": "Hi", "color": tone(), "x": 100, "y": 20 },
        framed(84, "#123456"),
        framed(120, "#123456"),
        tinted(140, "#ffffff", tone()),
        tinted(150, "#ffffff", tone()),
    ]);
    let window = json!({ "name": "w", "container": "generic_9x3", "children": children });
    let output = compile(&json!({ "windows": [window] })).unwrap();
    let entry = &output.manifest.windows["w"];

    let palette = BTreeMap::from([("ok".to_string(), "#3ddc84".to_string()), ("warn".into(), "#ffb020".into())]);
    assert_eq!(entry.tints, BTreeMap::from([("tone".to_string(), palette)]));

    // Tinted art draws where it was authored, the untinted art covering it draws above it, and adjacent art of one
    // tint shares a layer. Untinted art clear of it stays in the static art.
    let label = entry.slots.iter().find(|(_, slot)| slot.text.is_some()).unwrap();
    let art = |index: u32| Layer::Art(format!("art/{index}"));
    assert_eq!(entry.layers, [Layer::Slot("title".into()), art(0), Layer::Slot(label.0.clone()), art(1), art(2)]);
    let tints: Vec<Option<&str>> = entry.art.values().map(|art| art.tint.as_deref()).collect();
    assert_eq!(tints, [Some("tone"), None, Some("tone")]);
    assert!(entry.art.values().all(|art| !art.static_text.is_empty() && art.tint_markers.is_empty()));
    assert_eq!(label.1.tint.as_deref(), Some("tone"));
    assert_eq!(label.1.color, "#3ddc84");

    // A fixed color is multiplied into the static art; a palette's is left to the runtime.
    assert_eq!(first_pixel(&output, "assets/window/textures/font/w.png"), [0x80, 0, 0, 0xff]);
    assert_eq!(first_pixel(&output, "assets/window/textures/font/w_art/0.png"), [0xff; 4]);

    let files = generate_kotlin(&output.manifest, "demo", KotlinTarget::Minestom).unwrap();
    let source = |path: &str| {
        let file = files.iter().find(|file| file.path == path).unwrap();
        String::from_utf8(file.contents.as_bytes().to_vec()).unwrap()
    };
    let view = source("WView.kt");
    for line in [
        "public enum class Tone(public val value: String) {",
        "WARN(\"warn\"),",
        "protected abstract fun tone(): Tone",
        "tint(\"tone\") { tone().value }",
    ] {
        assert!(view.contains(line), "missing `{line}` in:\n{view}");
    }
    assert!(source("WindowEntries.kt").contains("LayerEntry(LayerKind.ART, \"art/0\")"));
}

#[test]
fn art_inside_a_switch_case_belongs_to_the_case() {
    let flag = json!({ "kind": "flag", "id": "open" });
    let children = json!([{ "type": "switch", "handle": flag, "x": 8, "y": 20, "children": [
        { "type": "case", "value": "true", "children": [tinted(8, "#ffffff", tone())] },
        { "type": "case", "value": "false", "children": [] }
    ] }]);
    let window = json!({ "name": "w", "container": "generic_9x3", "children": children });
    let output = compile(&json!({ "windows": [window] })).unwrap();
    let cases = &output.manifest.windows["w"].switches["open"].cases;
    assert_eq!(cases[0].art, ["art/0"]);
    assert!(cases[1].art.is_empty());
}

#[test]
fn shader_placed_huds_get_a_marker_per_palette_entry() {
    let hud = |name: &str, shader: bool| {
        let mut hud = json!({ "name": name, "channel": "actionbar", "width": 120, "height": 20, "children": [
            tinted(0, "#ffffff", tone()),
            { "type": "slot", "handle": { "kind": "text", "id": "time" }, "color": tone(), "x": 20, "y": 2, "width": 60 }
        ] });
        if shader {
            hud["shader"] =
                json!({ "source_bottom": 59, "origin": { "x": 0.5, "y": 0.0 }, "anchor": { "x": 0.5, "y": 0.0 } });
        }
        hud
    };
    let project = json!({
        "options": { "core_shaders": { "hud": true } },
        "target": { "pack_format": 88 },
        "huds": [hud("placed", true), hud("plain", false)]
    });
    let output = compile(&project).unwrap();

    let placed = &output.manifest.huds["placed"];
    let (art, slot) = (&placed.art["art/0"], &placed.slots["time"]);
    let entries = |markers: &BTreeMap<String, String>| markers.keys().cloned().collect::<Vec<_>>();
    assert_eq!(entries(&art.tint_markers), ["ok", "warn"]);
    assert_eq!(entries(&slot.tint_markers), ["ok", "warn"]);
    let shader = placed.shader.as_ref().unwrap();
    let markers: Vec<&String> = (art.tint_markers.values().chain(slot.tint_markers.values()))
        .chain([&shader.static_marker, slot.shader_marker.as_ref().unwrap()])
        .collect();
    assert!(markers.iter().all(|marker| &marker[3..5] == "00"), "{markers:?}");
    assert_eq!(markers.iter().collect::<BTreeSet<_>>().len(), markers.len());

    // Without a shader the runtime sends the palette colors themselves.
    let plain = &output.manifest.huds["plain"];
    assert!(plain.art["art/0"].tint_markers.is_empty() && plain.slots["time"].tint_markers.is_empty());
    assert_eq!(plain.tints, placed.tints);
}

#[test]
fn invalid_tints_are_rejected_with_the_cause() {
    let palette =
        |values: Value, colors: Value| json!({ "kind": "tint", "id": "tone", "values": values, "colors": colors });
    let cases = [
        (palette(json!(["ok", "warn"]), json!(["#3ddc84", "orange"])), "tint `tone` entry `warn` has invalid color"),
        (palette(json!(["ok", "ok"]), json!(["#3ddc84", "#ffb020"])), "tint `tone` repeats value `ok`"),
        (palette(json!(["Ok"]), json!(["#3ddc84"])), "tint `tone` value"),
        (palette(json!([]), json!([])), "tint `tone` requires at least one value"),
        (json!({ "kind": "value", "id": "mode", "values": ["a"] }), "a value handle cannot be used as a tint"),
    ];
    for (tint, expected) in cases {
        let message = error(json!([tinted(8, "#ffffff", tint)]));
        assert!(message.contains(expected), "expected `{expected}` in: {message}");
    }

    let names: Vec<String> = (0..65).map(|i| format!("c{i}")).collect();
    let colors = vec!["#ffffff"; 65];
    let message = error(json!([tinted(8, "#ffffff", palette(json!(names), json!(colors)))]));
    assert!(message.contains("a tint takes at most 64"), "{message}");

    let mut unframed = tinted(8, "#ffffff", tone());
    unframed.as_object_mut().unwrap().remove("frame");
    assert!(error(json!([unframed])).contains("flex element sets `tint` without a `frame` to tint"));

    let section = json!({ "type": "section", "section": "container", "tint": tone() });
    assert!(error(json!([section])).contains("section element does not accept field `tint`"));
}
