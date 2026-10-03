//! End-to-end frontend test: a realistic TypeScript-authored shop window parsed and
//! solved into the layout IR, asserting exact rects, draw order, generated
//! label names, button mappings, and the text-alignment metadata.

use std::collections::HashMap;

use window_core::authoring::{ParsedProject, project_from_json};
use window_core::geometry::{Rect, Size};
use window_core::ir::{Align, ButtonDefault, Draw};
use window_core::layout::solve;
use window_core::surface::Surface;
use window_core::text_font::TextFonts;
use window_core::vanilla;

fn textures(entries: &[(&str, u32, u32)]) -> impl Fn(&str) -> Option<Size> {
    let map: HashMap<String, Size> = entries.iter().map(|(p, w, h)| (p.to_string(), Size::new(*w, *h))).collect();
    move |p: &str| map.get(p).copied()
}

fn project() -> ParsedProject {
    let doc = r##"{
      "theme": {
        "frames": {
          "panel": { "texture": "window/sprites/panel.png", "insets": 8 },
          "button": { "texture": "window/sprites/button.png", "insets": 4 }
        },
        "sprites": {
          "coin": { "texture": "window/sprites/coin.png" }
        }
      },
      "windows": [{
        "name": "shop",
        "container": "generic_9x6",
        "children": [{
          "type": "panel",
          "frame": "panel",
          "x": 0,
          "y": 0,
          "width": 176,
          "height": 222,
          "children": [{
            "type": "column",
            "x": 8,
            "y": 6,
            "gap": 6,
            "children": [
              {
                "type": "slot",
                "name": "title",
                "width": 160,
                "align": "center",
                "color": "#ffd700"
              },
              {
                "type": "row",
                "gap": 4,
                "children": [
                  {
                    "type": "button",
                    "name": "minus",
                    "frame": "button",
                    "width": 18,
                    "height": 18,
                    "children": [{
                      "type": "label",
                      "text": "-",
                      "align": "center",
                      "color": "#ffffff"
                    }]
                  },
                  {
                    "type": "slot",
                    "name": "quantity",
                    "width": 40,
                    "align": "center"
                  },
                  {
                    "type": "button",
                    "name": "plus",
                    "frame": "button",
                    "width": 18,
                    "height": 18,
                    "children": [{
                      "type": "label",
                      "text": "+",
                      "align": "center",
                      "color": "#ffffff"
                    }]
                  }
                ]
              },
              { "type": "sprite", "name": "coin" },
              {
                "type": "button",
                "name": "buy",
                "frame": "button",
                "width": 72,
                "height": 20,
                "children": [{
                  "type": "label",
                  "text": "Buy",
                  "align": "center",
                  "color": "#ffffff"
                }]
              },
              {
                "type": "button",
                "name": "exit",
                "frame": "button",
                "width": 72,
                "height": 20,
                "default": "close",
                "children": [{
                  "type": "label",
                  "text": "Close",
                  "align": "center"
                }]
              }
            ]
          }]
        }]
      }]
    }"##;
    project_from_json(doc.as_bytes()).expect("parse")
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
    // column content origin = (8,6); title is first child.
    assert_eq!(title.rect, Rect::new(8, 6, 160, 8));
    assert_eq!(title.align, Align::Center);
    assert_eq!(title.color.to_hex(), "#ffd700");
    assert!(title.text.is_none(), "dynamic slot carries no text");
}

#[test]
fn quantity_row_flows_with_gap_and_cross_start() {
    let w = solved();
    // Row begins below title: y = 6 + 8 (title) + 6 (gap) = 20.
    let minus = w.buttons.iter().find(|b| b.name == "minus").unwrap();
    let plus = w.buttons.iter().find(|b| b.name == "plus").unwrap();
    let quantity = w.slots.iter().find(|s| s.name == "quantity").unwrap();
    assert_eq!(minus.rect, Rect::new(8, 20, 18, 18));
    // quantity after minus(18)+gap(4) → x=30; default cross align start → y=20.
    assert_eq!(quantity.rect, Rect::new(30, 20, 40, 8));
    // plus after quantity(40)+gap(4) → x=74.
    assert_eq!(plus.rect, Rect::new(74, 20, 18, 18));
}

#[test]
fn coin_sprite_after_row() {
    let w = solved();
    // Row height = max child height = 18. Next column child y = 20 + 18 + 6 = 44.
    let coin = w
        .draws
        .iter()
        .find_map(|d| match d {
            Draw::Sprite { dest, texture } if texture.0.contains("coin") => Some(*dest),
            _ => None,
        })
        .expect("coin sprite draw");
    assert_eq!(coin, Rect::new(8, 44, 16, 16));
}

#[test]
fn buy_and_exit_buttons_stack() {
    let w = solved();
    // After coin (16 tall) + gap 6: buy y = 44 + 16 + 6 = 66.
    let buy = w.buttons.iter().find(|b| b.name == "buy").unwrap();
    assert_eq!(buy.rect, Rect::new(8, 66, 72, 20));
    assert_eq!(buy.default, None);
    // exit after buy (20 tall) + gap 6: y = 66 + 20 + 6 = 92.
    let exit = w.buttons.iter().find(|b| b.name == "exit").unwrap();
    assert_eq!(exit.rect, Rect::new(8, 92, 72, 20));
    assert_eq!(exit.default, Some(ButtonDefault::Close));
}

#[test]
fn draw_order_parents_before_children() {
    let w = solved();
    // First draw is the panel frame (drawn before its children's frames).
    match &w.draws[0] {
        Draw::NineSlice { texture, .. } => assert!(texture.0.contains("panel")),
        other => panic!("expected panel frame first, got {other:?}"),
    }
    // Total NineSlice draws: panel + 4 button frames = 5.
    let nine = w.draws.iter().filter(|d| matches!(d, Draw::NineSlice { .. })).count();
    assert_eq!(nine, 5);
}

#[test]
fn labels_get_generated_names_in_document_order() {
    let w = solved();
    let labels: Vec<&str> = w.slots.iter().filter(|s| s.text.is_some()).map(|s| s.name.as_str()).collect();
    assert_eq!(labels, ["label_0", "label_1", "label_2", "label_3"]);
    // Their texts, in order: "-", "+", "Buy", "Close".
    let texts: Vec<&str> = w.slots.iter().filter_map(|s| s.text.as_deref()).collect();
    assert_eq!(texts, ["-", "+", "Buy", "Close"]);
}

#[test]
fn label_default_color_when_unspecified() {
    let w = solved();
    // The "Close" label has no color → default #404040.
    let close = w.slots.iter().find(|s| s.text.as_deref() == Some("Close")).unwrap();
    assert_eq!(close.color.to_hex(), "#404040");
}

#[test]
fn buy_label_centered_width_matches_text() {
    let w = solved();
    let buy = w.slots.iter().find(|s| s.text.as_deref() == Some("Buy")).unwrap();
    assert_eq!(buy.rect.width, vanilla::text_visible_width("Buy"));
    assert_eq!(buy.align, Align::Center);
}

#[test]
fn all_buttons_map_into_container_slots() {
    let w = solved();
    let kind = window_core::surface::ContainerKind::Generic9x6;
    // Every button must overlap at least one container slot for clicks to land.
    for b in &w.buttons {
        let overlap = kind.slots_overlapping(&b.rect);
        assert!(!overlap.is_empty(), "button `{}` at {:?} overlaps no container slot", b.name, b.rect);
    }
}

#[test]
fn no_warnings_for_in_bounds_shop() {
    let w = solved();
    assert!(w.warnings.is_empty(), "unexpected warnings: {:?}", w.warnings);
}
