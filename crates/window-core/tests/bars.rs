//! Bars: a fill cut into power-of-two clips that compose to any column count, with a track in the art underneath.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use window_core::codegen::{KotlinTarget, generate_kotlin};
use window_core::compose::Texture;
use window_core::ir::Layer;
use window_core::manifest::BarEntry;
use window_core::pipeline::{CompileInput, CompileOutput, compile_project_json};

const WIDTH: u32 = 37;
const HEIGHT: u32 = 3;

/// A fill whose every column differs, with a transparent column and a column whose only ink is its bottom row.
fn uneven_fill() -> Texture {
    let mut fill = Texture::transparent(WIDTH, HEIGHT);
    for x in (0..WIDTH).filter(|x| *x != 5) {
        for y in (0..HEIGHT).filter(|y| x != 20 || *y == HEIGHT - 1) {
            let i = ((y * WIDTH + x) * 4) as usize;
            fill.rgba[i..i + 4].copy_from_slice(&[x as u8 * 6, y as u8 * 80, 0x40, 0xff]);
        }
    }
    fill
}

fn hp() -> Value {
    json!({ "kind": "fraction", "id": "hp" })
}

fn flat(fill: &str) -> Value {
    json!({ "art": "shape", "kind": "panel", "fill": fill, "border_width": 0, "radius": 0, "inset_depth": 0 })
}

fn bar(width: u32, fill: Value) -> Value {
    json!({ "type": "bar", "handle": hp(), "fill": fill, "x": 8, "y": 20, "width": width, "height": HEIGHT })
}

fn compile(project: &Value, files: BTreeMap<String, Vec<u8>>) -> window_core::Result<CompileOutput> {
    compile_project_json(project.to_string().as_bytes(), &CompileInput::new(files))
}

fn window(children: Value) -> Value {
    json!({ "windows": [{ "name": "w", "container": "generic_9x3", "children": children }] })
}

fn error(children: Value) -> String {
    compile(&window(children), BTreeMap::new()).expect_err("project should be rejected").to_string()
}

fn texture(output: &CompileOutput, path: &str) -> Texture {
    let file = output.files.iter().find(|file| file.path == path).unwrap_or_else(|| panic!("no file {path}"));
    Texture::decode_png(file.contents.as_bytes()).unwrap()
}

/// The texture of each glyph of the main font.
fn glyph_textures(output: &CompileOutput) -> BTreeMap<char, Texture> {
    let font = output.files.iter().find(|file| file.path == "assets/window/font/ui.json").unwrap();
    let font: Value = serde_json::from_slice(font.contents.as_bytes()).unwrap();
    let bitmaps = font["providers"].as_array().unwrap().iter().filter(|provider| provider["type"] == "bitmap");
    bitmaps
        .map(|provider| {
            let glyph = provider["chars"][0].as_str().unwrap().chars().next().unwrap();
            let file = provider["file"].as_str().unwrap().strip_prefix("window:").unwrap();
            (glyph, texture(output, &format!("assets/window/textures/{file}")))
        })
        .collect()
}

/// The pixels `text` draws over a strip starting at the bar's left edge, and where its cursor ends.
fn draw(output: &CompileOutput, bar: &BarEntry, text: &str) -> (Texture, i32) {
    let glyphs = glyph_textures(output);
    let mut strip = Texture::transparent(bar.width, HEIGHT);
    let mut cursor = 0;
    for character in text.chars() {
        if let Some(advance) = output.manifest.spacers.get(&(character as u32)) {
            cursor += advance;
            continue;
        }
        let glyph = &glyphs[&character];
        for x in 0..glyph.width {
            for y in 0..HEIGHT {
                let at = ((y as i32 * bar.width as i32 + cursor - bar.x + x as i32) * 4) as usize;
                let from = ((y * glyph.width + x) * 4) as usize;
                assert_eq!(strip.rgba[at + 3], 0, "a column is drawn twice");
                strip.rgba[at..at + 4].copy_from_slice(&glyph.rgba[from..from + 4]);
            }
        }
        cursor += bar.advances[&character] as i32;
    }
    (strip, cursor)
}

#[test]
fn every_column_count_draws_exactly_the_leftmost_columns_of_the_fill_and_returns_the_cursor() {
    let fill = uneven_fill();
    let files = BTreeMap::from([("fill.png".to_string(), fill.encode_png().unwrap())]);
    let project = window(json!([bar(WIDTH, json!({ "art": "texture", "texture": "fill.png" }))]));
    let output = compile(&project, files).unwrap();
    let entry = &output.manifest.windows["w"];
    let art = &entry.art["art/0"];
    let bar = art.bar.as_ref().unwrap();

    assert_eq!(entry.layers, [Layer::Art("art/0".into())]);
    assert_eq!((bar.name.as_str(), bar.width, bar.levels.len()), ("hp", WIDTH, 6));
    assert_eq!(art.static_text, bar.compose(WIDTH));
    for columns in 0..=WIDTH {
        let text = bar.compose(columns);
        let (strip, cursor) = draw(&output, bar, &text);
        assert_eq!(cursor, 0, "{columns} columns leave the cursor where they started");
        let mut expected = Texture::transparent(WIDTH, HEIGHT);
        for y in 0..HEIGHT as usize {
            let row = y * WIDTH as usize * 4;
            let end = row + columns as usize * 4;
            expected.rgba[row..end].copy_from_slice(&fill.rgba[row..end]);
        }
        assert_eq!(strip.rgba, expected.rgba, "{columns} columns");
        // One glyph per set bit of the column count, less the clips with nothing to draw.
        let glyphs = text.chars().filter(|character| bar.advances.contains_key(character)).count();
        assert!(glyphs <= columns.count_ones() as usize, "{columns} columns took {glyphs} glyphs");
    }
    assert_eq!(bar.compose(WIDTH + 9), bar.compose(WIDTH), "a count past the width draws the whole fill");
}

#[test]
fn a_fill_whose_columns_repeat_costs_one_glyph_per_clip_width() {
    let mut element = bar(120, flat("#ffffff"));
    element["track"] = flat("#202020");
    element["tint"] = json!("#ff0000");
    let output = compile(&window(json!([element])), BTreeMap::new()).unwrap();
    let entry = &output.manifest.windows["w"];
    let bar = entry.art["art/0"].bar.as_ref().unwrap();

    assert_eq!(bar.advances.len(), 7);
    assert_eq!(bar.levels.iter().map(|level| level.chars().count()).collect::<Vec<_>>(), [60, 30, 15, 8, 4, 2, 1]);
    let fills = output.files.iter().filter(|file| file.path.starts_with("assets/window/textures/font/w_art/"));
    assert_eq!(fills.count(), 7);
    // A fixed tint is baked into the fill, and the track is part of the static art underneath.
    assert_eq!(texture(&output, "assets/window/textures/font/w_art/0_6_0.png").rgba[..4], [0xff, 0, 0, 0xff]);
    assert_eq!(texture(&output, "assets/window/textures/font/w.png").rgba[..4], [0x20, 0x20, 0x20, 0xff]);

    let files = generate_kotlin(&output.manifest, "demo", KotlinTarget::Minestom).unwrap();
    let source = |path: &str| {
        let file = files.iter().find(|file| file.path == path).unwrap();
        String::from_utf8(file.contents.as_bytes().to_vec()).unwrap()
    };
    let view = source("WView.kt");
    for line in ["protected abstract fun hp(): Float", "bar(\"hp\") { hp() }"] {
        assert!(view.contains(line), "missing `{line}` in:\n{view}");
    }
    assert!(source("WindowEntries.kt").contains("bar = BarEntry("));
}

#[test]
fn a_tinted_bar_in_a_shader_placed_hud_gets_a_marker_per_palette_entry() {
    let tone = json!({ "kind": "tint", "id": "tone", "values": ["ok", "low"], "colors": ["#3ddc84", "#ff4d4d"] });
    let mut element = bar(64, flat("#ffffff"));
    element["tint"] = tone;
    let shader = json!({ "source_bottom": 59, "origin": { "x": 0.5, "y": 0.0 }, "anchor": { "x": 0.5, "y": 0.0 } });
    let hud = json!({ "name": "status", "channel": "actionbar", "width": 120, "height": 30, "shader": shader,
        "children": [element] });
    let project = json!({
        "options": { "core_shaders": { "hud": true } },
        "target": { "pack_format": 88 },
        "huds": [hud]
    });
    let output = compile(&project, BTreeMap::new()).unwrap();
    let art = &output.manifest.huds["status"].art["art/0"];

    assert_eq!(art.tint.as_deref(), Some("tone"));
    assert_eq!(art.tint_markers.keys().collect::<Vec<_>>(), ["low", "ok"]);
    assert_eq!(art.bar.as_ref().unwrap().x, 8);
}

#[test]
fn invalid_bars_are_rejected_with_the_cause() {
    let sized = |width: u32, height: u32| {
        let mut element = bar(width, flat("#ffffff"));
        element["height"] = json!(height);
        element
    };
    let mut unfilled = bar(40, flat("#ffffff"));
    unfilled.as_object_mut().unwrap().remove("fill");
    let mut on_text = bar(40, flat("#ffffff"));
    on_text["handle"] = json!({ "kind": "text", "id": "hp" });
    let mut missing = bar(40, json!({ "art": "texture", "texture": "nowhere.png" }));
    missing["debug_name"] = json!("health");
    let cases = [
        (sized(0, 4), "bar element is 0x4 px; its `width` and `height` must each be from 1 to 256"),
        (sized(300, 4), "bar element is 300x4 px; its `width` and `height` must each be from 1 to 256"),
        (sized(40, 0), "bar element is 40x0 px"),
        (unfilled, "bar element requires `fill`"),
        (on_text, "a text handle cannot be used as a bar's fill"),
        (missing, "nowhere.png"),
    ];
    for (element, expected) in cases {
        let message = error(json!([element]));
        assert!(message.contains(expected), "expected `{expected}` in: {message}");
    }
    let slot = json!({ "type": "slot", "handle": hp(), "x": 8, "y": 20, "width": 40 });
    assert!(error(json!([slot])).contains("a fraction handle cannot be used as text"));
}
