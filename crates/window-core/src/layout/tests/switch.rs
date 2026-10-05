use serde_json::json;

use super::{sizes, solve, solve_one, themed};
use crate::geometry::Rect;

fn boxed(width: u32, height: u32) -> serde_json::Value {
    json!({ "type": "flex", "style": { "width": width, "height": height } })
}

#[test]
fn switch_takes_its_largest_case_and_keeps_case_art_out_of_the_static_layer() {
    let laid = solve_one(
        themed(
            json!({ "frames": { "f": { "kind": "panel" } } }),
            json!([{
                "type": "flex",
                "x": 10,
                "y": 20,
                "children": [
                    {
                        "type": "switch",
                        "name": "mode",
                        "children": [
                            { "type": "case", "value": "buy", "frame": "f", "children": [
                                { "type": "slot", "name": "price", "width": 20 },
                            ] },
                            { "type": "case", "value": "sell", "style": { "direction": "column" }, "children": [
                                boxed(50, 12),
                                { "type": "label", "text": "Sell" },
                            ] },
                        ],
                    },
                    { "type": "flex", "frame": "f", "style": { "width": 5, "height": 5 } },
                ],
            }]),
        ),
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
fn slot_bound_controls_inside_a_case_are_rejected() {
    let project = themed(
        json!({}),
        json!([{
            "type": "switch",
            "name": "mode",
            "children": [{ "type": "case", "value": "buy", "children": [{
                "type": "flex",
                "children": [{ "type": "button", "name": "buy", "width": 18, "height": 18 }],
            }] }],
        }]),
    );
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("button `buy` cannot be inside switch `mode`"), "{err}");
}
