use std::collections::HashMap;

use super::*;
use crate::authoring::project_from_json;
use crate::ir::{Align, Rgb};
use serde_json::{Value, json};

/// Build a texture-size closure from a name→(w,h) map.
fn sizes(entries: &[(&str, u32, u32)]) -> impl Fn(&str) -> Option<Size> + 'static {
    let map: HashMap<String, Size> = entries.iter().map(|(p, w, h)| (p.to_string(), Size::new(*w, *h))).collect();
    move |p: &str| map.get(p).copied()
}

fn project(value: Value) -> ParsedProject {
    project_from_json(value.to_string().as_bytes()).expect("parse project")
}

fn window(children: Value) -> ParsedProject {
    project(json!({
        "windows": [{
            "name": "s",
            "container": "generic_9x3",
            "children": children,
        }],
    }))
}

fn themed(theme: Value, children: Value) -> ParsedProject {
    project(json!({
        "theme": theme,
        "windows": [{
            "name": "s",
            "container": "generic_9x6",
            "children": children,
        }],
    }))
}

fn solve_one(project: ParsedProject, tx: &dyn Fn(&str) -> Option<Size>) -> LaidOutWindow {
    solve(&project, tx).expect("solve").pop().expect("window")
}

#[test]
fn column_flow_stacks_with_gap() {
    let w = solve_one(
        window(json!([{
            "type": "column", "x": 8, "y": 6, "gap": 4,
            "children": [
                { "type": "slot", "name": "a", "width": 10 },
                { "type": "slot", "name": "b", "width": 20 }
            ]
        }])),
        &sizes(&[]),
    );
    let a = w.slots.iter().find(|s| s.name == "a").unwrap();
    let b = w.slots.iter().find(|s| s.name == "b").unwrap();
    // a at content origin (8,6); b below by height(8)+gap(4) = 12 → y=18.
    assert_eq!(a.rect, Rect::new(8, 6, 10, 8));
    assert_eq!(b.rect, Rect::new(8, 18, 20, 8));
}

#[test]
fn row_flow_advances_with_gap() {
    let w = solve_one(
        window(json!([{
            "type": "row", "x": 8, "y": 6, "gap": 4,
            "children": [
                { "type": "slot", "name": "a", "width": 10 },
                { "type": "slot", "name": "b", "width": 20 }
            ]
        }])),
        &sizes(&[]),
    );
    let a = w.slots.iter().find(|s| s.name == "a").unwrap();
    let b = w.slots.iter().find(|s| s.name == "b").unwrap();
    assert_eq!(a.rect, Rect::new(8, 6, 10, 8));
    // b after a(10) + gap(4) → x=22.
    assert_eq!(b.rect, Rect::new(22, 6, 20, 8));
}

#[test]
fn row_cross_alignment() {
    // A tall sprite (16px) and a short slot (8px); center alignment pushes
    // the slot down by (16-8)/2 = 4.
    let w = solve_one(
        themed(
            json!({ "sprites": { "tall": { "texture": "tall.png" } } }),
            json!([{
                "type": "row", "x": 8, "y": 6, "align": "center",
                "children": [
                    { "type": "sprite", "name": "tall" },
                    { "type": "slot", "name": "short", "width": 10 }
                ]
            }]),
        ),
        &sizes(&[("tall.png", 16, 16)]),
    );
    let short = w.slots.iter().find(|s| s.name == "short").unwrap();
    assert_eq!(short.rect.y, 6 + 4);
}

#[test]
fn explicit_pos_is_out_of_flow() {
    // The middle element has explicit pos; it should not consume flow.
    let w = solve_one(
        window(json!([{
            "type": "row", "x": 8, "y": 6,
            "children": [
                { "type": "slot", "name": "a", "width": 10 },
                { "type": "slot", "name": "floating", "width": 5, "x": 100, "y": 20 },
                { "type": "slot", "name": "b", "width": 10 }
            ]
        }])),
        &sizes(&[]),
    );
    let a = w.slots.iter().find(|s| s.name == "a").unwrap();
    let b = w.slots.iter().find(|s| s.name == "b").unwrap();
    let f = w.slots.iter().find(|s| s.name == "floating").unwrap();
    assert_eq!(a.rect, Rect::new(8, 6, 10, 8));
    // b flows right after a (10) with no contribution from floating.
    assert_eq!(b.rect, Rect::new(18, 6, 10, 8));
    // floating is content_origin (8,6) + (100,20).
    assert_eq!(f.rect, Rect::new(108, 26, 5, 8));
}

#[test]
fn panel_children_overlay_without_pos() {
    let w = solve_one(
        themed(
            json!({ "frames": { "p": { "texture": "p.png", "insets": 4 } } }),
            json!([{
                "type": "panel", "frame": "p", "x": 0, "y": 0, "width": 176, "height": 40, "padding": 8,
                "children": [
                    { "type": "slot", "name": "a", "width": 10 },
                    { "type": "slot", "name": "b", "width": 20, "x": 4, "y": 4 }
                ]
            }]),
        ),
        &sizes(&[("p.png", 16, 16)]),
    );
    let a = w.slots.iter().find(|s| s.name == "a").unwrap();
    let b = w.slots.iter().find(|s| s.name == "b").unwrap();
    // content origin = panel(0,0) + padding(8,8).
    assert_eq!(a.rect, Rect::new(8, 8, 10, 8));
    // b at content_origin + pos.
    assert_eq!(b.rect, Rect::new(12, 12, 20, 8));
    // Frame drawn as NineSlice over the full panel.
    assert_eq!(w.draws.len(), 1);
    assert!(matches!(w.draws[0], Draw::NineSlice { .. }));
}

#[test]
fn label_auto_width_from_text() {
    let w = solve_one(
        window(json!([
            { "type": "label", "text": "Hi", "x": 8, "y": 6 }
        ])),
        &sizes(&[]),
    );
    let l = &w.slots[0];
    assert_eq!(l.name, "label_0");
    assert_eq!(l.text.as_deref(), Some("Hi"));
    assert_eq!(l.rect.width, vanilla::text_visible_width("Hi"));
}

#[test]
fn bold_label_auto_width_uses_bold_visible_width() {
    let w = solve_one(
        window(json!([
            { "type": "label", "text": "Hi", "x": 8, "y": 6, "bold": true }
        ])),
        &sizes(&[]),
    );
    let l = &w.slots[0];
    assert!(l.bold);
    assert_eq!(l.rect.width, vanilla::bold_text_visible_width("Hi"));
}

#[test]
fn generated_label_names_in_order() {
    let w = solve_one(
        window(json!([{
            "type": "column", "x": 8, "y": 6,
            "children": [
                { "type": "label", "text": "a" },
                { "type": "label", "text": "b" }
            ]
        }])),
        &sizes(&[]),
    );
    assert_eq!(w.slots[0].name, "label_0");
    assert_eq!(w.slots[1].name, "label_1");
}

#[test]
fn sprite_intrinsic_size_and_draw() {
    let w = solve_one(
        themed(
            json!({ "sprites": { "coin": { "texture": "coin.png" } } }),
            json!([{ "type": "sprite", "name": "coin", "x": 8, "y": 6 }]),
        ),
        &sizes(&[("coin.png", 12, 12)]),
    );
    match &w.draws[0] {
        Draw::Sprite { dest, texture } => {
            assert_eq!(*dest, Rect::new(8, 6, 12, 12));
            assert_eq!(texture.0, "coin.png");
        }
        other => panic!("expected sprite, got {other:?}"),
    }
}

#[test]
fn generated_sprite_intrinsic_size_and_draw() {
    let w = solve_one(
        themed(
            json!({
                "sprites": {
                    "badge": {
                        "kind": "badge",
                        "width": 18,
                        "height": 14,
                        "fill": "#ff8700"
                    }
                }
            }),
            json!([{ "type": "sprite", "name": "badge", "x": 8, "y": 6 }]),
        ),
        &sizes(&[]),
    );
    match &w.draws[0] {
        Draw::Generated { dest, style } => {
            assert_eq!(*dest, Rect::new(8, 6, 18, 14));
            assert_eq!(style.fill, Rgb::new(0xff, 0x87, 0x00));
        }
        other => panic!("expected generated sprite, got {other:?}"),
    }
}

#[test]
fn missing_sprite_texture_errors() {
    let project = themed(
        json!({ "sprites": { "coin": { "texture": "coin.png" } } }),
        json!([{ "type": "sprite", "name": "coin", "x": 8, "y": 6 }]),
    );
    let err = solve(&project, &sizes(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("missing texture"), "{msg}");
    assert!(msg.contains("coin.png"), "{msg}");
}

#[test]
fn generated_frame_draws_without_texture_lookup() {
    let w = solve_one(
        themed(
            json!({
                "frames": {
                    "panel": {
                        "kind": "panel",
                        "fill": "#123456",
                        "border_color": "#abcdef"
                    }
                }
            }),
            json!([{ "type": "panel", "frame": "panel", "x": 0, "y": 0, "width": 40, "height": 20 }]),
        ),
        &sizes(&[]),
    );
    match &w.draws[0] {
        Draw::Generated { dest, style } => {
            assert_eq!(*dest, Rect::new(0, 0, 40, 20));
            assert_eq!(style.border_color, Rgb::new(0xab, 0xcd, 0xef));
        }
        other => panic!("expected generated frame, got {other:?}"),
    }
}

#[test]
fn frame_too_small_for_insets_errors() {
    // insets uniform 8 → needs >=17x17, but texture is 10x10.
    let project = themed(
        json!({ "frames": { "p": { "texture": "p.png", "insets": 8 } } }),
        json!([{ "type": "panel", "frame": "p", "x": 0, "y": 0, "width": 176, "height": 40 }]),
    );
    let err = solve(&project, &sizes(&[("p.png", 10, 10)])).unwrap_err();
    assert!(err.to_string().contains("too small"), "{err}");
}

#[test]
fn duplicate_names_within_window_error() {
    let project = themed(
        json!({}),
        json!([{
            "type": "column", "x": 8, "y": 6,
            "children": [
                { "type": "slot", "name": "dup", "width": 10 },
                { "type": "button", "name": "dup", "width": 10, "height": 10 }
            ]
        }]),
    );
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("duplicate name `dup`"), "{err}");
}

#[test]
fn slot_outside_gui_errors() {
    let project = window(json!([
        { "type": "slot", "name": "a", "width": 10, "x": 200, "y": 6 }
    ]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("outside"), "{msg}");
}

#[test]
fn text_above_ascent_limit_errors() {
    // title_origin.y = 6 → min y is 5; y=0 is too high.
    let project = window(json!([
        { "type": "slot", "name": "a", "width": 10, "x": 8, "y": 0 }
    ]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("ascent limit"), "{err}");
}

#[test]
fn draw_overflow_is_a_warning() {
    // A sprite extending past the right edge warns but does not error.
    let w = solve_one(
        themed(
            json!({ "sprites": { "wide": { "texture": "wide.png" } } }),
            json!([{ "type": "sprite", "name": "wide", "x": 170, "y": 6 }]),
        ),
        &sizes(&[("wide.png", 20, 8)]),
    );
    assert_eq!(w.slots.len(), 0);
    assert_eq!(w.draws.len(), 1);
    assert_eq!(w.warnings.len(), 1);
    assert!(w.warnings[0].contains("extends outside"), "{:?}", w.warnings);
}

#[test]
fn bleed_allows_visual_overflow_but_not_text_overflow() {
    let visual_project = project(json!({
        "theme": {
            "frames": {
                "panel": { "kind": "panel" }
            }
        },
        "windows": [{
            "name": "s",
            "container": "generic_9x3",
            "bleed": { "top": 10, "left": 6 },
            "children": [
                { "type": "panel", "frame": "panel", "x": -6, "y": -10, "width": 20, "height": 20 }
            ]
        }]
    }));
    let w = solve(&visual_project, &sizes(&[])).unwrap().pop().unwrap();
    assert!(w.warnings.is_empty(), "bleed should suppress visual overflow warnings");

    let text_project = project(json!({
        "windows": [{
            "name": "s",
            "container": "generic_9x3",
            "bleed": { "left": 10 },
            "children": [
                { "type": "slot", "name": "bad", "x": -4, "y": 6, "width": 10 }
            ]
        }]
    }));
    let err = solve(&text_project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("outside"), "{err}");
}

#[test]
fn slot_rect_pattern_resolves_player_section_to_backing_slots_and_rect() {
    let w = solve_one(
        project(json!({
            "windows": [{
                "name": "s",
                "container": "generic_9x6",
                "children": [{
                    "type": "button",
                    "name": "buy",
                    "pattern": {
                        "kind": "rect",
                        "section": "player",
                        "x": 6,
                        "y": 0,
                        "width": 3,
                        "height": 2
                    }
                }]
            }]
        })),
        &sizes(&[]),
    );
    let buy = w.buttons.iter().find(|b| b.name == "buy").unwrap();
    assert_eq!(buy.rect, Rect::new(116, 140, 52, 34));
    let slots: Vec<_> = buy.slots.as_ref().unwrap().iter().map(|slot| slot.index).collect();
    assert_eq!(slots, [15, 16, 17, 24, 25, 26]);
}

#[test]
fn slot_rects_draw_once_per_slot_and_carry_claim() {
    let w = solve_one(
        themed(
            json!({ "frames": { "slot": { "kind": "slot" } } }),
            json!([{
                "type": "slot_rects",
                "name": "inventory_fill",
                "frame": "slot",
                "claim": "unowned",
                "pattern": {
                    "kind": "rect",
                    "section": "hotbar",
                    "x": 0,
                    "y": 0,
                    "width": 2,
                    "height": 1
                }
            }]),
        ),
        &sizes(&[]),
    );
    assert_eq!(w.draws.len(), 2);
    assert_eq!(w.slot_rects.len(), 1);
    assert_eq!(w.slot_rects[0].name, "inventory_fill");
    let slots: Vec<_> = w.slot_rects[0].slots.iter().map(|slot| slot.index).collect();
    assert_eq!(slots, [0, 1]);
}

#[test]
fn slot_rects_can_claim_without_drawing_frames() {
    let w = solve_one(
        themed(
            json!({ "frames": { "slot": { "kind": "slot" } } }),
            json!([{
                "type": "slot_rects",
                "name": "inventory_fill",
                "claim": "unowned",
                "pattern": {
                    "kind": "rect",
                    "section": "hotbar",
                    "x": 0,
                    "y": 0,
                    "width": 2,
                    "height": 1
                }
            }]),
        ),
        &sizes(&[]),
    );
    assert!(w.draws.is_empty());
    assert_eq!(w.slot_rects.len(), 1);
    let slots: Vec<_> = w.slot_rects[0].slots.iter().map(|slot| slot.index).collect();
    assert_eq!(slots, [0, 1]);
}

#[test]
fn repeater_flattens_cells_and_attaches_group_metadata() {
    let w = solve_one(
        themed(
            json!({ "frames": { "card": { "kind": "button" } } }),
            json!([{
                "type": "repeater",
                "name": "entry",
                "frame": "card",
                "pattern": {
                    "kind": "grid",
                    "section": "container",
                    "x": 0,
                    "y": 2,
                    "columns": 2,
                    "rows": 1,
                    "cell_width": 3,
                    "cell_height": 2
                },
                "children": [
                    { "type": "sprite_slot", "name": "icon", "x": 17, "y": 8, "width": 18, "height": 18 },
                    { "type": "slot", "name": "price", "x": 21, "y": 28, "width": 29, "align": "center" }
                ]
            }]),
        ),
        &sizes(&[]),
    );

    let buttons: Vec<_> = w.buttons.iter().map(|button| button.name.as_str()).collect();
    assert_eq!(buttons, ["entry_0", "entry_1"]);
    assert_eq!(w.buttons[0].rect, Rect::new(8, 54, 52, 34));
    assert_eq!(w.buttons[1].rect, Rect::new(62, 54, 52, 34));
    assert_eq!(w.buttons[0].repeat.as_ref().unwrap().index, 0);

    let price = w.slots.iter().find(|slot| slot.name == "entry_price_1").unwrap();
    let repeat = price.repeat.as_ref().unwrap();
    assert_eq!(repeat.group, "entry");
    assert_eq!(repeat.field.as_deref(), Some("price"));
    assert_eq!(repeat.index, 1);

    let icon = w.sprite_slots.iter().find(|slot| slot.name == "entry_icon_0").unwrap();
    assert_eq!(icon.repeat.as_ref().unwrap().field.as_deref(), Some("icon"));
}

/// A 3x2 grid repeater whose cells each carry an item pinned to `cell_slot`.
fn repeater_with_cell_item(cell_slot: Value) -> ParsedProject {
    themed(
        json!({ "frames": { "card": { "kind": "button" } } }),
        json!([{
            "type": "repeater",
            "name": "entry",
            "frame": "card",
            "pattern": {
                "kind": "grid",
                "section": "container",
                "x": 0,
                "y": 2,
                "columns": 2,
                "rows": 1,
                "cell_width": 3,
                "cell_height": 2
            },
            "children": [
                { "type": "item", "name": "icon", "cell_slot": cell_slot },
                { "type": "slot", "name": "price", "x": 21, "y": 28, "width": 29 }
            ]
        }]),
    )
}

#[test]
fn repeater_cell_item_resolves_per_cell_and_is_yielded_by_the_cell_button() {
    let w = solve_one(repeater_with_cell_item(json!(1)), &sizes(&[]));

    // Cell 0 covers container slots 18,19,20,27,28,29; cell 1 covers 21..23,30..32.
    let items: Vec<(&str, Vec<u32>)> =
        w.items.iter().map(|item| (item.name.as_str(), item.slots.iter().map(|slot| slot.index).collect())).collect();
    assert_eq!(items, [("entry_icon_0", vec![18]), ("entry_icon_1", vec![21])]);

    let repeat = w.items[1].repeat.as_ref().unwrap();
    assert_eq!(repeat.group, "entry");
    assert_eq!(repeat.field.as_deref(), Some("icon"));
    assert_eq!(repeat.index, 1);

    // The cell button still routes every cell slot, but yields the item's slot.
    let cell = &w.buttons[0];
    let routed: Vec<u32> = cell.slots.as_ref().unwrap().iter().map(|slot| slot.index).collect();
    assert_eq!(routed, [18, 19, 20, 27, 28, 29]);
    let yielded: Vec<u32> = cell.yielded_slots.iter().map(|slot| slot.index).collect();
    assert_eq!(yielded, [18]);
    assert_eq!(w.buttons[1].yielded_slots.iter().map(|slot| slot.index).collect::<Vec<_>>(), [21]);
}

#[test]
fn repeater_cell_item_accepts_the_last_slot_of_the_cell() {
    let w = solve_one(repeater_with_cell_item(json!(6)), &sizes(&[]));
    assert_eq!(w.items[0].slots[0].index, 29);
    assert_eq!(w.buttons[0].yielded_slots[0].index, 29);
}

#[test]
fn cell_slot_out_of_range_names_the_repeater_and_the_cell_size() {
    let err = solve(&repeater_with_cell_item(json!(7)), &sizes(&[])).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("`cell_slot` 7"), "{message}");
    assert!(message.contains("repeater `entry`"), "{message}");
    assert!(message.contains("6 slot(s) per cell"), "{message}");
    assert!(message.contains("1..=6"), "{message}");
}

#[test]
fn cell_slot_outside_a_repeater_is_an_error() {
    let project = window(json!([{ "type": "item", "name": "icon", "cell_slot": 1 }]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("outside a repeater"), "{message}");
}

#[test]
fn two_cell_items_cannot_claim_the_same_cell_slot() {
    let project = themed(
        json!({ "frames": { "card": { "kind": "button" } } }),
        json!([{
            "type": "repeater",
            "name": "entry",
            "frame": "card",
            "transform": { "section": "container", "x": 0, "y": 2, "width": 3, "height": 2 },
            "children": [
                { "type": "item", "name": "icon", "cell_slot": 1 },
                { "type": "item", "name": "badge", "cell_slot": 1 }
            ]
        }]),
    );
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("another item already claims"), "{err}");
}

#[test]
fn absolute_item_slots_inside_a_repeater_still_resolve_absolutely() {
    let project = themed(
        json!({ "frames": { "card": { "kind": "button" } } }),
        json!([{
            "type": "repeater",
            "name": "entry",
            "frame": "card",
            "pattern": {
                "kind": "grid",
                "section": "container",
                "x": 0,
                "y": 2,
                "columns": 2,
                "rows": 1,
                "cell_width": 3,
                "cell_height": 2
            },
            "children": [
                { "type": "item", "name": "icon", "slots": [4] }
            ]
        }]),
    );
    let w = solve(&project, &sizes(&[])).expect("solve").pop().unwrap();
    // Both cells resolve to the same absolute slot, exactly as before this feature.
    assert_eq!(w.items[0].slots[0].index, 4);
    assert_eq!(w.items[1].slots[0].index, 4);
    assert!(w.buttons[0].yielded_slots.is_empty());
    assert!(w.buttons[1].yielded_slots.is_empty());
}

#[test]
fn nested_shop_window_exact_rects() {
    // Realistic shop: panel → column → (slot, row of [button,slot,button]).
    let project = project(json!({
        "theme": {
            "frames": {
                "panel": { "texture": "panel.png", "insets": 8 },
                "button": { "texture": "button.png", "insets": 4 }
            }
        },
        "windows": [{
            "name": "shop",
            "container": "generic_9x6",
            "children": [{
                "type": "panel", "frame": "panel", "x": 0, "y": 0, "width": 176, "height": 222,
                "children": [{
                    "type": "column", "x": 8, "y": 6, "gap": 6,
                    "children": [
                        { "type": "slot", "name": "title", "width": 160, "align": "center" },
                        {
                            "type": "row", "gap": 4,
                            "children": [
                                { "type": "button", "name": "minus", "frame": "button", "width": 18, "height": 18,
                                  "children": [{ "type": "label", "text": "-", "align": "center" }] },
                                { "type": "slot", "name": "quantity", "width": 40, "align": "center" },
                                { "type": "button", "name": "plus", "frame": "button", "width": 18, "height": 18,
                                  "children": [{ "type": "label", "text": "+", "align": "center" }] }
                            ]
                        }
                    ]
                }]
            }]
        }]
    }));
    let tx = sizes(&[("panel.png", 24, 24), ("button.png", 12, 12)]);
    let w = solve(&project, &tx).unwrap().pop().unwrap();

    let title = w.slots.iter().find(|s| s.name == "title").unwrap();
    assert_eq!(title.rect, Rect::new(8, 6, 160, 8));
    assert_eq!(title.align, Align::Center);

    // Row begins below title: y = 6 + 8 + 6 = 20.
    let minus = w.buttons.iter().find(|b| b.name == "minus").unwrap();
    assert_eq!(minus.rect, Rect::new(8, 20, 18, 18));
    // quantity: after minus(18)+gap(4) → x=30; row height 18, slot 8 tall,
    // default cross align start → y=20.
    let quantity = w.slots.iter().find(|s| s.name == "quantity").unwrap();
    assert_eq!(quantity.rect, Rect::new(30, 20, 40, 8));
    // plus: after quantity(40)+gap(4) → x=74.
    let plus = w.buttons.iter().find(|b| b.name == "plus").unwrap();
    assert_eq!(plus.rect, Rect::new(74, 20, 18, 18));

    // Button frames + panel frame = 3 NineSlice draws.
    let nine = w.draws.iter().filter(|d| matches!(d, Draw::NineSlice { .. })).count();
    assert_eq!(nine, 3);

    // Labels without an explicit width reserve their visible ink width.
    let dash = w.slots.iter().find(|s| s.text.as_deref() == Some("-")).unwrap();
    // Direct, unpositioned button children center as a line box.
    assert_eq!(dash.rect.x, 8 + (18 - vanilla::text_visible_width("-") as i32).div_euclid(2));
    assert_eq!(dash.rect.y, 25);
    assert_eq!(dash.rect.width, vanilla::text_visible_width("-"));
}

#[test]
fn button_centers_unpositioned_static_and_dynamic_text() {
    let w = solve_one(
        window(json!([
            {
                "type": "button", "name": "static", "x": 8, "y": 18,
                "width": 34, "height": 16, "padding": 2,
                "children": [{ "type": "label", "text": "OK", "bold": true }]
            },
            {
                "type": "button", "name": "dynamic",
                "transform": { "section": "container", "x": 2, "y": 0, "width": 1, "height": 1 },
                "children": [{ "type": "slot", "name": "caption" }]
            }
        ])),
        &sizes(&[]),
    );

    let label = w.slots.iter().find(|slot| slot.text.as_deref() == Some("OK")).unwrap();
    let label_width = vanilla::bold_text_visible_width("OK");
    assert_eq!(label.rect, Rect::new(10 + (30 - label_width as i32) / 2, 22, label_width, 8));
    assert_eq!(label.align, Align::Center);

    let caption = w.slots.iter().find(|slot| slot.name == "caption").unwrap();
    assert_eq!(caption.rect, Rect::new(44, 22, 16, 8));
    assert_eq!(caption.align, Align::Center);
}

#[test]
fn explicit_button_child_position_and_alignment_are_preserved() {
    let w = solve_one(
        window(json!([{
            "type": "button", "name": "b", "x": 8, "y": 18,
            "width": 34, "height": 16, "padding": 2,
            "children": [{
                "type": "slot", "name": "caption", "x": 3, "y": 1,
                "width": 20, "align": "right"
            }]
        }])),
        &sizes(&[]),
    );
    let caption = w.slots.iter().find(|slot| slot.name == "caption").unwrap();
    assert_eq!(caption.rect, Rect::new(13, 21, 20, 8));
    assert_eq!(caption.align, Align::Right);
}

#[test]
fn widthless_slot_outside_button_is_rejected() {
    let error =
        solve(&window(json!([{ "type": "slot", "name": "caption", "x": 8, "y": 18 }])), &sizes(&[])).unwrap_err();
    assert!(error.to_string().contains("direct, unpositioned button child"));
}

#[test]
fn explicit_controls_warn_when_they_cross_reserved_gutters() {
    let w = solve_one(
        window(json!([{
            "type": "button", "name": "crossing", "x": 8, "y": 136,
            "width": 16, "height": 12
        }])),
        &sizes(&[]),
    );
    assert!(w.warnings.iter().any(|warning| warning.contains("player-to-hotbar gutter")));
}

#[test]
fn exact_section_controls_do_not_warn_about_gutters() {
    let w = solve_one(
        window(json!([{
            "type": "button", "name": "exact",
            "transform": { "section": "hotbar", "x": 0, "y": 0, "width": 1, "height": 1 }
        }])),
        &sizes(&[]),
    );
    assert!(w.warnings.is_empty());
    assert_eq!(w.buttons[0].rect, Rect::new(8, 144, 16, 16));
}

#[test]
fn explicit_control_warns_when_artwork_differs_from_backing_slots() {
    let w = solve_one(
        window(json!([{
            "type": "button", "name": "misaligned", "x": 152, "y": 7,
            "width": 16, "height": 16,
            "transform": { "section": "container", "x": 8, "y": 0, "width": 1, "height": 1 }
        }])),
        &sizes(&[]),
    );
    assert_eq!(w.buttons[0].rect, Rect::new(152, 7, 16, 16));
    assert!(w.warnings.iter().any(|warning| {
        warning.contains("control `misaligned` draws at")
            && warning.contains("backing slots are bounded by")
            && warning.contains("omit x/y/width/height")
    }));
}

#[test]
fn sparse_anvil_control_warns_but_keeps_native_bounds() {
    let w = solve_one(
        project(json!({
            "windows": [{
                "name": "search", "container": "anvil",
                "children": [{
                    "type": "button", "name": "wide",
                    "transform": { "section": "container", "x": 1, "y": 0, "width": 2, "height": 1 }
                }]
            }]
        })),
        &sizes(&[]),
    );
    assert_eq!(w.buttons[0].rect, Rect::new(76, 47, 74, 16));
    assert!(w.warnings.iter().any(|warning| warning.contains("nonuniform anvil container slots")));
}

#[test]
fn static_label_width_overflow_is_reported() {
    let w = solve_one(
        window(json!([{
            "type": "label", "text": "OVERFLOW", "x": 8, "y": 18, "width": 8
        }])),
        &sizes(&[]),
    );
    assert!(w.warnings.iter().any(|warning| warning.contains("text `OVERFLOW`") && warning.contains("reserves 8px")));
}

#[test]
fn static_label_warns_for_unsupported_shifted_font_glyphs() {
    let w = solve_one(
        window(json!([{
            "type": "label", "text": "STATUS 😀", "x": 8, "y": 18
        }])),
        &sizes(&[]),
    );
    assert!(
        w.warnings
            .iter()
            .any(|warning| { warning.contains("unsupported shifted-font glyph") && warning.contains("U+1F600") })
    );
}

#[test]
fn static_label_accepts_standard_ui_markers() {
    let w = solve_one(
        window(json!([{
            "type": "label", "text": "× ▲ ▼ ◆ ● ·", "x": 8, "y": 18
        }])),
        &sizes(&[]),
    );
    assert!(w.warnings.is_empty(), "{:?}", w.warnings);
}

#[test]
fn collection_frame_draws_each_resolved_inventory_cell() {
    let project = project(json!({
        "theme": {
            "frames": {
                "cell": {
                    "kind": "slot",
                    "fill": "#0b536d",
                    "border_color": "#22b8e6"
                }
            }
        },
        "windows": [{
            "name": "browser",
            "container": "generic_9x3",
            "children": [{
                "type": "collection",
                "name": "results",
                "frame": "cell",
                "transform": {
                    "section": "container",
                    "x": 1,
                    "y": 0,
                    "width": 3,
                    "height": 2
                }
            }]
        }]
    }));
    let w = solve(&project, &sizes(&[])).unwrap().pop().unwrap();

    assert_eq!(w.collections[0].slots.len(), 6);
    let frame_rects: Vec<Rect> = w
        .draws
        .iter()
        .filter_map(|draw| match draw {
            Draw::Generated { dest, .. } => Some(*dest),
            _ => None,
        })
        .collect();
    assert_eq!(
        frame_rects,
        vec![
            Rect::new(26, 18, 16, 16),
            Rect::new(44, 18, 16, 16),
            Rect::new(62, 18, 16, 16),
            Rect::new(26, 36, 16, 16),
            Rect::new(44, 36, 16, 16),
            Rect::new(62, 36, 16, 16),
        ]
    );
}

#[test]
fn anvil_input_requires_anvil_and_reserves_the_first_slot() {
    let search_project = project(json!({
        "windows": [{
            "name": "search",
            "container": "anvil",
            "children": [{
                "type": "anvil_input",
                "name": "query",
                "initial": "Search...",
                "item_model": "demo:gui/search"
            }]
        }]
    }));
    let w = solve(&search_project, &sizes(&[])).unwrap().pop().unwrap();
    assert_eq!(w.inputs.len(), 1);
    assert_eq!(w.inputs[0].name, "query");
    assert_eq!(w.inputs[0].initial, "Search...");
    assert_eq!(w.inputs[0].item_model.as_deref(), Some("demo:gui/search"));

    let wrong_surface = project(json!({
        "windows": [{
            "name": "search",
            "container": "generic_9x3",
            "children": [{ "type": "anvil_input", "name": "query" }]
        }]
    }));
    let error = solve(&wrong_surface, &sizes(&[])).unwrap_err();
    assert!(error.to_string().contains("requires container `anvil`"));
}
