use std::collections::BTreeMap;

use serde_json::Value;

use super::*;
use crate::manifest::{SlotAreaEntry, SlotRefEntry};
use crate::pipeline::{CompileInput, FileContents, compile_project_json};

fn compile(project: &str) -> CompileOutput {
    compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap()
}

fn project() -> &'static str {
    r##"{
      "windows": [{
        "name": "validation",
        "container": "generic_9x3",
        "children": [
          {"type":"panel","frame":"panel","width":32,"height":16,"x":8,"y":0},
          {"type":"slot","name":"title","width":30,"x":9,"y":5},
          {"type":"button","name":"buy","width":16,"height":16,"x":8,"y":18}
        ]
      }],
      "theme": {"frames": {"panel": {
        "kind":"panel", "fill":"#123456", "border_color":"#abcdef",
        "border_width":1, "radius":0, "inset_depth":0
      }}}
    }"##
}

#[test]
fn validates_real_compiler_output() {
    let report = validate_compile_output(&compile(project()));
    report.assert_valid();
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.checks.len(), validation_catalog().len());
    assert!(report.checks.iter().all(|check| check.status == ValidationStatus::Passed));
}

#[test]
fn exports_named_checks_and_findings_as_json() {
    let mut output = compile(project());
    output.files.push(output.files[0].clone());
    let report = validate_compile_output(&output);
    let document: Value = serde_json::from_slice(&report.to_json_pretty().unwrap()).unwrap();

    assert_eq!(document["schema_version"], 1);
    assert_eq!(document["valid"], false);
    let artifact_check =
        document["checks"].as_array().unwrap().iter().find(|check| check["id"] == "artifact_set").unwrap();
    assert_eq!(artifact_check["status"], "failed");
    assert!(artifact_check["finding_count"].as_u64().unwrap() > 0);
    assert_eq!(document["issues"][0]["code"], "artifact.order.unstable");
}

#[test]
fn reports_all_duplicate_and_unsorted_paths() {
    let mut output = compile(project());
    let duplicate = output.files[0].clone();
    output.files.push(duplicate);
    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "artifact.path.duplicate"));
    assert!(report.issues.iter().any(|issue| issue.code == "artifact.order.unstable"));
}

#[test]
fn catches_corrupt_font_json_without_panicking() {
    let mut output = compile(project());
    let main = output.files.iter_mut().find(|file| file.path.ends_with("/font/ui.json")).unwrap();
    main.contents = FileContents::Text("not json".into());
    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "artifact.json.invalid"));
}

#[test]
fn catches_debug_descriptor_drift_from_compiled_definition() {
    let mut output = compile(project());
    output.manifest.windows.get_mut("validation").unwrap().slots.get_mut("title").unwrap().x += 1;

    let report = validate_compile_output(&output);

    assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.mismatch"));
}

#[test]
fn catches_debug_descriptor_drift_from_generated_asset_bytes() {
    let mut output = compile(project());
    let model = output.files.iter_mut().find(|file| file.path.ends_with("/models/gui/hitbox.json")).unwrap();
    model.contents = FileContents::Text(format!("{}\n", std::str::from_utf8(model.contents.as_bytes()).unwrap()));

    let report = validate_compile_output(&output);

    assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.mismatch"));
}

#[test]
fn catches_invalid_debug_descriptor_schema() {
    let mut output = compile(project());
    let descriptor = output.files.iter_mut().find(|file| file.path.ends_with("/window/debug.json")).unwrap();
    let mut document: Value = serde_json::from_slice(descriptor.contents.as_bytes()).unwrap();
    document["schema_version"] = Value::from(999);
    descriptor.contents = FileContents::Text(serde_json::to_string(&document).unwrap());

    let report = validate_compile_output(&output);

    assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.invalid"));
    assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.mismatch"));
}

#[test]
fn catches_actual_minecraft_cursor_drift() {
    let mut output = compile(project());
    let main = output.files.iter_mut().find(|file| file.path.ends_with("/font/ui.json")).unwrap();
    let mut document: Value = serde_json::from_slice(main.contents.as_bytes()).unwrap();
    let advances = document["providers"][0]["advances"].as_object_mut().unwrap();
    let key = advances.keys().next().unwrap().clone();
    *advances.get_mut(&key).unwrap() = Value::from(-999);
    main.contents = FileContents::Text(serde_json::to_string(&document).unwrap());
    let report = validate_compile_output(&output);
    assert!(
        report
            .issues
            .iter()
            .any(|issue| { issue.code == "spacer.advance.mismatch" || issue.code == "segment.advance.mismatch" })
    );
}

#[test]
fn catches_shifted_font_ascent_drift() {
    let mut output = compile(project());
    let shifted = output.files.iter_mut().find(|file| file.path.contains("/font/ym1.json")).unwrap();
    let mut document: Value = serde_json::from_slice(shifted.contents.as_bytes()).unwrap();
    document["providers"][1]["ascent"] = Value::from(0);
    shifted.contents = FileContents::Text(serde_json::to_string(&document).unwrap());
    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "font.shifted.ascent_mismatch"));
}

#[test]
fn catches_shifted_font_provider_metric_drift() {
    let mut output = compile(project());
    let shifted = output.files.iter_mut().find(|file| file.path.contains("/font/ym1.json")).unwrap();
    let mut document: Value = serde_json::from_slice(shifted.contents.as_bytes()).unwrap();
    document["providers"][2]["chars"][56] = Value::from("\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    shifted.contents = FileContents::Text(serde_json::to_string(&document).unwrap());
    let report = validate_compile_output(&output);
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.code == "font.shifted.glyph_missing" && issue.message.contains("U+25C6"))
    );
}

#[test]
fn catches_fill_slots_that_are_not_routed() {
    let mut output = compile(project());
    let window = output.manifest.windows.get_mut("validation").unwrap();
    let buy = window.buttons.get_mut("buy").unwrap();
    buy.fill_slots = Some(vec![SlotRefEntry { area: SlotAreaEntry::Container, index: 8 }]);
    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.fill_not_routed"), "{report}");
}

#[test]
fn catches_two_buttons_routing_one_slot() {
    let mut output = compile(project());
    let window = output.manifest.windows.get_mut("validation").unwrap();
    let buy = window.buttons.get("buy").unwrap().clone();
    // A second button routes `buy`'s slots but fills none of them, so the
    // ownership map stays clean and only the routing map catches it.
    window.buttons.insert(
        "shadow".into(),
        crate::manifest::ButtonEntry { fill_slots: Some(buy.slots[..1].to_vec()), ..buy.clone() },
    );
    window.buttons.get_mut("buy").unwrap().fill_slots = Some(buy.slots[1..].to_vec());
    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.duplicate_route"), "{report}");
}

#[test]
fn catches_inventory_ownership_and_bounds_corruption() {
    let mut output = compile(project());
    let window = output.manifest.windows.get_mut("validation").unwrap();
    let buy = window.buttons.get_mut("buy").unwrap();
    buy.slots.push(SlotRefEntry { area: SlotAreaEntry::Container, index: 99 });
    window.items.insert("duplicate".into(), crate::manifest::ItemEntry { slots: buy.slots.clone() });
    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.out_of_bounds"));
    assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.duplicate_owner"));
}

#[test]
fn catches_corrupt_anvil_input_slot() {
    let mut output = compile(
        r#"{
          "windows": [{
            "name": "search",
            "container": "anvil",
            "children": [{ "type": "anvil_input", "name": "query" }]
          }]
        }"#,
    );
    output.manifest.windows.get_mut("search").unwrap().inputs.get_mut("query").unwrap().slot.index = 1;

    let report = validate_compile_output(&output);
    assert!(report.issues.iter().any(|issue| issue.code == "inventory.input.slot_mismatch"));
}
