use super::*;

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
    let buy = w.regions.iter().find(|b| b.name == "buy").unwrap();
    assert_eq!(buy.rect, Rect::new(116, 139, 52, 34));
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
    assert_eq!(w.regions.len(), 1);
    assert_eq!(w.regions[0].name, "inventory_fill");
    let slots: Vec<_> = w.regions[0].slots.as_ref().unwrap().iter().map(|slot| slot.index).collect();
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
    assert_eq!(w.regions.len(), 1);
    let slots: Vec<_> = w.regions[0].slots.as_ref().unwrap().iter().map(|slot| slot.index).collect();
    assert_eq!(slots, [0, 1]);
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
            Rect::new(25, 17, 18, 18),
            Rect::new(43, 17, 18, 18),
            Rect::new(61, 17, 18, 18),
            Rect::new(25, 35, 18, 18),
            Rect::new(43, 35, 18, 18),
            Rect::new(61, 35, 18, 18),
        ]
    );
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

    let buttons: Vec<_> = w.regions.iter().map(|button| button.name.as_str()).collect();
    assert_eq!(buttons, ["entry_0", "entry_1"]);
    assert_eq!(w.regions[0].rect, Rect::new(8, 54, 52, 34));
    assert_eq!(w.regions[1].rect, Rect::new(62, 54, 52, 34));
    assert_eq!(w.regions[0].repeat.as_ref().unwrap().index, 0);

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
    let cell = &w.regions[0];
    let routed: Vec<u32> = cell.slots.as_ref().unwrap().iter().map(|slot| slot.index).collect();
    assert_eq!(routed, [18, 19, 20, 27, 28, 29]);
    let yielded: Vec<u32> = cell.yielded_slots.iter().map(|slot| slot.index).collect();
    assert_eq!(yielded, [18]);
    assert_eq!(w.regions[1].yielded_slots.iter().map(|slot| slot.index).collect::<Vec<_>>(), [21]);
}

#[test]
fn repeater_cell_item_accepts_the_last_slot_of_the_cell() {
    let w = solve_one(repeater_with_cell_item(json!(6)), &sizes(&[]));
    assert_eq!(w.items[0].slots[0].index, 29);
    assert_eq!(w.regions[0].yielded_slots[0].index, 29);
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
    assert!(w.regions[0].yielded_slots.is_empty());
    assert!(w.regions[1].yielded_slots.is_empty());
}
