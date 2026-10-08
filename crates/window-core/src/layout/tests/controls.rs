use super::*;

#[test]
fn anvil_input_requires_anvil_and_reserves_the_first_slot() {
    let input = |container: &str| {
        project(json!({
            "windows": [{
                "name": "search",
                "container": container,
                "children": [{
                    "type": "anvil_input",
                    "handle": { "kind": "input", "id": "query" },
                    "initial": "Search...",
                    "item_model": "demo:gui/search"
                }]
            }]
        }))
    };
    let w = solve(&input("anvil"), &sizes(&[])).unwrap().pop().unwrap();
    assert_eq!(w.inputs.len(), 1);
    assert_eq!(w.inputs[0].name, "query");
    assert_eq!(w.inputs[0].initial, "Search...");
    assert_eq!(w.inputs[0].item_model.as_deref(), Some("demo:gui/search"));

    let error = solve(&input("generic_9x3"), &sizes(&[])).unwrap_err();
    assert!(error.to_string().contains("requires container `anvil`"));
}

#[test]
fn layers_follow_the_authored_tree() {
    let w = solve_one(
        window(json!([{
            "type": "flex", "x": 8, "y": 18, "style": { "direction": "column" }, "children": [
                { "type": "label", "text": "Pinned", "x": 0, "y": 40 },
                { "type": "sprite_slot", "handle": { "kind": "sprite", "id": "icon" }, "width": 8, "height": 8 },
                { "type": "slot", "handle": text("price"), "width": 20 },
            ],
        }])),
        &sizes(&[]),
    );
    // A positioned child keeps its authored place among in-flow siblings.
    assert_eq!(
        w.layers,
        vec![Layer::Slot("label_0".into()), Layer::SpriteSlot("icon".into()), Layer::Slot("price".into())]
    );
    assert_eq!(w.slots[0].rect.y, 58);
}
