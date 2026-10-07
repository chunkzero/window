use super::*;
use crate::inventory::{InventorySlotRef, SlotRectClaim};
use crate::ir::SlotRectIr;

#[test]
fn indexed_switch_art_uses_valid_resource_paths() {
    let project = r##"{
      "theme":{"sprites":{"dot":{"kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0,"width":4,"height":4}}},
      "windows":[{"name":"shop","container":"generic_9x3","children":[
        {"type":"switch","name":"lamp","index":0,"x":8,"y":20,"children":[
            {"type":"case","value":"on","children":[{"type":"sprite","name":"dot"}]}
        ]}
      ]}]
    }"##;
    let input = crate::pipeline::CompileInput::new(std::collections::BTreeMap::new());
    let out = crate::pipeline::compile_project_json(project.as_bytes(), &input).unwrap();
    let paths: Vec<&str> = out.files.iter().map(|file| file.path.as_str()).collect();
    assert!(paths.iter().any(|path| path.contains("_switch/lamp-0/on")), "{paths:?}");
}

#[test]
fn emits_expected_paths() {
    let (w, textures) = sample_window();
    let out = compile_windows(&[w], &textures, "window").unwrap();
    let paths: Vec<&str> = out.files.iter().map(|f| f.path.as_str()).collect();
    // Two distinct slot offsets: title y=6 (k=0), label y=30 (k=24).
    assert_eq!(
        paths,
        vec![
            "assets/window/font/ui.json",
            "assets/window/font/y0.json",
            "assets/window/font/y24.json",
            "assets/window/items/gui/hitbox.json",
            "assets/window/models/gui/hitbox.json",
            "assets/window/textures/font/shop.png",
            "assets/window/textures/gui/hitbox.png",
            "assets/window/window/debug.json",
        ]
    );
    // Window warning propagated.
    assert_eq!(out.warnings, vec!["overlay overflow".to_string()]);
}

#[test]
fn bitmap_provider_ascent_math() {
    // Glyph top at y=0, title y=6 → ascent 13, height 16.
    let (w, textures) = sample_window();
    let out = compile_windows(&[w], &textures, "window").unwrap();
    let ui: serde_json::Value =
        serde_json::from_slice(find(&out, "assets/window/font/ui.json").contents.as_bytes()).unwrap();
    let providers = ui["providers"].as_array().unwrap();
    // [0] = space, [1] = shop bitmap glyph.
    assert_eq!(providers[0]["type"], "space");
    let bm = &providers[1];
    assert_eq!(bm["type"], "bitmap");
    assert_eq!(bm["file"], "window:font/shop.png");
    assert_eq!(bm["height"], 16);
    assert_eq!(bm["ascent"], 13);
}

#[test]
fn static_bake_uses_alpha_trimmed_minecraft_advance() {
    let mut textures = BTreeMap::new();
    textures.insert("trimmed.png".to_string(), with_transparent_right_edge(solid(16, 8, [255, 255, 255, 255]), 6));
    let draw = Draw::Sprite { texture: TextureKey("trimmed.png".into()), dest: Rect::new(8, 6, 16, 8) };
    let window = bare_window("trimmed", ContainerKind::Generic9x3, vec![draw]);

    let output = compile_windows(&[window], &textures, "window").unwrap();
    crate::validation::validate_compile_output(&output).assert_valid();
}

#[test]
fn negative_visual_top_pads_bitmap_provider_height() {
    let mut textures = BTreeMap::new();
    textures.insert("cap.png".to_string(), solid(4, 4, [255, 128, 0, 255]));
    let draw = Draw::Sprite { texture: TextureKey("cap.png".into()), dest: Rect::new(0, -20, 4, 4) };
    let w = bare_window("overhang", ContainerKind::Generic9x3, vec![draw]);

    let out = compile_windows(&[w], &textures, "window").unwrap();
    let ui: serde_json::Value =
        serde_json::from_slice(find(&out, "assets/window/font/ui.json").contents.as_bytes()).unwrap();
    let bm = &ui["providers"].as_array().unwrap()[1];
    assert_eq!(bm["height"], 33);
    assert_eq!(bm["ascent"], 33);
    let png = find(&out, "assets/window/textures/font/overhang.png").contents.as_bytes();
    let texture = Texture::decode_png(png).unwrap();
    assert_eq!(texture.height, 33);
    assert_eq!(&texture.rgba[0..4], &[255, 128, 0, 255]);
}

#[test]
fn text_font_slots_draw_with_their_sheet_at_the_slot_offset() {
    let (mut w, textures) = sample_window();
    w.slots.iter_mut().find(|s| s.name == "buy_label").unwrap().font = Some("small_caps".into());
    let out = compile_windows(&[w], &textures, "window").unwrap();
    assert_eq!(out.manifest.windows["shop"].slots["buy_label"].font, "window:small_caps/y24");
    assert_eq!(out.manifest.font_metrics["window:small_caps/y24"].advances[&'a'], 5);
    let font: serde_json::Value =
        serde_json::from_slice(find(&out, "assets/window/font/small_caps/y24.json").contents.as_bytes()).unwrap();
    assert_eq!(font["providers"][0]["file"], "window:font/text/small_caps.png");
    assert_eq!(font["providers"][0]["ascent"], 7 - 24);
    find(&out, "assets/window/textures/font/text/small_caps.png");
}

#[test]
fn manifest_parses_and_matches() {
    let (w, textures) = sample_window();
    let out = compile_windows(&[w], &textures, "window").unwrap();
    let m = &out.manifest;
    assert_eq!(m.version, VERSION);
    assert_eq!(m.namespace, "window");
    assert_eq!(m.font, "window:ui");
    assert_eq!(m.spacers.len(), 22);
    assert_eq!(m.font_metrics["minecraft:default"].bold_advance, 1);
    assert_eq!(m.font_metrics["window:y0"].advances[&'H'], vanilla::text_width("H"));

    let shop = &m.windows["shop"];
    assert_eq!(shop.surface.container, "generic_9x6");
    assert_eq!(shop.surface.size, [176, 222]);
    assert_eq!(shop.surface.title_origin, [8, 6]);
    assert!(!shop.static_text.is_empty());

    let title = &shop.slots["title"];
    assert_eq!(title.font, "window:y0");
    assert_eq!(title.color, "#404040");
    assert_eq!(title.align, Align::Center);
    assert!(title.text.is_none());
    let label = &shop.slots["buy_label"];
    assert_eq!(label.font, "window:y24");
    assert_eq!(label.color, "#ffffff");
    assert_eq!(label.text.as_deref(), Some("Buy"));

    // Button mapped to container slot 0.
    let buy = &shop.buttons["buy"];
    assert_eq!(buy.slots, vec![InventorySlotRef::container(0).into()]);
    assert_eq!(buy.default, Some(ButtonDefault::Close));
    assert!(buy.action);
    assert_eq!(buy.tooltip.as_ref().unwrap().title, "Buy");
    assert_eq!(buy.states["disabled"].item_model.as_deref(), Some("demo:gui/buy_disabled"));
}

#[test]
fn unowned_slot_rect_claims_skip_existing_controls() {
    let (mut w, textures) = sample_window();
    w.slot_rects.push(SlotRectIr {
        name: "fill".into(),
        slots: vec![InventorySlotRef::container(0), InventorySlotRef::container(1), InventorySlotRef::player(0)],
        claim: SlotRectClaim::Unowned,
    });

    let out = compile_windows(&[w], &textures, "window").unwrap();
    let fill = &out.manifest.windows["shop"].slot_rects["fill"];
    assert_eq!(fill.slots, vec![InventorySlotRef::container(1).into(), InventorySlotRef::player(0).into()]);
}

#[test]
fn output_is_byte_identical_across_runs() {
    let (w1, t1) = sample_window();
    let (w2, t2) = sample_window();
    let a = compile_windows(&[w1], &t1, "window").unwrap();
    let b = compile_windows(&[w2], &t2, "window").unwrap();
    assert_eq!(a.files, b.files);
}

#[test]
fn button_over_no_slots_errors() {
    let (mut w, textures) = sample_window();
    // Move the button below the container and player grids (no overlap).
    w.buttons[0].rect = Rect::new(52, 216, 72, 20);
    let err = compile_windows(&[w], &textures, "window").unwrap_err();
    match err {
        Error::Validation(msg) => assert!(msg.contains("shop") && msg.contains("buy")),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn overlapping_buttons_error() {
    let (mut w, textures) = sample_window();
    w.buttons.push(ButtonIr {
        name: "other".into(),
        rect: Rect::new(8, 18, 16, 16),
        slots: None,
        yielded_slots: Vec::new(),
        default: None,
        action: true,
        tooltip: None,
        states: BTreeMap::new(),
        repeat: None,
    });

    let err = compile_windows(&[w], &textures, "window").unwrap_err();
    match err {
        Error::Validation(msg) => assert!(msg.contains("both own container slot 0"), "{msg}"),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn oversized_composite_height_errors() {
    let mut textures = BTreeMap::new();
    textures.insert("big.png".to_string(), solid(4, 4, [1, 1, 1, 255]));
    // A nine-slice stretched to a 600px-tall composite exceeds the 512px
    // provider limit.
    let draw = Draw::NineSlice {
        texture: TextureKey("big.png".into()),
        insets: Insets::uniform(1),
        dest: Rect::new(0, 0, 4, 600),
    };
    let w = bare_window("tall", ContainerKind::Generic9x6, vec![draw]);
    let err = compile_windows(&[w], &textures, "window").unwrap_err();
    assert!(matches!(err, Error::Font(_)));
}

#[test]
fn empty_draws_produce_empty_static_and_no_png() {
    let textures = BTreeMap::new();
    let mut w = bare_window("bare", ContainerKind::Generic9x3, vec![]);
    w.slots = vec![text_slot("title", None, Rect::new(8, 6, 100, 8), Align::Left, Rgb::DEFAULT_TEXT)];
    let out = compile_windows(&[w], &textures, "window").unwrap();
    assert!(!out.files.iter().any(|f| f.path == "assets/window/textures/font/bare.png"));
    assert_eq!(out.manifest.windows["bare"].static_text, "");
    // ui.json has only the space provider (no bitmap).
    let ui: serde_json::Value =
        serde_json::from_slice(find(&out, "assets/window/font/ui.json").contents.as_bytes()).unwrap();
    assert_eq!(ui["providers"].as_array().unwrap().len(), 1);
}

#[test]
fn invalid_namespace_errors() {
    let (w, textures) = sample_window();
    let err = compile_windows(&[w], &textures, "Bad NS").unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}

#[test]
fn static_glyph_codepoint_in_private_use_area() {
    let (w, textures) = sample_window();
    let out = compile_windows(&[w], &textures, "window").unwrap();
    let glyph = out.manifest.windows["shop"].static_text.chars().find(|&c| {
        let cp = c as u32;
        (crate::font::GLYPH_BASE..crate::font::GLYPH_END).contains(&cp)
    });
    assert!(glyph.is_some(), "static string contains a glyph codepoint");
}

fn compile_anvil_search(child: &str, options: &str) -> crate::Result<CompileOutput> {
    let project = format!(
        r##"{{
      "theme":{{
        "frames":{{"recess":{{"kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0}}}},
        "sprites":{{"field":{{"kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0,"width":110,"height":16}}}}
      }},
      "windows":[{{"name":"search","container":"anvil","children":[
        {{"type":"panel","frame":"recess","x":0,"y":0,"width":176,"height":166}},
        {child},
        {{"type":"anvil_input","name":"query"}}
      ]}}],
      "options":{options}
    }}"##
    );
    crate::pipeline::compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new()))
}

fn search_art_alpha(out: &CompileOutput, x: u32, y: u32) -> u8 {
    let art = Texture::decode_png(find(out, "assets/window/textures/font/search.png").contents.as_bytes()).unwrap();
    art.rgba[((y * art.width + x) * 4 + 3) as usize]
}

#[test]
fn anvil_inputs_open_the_art_over_the_native_field_and_can_restyle_it() {
    let label = r#"{"type":"label","text":"Query","x":62,"y":24,"width":103}"#;
    let out = compile_anvil_search(label, r#"{"anvil_field_sprite":"field"}"#).unwrap();
    for path in [
        "assets/minecraft/textures/gui/sprites/container/anvil/text_field.png",
        "assets/minecraft/textures/gui/sprites/container/anvil/text_field_disabled.png",
    ] {
        let field = Texture::decode_png(find(&out, path).contents.as_bytes()).unwrap();
        assert_eq!((field.width, field.height), (110, 16));
    }
    assert_eq!((search_art_alpha(&out, 59, 20), search_art_alpha(&out, 168, 35)), (0, 0));
    assert_eq!((search_art_alpha(&out, 58, 20), search_art_alpha(&out, 59, 36)), (255, 255));
    assert!(out.warnings.iter().any(|w| w.contains("draws over the native anvil text field")));
}

#[test]
fn anvil_inputs_hide_vanilla_anvil_art_and_share_their_slot_with_a_button() {
    let back = r#"{"type":"button","name":"back","x":26,"y":46,"width":18,"height":18}"#;
    let out = compile_anvil_search(back, "{}").unwrap();
    for path in [
        "assets/minecraft/textures/gui/container/anvil.png",
        "assets/minecraft/textures/gui/sprites/container/anvil/error.png",
    ] {
        let art = Texture::decode_png(find(&out, path).contents.as_bytes()).unwrap();
        assert!(art.rgba.chunks(4).all(|pixel| pixel[3] == 0), "{path}");
    }
    let back = &out.manifest.windows["search"].buttons["back"];
    assert_eq!((back.slots.len(), back.fill_slots.as_deref()), (1, Some(&[][..])));
    assert_eq!(out.manifest.windows["search"].inputs["query"].slot, back.slots[0]);
}

#[test]
fn anvil_title_updates_require_the_experimental_option() {
    let slot = r#"{"type":"slot","name":"count","x":8,"y":70,"width":60}"#;
    let err = compile_anvil_search(slot, "{}").unwrap_err();
    assert!(err.to_string().contains("`count` changes the title of an anvil input window"), "{err}");

    let out = compile_anvil_search(slot, r#"{"experimental_anvil_updates":true}"#).unwrap();
    assert!(out.warnings.iter().any(|w| w.contains("`count` reopens the anvil")));
}

#[test]
fn switch_cases_bake_their_own_net_zero_glyphs() {
    let mut textures = BTreeMap::new();
    textures.insert("badge.png".to_string(), solid(8, 8, [255, 0, 0, 255]));
    let badge = |x| Draw::Sprite { texture: TextureKey("badge.png".into()), dest: Rect::new(x, 20, 8, 8) };
    let mut w = bare_window("shop", ContainerKind::Generic9x3, vec![badge(4)]);
    w.switches = vec![crate::ir::SwitchIr {
        name: "mode".into(),
        cases: vec![
            crate::ir::SwitchCaseIr {
                value: "buy".into(),
                draws: vec![badge(30)],
                slots: vec!["price".into()],
                sprite_slots: vec![],
            },
            crate::ir::SwitchCaseIr { value: "sell".into(), draws: vec![], slots: vec![], sprite_slots: vec![] },
        ],
    }];

    let out = compile_windows(&[w], &textures, "window").unwrap();
    crate::validation::validate_compile_output(&out).assert_valid();
    find(&out, "assets/window/textures/font/shop_switch/mode/buy.png");
    let entry = &out.manifest.windows["shop"];
    let cases = &entry.switches["mode"].cases;
    assert_eq!(cases[0].slots, vec!["price"]);
    assert!(!cases[0].static_text.is_empty() && cases[0].static_text != entry.static_text);
    assert_eq!((cases[1].value.as_str(), cases[1].static_text.as_str()), ("sell", ""));
}
