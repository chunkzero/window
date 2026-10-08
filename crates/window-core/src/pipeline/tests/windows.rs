use super::*;
use crate::inventory::InventorySlotRef;
use crate::manifest::SlotRefEntry;

#[test]
fn indexed_switch_art_uses_valid_resource_paths() {
    let project = r##"{
      "windows":[{"name":"shop","container":"generic_9x3","children":[
        {"type":"switch","handle":{"kind":"flag","id":"lamp","shape":[1],"at":[0]},"x":8,"y":20,"children":[
            {"type":"case","value":"true","children":[{"type":"sprite","art":{"art":"shape","kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0,"width":4,"height":4}}]},
            {"type":"case","value":"false","children":[]}
        ]}
      ]}]
    }"##;
    let input = crate::pipeline::CompileInput::new(std::collections::BTreeMap::new());
    let out = crate::pipeline::compile_project_json(project.as_bytes(), &input).unwrap();
    let paths: Vec<&str> = out.files.iter().map(|file| file.path.as_str()).collect();
    assert!(paths.iter().any(|path| path.contains("_switch/lamp-0/true")), "{paths:?}");
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

    // Region mapped to container slot 0.
    let buy = &shop.regions["buy"];
    assert_eq!(buy.slots, vec![InventorySlotRef::container(0).into()]);
    assert_eq!(buy.action.as_deref(), Some("buy"));
    assert_eq!(buy.default_action.as_deref(), Some(CLOSE_ACTION));
    assert_eq!(buy.hitbox.as_ref().unwrap().tooltip.as_ref().unwrap().title, "Buy");
}

#[test]
fn unowned_slot_rect_claims_skip_existing_controls() {
    let (mut w, textures) = sample_window();
    let slots = vec![InventorySlotRef::container(0), InventorySlotRef::container(1), InventorySlotRef::player(0)];
    w.regions.push(RegionIr { unowned: true, ..region("fill", Rect::default(), Some(slots)) });

    let out = compile_windows(&[w], &textures, "window").unwrap();
    let fill = &out.manifest.windows["shop"].regions["fill"];
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
    w.regions[0].rect = Rect::new(52, 216, 72, 20);
    let err = compile_windows(&[w], &textures, "window").unwrap_err();
    match err {
        Error::Validation(msg) => assert!(msg.contains("shop") && msg.contains("buy")),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn overlapping_buttons_error() {
    let (mut w, textures) = sample_window();
    w.regions.push(region("other", Rect::new(8, 18, 16, 16), None));

    let err = compile_windows(&[w], &textures, "window").unwrap_err();
    match err {
        Error::Validation(msg) => assert!(msg.contains("both route container slot 0"), "{msg}"),
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
      "sprites":{{"field":{{"art":"shape","kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0,"width":110,"height":16}}}},
      "windows":[{{"name":"search","container":"anvil","children":[
        {{"type":"flex","frame":{{"art":"shape","kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0}},
          "x":0,"y":0,"style":{{"width":176,"height":166}}}},
        {child},
        {{"type":"anvil_input","handle":{{"kind":"input","id":"query"}}}}
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
    let back = r#"{"type":"flex","x":26,"y":46,"children":[
        {"type":"region","on_click":{"kind":"action","id":"back"},"width":18,"height":18}]}"#;
    let out = compile_anvil_search(back, "{}").unwrap();
    for path in [
        "assets/minecraft/textures/gui/container/anvil.png",
        "assets/minecraft/textures/gui/sprites/container/anvil/error.png",
    ] {
        let art = Texture::decode_png(find(&out, path).contents.as_bytes()).unwrap();
        assert!(art.rgba.chunks(4).all(|pixel| pixel[3] == 0), "{path}");
    }
    let back = &out.manifest.windows["search"].regions["back"];
    assert_eq!((back.slots.len(), back.fill_slots.as_deref()), (1, Some(&[][..])));
    assert_eq!(out.manifest.windows["search"].inputs["query"].slot, back.slots[0]);
}

#[test]
fn anvil_title_updates_require_the_experimental_option() {
    let slot = r#"{"type":"slot","handle":{"kind":"text","id":"count"},"x":8,"y":70,"width":60}"#;
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
        source: None,
        cases: vec![
            crate::ir::SwitchCaseIr {
                value: "buy".into(),
                draws: vec![badge(30)],
                slots: vec!["price".into()],
                ..Default::default()
            },
            crate::ir::SwitchCaseIr { value: "sell".into(), ..Default::default() },
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

fn compile_children(children: &str) -> crate::Result<CompileOutput> {
    let project = format!(r#"{{"windows":[{{"name":"shop","container":"generic_9x3","children":{children}}}]}}"#);
    crate::pipeline::compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new()))
}

#[test]
fn multi_line_slots_reserve_their_lines_and_a_font_per_line_position() {
    let out = compile_children(
        r#"[{"type":"flex","x":8,"y":20,"style":{"direction":"column"},"children":[
            {"type":"slot","handle":{"kind":"text","id":"name"},"width":46,"lines":2,"line_height":7},
            {"type":"label","text":"Below"}
        ]}]"#,
    )
    .unwrap();
    crate::validation::validate_compile_output(&out).assert_valid();
    let slots = &out.manifest.windows["shop"].slots;
    let name = &slots["name"];
    assert_eq!((name.y, name.height(), name.font.as_str()), (20, 15, "window:y14"));
    assert_eq!(name.overflow, Some(crate::manifest::TextOverflow::Ellipsis));
    let lines = name.lines.as_ref().unwrap();
    assert_eq!((lines.count, lines.line_height), (2, 7));
    assert_eq!(lines.fonts, ["window:y14", "window:y17", "window:y21"]);
    find(&out, "assets/window/font/y17.json");
    assert_eq!(slots["label_0"].y, 35);

    let err = compile_children(r#"[{"type":"label","text":"Hi","x":8,"y":20,"overflow":"ellipsis"}]"#).unwrap_err();
    assert!(err.to_string().contains("label element does not accept `overflow`"), "{err}");
    let err = compile_children(
        r#"[{"type":"slot","handle":{"kind":"text","id":"n"},"width":9,"lines":2,"line_height":4294967295}]"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("`lines` span more than 1024 pixels"), "{err}");
}

#[test]
fn regions_in_exclusive_cases_share_slots_and_real_overlaps_are_rejected() {
    let region = |id: &str| format!(r#"{{"type":"region","on_click":{{"kind":"action","id":"{id}"}}}}"#);
    let switch = format!(
        r#"{{"type":"switch","handle":{{"kind":"flag","id":"open"}},"x":7,"y":17,"children":[
            {{"type":"case","value":"true","style":{{"width":18,"height":18}},"children":[{}]}},
            {{"type":"case","value":"false","style":{{"width":18,"height":18}},"children":[{}]}}]}}"#,
        region("buy"),
        region("sell"),
    );
    let out = compile_children(&format!("[{switch}]")).unwrap();
    crate::validation::validate_compile_output(&out).assert_valid();
    let shop = &out.manifest.windows["shop"];
    assert_eq!(shop.regions["buy"].slots, vec![InventorySlotRef::container(0).into()]);
    assert_eq!(shop.regions["sell"].slots, shop.regions["buy"].slots);

    let info =
        format!(r#"{{"type":"flex","x":7,"y":17,"style":{{"width":18,"height":18}},"children":[{}]}}"#, region("info"));
    let err = compile_children(&format!("[{switch},{info}]")).unwrap_err();
    assert!(err.to_string().contains("both route container slot 0"), "{err}");
}

#[test]
fn regions_route_clicks_on_item_slots_and_fill_only_unowned_slots() {
    let item = |id: &str, index: u32| {
        format!(
            r#"{{"type":"item","handle":{{"kind":"items","id":"{id}"}},"slots":[{{"area":"container","index":{index}}}]}}"#
        )
    };
    let region = |id: &str, width: u32| {
        format!(
            r#"{{"type":"flex","x":7,"y":17,"style":{{"width":{width},"height":18}},"children":[
                {{"type":"region","on_click":{{"kind":"action","id":"{id}"}}}}]}}"#
        )
    };
    let out = compile_children(&format!("[{},{},{}]", item("icon", 0), item("price", 1), region("cell", 54))).unwrap();
    crate::validation::validate_compile_output(&out).assert_valid();
    let shop = &out.manifest.windows["shop"];
    let refs = |indices: &[u32]| -> Vec<SlotRefEntry> {
        indices.iter().map(|index| InventorySlotRef::container(*index).into()).collect()
    };
    assert_eq!(shop.regions["cell"].slots, refs(&[0, 1, 2]));
    assert_eq!(shop.regions["cell"].fill_slots, Some(refs(&[2])));
    assert_eq!(shop.items["icon"].slots, refs(&[0]));

    let out = compile_children(&format!("[{},{}]", item("icon", 0), region("cell", 18))).unwrap();
    crate::validation::validate_compile_output(&out).assert_valid();
    assert_eq!(out.manifest.windows["shop"].regions["cell"].fill_slots, Some(Vec::new()));

    let err = compile_children(&format!("[{},{}]", item("icon", 0), item("other", 0))).unwrap_err();
    assert!(err.to_string().contains("both own container slot 0"), "{err}");
}

#[test]
fn conditional_content_owners_keep_the_region_fill() {
    let region = r#"{"type":"flex","x":7,"y":17,"style":{"width":18,"height":18},"children":[
        {"type":"region","on_click":{"kind":"action","id":"tip"}}]}"#;
    let item = r#"{"type":"item","handle":{"kind":"items","id":"icon"},"slots":[{"area":"container","index":0}]}"#;
    let switch = format!(
        r#"{{"type":"switch","handle":{{"kind":"flag","id":"shown"}},"x":7,"y":17,"children":[
            {{"type":"case","value":"true","style":{{"width":18,"height":18}},"children":[{item}]}},
            {{"type":"case","value":"false","style":{{"width":18,"height":18}},"children":[]}}]}}"#
    );
    let out = compile_children(&format!("[{switch},{region}]")).unwrap();
    crate::validation::validate_compile_output(&out).assert_valid();
    assert_eq!(out.manifest.windows["shop"].regions["tip"].fill_slots, None);

    let out = compile_children(&format!("[{item},{region}]")).unwrap();
    assert_eq!(out.manifest.windows["shop"].regions["tip"].fill_slots, Some(Vec::new()));
}

fn compile_with_png(project: &str, path: &str) -> CompileOutput {
    let png = solid(4, 4, [255, 0, 0, 255]).encode_png().unwrap();
    let input = crate::pipeline::CompileInput::new(BTreeMap::from([(path.to_string(), png)]));
    crate::pipeline::compile_project_json(project.as_bytes(), &input).unwrap()
}

#[test]
fn inline_resource_texture_image_resolves_to_its_source_path() {
    let project = r#"{"windows":[{"name":"shop","container":"generic_9x3","children":[
      {"type":"sprite","art":{"art":"texture","texture":"demo:item/coin.png","width":4,"height":4},"x":8,"y":20}
    ]}]}"#;
    compile_with_png(project, "assets/demo/textures/item/coin.png");
}

#[test]
fn inline_resource_texture_frame_resolves_to_its_source_path() {
    let project = r#"{"windows":[{"name":"shop","container":"generic_9x3","children":[
      {"type":"flex","frame":{"art":"texture","texture":"demo:item/coin.png","insets":1},"x":8,"y":20,
       "style":{"width":12,"height":12}}
    ]}]}"#;
    compile_with_png(project, "assets/demo/textures/item/coin.png");
}
