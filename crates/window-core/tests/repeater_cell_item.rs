//! End-to-end coverage for repeater cells that display a real `ItemStack` in
//! one of their own slots while every click in the cell still routes to the
//! cell button.

use window_core::pipeline::{CompileInput, CompileOutput, compile_project_json};
use window_core::validation::validate_compile_output;

/// A 3x2-cell catalog repeater. `cell_slot` is the authored slot-source suffix
/// applied to the cell's item child.
fn project(cell_slot: &str) -> String {
    format!(
        r##"{{
          "theme": {{ "frames": {{ "card": {{ "kind": "button" }} }} }},
          "windows": [{{
            "name": "catalog",
            "container": "generic_9x6",
            "children": [{{
              "type": "repeater",
              "name": "entry",
              "frame": "card",
              "pattern": {{
                "kind": "grid",
                "section": "container",
                "x": 0,
                "y": 2,
                "columns": 3,
                "rows": 2,
                "cell_width": 3,
                "cell_height": 2
              }},
              "children": [
                {{ "type": "item", "name": "icon"{cell_slot} }},
                {{ "type": "slot", "name": "price", "x": 4, "y": 24, "width": 40 }}
              ]
            }}]
          }}]
        }}"##
    )
}

fn compile(source: &str) -> CompileOutput {
    compile_project_json(source.as_bytes(), &CompileInput::new(Default::default())).expect("project compiles")
}

#[test]
fn cell_item_owns_one_slot_while_the_cell_button_routes_all_of_them() {
    let output = compile(&project(r#", "cell_slot": 2"#));
    validate_compile_output(&output).assert_valid();

    let window = &output.manifest.windows["catalog"];
    assert_eq!(window.groups["entry"].count, 6);
    assert_eq!(
        window.groups["entry"].items["icon"],
        ["entry_icon_0", "entry_icon_1", "entry_icon_2", "entry_icon_3", "entry_icon_4", "entry_icon_5",]
    );

    // Cell 0 covers container slots 18,19,20,27,28,29. `cell_slot = 2` is 19.
    let cell = &window.buttons["entry_0"];
    let routed: Vec<u32> = cell.slots.iter().map(|slot| slot.index).collect();
    assert_eq!(routed, [18, 19, 20, 27, 28, 29]);
    let filled: Vec<u32> = cell.filled_slots().iter().map(|slot| slot.index).collect();
    assert_eq!(filled, [18, 20, 27, 28, 29]);
    assert_eq!(window.items["entry_icon_0"].slots[0].index, 19);

    // Cell 3 opens the second grid row: container slots 36..38 and 45..47.
    let cell = &window.buttons["entry_3"];
    assert_eq!(cell.slots.iter().map(|slot| slot.index).collect::<Vec<_>>(), [36, 37, 38, 45, 46, 47]);
    let icon = window.items["entry_icon_3"].slots[0];
    assert_eq!(icon.index, 37);
    assert!(cell.slots.contains(&icon), "the cell still routes the item slot");
    assert!(!cell.filled_slots().contains(&icon), "the cell must not overwrite the item's stack");
}

#[test]
fn an_item_with_absolute_slots_inside_a_repeater_is_unchanged() {
    // Single cell, absolute item slot: the item lands on the authored slot and
    // the cell button yields nothing, exactly as before this feature.
    let source = r##"{
      "theme": { "frames": { "card": { "kind": "button" } } },
      "windows": [{
        "name": "catalog",
        "container": "generic_9x6",
        "children": [{
          "type": "repeater",
          "name": "entry",
          "frame": "card",
          "transform": { "section": "container", "x": 0, "y": 2, "width": 3, "height": 2 },
          "children": [{ "type": "item", "name": "icon", "slots": [4] }]
        }]
      }]
    }"##;
    let output = compile(source);
    validate_compile_output(&output).assert_valid();
    let window = &output.manifest.windows["catalog"];
    assert_eq!(window.items["entry_icon_0"].slots[0].index, 4);
    assert!(window.buttons["entry_0"].fill_slots.is_none());
    assert_eq!(window.buttons["entry_0"].filled_slots().len(), 6);
}

#[test]
fn absolute_item_slots_still_collide_across_repeater_cells() {
    // Multi-cell absolute slots were, and remain, a duplicate-ownership error.
    let error = compile_project_json(project(r#", "slots": [4]"#).as_bytes(), &CompileInput::new(Default::default()))
        .unwrap_err()
        .to_string();
    assert!(error.contains("both own container slot 4"), "{error}");
}

#[test]
fn cell_slot_out_of_range_fails_the_build() {
    let error = compile_project_json(project(r#", "cell_slot": 7"#).as_bytes(), &CompileInput::new(Default::default()))
        .unwrap_err()
        .to_string();
    assert!(error.contains("repeater `entry`"), "{error}");
    assert!(error.contains("6 slot(s) per cell"), "{error}");
}

#[test]
fn cell_slot_outside_a_repeater_fails_the_build() {
    let source = r##"{
      "windows": [{
        "name": "catalog",
        "container": "generic_9x6",
        "children": [{ "type": "item", "name": "icon", "cell_slot": 1 }]
      }]
    }"##;
    let error =
        compile_project_json(source.as_bytes(), &CompileInput::new(Default::default())).unwrap_err().to_string();
    assert!(error.contains("outside a repeater"), "{error}");
}

#[test]
fn cell_slot_rejects_a_competing_absolute_slot_source() {
    let error = compile_project_json(
        project(r#", "cell_slot": 1, "slots": [4]"#).as_bytes(),
        &CompileInput::new(Default::default()),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("cannot be combined"), "{error}");
}

#[test]
fn a_cell_may_not_yield_every_slot() {
    let source = r##"{
      "theme": { "frames": { "card": { "kind": "button" } } },
      "windows": [{
        "name": "catalog",
        "container": "generic_9x6",
        "children": [{
          "type": "repeater",
          "name": "entry",
          "frame": "card",
          "transform": { "section": "container", "x": 0, "y": 2, "width": 1, "height": 1 },
          "children": [{ "type": "item", "name": "icon", "cell_slot": 1 }]
        }]
      }]
    }"##;
    let error =
        compile_project_json(source.as_bytes(), &CompileInput::new(Default::default())).unwrap_err().to_string();
    assert!(error.contains("yielded every backing slot"), "{error}");
}
