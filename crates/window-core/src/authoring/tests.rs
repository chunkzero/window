use serde_json::json;

use super::project_from_json;
use crate::geometry::{Insets, Size};
use crate::inventory::InventorySlotRef;
use crate::ir::{HudChannel, Rgb};
use crate::model::{Element, Frame, SpriteDef};

const PROJECT_JSON: &[u8] = br##"{
      "fonts": { "runes": { "texture": "window/fonts/runes.png", "chars": ["ab"] } },
      "windows": [{
        "name": "shop",
        "container": "generic_9x6",
        "children": [
          { "type": "flex", "frame": { "art": "texture", "texture": "window/sprites/panel.png", "insets": 8 },
            "x": 0, "y": 0, "style": { "width": 176, "height": 222 },
            "children": [
              { "type": "slot", "handle": { "kind": "text", "id": "title" }, "width": 160, "align": "center",
                "color": "#ffd700" },
              { "type": "region", "on_click": { "kind": "action", "id": "buy" }, "width": 18, "height": 18,
                "tooltip": { "title": "Buy", "lines": ["Spend coins"] } },
              { "type": "sprite", "art": { "art": "texture", "texture": "window/sprites/coin.png" } }
            ]
          }
        ]
      }]
    }"##;

#[test]
fn parses_project_json() {
    let project = project_from_json(PROJECT_JSON).unwrap();
    let frames: Vec<&Frame> = project.art.frames.values().collect();
    assert_eq!(frames, [&Frame::Texture { texture: "window/sprites/panel.png".into(), insets: Insets::uniform(8) }]);
    let sprites: Vec<&SpriteDef> = project.art.sprites.values().collect();
    assert_eq!(sprites, [&SpriteDef::Texture { texture: "window/sprites/coin.png".into(), size: None }]);
    assert_eq!(project.fonts["runes"].chars, ["ab"]);
    assert_eq!(project.windows[0].name, "shop");
    let Element::Flex(panel) = &project.windows[0].children[0] else {
        panic!("expected flex");
    };
    let Element::Slot { name, .. } = &panel.children[0].element else {
        panic!("expected slot");
    };
    assert_eq!(name, "title");
    let Element::Region(region) = &panel.children[1].element else {
        panic!("expected region");
    };
    assert_eq!(region.tooltip.as_ref().unwrap().title, "Buy");
    assert_eq!(region.name.as_deref(), Some("buy"));
}

#[test]
fn rejects_named_art_references() {
    let err = project_from_json(
        json!({ "windows": [{ "name": "s", "container": "generic_9x3", "children": [
            { "type": "sprite", "art": "coin", "x": 0, "y": 0 },
        ] }] })
        .to_string()
        .as_bytes(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("expected an art value"), "{err}");
}

#[test]
fn rejects_bad_names() {
    let err = project_from_json(br#"{"windows":[{"name":"Shop","container":"generic_9x3"}]}"#).unwrap_err();
    assert!(err.to_string().contains("window name `Shop` is invalid"));
}

#[test]
fn expands_slot_ranges_in_rust() {
    let json = br#"{
      "windows": [{
        "name": "shop",
        "container": "generic_9x3",
        "children": [{
          "type": "item",
          "handle": { "kind": "items", "id": "buy" },
          "slots": [
            { "area": "container", "first": 3, "last": 1 },
            { "area": "player", "first": 0, "last": 2 }
          ]
        }]
      }]
    }"#;

    let project = project_from_json(json).unwrap();
    let Element::Item { slots: Some(slots), .. } = &project.windows[0].children[0] else {
        panic!("expected item with slots");
    };
    assert_eq!(
        slots.as_slice(),
        &[
            InventorySlotRef::container(3),
            InventorySlotRef::container(2),
            InventorySlotRef::container(1),
            InventorySlotRef::player(0),
            InventorySlotRef::player(1),
            InventorySlotRef::player(2),
        ]
    );
}

#[test]
fn rejects_fractional_slot_range_endpoints() {
    let err = project_from_json(
        br#"{
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "item",
              "handle": { "kind": "items", "id": "buy" },
              "slots": [{ "area": "container", "first": 0, "last": 2.5 }]
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("invalid type"), "{err}");
}

#[test]
fn rejects_fields_that_do_not_belong_to_element_kind() {
    let err = project_from_json(
        br#"{
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "region",
              "width": 18,
              "height": 18,
              "claim": "unowned"
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("region element does not accept field `claim`"), "{err}");
}

#[test]
fn rejects_unknown_element_fields() {
    let err = project_from_json(
        br#"{
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "region",
              "width": 18,
              "height": 18,
              "tooltp": "Buy"
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("region element does not accept field `tooltp`"), "{err}");
}

#[test]
fn rejects_fields_that_do_not_belong_to_pattern_kind() {
    let err = project_from_json(
        br#"{
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "item",
              "handle": { "kind": "items", "id": "entry" },
              "pattern": { "kind": "slots", "slots": [1], "x": 0 }
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("slot pattern does not accept field `x`"), "{err}");
}

#[test]
fn rejects_unknown_art_fields() {
    let err = project_from_json(
        json!({ "windows": [{ "name": "s", "container": "generic_9x3", "children": [
            { "type": "flex", "frame": json!({ "art": "shape", "kind": "panel", "widht": 18 }), "x": 0, "y": 0, "style": { "width": 18, "height": 18 } },
        ] }] })
        .to_string()
        .as_bytes(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("does not accept field `widht`"), "{err}");
}

#[test]
fn rejects_indicator_color_on_kinds_that_do_not_draw_it() {
    let err = project_from_json(
        json!({ "windows": [{ "name": "s", "container": "generic_9x3", "children": [
            { "type": "flex", "frame": json!({ "art": "shape", "kind": "hazard_bar", "indicator_color": "#ff8300" }), "x": 0, "y": 0, "style": { "width": 18, "height": 18 } },
        ] }] })
        .to_string()
        .as_bytes(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("`indicator_color` only applies to panel, button, and slot kinds"), "{err}");
}

#[test]
fn small_caps_selects_the_bundled_font_unless_font_is_set() {
    let label = |fields: &str| {
        let json = format!(
            r#"{{"windows":[{{"name":"s","container":"generic_9x3","children":[{{"type":"label","text":"Hi",{fields}}}]}}]}}"#
        );
        project_from_json(json.as_bytes())
    };
    let project = label(r#""small_caps":true"#).unwrap();
    let Element::Label { style, .. } = &project.windows[0].children[0] else {
        panic!("expected label");
    };
    assert_eq!(style.font.as_deref(), Some("small_caps"));
    let err = label(r#""small_caps":true,"font":"runes""#).unwrap_err();
    assert!(err.to_string().contains("sets both `font` and `small_caps`"), "{err}");
}

#[test]
fn parses_generated_art() {
    let frame = json!({
        "art": "shape",
        "kind": "panel",
        "fill": "#123456",
        "border_color": "#abcdef",
        "border_width": 3,
        "radius": 5,
        "inset_depth": 2
    });
    let badge = json!({ "art": "shape", "kind": "badge", "width": 18, "height": 14, "accent_color": "#00ffff" });
    let project = project_from_json(
        json!({ "windows": [{ "name": "s", "container": "generic_9x3", "children": [
            { "type": "flex", "frame": frame, "x": 0, "y": 0, "style": { "width": 18, "height": 18 } },
            { "type": "sprite", "art": badge, "x": 0, "y": 20 },
        ] }] })
        .to_string()
        .as_bytes(),
    )
    .unwrap();
    let Some(Frame::Generated(panel)) = project.art.frames.values().next() else {
        panic!("expected generated panel");
    };
    assert_eq!(panel.fill, Rgb::new(0x12, 0x34, 0x56));
    assert_eq!(panel.border_color, Rgb::new(0xab, 0xcd, 0xef));
    assert_eq!(panel.border_width, 3);
    assert_eq!(panel.radius, 5);
    let Some(SpriteDef::Generated { size, style }) = project.art.sprites.values().next() else {
        panic!("expected generated sprite");
    };
    assert_eq!(*size, Size::new(18, 14));
    assert_eq!(style.accent_color, Some(Rgb::new(0x00, 0xff, 0xff)));
}

#[test]
fn parses_hud_project_json() {
    let json = br##"{
      "options": { "hud_shaders": true },
      "target": { "pack_format": 84 },
      "huds": [{
        "name": "status",
        "channel": "actionbar",
        "width": 120,
        "height": 16,
        "shader": {
          "source_bottom": 59,
          "origin": { "x": 0.5, "y": 0.08 },
          "anchor": { "x": 0.5, "y": 0.0 },
          "x": 0,
          "y": 0
        },
        "children": [
          { "type": "sprite", "art": { "art": "texture", "texture": "window/sprites/meter.png" }, "x": 0, "y": 0 },
          { "type": "slot", "handle": { "kind": "text", "id": "coins" }, "x": 12, "y": 4, "width": 80 }
        ]
      }]
    }"##;

    let project = project_from_json(json).unwrap();
    assert!(project.options.hud_shaders);
    assert_eq!(project.target.pack_format, Some(84));
    assert_eq!(project.huds[0].name, "status");
    assert_eq!(project.huds[0].channel, HudChannel::ActionBar);
    assert_eq!(project.huds[0].size, Some(Size::new(120, 16)));
    let shader = project.huds[0].shader.unwrap();
    assert_eq!(shader.origin_x, 0.5);
    assert_eq!(shader.origin_y, 0.08);
    assert_eq!(shader.anchor_x, 0.5);
    assert_eq!(shader.anchor_y, 0.0);
    assert_eq!(shader.offset_y, 0);
}

#[test]
fn rejects_invalid_flex_numeric_styles() {
    for fields in [
        serde_json::json!({ "style": { "columns": ["NaNfr"] } }),
        serde_json::json!({ "style": { "columns": ["-1fr"] } }),
        serde_json::json!({ "style": { "aspect_ratio": 0 } }),
        serde_json::json!({ "style": { "aspect_ratio": -1 } }),
        serde_json::json!({ "children": [{ "type": "label", "text": "Hi", "layout": { "grow": -1 } }] }),
        serde_json::json!({ "children": [{ "type": "label", "text": "Hi", "layout": { "shrink": -1 } }] }),
    ] {
        let mut element = fields;
        element["type"] = serde_json::json!("flex");
        let json = serde_json::json!({
            "windows": [{ "name": "test", "container": "generic_9x1", "children": [element] }],
        });
        let err = project_from_json(json.to_string().as_bytes()).unwrap_err();
        assert!(err.to_string().contains("must be"), "{err}");
    }
}

#[test]
fn inline_art_is_shared_by_content_and_named_by_hash() {
    let button = json!({ "art": "shape", "kind": "button", "name": "industrial/button" });
    let project = project_from_json(
        json!({
            "windows": [{ "name": "w", "container": "generic_9x3", "children": [
                { "type": "flex", "frame": button, "x": 0, "y": 0, "style": { "width": 20, "height": 10 } },
                { "type": "flex", "frame": button, "x": 0, "y": 20, "style": { "width": 30, "height": 10 } },
                { "type": "flex", "frame": { "art": "shape", "kind": "panel" }, "x": 0, "y": 40,
                  "style": { "width": 10, "height": 10 } },
            ] }],
        })
        .to_string()
        .as_bytes(),
    )
    .unwrap();
    let frames: Vec<&str> = project.art.frames.keys().map(String::as_str).collect();
    let [hashed, named] = frames.as_slice() else { panic!("two frames: {frames:?}") };
    assert!(named.starts_with("art/industrial-button-") && named.len() == "art/industrial-button-".len() + 6);
    assert!(hashed.starts_with("art/") && hashed.len() == "art/".len() + 6, "{hashed}");
}

#[test]
fn sprite_catalog_holds_inline_art_under_its_key() {
    let lamp = |fill: &str| json!({ "art": "shape", "kind": "badge", "width": 4, "height": 4, "fill": fill });
    let project = project_from_json(
        json!({ "sprites": { "lamp_on": lamp("#00ff00"), "lamp_off": lamp("#ff0000") }, "windows": [] })
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    assert!(matches!(project.art.sprites["lamp_on"], SpriteDef::Generated { .. }));
    assert!(project.art.sprites.contains_key("lamp_off"));
    let err = project_from_json(json!({ "sprites": { "lamp": "coin" }, "windows": [] }).to_string().as_bytes())
        .unwrap_err()
        .to_string();
    assert!(err.contains("expected an art value"), "{err}");
}

#[test]
fn camel_case_catalog_keys_and_only_entries_become_snake_case() {
    let lamp = |fill: &str| json!({ "art": "shape", "kind": "badge", "width": 4, "height": 4, "fill": fill });
    let handle = json!({ "kind": "sprite", "id": "lamp", "only": ["lampOn", "lamp_off"] });
    let project = project_from_json(
        json!({
            "sprites": { "lampOn": lamp("#00ff00"), "lamp_off": lamp("#ff0000") },
            "windows": [{ "name": "s", "container": "generic_9x3", "children": [
                { "type": "sprite_slot", "handle": handle, "width": 4, "height": 4 },
            ] }],
        })
        .to_string()
        .as_bytes(),
    )
    .unwrap();
    assert!(project.art.sprites.contains_key("lamp_on"));
    assert_eq!(project.windows[0].handles["lamp"].only, ["lamp_on", "lamp_off"]);

    let clash = project_from_json(
        json!({ "sprites": { "lampOn": lamp("#00ff00"), "lamp_on": lamp("#ff0000") }, "windows": [] })
            .to_string()
            .as_bytes(),
    )
    .unwrap_err()
    .to_string();
    assert!(clash.contains("`lampOn` and `lamp_on` both become `lamp_on`"), "{clash}");
    let mixed =
        project_from_json(json!({ "sprites": { "Lamp_on": lamp("#00ff00") }, "windows": [] }).to_string().as_bytes());
    assert!(mixed.is_err());
}
