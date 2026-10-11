//! Hover outlines: which regions get one, what the pack gains for them, and the vanilla slot highlight override.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use window_core::codegen::{KotlinTarget, generate_kotlin};
use window_core::compose::Texture;
use window_core::font::spacers_for;
use window_core::pipeline::{CompileInput, CompileOutput, compile_project_json};

fn compile(children: Value, options: Value, pack_format: u32) -> window_core::Result<CompileOutput> {
    let window = json!({ "name": "shop", "container": "generic_9x3", "children": children });
    let project = json!({ "windows": [window], "options": options, "target": { "pack_format": pack_format } });
    compile_project_json(project.to_string().as_bytes(), &CompileInput::new(BTreeMap::new()))
}

/// A 16x16 region over the container slot in column `column` of the first row, with `fields` added.
fn region(column: i32, fields: Value) -> Value {
    let mut region = json!({ "type": "region", "width": 16, "height": 16 });
    region.as_object_mut().unwrap().extend(fields.as_object().unwrap().clone());
    json!({ "type": "flex", "x": 8 + column * 18, "y": 18, "children": [region] })
}

fn click(id: &str) -> Value {
    json!({ "kind": "action", "id": id })
}

fn paths(output: &CompileOutput) -> Vec<&str> {
    output.files.iter().map(|file| file.path.as_str()).collect()
}

#[test]
fn clickable_regions_are_outlined_unless_they_opt_out() {
    let children = json!([
        region(0, json!({ "on_click": click("buy"), "tooltip": "Buy" })),
        region(1, json!({ "on_click": click("sell") })),
        region(2, json!({ "on_click": click("quiet"), "hover": false })),
        region(3, json!({ "tooltip": "Info" })),
        region(4, json!({ "tooltip": "More", "hover": true })),
        json!({ "type": "flex", "x": 8, "y": 36, "children": [
            { "type": "region", "on_click": click("wide"), "width": 34, "height": 16 }
        ] }),
    ]);
    let output = compile(children, json!({ "core_shaders": { "hover": true } }), 88).unwrap();
    window_core::validation::validate_compile_output(&output).assert_valid();
    let regions = &output.manifest.windows["shop"].regions;

    // generic_9x3 is 176x168, so the region at (8, 18) starts 80 px left of and 66 px above the GUI center.
    let buy = regions["buy"].hover.as_ref().expect("a clickable region is outlined");
    assert_eq!((buy.x, buy.y), (-80, -66));
    let glyph = buy.text.chars().next().unwrap();
    assert_eq!(buy.text, format!("{glyph}{}", spacers_for(-17)));
    assert!(regions["buy"].hitbox.as_ref().unwrap().tooltip.is_some());

    // Without a tooltip or item model the region has no hitbox entry; its hover entry alone makes the runtime fill it.
    let sell = regions["sell"].hover.as_ref().expect("a tooltip-less clickable region is outlined");
    assert_eq!((sell.x, sell.text.as_str()), (-62, buy.text.as_str()));
    assert_eq!(regions["sell"].hitbox, None);

    assert_eq!(regions["quiet"].hover, None);
    let hover_only: Vec<bool> = regions
        .iter()
        .filter(|(_, region)| region.action.is_none())
        .map(|(_, region)| region.hover.is_some())
        .collect();
    assert_eq!(hover_only, [false, true], "a region without a click is outlined only when it asks");
    assert_ne!(regions["wide"].hover.as_ref().unwrap().text, buy.text);

    let files = paths(&output);
    let hover_files: Vec<&str> = files.iter().copied().filter(|path| path.contains("hover")).collect();
    assert_eq!(
        hover_files,
        [
            "assets/window/textures/font/hover/16x16.png",
            "assets/window/textures/font/hover/34x16.png",
            "assets/window/textures/gui/sprites/tooltip/hover_background.png",
            "assets/window/textures/gui/sprites/tooltip/hover_frame.png",
        ]
    );
    let shaders: Vec<&str> = files.iter().copied().filter(|path| path.contains("/shaders/")).collect();
    assert_eq!(shaders, ["assets/minecraft/shaders/core/text.fsh", "assets/minecraft/shaders/core/text.vsh"]);
    let font = output.files.iter().find(|file| file.path == "assets/window/font/ui.json").unwrap();
    let font = String::from_utf8(font.contents.as_bytes().to_vec()).unwrap();
    assert!(font.contains(r#""file":"window:font/hover/16x16.png","height":18,"type":"bitmap""#), "{font}");

    let kotlin = generate_kotlin(&output.manifest, "demo", KotlinTarget::Agnostic).unwrap();
    let entries = kotlin.iter().find(|file| file.path == "WindowEntries.kt").unwrap();
    let entries = String::from_utf8(entries.contents.as_bytes().to_vec()).unwrap();
    assert!(entries.contains("import com.chunkzero.window.manifest.HoverEntry\n"));
    assert_eq!(entries.matches("hover = HoverEntry(").count(), 4);
}

#[test]
fn hover_needs_its_core_shader_option_and_a_supported_pack_format() {
    let hover = |value: bool| json!([region(0, json!({ "on_click": click("buy"), "hover": value }))]);
    for value in [true, false] {
        let err = compile(hover(value), json!({}), 88).unwrap_err().to_string();
        assert!(
            err.contains("window `shop`: region `buy` sets `hover`, but `coreShaders.hover` is off"),
            "hover={value}: {err}"
        );
    }

    let plain = json!([region(0, json!({ "on_click": click("buy") }))]);
    let off = compile(plain.clone(), json!({}), 88).unwrap();
    assert_eq!(off.manifest.windows["shop"].regions["buy"].hover, None);
    assert!(!paths(&off).iter().any(|path| path.contains("hover") || path.contains("/shaders/")));

    let err = compile(plain, json!({ "core_shaders": { "hover": true } }), 84).unwrap_err().to_string();
    assert!(err.contains("`coreShaders.hover` needs pack_format 85 through 88"), "{err}");

    let narrow = json!([{ "type": "flex", "x": 8, "y": 18, "children": [
        { "type": "region", "on_click": click("thin"), "width": 3, "height": 16 }
    ] }]);
    let err = compile(narrow, json!({ "core_shaders": { "hover": true } }), 88).unwrap_err().to_string();
    assert!(err.contains("region `thin` has a 3x16 px hover outline, which must be 4 to 256 px wide"), "{err}");
}

#[test]
fn hover_art_keeps_its_colors_and_is_stretched_or_centered_on_its_region() {
    let shape = |fill: &str, size: Option<u32>| {
        let mut shape = json!({ "art": "shape", "kind": "panel", "fill": fill, "border_width": 0, "radius": 0,
            "inset_depth": 0 });
        if let Some(size) = size {
            shape.as_object_mut().unwrap().extend([("width".into(), json!(size)), ("height".into(), json!(size))]);
        }
        shape
    };
    let children = json!([
        region(0, json!({ "on_click": click("frame"), "hover": shape("#ff0000", None) })),
        region(1, json!({ "on_click": click("image"), "hover": shape("#00ff00", Some(19)) })),
        region(2, json!({ "on_click": click("same"), "hover": shape("#00ff00", Some(19)) })),
        region(3, json!({ "on_click": click("plain") })),
    ]);
    let output = compile(children, json!({ "core_shaders": { "hover": true } }), 88).unwrap();
    window_core::validation::validate_compile_output(&output).assert_valid();
    let regions = &output.manifest.windows["shop"].regions;
    let hover = |name: &str| regions[name].hover.as_ref().unwrap();

    // Art without a size of its own stretches over the region, so it starts where the region does.
    assert_eq!((hover("frame").x, hover("frame").y), (-80, -66));
    assert_ne!(hover("frame").text, hover("plain").text, "art of the outline's size is another glyph");
    // The 19x19 image is centered on its 16x16 region, the odd pixel toward the top-left.
    assert_eq!((hover("image").x, hover("image").y), (-64, -68));
    let glyph = hover("image").text.chars().next().unwrap();
    assert_eq!(hover("image").text, format!("{glyph}{}", spacers_for(-20)));
    assert_eq!(hover("same").text, hover("image").text, "regions with the same art share its glyph");
    assert_eq!(hover("same").x, -46);

    let glyphs: Vec<&str> = paths(&output).into_iter().filter(|path| path.contains("/font/hover/")).collect();
    let [outline, frame, image] = glyphs[..] else { panic!("expected three hover glyphs: {glyphs:?}") };
    assert_eq!(outline, "assets/window/textures/font/hover/16x16.png");
    assert!(
        image.starts_with("assets/window/textures/font/hover/image/art/") && image.ends_with("/19x19.png"),
        "{image}"
    );
    assert!(
        frame.starts_with("assets/window/textures/font/hover/frame/art/") && frame.ends_with("/16x16.png"),
        "{frame}"
    );

    let file = output.files.iter().find(|file| file.path == frame).unwrap();
    let texture = Texture::decode_png(file.contents.as_bytes()).unwrap();
    assert_eq!((texture.width, texture.height), (16, 18), "the art sits between two data rows");
    let pixel = |x: usize, y: usize| &texture.rgba[(y * 16 + x) * 4..][..4];
    assert_eq!(pixel(0, 1), [0xff, 0, 0, 0xff]);
    assert_eq!(pixel(15, 16), [0xff, 0, 0, 0xff]);
    assert_eq!(pixel(1, 0), [15, 15, 0xa9, 0xff], "a data row holds the art's size minus one");
    assert_eq!(pixel(8, 17)[3], 0);

    let thin = json!([region(0, json!({ "on_click": click("thin"), "hover": shape("#ff0000", Some(3)) }))]);
    let err = compile(thin, json!({ "core_shaders": { "hover": true } }), 88).unwrap_err().to_string();
    assert!(err.contains("region `thin` has a 3x3 px hover art, which must be 4 to 256 px wide"), "{err}");
    let err = compile(json!([region(0, json!({ "hover": "#ffffff" }))]), json!({}), 88).unwrap_err().to_string();
    assert!(err.contains("`hover` must be a boolean or an art value"), "{err}");
}

#[test]
fn slot_highlight_is_blanked_or_replaced_by_a_catalog_sprite() {
    let back = "assets/minecraft/textures/gui/sprites/container/slot_highlight_back.png";
    let front = "assets/minecraft/textures/gui/sprites/container/slot_highlight_front.png";
    let pixels = |output: &CompileOutput, path: &str| {
        let file = output.files.iter().find(|file| file.path == path).unwrap_or_else(|| panic!("no file {path}"));
        Texture::decode_png(file.contents.as_bytes()).unwrap()
    };
    let project = |highlight: Value, pack_format: u32| {
        let ring = json!({ "art": "shape", "kind": "panel", "fill": "#ff0000", "border_width": 0, "radius": 0,
            "inset_depth": 0, "width": 24, "height": 24 });
        let project = json!({
            "sprites": { "ring": ring },
            "options": { "vanilla": { "slotHighlight": highlight } },
            "target": { "pack_format": pack_format },
        });
        compile_project_json(project.to_string().as_bytes(), &CompileInput::new(BTreeMap::new()))
    };

    let blank = project(json!(false), 88).unwrap();
    for path in [back, front] {
        let texture = pixels(&blank, path);
        assert_eq!((texture.width, texture.height), (24, 24));
        assert!(texture.rgba.iter().all(|&channel| channel == 0));
    }

    let sprite = project(json!("ring"), 88).unwrap();
    assert!(pixels(&sprite, back).rgba.iter().all(|&channel| channel == 0));
    assert_eq!(pixels(&sprite, front).rgba[..4], [0xff, 0, 0, 0xff]);

    assert!(!paths(&project(json!(true), 88).unwrap()).iter().any(|path| path.contains("slot_highlight")));
    let err = project(json!(false), 34).unwrap_err().to_string();
    assert!(err.contains("`vanilla.slotHighlight` replaces GUI sprites that need pack format 42"), "{err}");
}
