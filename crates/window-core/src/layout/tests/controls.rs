use super::*;

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
