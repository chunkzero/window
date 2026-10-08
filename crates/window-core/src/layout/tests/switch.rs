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
        (
            status,
            json!({ "type": "slot", "name": "status", "width": 40 }),
            "binding `status` appears both inside and outside the cases of switch `kind`",
        ),
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
    // A switch nested in the first case does not move the second case to another switch.
    let inner =
        json!({ "type": "switch", "name": "inner", "children": [{ "type": "case", "value": "on", "children": [] }] });
    let nested = themed(json!({}), cases(json!([power(0), inner, power(1)]), json!([power(0), power(1)])));
    assert_eq!(solve_one(nested, &sizes(&[])).switches[1].cases[0].switches, vec!["inner"]);
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

/// `{ outer: a { slot status, inner: x { slot status, sprite_slot status_icon } | y { slot status } } | b { slot status } }`
fn nested_switches() -> serde_json::Value {
    let status = json!({ "type": "slot", "name": "status", "width": 40 });
    json!([{
        "type": "switch",
        "name": "outer",
        "children": [
            { "type": "case", "value": "a", "children": [{
                "type": "switch",
                "name": "inner",
                "children": [
                    { "type": "case", "value": "x", "children": [
                        status,
                        { "type": "sprite_slot", "name": "icon", "width": 8, "height": 8 },
                    ] },
                    { "type": "case", "value": "y", "children": [status] },
                ],
            }, { "type": "label", "text": "A" }] },
            { "type": "case", "value": "b", "children": [status] },
        ],
    }])
}

#[test]
fn nested_switches_key_shared_bindings_by_their_case_path() {
    let laid = solve_one(themed(json!({}), nested_switches()), &sizes(&[]));
    let keys: Vec<(&str, Option<&str>)> =
        laid.slots.iter().map(|slot| (slot.name.as_str(), slot.binding.as_deref())).collect();
    assert_eq!(
        keys,
        vec![
            ("status.a.x", Some("status")),
            ("status.a.y", Some("status")),
            ("label_0", None),
            ("status.b", Some("status")),
        ]
    );
    let [inner, outer] = laid.switches.as_slice() else { panic!("two switches") };
    assert_eq!((inner.name.as_str(), outer.name.as_str()), ("inner", "outer"));
    // A case lists only what sits directly in it; the nested switch's entries stay with its own cases.
    assert_eq!(outer.cases[0].slots, vec!["label_0"]);
    assert_eq!(outer.cases[0].switches, vec!["inner"]);
    assert_eq!(outer.cases[1].slots, vec!["status.b"]);
    assert_eq!(inner.cases[0].slots, vec!["status.a.x"]);
    assert_eq!(inner.cases[0].sprite_slots, vec!["icon"]);
}

#[test]
fn shared_switches_must_have_the_same_case_values() {
    let inner = |values: [&str; 2]| {
        let cases: Vec<_> = values.iter().map(|v| json!({ "type": "case", "value": v, "children": [] })).collect();
        json!({ "type": "switch", "name": "inner", "children": cases })
    };
    let outer = |a: [&str; 2], b: [&str; 2]| {
        themed(
            json!({}),
            json!([{ "type": "switch", "name": "outer", "children": [
                { "type": "case", "value": "a", "children": [inner(a)] },
                { "type": "case", "value": "b", "children": [inner(b)] },
            ] }]),
        )
    };
    let err = solve(&outer(["one", "two"], ["three", "four"]), &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("switch `inner` is shared across exclusive cases but has cases [one, two] in one and [four, three] in another"), "{err}");
    assert!(solve(&outer(["one", "two"], ["two", "one"]), &sizes(&[])).is_ok());
}

#[test]
fn bindings_repeated_in_one_nested_case_are_rejected() {
    let status = json!({ "type": "slot", "name": "status", "width": 40 });
    let children = json!([{
        "type": "switch",
        "name": "outer",
        "children": [
            { "type": "case", "value": "a", "children": [status, {
                "type": "switch",
                "name": "inner",
                "children": [{ "type": "case", "value": "x", "children": [status] }],
            }] },
            { "type": "case", "value": "b", "children": [status] },
        ],
    }]);
    let err = solve(&themed(json!({}), children), &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("binding `status` appears more than once in case `a` of switch `outer`"), "{err}");
}

#[test]
fn indexed_families_repeat_only_along_exclusive_case_paths() {
    let power = json!({ "type": "slot", "name": "power", "index": 0, "width": 9 });
    let nested = |inner_cases: serde_json::Value, outer_extra: serde_json::Value| {
        json!({ "windows": [{ "name": "s", "container": "generic_9x3", "children": [{
            "type": "switch",
            "name": "outer",
            "children": [
                { "type": "case", "value": "a", "children": [outer_extra, {
                    "type": "switch", "name": "inner", "children": inner_cases,
                }] },
                { "type": "case", "value": "b", "children": [power] },
            ],
        }] }] })
        .to_string()
    };
    let cases = json!([
        { "type": "case", "value": "x", "children": [power] },
        { "type": "case", "value": "y", "children": [power] },
    ]);
    let project = super::project_from_json(nested(cases.clone(), json!({ "type": "column" })).as_bytes()).unwrap();
    let laid = solve_one(project, &sizes(&[]));
    let keys: Vec<&str> = laid.slots.iter().map(|slot| slot.name.as_str()).collect();
    assert_eq!(keys, ["power[0].a.x", "power[0].a.y", "power[0].b"]);

    let err = super::project_from_json(nested(cases, power.clone()).as_bytes()).unwrap_err();
    assert!(err.to_string().contains("indexed binding `power` repeats index [0]"), "{err}");
}

#[test]
fn item_and_collection_handles_repeat_across_exclusive_cases() {
    let control = |kind: &str, handle: &str, id: &str| json!({ "type": kind, "handle": { "kind": handle, "id": id } });
    let case = |value: &str| {
        let children = [control("item", "items", "stack"), control("collection", "collection", "grid")];
        json!({ "type": "case", "value": value, "children": children })
    };
    let children = json!([{ "type": "section", "section": "container", "claim": "none", "children": [
        { "type": "switch", "name": "kind", "children": [case("good"), case("bad")] },
    ] }]);
    let laid = solve_one(themed(json!({}), children), &sizes(&[]));
    let items: Vec<&str> = laid.items.iter().map(|item| item.name.as_str()).collect();
    let collections: Vec<&str> = laid.collections.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(items, ["stack", "stack~2"]);
    assert_eq!(collections, ["grid", "grid~2"]);
}
