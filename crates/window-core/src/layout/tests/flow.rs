use super::*;

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

/// Panel → column → (slot, row of [button, slot, button]).
fn nested_shop_project() -> ParsedProject {
    project(json!({
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
    }))
}

#[test]
fn nested_shop_window_exact_rects() {
    let project = nested_shop_project();
    let tx = sizes(&[("panel.png", 24, 24), ("button.png", 12, 12)]);
    let w = solve(&project, &tx).unwrap().pop().unwrap();

    let title = w.slots.iter().find(|s| s.name == "title").unwrap();
    assert_eq!(title.rect, Rect::new(8, 6, 160, 8));
    assert_eq!(title.align, Align::Center);

    // Row begins below title: y = 6 + 8 + 6 = 20.
    let minus = w.regions.iter().find(|b| b.name == "minus").unwrap();
    assert_eq!(minus.rect, Rect::new(8, 20, 18, 18));
    // quantity: after minus(18)+gap(4) → x=30; row height 18, slot 8 tall,
    // default cross align start → y=20.
    let quantity = w.slots.iter().find(|s| s.name == "quantity").unwrap();
    assert_eq!(quantity.rect, Rect::new(30, 20, 40, 8));
    // plus: after quantity(40)+gap(4) → x=74.
    let plus = w.regions.iter().find(|b| b.name == "plus").unwrap();
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
