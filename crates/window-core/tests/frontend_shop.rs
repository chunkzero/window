//! End-to-end frontend test: a shop window built from primitives, parsed and solved into the layout IR, asserting
//! exact rects, draw order, generated label names, region mappings, and text alignment.

use std::collections::HashMap;

use serde_json::{Value, json};
use window_core::authoring::{ParsedProject, project_from_json};
use window_core::geometry::{Rect, Size};
use window_core::ir::{Align, CLOSE_ACTION, Draw};
use window_core::layout::solve;
use window_core::surface::Surface;
use window_core::text_font::TextFonts;

fn textures(entries: &[(&str, u32, u32)]) -> impl Fn(&str) -> Option<Size> {
    let map: HashMap<String, Size> = entries.iter().map(|(p, w, h)| (p.to_string(), Size::new(*w, *h))).collect();
    move |p: &str| map.get(p).copied()
}

/// A framed, fixed-size box whose region runs `click` and whose label is centered.
fn button(click: Value, width: u32, height: u32, label: Value) -> Value {
    json!({
        "type": "flex",
        "frame": { "art": "texture", "texture": "window/sprites/button.png", "insets": 4 },
        "style": { "width": width, "height": height, "justify": "center", "align": "center" },
        "children": [{ "type": "region", "on_click": click }, label],
    })
}

fn action(id: &str) -> Value {
    json!({ "kind": "action", "id": id })
}

fn project() -> ParsedProject {
    let white = |text: &str| json!({ "type": "label", "text": text, "color": "#ffffff" });
    let doc = json!({
      "windows": [{
        "name": "shop",
        "container": "generic_9x6",
        "children": [{
          "type": "flex",
          "frame": { "art": "texture", "texture": "window/sprites/panel.png", "insets": 8 },
          "x": 0,
          "y": 0,
          "style": { "width": 176, "height": 222, "padding": { "left": 8, "top": 6 } },
          "children": [{
            "type": "flex",
            "style": { "direction": "column", "gap": 6, "align": "start" },
            "children": [
              { "type": "slot", "handle": { "kind": "text", "id": "title" }, "width": 160, "align": "center",
                "color": "#ffd700" },
              {
                "type": "flex",
                "style": { "gap": 4, "align": "start" },
                "children": [
                  button(action("minus"), 18, 18, white("-")),
                  { "type": "slot", "handle": { "kind": "text", "id": "quantity" }, "width": 40, "align": "center" },
                  button(action("plus"), 18, 18, white("+")),
                ]
              },
              { "type": "sprite", "art": { "art": "texture", "texture": "window/sprites/coin.png" } },
              button(action("buy"), 72, 20, white("Buy")),
              button(json!({ "kind": "builtin", "id": CLOSE_ACTION }), 72, 20,
                json!({ "type": "label", "text": "Close" })),
            ]
          }]
        }]
      }]
    });
    project_from_json(doc.to_string().as_bytes()).expect("parse")
}

fn solved() -> window_core::ir::LaidOutWindow {
    let tx = textures(&[
        ("window/sprites/panel.png", 24, 24),
        ("window/sprites/button.png", 16, 16),
        ("window/sprites/coin.png", 16, 16),
    ]);
    solve(&project(), &tx, &TextFonts::new()).expect("solve").pop().expect("one window")
}

#[test]
fn surface_and_identity() {
    let w = solved();
    assert_eq!(w.name, "shop");
    assert_eq!(w.surface, Surface::Container(window_core::surface::ContainerKind::Generic9x6));
    assert_eq!(w.surface.gui_size(), Size::new(176, 222));
}

#[test]
fn title_slot_rect_and_style() {
    let w = solved();
    let title = w.slots.iter().find(|s| s.name == "title").unwrap();
    assert_eq!(title.rect, Rect::new(8, 6, 160, 8));
    assert_eq!(title.align, Align::Center);
    assert_eq!(title.color.to_hex(), "#ffd700");
    assert!(title.text.is_none(), "dynamic slot carries no text");
}

#[test]
fn rows_and_columns_flow_with_their_gaps() {
    let w = solved();
    let region = |name: &str| w.regions.iter().find(|r| r.name == name).unwrap().rect;
    // The row starts below the title: y = 6 + 8 + 6.
    assert_eq!(region("minus"), Rect::new(8, 20, 18, 18));
    let quantity = w.slots.iter().find(|s| s.name == "quantity").unwrap();
    assert_eq!(quantity.rect, Rect::new(30, 20, 40, 8));
    assert_eq!(region("plus"), Rect::new(74, 20, 18, 18));
    let coin = w
        .draws
        .iter()
        .find_map(|d| match d {
            Draw::Sprite { dest, texture } if texture.0.contains("coin") => Some(*dest),
            _ => None,
        })
        .expect("coin sprite draw");
    assert_eq!(coin, Rect::new(8, 44, 16, 16));
    assert_eq!(region("buy"), Rect::new(8, 66, 72, 20));
    let close = w.regions.iter().find(|r| r.name == CLOSE_ACTION).unwrap();
    assert_eq!((close.rect, close.default_action.as_deref()), (Rect::new(8, 92, 72, 20), Some(CLOSE_ACTION)));
}

#[test]
fn draw_order_parents_before_children() {
    let w = solved();
    match &w.draws[0] {
        Draw::NineSlice { texture, .. } => assert!(texture.0.contains("panel")),
        other => panic!("expected panel frame first, got {other:?}"),
    }
    let nine = w.draws.iter().filter(|d| matches!(d, Draw::NineSlice { .. })).count();
    assert_eq!(nine, 5);
}

#[test]
fn labels_get_generated_names_in_document_order() {
    let w = solved();
    let labels: Vec<(&str, &str)> =
        w.slots.iter().filter_map(|s| Some((s.name.as_str(), s.text.as_deref()?))).collect();
    assert_eq!(labels, [("label_0", "-"), ("label_1", "+"), ("label_2", "Buy"), ("label_3", "Close")]);
    let close = w.slots.iter().find(|s| s.text.as_deref() == Some("Close")).unwrap();
    assert_eq!(close.color.to_hex(), "#404040");
}

#[test]
fn every_region_maps_into_container_slots() {
    let w = solved();
    for region in &w.regions {
        assert!(region.slots.as_ref().is_some_and(|slots| !slots.is_empty()), "region `{}`", region.name);
    }
    assert!(w.warnings.is_empty(), "unexpected warnings: {:?}", w.warnings);
}
