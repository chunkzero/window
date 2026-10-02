use super::project_from_json;
use crate::geometry::{Insets, Size};
use crate::inventory::InventorySlotRef;
use crate::ir::{HudChannel, Rgb};
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
    assert_eq!(project.huds[0].size, Size::new(120, 16));
    let shader = project.huds[0].shader.unwrap();
    assert_eq!(shader.origin_x, 0.5);
    assert_eq!(shader.origin_y, 0.08);
    assert_eq!(shader.anchor_x, 0.5);
    assert_eq!(shader.anchor_y, 0.0);
    assert_eq!(shader.offset_y, 0);
}
