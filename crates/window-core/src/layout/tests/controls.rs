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
    assert_eq!(w.regions[0].rect, Rect::new(8, 143, 16, 16));
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
    assert_eq!(w.regions[0].rect, Rect::new(152, 7, 16, 16));
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
    assert_eq!(w.regions[0].rect, Rect::new(76, 47, 74, 16));
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

/// A theme with frames `f` and `g` and an 8x8 sprite `dot`.
fn state_theme() -> Value {
    json!({
        "frames": { "f": { "kind": "panel" }, "g": { "kind": "button" } },
        "sprites": { "dot": { "kind": "badge", "width": 8, "height": 8 } },
    })
}

#[test]
fn button_states_become_a_state_switch_of_regions() {
    let w = solve_one(
        themed(
            state_theme(),
            json!([{
                "type": "button", "name": "buy", "x": 8, "y": 18, "width": 18, "height": 18, "frame": "f",
                "tooltip": "Buy",
                "states": {
                    "on": { "item_model": "demo:on", "sprite": "dot" },
                    "off": { "tooltip": "Sold out" },
                },
                "children": [{ "type": "label", "text": "B" }],
            }]),
        ),
        &sizes(&[]),
    );
    // Without state frames the button frame stays static, and every state is a case with its own region.
    assert_eq!(w.draws.len(), 1);
    let [switch] = w.switches.as_slice() else { panic!("one state switch") };
    assert!(switch.states);
    assert_eq!((switch.initial.as_deref(), switch.source.as_deref()), (Some("default"), Some("button `buy`")));
    let values: Vec<&str> = switch.cases.iter().map(|case| case.value.as_str()).collect();
    assert_eq!(values, ["default", "off", "on"]);
    let regions: Vec<&str> = w.regions.iter().map(|region| region.name.as_str()).collect();
    assert_eq!(regions, ["buy.default", "buy.off", "buy.on"]);
    assert!(w.regions.iter().all(|region| region.action.as_deref() == Some("buy")));
    let tooltip = |i: usize| w.regions[i].hitbox.as_ref().and_then(|h| h.tooltip.as_ref()).map(|t| t.title.as_str());
    assert_eq!((tooltip(0), tooltip(1), tooltip(2)), (Some("Buy"), Some("Sold out"), Some("Buy")));
    assert_eq!(w.regions[2].hitbox.as_ref().unwrap().item_model.as_deref(), Some("demo:on"));

    // A state sprite is a fixed image in its case at the button's corner, drawn before the button content.
    assert_eq!(switch.cases[2].sprite_slots, vec!["buy.on"]);
    assert_eq!((w.sprite_slots[0].rect, w.sprite_slots[0].sprite.as_deref()), (Rect::new(8, 18, 18, 18), Some("dot")));
    assert_eq!(
        w.layers,
        vec![Layer::Switch("buy".into()), Layer::SpriteSlot("buy.on".into()), Layer::Slot("label_0".into())]
    );
}

#[test]
fn state_frames_replace_the_button_frame_in_each_case() {
    let w = solve_one(
        themed(
            state_theme(),
            json!([{
                "type": "button", "name": "buy", "x": 8, "y": 18, "width": 18, "height": 18, "frame": "f",
                "states": { "enabled": {}, "disabled": { "frame": "g" } },
                "children": [{ "type": "sprite", "name": "dot" }],
            }]),
        ),
        &sizes(&[]),
    );
    // The frame and the content's static art move into every case, so each state draws its frame below the content.
    assert!(w.draws.is_empty());
    let generated = |draw: &Draw| match draw {
        Draw::Generated { style, dest } => (style.kind, *dest),
        other => panic!("expected generated art, got {other:?}"),
    };
    let [switch] = w.switches.as_slice() else { panic!("one state switch") };
    for case in &switch.cases {
        assert_eq!(case.draws.len(), 2, "case `{}`", case.value);
        assert_eq!(generated(&case.draws[0]).1, Rect::new(8, 18, 18, 18));
        assert_eq!(generated(&case.draws[1]).1, Rect::new(13, 23, 8, 8));
    }
    let kind = |value: &str| generated(&switch.cases.iter().find(|c| c.value == value).unwrap().draws[0]).0;
    assert_eq!((kind("default"), kind("enabled")), (kind("enabled"), kind("enabled")));
    assert_ne!(kind("disabled"), kind("enabled"));
}

#[test]
fn layers_follow_the_authored_tree() {
    let w = solve_one(
        themed(
            state_theme(),
            json!([{
                "type": "column", "x": 8, "y": 18, "children": [
                    { "type": "label", "text": "Pinned", "x": 0, "y": 40 },
                    { "type": "sprite_slot", "name": "icon", "width": 8, "height": 8 },
                    { "type": "slot", "name": "price", "width": 20 },
                ],
            }]),
        ),
        &sizes(&[]),
    );
    // A positioned child keeps its authored place among in-flow siblings.
    assert_eq!(
        w.layers,
        vec![Layer::Slot("label_0".into()), Layer::SpriteSlot("icon".into()), Layer::Slot("price".into())]
    );
    assert_eq!(w.slots[0].rect.y, 58);
}
