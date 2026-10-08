use super::*;

#[test]
fn rect_pattern_resolves_the_player_section_to_backing_slots() {
    let w = solve_one(
        tall(json!([{
            "type": "item",
            "handle": { "kind": "items", "id": "buy" },
            "pattern": { "kind": "rect", "section": "player", "x": 6, "y": 0, "width": 3, "height": 2 }
        }])),
        &sizes(&[]),
    );
    let slots: Vec<_> = w.items[0].slots.iter().map(|slot| slot.index).collect();
    assert_eq!(slots, [15, 16, 17, 24, 25, 26]);
}

#[test]
fn collection_frame_draws_each_resolved_inventory_cell() {
    let frame = json!({ "art": "shape", "kind": "slot", "fill": "#0b536d", "border_color": "#22b8e6" });
    let w = solve_one(
        window(json!([{
            "type": "collection",
            "handle": { "kind": "collection", "id": "results" },
            "frame": frame,
            "transform": { "section": "container", "x": 1, "y": 0, "width": 3, "height": 2 }
        }])),
        &sizes(&[]),
    );

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
