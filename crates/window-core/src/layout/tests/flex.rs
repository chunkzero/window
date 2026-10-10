use serde_json::json;

use super::{panel, project, sizes, solve, solve_one, text, window};
use crate::geometry::Rect;
use crate::inventory::InventorySlotRef;
use crate::ir::Align;
use crate::vanilla;

fn region_slots(laid: &crate::ir::LaidOutWindow, name: &str) -> Vec<InventorySlotRef> {
    laid.regions.iter().find(|b| b.name == name).and_then(|b| b.slots.clone()).expect("region slots")
}

fn region(id: &str, layout: serde_json::Value) -> serde_json::Value {
    json!({ "type": "region", "on_click": { "kind": "action", "id": id }, "layout": layout })
}

#[test]
fn section_children_auto_flow_through_slot_tracks() {
    let laid = solve_one(
        window(json!([{
            "type": "section",
            "section": "container",
            "children": [
                region("a", json!({ "column": { "span": 3 } })),
                region("b", json!({ "column": { "span": 7 } })),
                region("c", json!({ "column": 9, "row": 3 })),
            ],
        }])),
        &sizes(&[]),
    );
    assert_eq!(region_slots(&laid, "a"), (0..3).map(InventorySlotRef::container).collect::<Vec<_>>());
    // `b` does not fit beside `a`, so it wraps to the next row.
    assert_eq!(region_slots(&laid, "b"), (9..16).map(InventorySlotRef::container).collect::<Vec<_>>());
    assert_eq!(region_slots(&laid, "c"), vec![InventorySlotRef::container(26)]);
    let claim = laid.regions.iter().find(|r| r.name == "container_section").expect("section claim");
    assert_eq!(claim.slots.as_ref().map(Vec::len), Some(27));
}

#[test]
fn section_patterns_are_relative_to_their_area() {
    let item = |id: &str, pattern: serde_json::Value| json!({ "type": "item", "handle": { "kind": "items", "id": id }, "pattern": pattern });
    let laid = solve_one(
        window(json!([{
            "type": "section",
            "section": "player",
            "claim": "none",
            "children": [
                item("skip", json!({ "x": 0, "y": 0, "width": 4, "height": 1 })),
                item("go", json!({ "x": 1, "y": 0, "width": 2, "height": 1 })),
            ],
        }])),
        &sizes(&[]),
    );
    // `go` auto-places after the 4-slot block with a 3x1 area and covers its last two slots.
    let go = laid.items.iter().find(|item| item.name == "go").expect("item");
    assert_eq!(go.slots, vec![InventorySlotRef::player(14), InventorySlotRef::player(15)]);
    assert!(laid.regions.iter().all(|r| r.name != "player_section"));
}

#[test]
fn section_overflow_is_an_error() {
    let err = solve(
        &window(json!([{
            "type": "section",
            "section": "hotbar",
            "children": [
                region("a", json!({ "column": { "span": 6 } })),
                region("b", json!({ "column": { "span": 6 } })),
            ],
        }])),
        &sizes(&[]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("region `b` does not fit the hotbar section"), "{err}");
}

#[test]
fn flex_text_fills_and_centers_in_its_box() {
    let laid = solve_one(
        window(json!([{
            "type": "flex",
            "x": 10,
            "y": 20,
            "style": { "width": 100, "height": 20, "direction": "row", "gap": 4, "padding": 2 },
            "children": [
                { "type": "label", "text": "Hi" },
                { "type": "slot", "handle": text("value"), "align": "right" },
                { "type": "slot", "handle": text("fixed"), "width": 10, "layout": { "translate": [0, 1] } },
            ],
        }])),
        &sizes(&[]),
    );
    let label = laid.slots.iter().find(|s| s.text.as_deref() == Some("Hi")).expect("label");
    let value = laid.slots.iter().find(|s| s.name == "value").expect("value");
    let fixed = laid.slots.iter().find(|s| s.name == "fixed").expect("fixed");
    assert_eq!(label.rect.x, 12);
    assert_eq!(label.rect.y, 26);
    // The unsized dynamic slot grows into the space between the label and the fixed slot.
    assert_eq!(value.rect, Rect::new(label.rect.right() + 4, 26, 78 - label.rect.width, 8));
    assert_eq!(value.align, Align::Right);
    assert_eq!(fixed.rect, Rect::new(98, 27, 10, 8));
}

#[test]
fn widthless_static_text_stretches_across_a_column_and_keeps_its_row_sizing() {
    let boxed = |direction: &str, width: u32| {
        let laid = solve_one(
            window(json!([{
                "type": "flex",
                "x": 10,
                "y": 20,
                "style": { "width": width, "height": 34, "direction": direction },
                "children": [
                    { "type": "label", "text": "Coins", "align": "center", "layout": { "shrink": 1 } },
                    { "type": "slot", "handle": text("coins"), "align": "center" },
                ],
            }])),
            &sizes(&[]),
        );
        laid.slots.iter().find(|s| s.text.as_deref() == Some("Coins")).expect("label").clone()
    };
    let column = boxed("column", 52);
    assert_eq!((column.rect.x, column.rect.width), (10, 52));
    assert_eq!(column.align, Align::Center);
    let row = boxed("row", 52);
    assert_eq!((row.rect.x, row.rect.width), (10, vanilla::text_visible_width("Coins")));
    // A row narrower than the text can still shrink it.
    assert_eq!(boxed("row", 20).rect.width, 20);
}

#[test]
fn auto_sized_box_fills_a_section_cell_and_draws_its_frame() {
    let laid = solve_one(
        project(json!({
            "windows": [{
                "name": "s",
                "container": "generic_9x3",
                "children": [{
                    "type": "section",
                    "section": "container",
                    "children": [{
                        "type": "flex",
                        "frame": panel(),
                        "layout": { "column": { "start": 2, "span": 3 }, "row": 2 },
                        "children": [{ "type": "slot", "handle": text("v") }],
                    }],
                }],
            }],
        })),
        &sizes(&[]),
    );
    // Columns 1..4 of row 1: 18x18 slot boxes starting at (7 + 18, 17 + 18).
    let cell = Rect::new(25, 35, 54, 18);
    assert_eq!(*laid.draws[0].dest(), cell);
    let v = laid.slots.iter().find(|s| s.name == "v").expect("slot");
    assert_eq!(v.rect, Rect::new(25, 40, 54, 8));
}

#[test]
fn hud_without_size_fits_its_content() {
    let project = project(json!({
        "huds": [{
            "name": "h",
            "children": [{
                "type": "flex",
                "style": { "direction": "column", "padding": 3, "gap": 2 },
                "children": [
                    { "type": "slot", "handle": text("a"), "width": 40 },
                    { "type": "slot", "handle": text("b"), "width": 20 },
                ],
            }],
        }],
    }));
    let fonts = crate::text_font::resolve(&Default::default(), &Default::default()).unwrap();
    let hud = crate::layout::solve_huds(&project, &sizes(&[]), &fonts).unwrap().pop().unwrap();
    assert_eq!((hud.width, hud.height), (46, 24));
}

#[test]
fn section_visual_translation_moves_the_subtree_in_pixels() {
    let laid = solve_one(
        window(json!([{
            "type": "section", "section": "container", "claim": "none",
            "children": [{
                "type": "flex", "layout": { "column": { "span": 3 }, "translate": [2, 1] },
                "children": [{ "type": "slot", "handle": text("value") }],
            }],
        }])),
        &sizes(&[]),
    );
    assert_eq!(laid.slots[0].rect, Rect::new(9, 23, 54, 8));
}

#[test]
fn section_patterns_must_fit_their_authored_grid_area() {
    let err = solve(
        &window(json!([{
            "type": "section", "section": "container",
            "children": [{
                "type": "item", "handle": { "kind": "items", "id": "wide" },
                "pattern": { "x": 0, "y": 0, "width": 3, "height": 1 },
                "layout": { "column": { "span": 1 } },
            }],
        }])),
        &sizes(&[]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("pattern does not fit its 1x1 slot grid area"), "{err}");
}

#[test]
fn section_rejects_implicit_negative_tracks_and_oversized_patterns() {
    let children = [
        region("bad", json!({ "column": -11 })),
        region("bad", json!({ "column": { "span": 65535 } })),
        json!({ "type": "item", "handle": { "kind": "items", "id": "bad" }, "pattern": { "x": 4294967295u32, "y": 0, "width": 2, "height": 1 } }),
    ];
    for child in children {
        let err =
            solve(&window(json!([{ "type": "section", "section": "container", "children": [child] }])), &sizes(&[]))
                .unwrap_err();
        assert!(err.to_string().contains("section"), "{err}");
    }
}

#[test]
fn section_rejects_pixel_layout_that_would_change_slot_ownership() {
    for layout in [json!({ "margin": 1 }), json!({ "justify_self": "center" }), json!({ "translate": [1, 0] })] {
        let err = solve(
            &window(json!([{
                "type": "section", "section": "container",
                "children": [region("bad", layout)],
            }])),
            &sizes(&[]),
        )
        .unwrap_err();
        assert!(err.to_string().contains("pixel layout") || err.to_string().contains("cannot be translated"), "{err}");
    }
}
