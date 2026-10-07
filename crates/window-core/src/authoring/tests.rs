use super::project_from_json;
use crate::geometry::{Insets, Size};
use crate::inventory::InventorySlotRef;
use crate::ir::{HudChannel, IndexedBinding, IndexedKind, Rgb};
use crate::model::{Element, Frame, SpriteDef};

const PROJECT_JSON: &[u8] = br##"{
      "theme": {
        "frames": {
          "panel": { "texture": "window/sprites/panel.png", "insets": 8 }
        },
        "sprites": {
          "coin": { "texture": "window/sprites/coin.png" }
        }
      },
      "windows": [{
        "name": "shop",
        "container": "generic_9x6",
        "children": [
          { "type": "panel", "frame": "panel", "x": 0, "y": 0, "width": 176, "height": 222,
            "children": [
              { "type": "slot", "name": "title", "width": 160, "align": "center", "color": "#ffd700" },
              { "type": "button", "name": "buy", "width": 18, "height": 18,
                "tooltip": { "title": "Buy", "lines": ["Spend coins"] },
                "states": {
                  "on": { "item_model": "demo:gui/buy_on", "sprite": "coin", "tooltip": "Ready" },
                  "off": { "item_model": "demo:gui/buy_off" }
                }
              },
              { "type": "hotspot", "name": "info", "width": 18, "height": 18,
                "tooltip": "Info"
              },
              { "type": "sprite", "name": "coin" }
            ]
          }
        ]
      }]
    }"##;

#[test]
fn parses_project_json() {
    let project = project_from_json(PROJECT_JSON).unwrap();
    assert_eq!(
        project.theme.frames["panel"],
        Frame::Texture { texture: "window/sprites/panel.png".into(), insets: Insets::uniform(8) }
    );
    assert_eq!(
        project.theme.sprites["coin"],
        SpriteDef::Texture { texture: "window/sprites/coin.png".into(), size: None }
    );
    assert_eq!(project.windows[0].name, "shop");
    assert_eq!(project.windows[0].children.len(), 1);
    let Element::Panel { children, .. } = &project.windows[0].children[0] else {
        panic!("expected panel");
    };
    let Element::Button { tooltip, states, .. } = &children[1] else {
        panic!("expected button");
    };
    assert_eq!(tooltip.as_ref().unwrap().title, "Buy");
    assert_eq!(states["on"].item_model.as_deref(), Some("demo:gui/buy_on"));
    assert_eq!(states["on"].sprite.as_deref(), Some("coin"));
    assert_eq!(states["on"].tooltip.as_ref().unwrap().title, "Ready");
    let Element::Hotspot { tooltip, .. } = &children[2] else {
        panic!("expected hotspot");
    };
    assert_eq!(tooltip.as_ref().unwrap().title, "Info");
}

#[test]
fn rejects_bad_names() {
    let err = project_from_json(br#"{"windows":[{"name":"Shop","container":"generic_9x3"}]}"#).unwrap_err();
    assert!(err.to_string().contains("window name `Shop` is invalid"));
}

#[test]
fn parses_fixed_sprite_slot() {
    let project = project_from_json(
        br#"{
          "theme": { "sprites": { "coin": { "kind": "badge", "width": 8, "height": 8 } } },
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "sprite_slot", "name": "coin_icon",
              "x": 8, "y": 6, "width": 8, "height": 8, "sprite": "coin"
            }]
          }]
        }"#,
    )
    .unwrap();
    let Element::SpriteSlot { sprite, .. } = &project.windows[0].children[0] else {
        panic!("expected sprite slot");
    };
    assert_eq!(sprite.as_deref(), Some("coin"));
}

#[test]
fn rejects_duplicate_theme_names_across_documents() {
    let err = project_from_json(
        br#"{
          "themes": [
            { "frames": { "panel": { "kind": "panel" } } },
            { "frames": { "panel": { "kind": "button" } } }
          ]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("duplicate frame name `panel`"));
}

#[test]
fn merges_theme_colors_across_documents() {
    let project = project_from_json(
        br##"{ "themes": [{ "colors": { "gold": "#FFD75E" } }, { "colors": { "muted": "#a9d9b5" } }] }"##,
    )
    .unwrap();
    assert_eq!(project.theme.colors.get("gold").map(|c| c.to_hex()).as_deref(), Some("#ffd75e"));
    assert_eq!(project.theme.colors.len(), 2);

    let err = project_from_json(
        br##"{ "themes": [{ "colors": { "gold": "#ffd75e" } }, { "colors": { "gold": "#000000" } }] }"##,
    )
    .unwrap_err();
    assert!(err.to_string().contains("duplicate color name `gold`"), "{err}");
    let err = project_from_json(br##"{ "theme": { "colors": { "gold": "ffd75e" } } }"##).unwrap_err();
    assert!(err.to_string().contains("expected #rrggbb"), "{err}");
}

#[test]
fn expands_slot_ranges_in_rust() {
    let json = br#"{
      "windows": [{
        "name": "shop",
        "container": "generic_9x3",
        "children": [{
          "type": "button",
          "name": "buy",
          "width": 18,
          "height": 18,
          "slots": [
            { "area": "container", "first": 3, "last": 1 },
            { "area": "player", "first": 0, "last": 2 }
          ]
        }]
      }]
    }"#;

    let project = project_from_json(json).unwrap();
    let Element::Button { slots: Some(slots), .. } = &project.windows[0].children[0] else {
        panic!("expected button with slots");
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
              "type": "button",
              "name": "buy",
              "width": 18,
              "height": 18,
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
              "type": "button",
              "name": "buy",
              "width": 18,
              "height": 18,
              "claim": "unowned"
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("button element does not accept field `claim`"), "{err}");
}

#[test]
fn rejects_unknown_element_fields() {
    let err = project_from_json(
        br#"{
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "button",
              "name": "buy",
              "width": 18,
              "height": 18,
              "tooltp": "Buy"
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("button element does not accept field `tooltp`"), "{err}");
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
              "name": "entry",
              "pattern": { "kind": "slots", "slots": [1], "x": 0 }
            }]
          }]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("slot pattern does not accept field `x`"), "{err}");
}

#[test]
fn rejects_unknown_theme_asset_fields() {
    let err = project_from_json(
        br##"{
          "theme": {
            "frames": {
              "panel": { "kind": "panel", "widht": 18 }
            }
          }
        }"##,
    )
    .unwrap_err();
    assert!(err.to_string().contains("generated theme asset does not accept field `widht`"), "{err}");
}

#[test]
fn rejects_indicator_color_on_kinds_that_do_not_draw_it() {
    let err = project_from_json(
        br##"{
          "theme": {
            "frames": {
              "warning": { "kind": "hazard_bar", "indicator_color": "#ff8300" }
            }
          }
        }"##,
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
fn parses_generated_theme_assets() {
    let json = br##"{
      "theme": {
        "frames": {
          "panel": {
            "kind": "panel",
            "fill": "#123456",
            "border_color": "#abcdef",
            "border_width": 3,
            "radius": 5,
            "inset_depth": 2
          }
        },
        "sprites": {
          "badge": {
            "kind": "badge",
            "width": 18,
            "height": 14,
            "accent_color": "#00ffff"
          }
        }
      }
    }"##;

    let project = project_from_json(json).unwrap();
    let Frame::Generated(panel) = &project.theme.frames["panel"] else {
        panic!("expected generated panel");
    };
    assert_eq!(panel.fill, Rgb::new(0x12, 0x34, 0x56));
    assert_eq!(panel.border_color, Rgb::new(0xab, 0xcd, 0xef));
    assert_eq!(panel.border_width, 3);
    assert_eq!(panel.radius, 5);
    let SpriteDef::Generated { size, style } = &project.theme.sprites["badge"] else {
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
      "theme": {
        "sprites": {
          "meter": { "texture": "window/sprites/meter.png" }
        }
      },
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
          { "type": "sprite", "name": "meter", "x": 0, "y": 0 },
          { "type": "slot", "name": "coins", "x": 12, "y": 4, "width": 80 }
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
fn flattens_indexed_bindings() {
    let window = |children: &str| {
        format!(r#"{{"windows":[{{"name":"card","container":"generic_9x3","children":[{children}]}}]}}"#)
    };
    let cell = |i: u32, j: u32| format!(r#"{{"type":"slot","name":"hole","index":[{i},{j}],"width":8}}"#);
    let cells: Vec<String> = (0..2).flat_map(|i| (0..3).map(move |j| cell(i, j))).collect();
    let project = project_from_json(window(&cells.join(",")).as_bytes()).unwrap();
    let card = &project.windows[0];
    assert_eq!(card.indexed["hole"], IndexedBinding { kind: IndexedKind::Slot, shape: vec![2, 3] });
    let Element::Slot { name, .. } = &card.children[5] else {
        panic!("expected slot");
    };
    assert_eq!(name, "hole[1][2]");
    let lamp = |i: u32| {
        format!(
            r#"{{"type":"switch","name":"lamp","index":{i},"children":[{{"type":"case","value":"on","children":[]}}]}}"#
        )
    };
    let project = project_from_json(window(&format!("{},{}", lamp(0), lamp(1))).as_bytes()).unwrap();
    let Element::Switch(switch) = &project.windows[0].children[1] else {
        panic!("expected switch");
    };
    assert_eq!(switch.name, "lamp[1]");

    let cases = [
        (cells[..5].join(","), "indexed binding `hole` is missing index [1, 2]"),
        (
            format!(r#"{},{{"type":"sprite_slot","name":"hole","index":[0,0],"width":8,"height":8}}"#, cell(0, 1)),
            "indexed binding `hole` mixes element kinds",
        ),
        (
            r#"{"type":"repeater","name":"row","pattern":{"kind":"grid","area":"container","x":0,"y":0,"columns":2,"rows":1},
                "children":[{"type":"slot","name":"hole","index":[0],"width":8}]}"#
                .to_string(),
            "indexed binding `hole` is inside repeater `row`",
        ),
        (
            r#"{"type":"sprite_slot","name":"dot","index":0,"width":8,"height":8,"sprite":"coin"}"#.to_string(),
            "indexed binding `dot` sets a fixed `sprite`",
        ),
        (r#"{"type":"slot","name":"hole","index":4294967295,"width":8}"#.to_string(), "too large to cover"),
    ];
    for (children, message) in cases {
        let err = project_from_json(window(&children).as_bytes()).unwrap_err();
        assert!(err.to_string().contains(message), "{err}");
    }
}
