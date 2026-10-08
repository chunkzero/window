use std::collections::{BTreeMap, HashMap};

use serde_json::{Value, json};

use super::{sizes, solve_one, themed};
use crate::compose::{Composite, compose_draws};
use crate::geometry::Rect;
use crate::inventory::InventorySlotRef;
use crate::ir::{Draw, LaidOutWindow, Layer, SwitchIr};
use crate::model::SpriteDef;

const VALUES: [&str; 3] = ["all", "gear", "magic"];

fn tab_theme() -> Value {
    json!({
        "frames": { "tab_frame": { "kind": "button" } },
        "sprites": {
            "tab": { "kind": "badge", "width": 10, "height": 4, "fill": "#3a3a3a" },
            "tab_selected": { "kind": "badge", "width": 10, "height": 4, "fill": "#ff8700" }
        }
    })
}

fn category(field: &str, value: &str) -> Value {
    json!({ "kind": "selection", "id": "category", "values": VALUES, "initial": "all", field: value })
}

fn in_container(children: Vec<Value>) -> Value {
    json!([{ "type": "section", "section": "container", "claim": "none", "children": children }])
}

/// What `<Tabs bind={category} span={3} frame sprite selectedSprite tooltip>` emits: one choice button per value.
fn tabs() -> Value {
    in_container(
        VALUES
            .iter()
            .map(|value| {
                let state = |sprite: &str| json!({ "tooltip": format!("Tab {value}"), "sprite": sprite });
                json!({
                    "type": "button",
                    "on_click": category("set", value),
                    "frame": "tab_frame",
                    "states": { "selected": state("tab_selected"), "unselected": state("tab") },
                    "layout": { "column": { "span": 3 } },
                })
            })
            .collect(),
    )
}

/// The same tab group built from boxes, switches, images, regions, and text.
fn primitive_tabs() -> Value {
    in_container(
        VALUES
            .iter()
            .map(|value| {
                let image = |sprite: &str| json!([{ "type": "sprite", "name": sprite }]);
                // A button spans its slots' 16px interiors; a section box spans their 18px cells.
                json!({
                    "type": "flex",
                    "layout": { "column": { "span": 3 } },
                    "style": { "padding": 1 },
                    "children": [{
                        "type": "flex",
                        "frame": "tab_frame",
                        "layout": { "grow": 1 },
                        "children": [
                            { "type": "switch", "handle": category("is", value), "x": 0, "y": 0, "children": [
                                { "type": "case", "value": "true", "children": image("tab_selected") },
                                { "type": "case", "value": "false", "children": image("tab") },
                            ] },
                            { "type": "region", "on_click": category("set", value), "tooltip": format!("Tab {value}") },
                        ],
                    }],
                })
            })
            .collect(),
    )
}

/// The case each switch shows while `selected` is the category: a button state switch by its state, a handle
/// switch by whether it tests `selected`.
fn case_for<'a>(switch: &'a SwitchIr, selected: &str) -> &'a str {
    let tab = VALUES.iter().find(|value| switch.name.contains(*value)).expect("switch names its tab");
    let on = *tab == selected;
    match (switch.states, on) {
        (true, true) => "selected",
        (true, false) => "unselected",
        (false, true) => "true",
        (false, false) => "false",
    }
}

/// The window's art and claimed regions while `selected` is the category, as the runtime composes them.
fn render(window: &LaidOutWindow, sprites: &BTreeMap<String, SpriteDef>, selected: &str) -> (Composite, Vec<Value>) {
    let mut active = HashMap::new();
    for switch in &window.switches {
        active.insert(switch.name.as_str(), switch.cases.iter().find(|c| c.value == case_for(switch, selected)));
    }
    let mut draws = window.draws.clone();
    let mut regions: Vec<&str> =
        window.regions.iter().map(|r| r.name.as_str()).filter(|name| !in_any_case(window, name)).collect();
    for layer in &window.layers {
        match layer {
            Layer::Switch(name) => {
                let case = active[name.as_str()].expect("active case");
                draws.extend(case.draws.iter().cloned());
                regions.extend(case.regions.iter().map(String::as_str));
            }
            Layer::SpriteSlot(name) => {
                let active_case = active.values().flatten().any(|case| case.sprite_slots.contains(name));
                let slot = window.sprite_slots.iter().find(|slot| &slot.name == name).unwrap();
                let Some(SpriteDef::Generated { style, size }) = slot.sprite.as_ref().map(|s| &sprites[s]) else {
                    panic!("fixed generated sprite");
                };
                if active_case {
                    let dest = Rect::new(slot.rect.x, slot.rect.y, size.width, size.height);
                    draws.push(Draw::Generated { style: style.clone(), dest });
                }
            }
            Layer::Slot(_) | Layer::Collection(_) => {}
        }
    }
    let mut claims: Vec<Value> = regions
        .iter()
        .map(|name| {
            let region = window.regions.iter().find(|r| r.name == *name).unwrap();
            let slots: Vec<u32> = region.slots.iter().flatten().map(|slot| slot.index).collect();
            json!({ "slots": slots, "action": region.action, "hitbox": format!("{:?}", region.hitbox) })
        })
        .collect();
    claims.sort_by_key(Value::to_string);
    (compose_draws(&draws, &window.name, &BTreeMap::new()).expect("compose"), claims)
}

fn in_any_case(window: &LaidOutWindow, region: &str) -> bool {
    window.switches.iter().flat_map(|s| &s.cases).any(|case| case.regions.iter().any(|r| r == region))
}

#[test]
fn a_tab_group_built_from_primitives_matches_tabs() {
    let tabs_project = themed(tab_theme(), tabs());
    let primitive_project = themed(tab_theme(), primitive_tabs());
    let tabs = solve_one(tabs_project.clone(), &sizes(&[]));
    let primitives = solve_one(primitive_project, &sizes(&[]));

    for selected in VALUES {
        let (tab_art, tab_claims) = render(&tabs, &tabs_project.theme.sprites, selected);
        let (art, claims) = render(&primitives, &tabs_project.theme.sprites, selected);
        assert_eq!(art.bounds, tab_art.bounds, "bounds with `{selected}` selected");
        assert!(art.texture.rgba == tab_art.texture.rgba, "pixels differ with `{selected}` selected");
        assert_eq!(claims, tab_claims, "regions with `{selected}` selected");
    }
}

#[test]
fn an_unsized_region_claims_the_slots_its_box_covers() {
    let laid = solve_one(
        themed(
            json!({}),
            in_container(vec![json!({
                "type": "flex",
                "layout": { "column": { "span": 2 }, "row": { "span": 2 } },
                "children": [{ "type": "region", "on_click": { "kind": "action", "id": "pick" } }],
            })]),
        ),
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
        themed(
            json!({}),
            in_container(vec![json!({
                "type": "switch",
                "name": "tab",
                "layout": { "column": { "span": 2 } },
                "children": [
                    { "type": "case", "value": "a", "children": [region("a")] },
                    { "type": "case", "value": "b", "children": [{
                        "type": "switch",
                        "name": "mode",
                        "x": 0,
                        "y": 0,
                        "children": [sized("x", region("x")), sized("y", json!({ "type": "item", "name": "coin" }))],
                    }] },
                ],
            })]),
        ),
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
    let project =
        themed(json!({}), json!([{ "type": "sprite", "name": "missing", "x": 0, "y": 0, "debug_name": "coin-icon" }]));
    let err = super::solve(&project, &sizes(&[])).unwrap_err().to_string();
    assert!(err.contains("(in `coin-icon`)"), "{err}");
}

#[test]
fn slots_carry_the_nearest_debug_name() {
    let laid = solve_one(
        themed(
            json!({}),
            json!([{ "type": "flex", "x": 8, "y": 6, "debug_name": "header", "children": [
                { "type": "label", "text": "Shop" },
                { "type": "slot", "name": "price", "width": 20, "debug_name": "price-text" },
            ] }]),
        ),
        &sizes(&[]),
    );
    let sources: Vec<Option<&str>> = laid.slots.iter().map(|slot| slot.source.as_deref()).collect();
    assert_eq!(sources, [Some("header"), Some("price-text")]);
}

#[test]
fn anonymous_regions_never_collide_with_authored_names() {
    let laid = solve_one(
        themed(
            json!({}),
            in_container(vec![
                json!({ "type": "region", "on_click": { "kind": "action", "id": "region_0" }, "layout": { "column": { "span": 1 } } }),
                json!({ "type": "region", "tooltip": "Hint", "layout": { "column": { "span": 1 } } }),
            ]),
        ),
        &sizes(&[]),
    );
    let actions: Vec<Option<&str>> = laid.regions.iter().map(|region| region.action.as_deref()).collect();
    assert_eq!(actions, [Some("region_0"), None]);
}

#[test]
fn windows_and_huds_name_themselves_in_errors() {
    let bad = json!([{ "type": "sprite", "name": "missing", "x": 0, "y": 0 }]);
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
