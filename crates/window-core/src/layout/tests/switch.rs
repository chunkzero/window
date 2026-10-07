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

#[test]
fn switch_in_a_flow_row_takes_its_largest_case() {
    let laid = solve_one(
        themed(
            json!({ "frames": { "f": { "kind": "panel" } } }),
            json!([{
                "type": "row",
                "x": 10,
                "y": 20,
                "gap": 2,
                "children": [
                    {
                        "type": "switch",
                        "name": "mode",
                        "children": [
                            { "type": "case", "value": "buy", "children": [boxed(30, 10)] },
                            { "type": "case", "value": "sell", "frame": "f", "children": [boxed(20, 10)] },
                        ],
                    },
                    { "type": "flex", "frame": "f", "style": { "width": 5, "height": 5 } },
                ],
            }]),
        ),
        &sizes(&[]),
    );

    // The sibling follows the 30px-wide buy case after the gap.
    let statics: Vec<Rect> = laid.draws.iter().map(|d| *d.dest()).collect();
    assert_eq!(statics, vec![Rect::new(42, 20, 5, 5)]);
    let [switch] = laid.switches.as_slice() else { panic!("one switch") };
    let sell_art: Vec<Rect> = switch.cases[1].draws.iter().map(|d| *d.dest()).collect();
    assert_eq!(sell_art, vec![Rect::new(10, 20, 30, 10)]);
}

#[test]
fn bindings_shared_across_cases_key_each_copy_by_case() {
    let switch = |good: serde_json::Value, extra: serde_json::Value| {
        json!([
            {
                "type": "switch",
                "name": "kind",
                "children": [
                    { "type": "case", "value": "good", "children": [good] },
                    { "type": "case", "value": "bad", "children": [
                        { "type": "slot", "name": "status", "width": 40, "color": "#ff0000" },
                    ] },
                ],
            },
            extra,
        ])
    };
    let status = json!({ "type": "slot", "name": "status", "width": 40, "color": "#00ff00" });
    let laid = solve_one(themed(json!({}), switch(status.clone(), json!({ "type": "column" }))), &sizes(&[]));
    let keys: Vec<(&str, Option<&str>)> =
        laid.slots.iter().map(|slot| (slot.name.as_str(), slot.binding.as_deref())).collect();
    assert_eq!(keys, vec![("status.good", Some("status")), ("status.bad", Some("status"))]);
    assert_eq!(laid.switches[0].cases[1].slots, vec!["status.bad"]);

    let cases = [
        (
            json!({ "type": "sprite_slot", "name": "status", "width": 8, "height": 8 }),
            json!({ "type": "column" }),
            "binding `status` is a sprite slot in one case of switch `kind` and a text slot in another",
        ),
        (status, json!({ "type": "slot", "name": "status", "width": 40 }), "duplicate name `status`"),
    ];
    for (good, extra, message) in cases {
        let err = solve(&themed(json!({}), switch(good, extra)), &sizes(&[])).unwrap_err();
        assert!(err.to_string().contains(message), "{err}");
    }
}

#[test]
fn indexed_bindings_repeat_once_per_case_and_fixed_sprites_do_not_share() {
    let cases = |good: serde_json::Value, bad: serde_json::Value| {
        json!([{
            "type": "switch",
            "name": "kind",
            "children": [
                { "type": "case", "value": "good", "children": good },
                { "type": "case", "value": "bad", "children": bad },
            ],
        }])
    };
    let power = |i: u32| json!({ "type": "slot", "name": "power", "index": i, "width": 9 });
    let project = themed(json!({}), cases(json!([power(0), power(1)]), json!([power(0), power(1)])));
    assert_eq!(project.windows[0].indexed["power"].shape, vec![2]);
    // A switch nested in the first case must not move the second case to another switch.
    let inner =
        json!({ "type": "switch", "name": "inner", "children": [{ "type": "case", "value": "on", "children": [] }] });
    let nested = themed(json!({}), cases(json!([power(0), inner, power(1)]), json!([power(0), power(1)])));
    let err = solve(&nested, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("switch `inner` cannot be nested"), "{err}");
    let laid = solve_one(project, &sizes(&[]));
    let keys: Vec<(&str, Option<&str>)> =
        laid.slots.iter().map(|slot| (slot.name.as_str(), slot.binding.as_deref())).collect();
    assert_eq!(
        keys,
        vec![
            ("power[0].good", Some("power[0]")),
            ("power[1].good", Some("power[1]")),
            ("power[0].bad", Some("power[0]")),
            ("power[1].bad", Some("power[1]")),
        ]
    );

    let err = super::project_from_json(
        json!({ "windows": [{ "name": "s", "container": "generic_9x3", "children": cases(json!([power(0), power(0)]), json!([])) }] })
            .to_string()
            .as_bytes(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("indexed binding `power` repeats index [0]"), "{err}");

    let icon = |sprite: Option<&str>| json!({ "type": "sprite_slot", "name": "icon", "width": 8, "height": 8, "sprite": sprite });
    let theme = json!({ "sprites": { "dot": { "kind": "badge", "width": 8, "height": 8 } } });
    let both = cases(json!([icon(None)]), json!([icon(None), { "type": "column", "children": [icon(Some("dot"))] }]));
    let err = solve(&themed(theme, both), &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("duplicate name `icon`"), "{err}");
}
