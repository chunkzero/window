use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{sizes, solve_one, tall, text};
use crate::inventory::InventorySlotRef;

fn missing() -> Value {
    json!({ "art": "texture", "texture": "missing.png" })
}

fn in_container(children: Vec<Value>) -> Value {
    json!([{ "type": "section", "section": "container", "claim": "none", "children": children }])
}

#[test]
fn an_unsized_region_claims_the_slots_its_box_covers() {
    let laid = solve_one(
        tall(in_container(vec![json!({
            "type": "flex",
            "layout": { "column": { "span": 2 }, "row": { "span": 2 } },
            "children": [{ "type": "region", "on_click": { "kind": "action", "id": "pick" } }],
        })])),
        &sizes(&[]),
    );
    let [region] = laid.regions.as_slice() else { panic!("one region") };
    let slots: Vec<InventorySlotRef> = [0, 1, 9, 10].into_iter().map(InventorySlotRef::container).collect();
    assert_eq!(region.slots.as_deref(), Some(slots.as_slice()));
    assert_eq!(region.action.as_deref(), Some("pick"));
}

#[test]
fn nested_cases_hold_regions_and_items_over_shared_slots() {
    let region = |id: &str| json!({ "type": "region", "on_click": { "kind": "action", "id": id } });
    let sized = |value: &str, child: Value| json!({ "type": "case", "value": value, "style": { "width": 36, "height": 18 }, "children": [child] });
    let laid = solve_one(
        tall(in_container(vec![json!({
            "type": "switch",
            "handle": { "kind": "value", "id": "tab", "values": ["a", "b"] },
            "layout": { "column": { "span": 2 } },
            "children": [
                { "type": "case", "value": "a", "children": [region("a")] },
                { "type": "case", "value": "b", "children": [{
                    "type": "switch",
                    "handle": { "kind": "value", "id": "mode", "values": ["x", "y"] },
                    "x": 0,
                    "y": 0,
                    "children": [sized("x", region("x")), sized("y", json!({ "type": "item", "handle": { "kind": "items", "id": "coin" } }))],
                }] },
            ],
        })])),
        &sizes(&[]),
    );
    let both: Vec<InventorySlotRef> = [0, 1].into_iter().map(InventorySlotRef::container).collect();
    for name in ["a", "x"] {
        let region = laid.regions.iter().find(|r| r.action.as_deref() == Some(name)).expect("region");
        assert_eq!(region.slots.as_deref(), Some(both.as_slice()), "region `{name}`");
    }
    assert_eq!(laid.items.iter().map(|item| &item.slots).collect::<Vec<_>>(), [&both]);
    let mode = laid.switches.iter().find(|s| s.name.starts_with("mode")).expect("nested switch");
    let tab = laid.switches.iter().find(|s| s.name == "tab").expect("outer switch");
    assert_eq!(tab.cases[1].switches, std::slice::from_ref(&mode.name));
    assert_eq!(mode.cases[1].items, ["coin"]);
}

#[test]
fn errors_name_the_primitive_they_come_from() {
    let project = tall(json!([{ "type": "sprite", "art": missing(), "x": 0, "y": 0, "debug_name": "coin-icon" }]));
    let err = super::solve(&project, &sizes(&[])).unwrap_err().to_string();
    assert!(err.contains("(in `coin-icon`)"), "{err}");
}

#[test]
fn nested_measurement_errors_name_the_primitive() {
    let project = tall(json!([{ "type": "flex", "x": 0, "y": 0, "children": [
            { "type": "sprite", "art": missing(), "debug_name": "coin-icon" },
        ] }]));
    let err = super::solve(&project, &sizes(&[])).unwrap_err().to_string();
    assert_eq!(err.matches("(in `coin-icon`)").count(), 1, "{err}");
}

#[test]
fn nested_containers_name_the_innermost_failing_scope() {
    let project = tall(json!([{ "type": "flex", "x": 0, "y": 0, "debug_name": "outer-box", "children": [
            { "type": "flex", "debug_name": "inner-box", "children": [{ "type": "sprite", "art": missing() }] },
        ] }]));
    let err = super::solve(&project, &sizes(&[])).unwrap_err().to_string();
    assert_eq!(err.matches("(in `").count(), 1, "{err}");
    assert!(err.contains("(in `inner-box`)"), "{err}");
}

#[test]
fn slots_carry_the_nearest_debug_name() {
    let laid = solve_one(
        tall(json!([{ "type": "flex", "x": 8, "y": 6, "debug_name": "header", "children": [
                { "type": "label", "text": "Shop" },
                { "type": "slot", "handle": text("price"), "width": 20, "debug_name": "price-text" },
            ] }])),
        &sizes(&[]),
    );
    let sources: Vec<Option<&str>> = laid.slots.iter().map(|slot| slot.source.as_deref()).collect();
    assert_eq!(sources, [Some("header"), Some("price-text")]);
}

#[test]
fn anonymous_regions_never_collide_with_authored_names() {
    let laid = solve_one(
        tall(in_container(vec![
            json!({ "type": "region", "on_click": { "kind": "action", "id": "region_0" }, "layout": { "column": { "span": 1 } } }),
            json!({ "type": "region", "tooltip": "Hint", "layout": { "column": { "span": 1 } } }),
        ])),
        &sizes(&[]),
    );
    let actions: Vec<Option<&str>> = laid.regions.iter().map(|region| region.action.as_deref()).collect();
    assert_eq!(actions, [Some("region_0"), None]);
}

#[test]
fn windows_and_huds_name_themselves_in_errors() {
    let bad = json!([{ "type": "sprite", "art": missing(), "x": 0, "y": 0 }]);
    let window = json!({ "name": "s", "container": "generic_9x3", "children": bad, "debug_name": "shop" });
    let project = super::project(json!({ "windows": [window] }));
    let err = super::solve(&project, &sizes(&[])).unwrap_err().to_string();
    assert!(err.contains("(in `shop`)"), "{err}");

    let hud = json!({ "name": "h", "children": bad, "debug_name": "bar" });
    let project = super::project(json!({ "huds": [hud] }));
    let err = crate::layout::solve_huds(
        &project,
        &sizes(&[]),
        &crate::text_font::resolve(&BTreeMap::new(), &BTreeMap::new()).unwrap(),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("(in `bar`)"), "{err}");
}
