use std::collections::BTreeMap;

use super::ValidationReport;
use crate::manifest::WindowEntry;

pub(super) fn validate_groups(window_name: &str, window: &WindowEntry, report: &mut ValidationReport) {
    for (group_name, group) in &window.groups {
        for (field, names) in &group.slots {
            validate_group_names(window_name, group_name, field, group.count, names, &window.slots, "slot", report);
        }
        for (field, names) in &group.sprite_slots {
            validate_group_names(
                window_name,
                group_name,
                field,
                group.count,
                names,
                &window.sprite_slots,
                "sprite slot",
                report,
            );
        }
        for (field, names) in &group.items {
            validate_group_names(window_name, group_name, field, group.count, names, &window.items, "item", report);
        }
        if !group.actions.is_empty() {
            let actions: BTreeMap<String, ()> =
                window.regions.values().filter_map(|region| region.action.clone()).map(|action| (action, ())).collect();
            validate_group_names(
                window_name,
                group_name,
                "actions",
                group.count,
                &group.actions,
                &actions,
                "region action",
                report,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_group_names<T>(
    window_name: &str,
    group_name: &str,
    field: &str,
    count: u32,
    names: &[String],
    entries: &BTreeMap<String, T>,
    kind: &str,
    report: &mut ValidationReport,
) {
    let location = format!("manifest.windows.{window_name}.groups.{group_name}.{field}");
    if names.len() != count as usize {
        report.push(
            "group.count.mismatch",
            &location,
            format!("contains {} names but group count is {count}", names.len()),
        );
    }
    for name in names {
        if !entries.contains_key(name) {
            report.push("group.control.missing", &location, format!("references missing {kind} `{name}`"));
        }
    }
}
