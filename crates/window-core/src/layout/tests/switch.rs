use serde_json::{Value, json};

use super::{panel, sizes, solve_one, tall, text};
use crate::geometry::Rect;

fn boxed(width: u32, height: u32) -> Value {
    json!({ "type": "flex", "style": { "width": width, "height": height } })
}

fn on(id: &str, values: &[&str]) -> Value {
    json!({ "kind": "value", "id": id, "values": values })
}

#[test]
fn switch_takes_its_largest_case_and_keeps_case_art_out_of_the_static_layer() {
    let laid = solve_one(
        tall(json!([{
            "type": "flex",
            "x": 10,
            "y": 20,
            "children": [
                {
                    "type": "switch",
                    "handle": on("mode", &["buy", "sell"]),
                    "children": [
                        { "type": "case", "value": "buy", "frame": panel(), "children": [
                            { "type": "slot", "handle": text("price"), "width": 20 },
                        ] },
                        { "type": "case", "value": "sell", "style": { "direction": "column" }, "children": [
                            boxed(50, 12),
                            { "type": "label", "text": "Sell" },
                        ] },
                    ],
                },
                { "type": "flex", "frame": panel(), "style": { "width": 5, "height": 5 } },
            ],
        }])),
        &sizes(&[]),
    );

    // The sibling follows the 50px-wide sell case, and only it stays in the static layer.
    let statics: Vec<Rect> = laid.draws.iter().map(|d| *d.dest()).collect();
    assert_eq!(statics, vec![Rect::new(60, 20, 5, 5)]);

    let [switch] = laid.switches.as_slice() else { panic!("one switch") };
    assert_eq!(switch.name, "mode");
    let [buy, sell] = switch.cases.as_slice() else { panic!("two cases") };
    // Every case box fills the switch, which is the size of the largest case.
    let buy_art: Vec<Rect> = buy.draws.iter().map(|d| *d.dest()).collect();
    assert_eq!(buy_art, vec![Rect::new(10, 20, 50, 20)]);
    assert_eq!(buy.slots, vec!["price"]);
    assert_eq!((sell.value.as_str(), sell.draws.len()), ("sell", 0));
    assert_eq!(sell.slots, vec!["label_0"]);
}

#[test]
fn nested_cases_list_only_their_direct_entries() {
    let status = json!({ "type": "slot", "handle": text("status"), "width": 40 });
    let laid = solve_one(
        tall(json!([{
            "type": "switch",
            "handle": on("outer", &["a", "b"]),
            "children": [
                { "type": "case", "value": "a", "children": [{
                    "type": "switch",
                    "handle": on("inner", &["x", "y"]),
                    "children": [
                        { "type": "case", "value": "x", "children": [
                            status,
                            { "type": "sprite_slot", "handle": { "kind": "sprite", "id": "icon" }, "width": 8, "height": 8 },
                        ] },
                        { "type": "case", "value": "y", "children": [status] },
                    ],
                }, { "type": "label", "text": "A" }] },
                { "type": "case", "value": "b", "children": [status] },
            ],
        }])),
        &sizes(&[]),
    );
    let names: Vec<&str> = laid.slots.iter().map(|slot| slot.name.as_str()).collect();
    assert_eq!(names, ["status", "status~2", "label_0", "status~3"]);
    let [inner, outer] = laid.switches.as_slice() else { panic!("two switches") };
    assert_eq!((inner.name.as_str(), outer.name.as_str()), ("inner", "outer"));
    assert_eq!(outer.cases[0].slots, vec!["label_0"]);
    assert_eq!(outer.cases[0].switches, vec!["inner"]);
    assert_eq!(outer.cases[1].slots, vec!["status~3"]);
    assert_eq!(inner.cases[0].slots, vec!["status"]);
    assert_eq!(inner.cases[0].sprite_slots, vec!["icon"]);
}

#[test]
fn item_and_collection_handles_repeat_across_exclusive_cases() {
    let control = |kind: &str, handle: &str, id: &str| json!({ "type": kind, "handle": { "kind": handle, "id": id } });
    let case = |value: &str| {
        let children = [control("item", "items", "stack"), control("collection", "collection", "grid")];
        json!({ "type": "case", "value": value, "children": children })
    };
    let children = json!([{ "type": "section", "section": "container", "claim": "none", "children": [
        { "type": "switch", "handle": on("kind", &["bad", "good"]), "children": [case("good"), case("bad")] },
    ] }]);
    let laid = solve_one(tall(children), &sizes(&[]));
    let items: Vec<&str> = laid.items.iter().map(|item| item.name.as_str()).collect();
    let collections: Vec<&str> = laid.collections.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(items, ["stack", "stack~2"]);
    assert_eq!(collections, ["grid", "grid~2"]);
}
