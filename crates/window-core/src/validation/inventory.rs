use std::collections::BTreeMap;

use super::ValidationReport;
use super::groups::validate_groups;
use crate::manifest::{
    CasePath, CasePaths, Manifest, RegionEntry, SlotAreaEntry, SlotRefEntry, WindowEntry, case_paths, exclusive_cases,
};
use crate::surface::ContainerKind;

pub(super) fn validate_inventory(manifest: &Manifest, report: &mut ValidationReport) {
    for (window_name, window) in &manifest.windows {
        let Some(kind) = ContainerKind::parse(&window.surface.container) else {
            report.push(
                "inventory.container.unknown",
                format!("manifest.windows.{window_name}.surface.container"),
                format!("unsupported container `{}`", window.surface.container),
            );
            continue;
        };
        let mut claims = SlotClaims { window_name, kind, owners: BTreeMap::new(), routes: BTreeMap::new() };
        let paths = case_paths(&window.switches);
        validate_regions(window, &paths, &mut claims, report);
        for (name, item) in &window.items {
            let path = paths.items.get(name).map(Vec::as_slice).unwrap_or_default();
            claims.own(&format!("item `{name}`"), path, &item.slots, report);
        }
        validate_inputs(window, &mut claims, report);
        validate_collections(window, &paths, &mut claims, report);
        validate_groups(window_name, window, report);
    }
}

/// Slot ownership tracks the slot a control fills; routing is a separate,
/// also-exclusive map so a repeater cell can route a slot it no longer fills. Regions in mutually exclusive switch
/// cases may share both.
struct SlotClaims<'a> {
    window_name: &'a str,
    kind: ContainerKind,
    owners: BTreeMap<SlotRefEntry, Vec<(String, CasePath)>>,
    routes: BTreeMap<SlotRefEntry, Vec<(String, CasePath)>>,
}

impl SlotClaims<'_> {
    fn own(&mut self, owner: &str, path: &[(String, String)], slots: &[SlotRefEntry], report: &mut ValidationReport) {
        for slot in slots {
            let valid = match slot.area {
                SlotAreaEntry::Container => slot.index < self.kind.slot_count(),
                SlotAreaEntry::Player => slot.index < 36,
            };
            if !valid {
                report.push(
                    "inventory.slot.out_of_bounds",
                    format!("manifest.windows.{}", self.window_name),
                    format!("{owner} owns invalid {:?} slot {}", slot.area, slot.index),
                );
            }
            let owners = self.owners.entry(*slot).or_default();
            if let Some((previous, _)) = owners.iter().find(|(_, other)| !exclusive_cases(path, other)) {
                report.push(
                    "inventory.slot.duplicate_owner",
                    format!("manifest.windows.{}", self.window_name),
                    format!("{:?} slot {} is owned by both {previous} and {owner}", slot.area, slot.index),
                );
            }
            owners.push((owner.to_string(), path.to_vec()));
        }
    }

    /// Route clicks on `slot` to `name`. `describe` renders `(previous, current)` router names for the
    /// duplicate-route finding.
    fn route(&mut self, slot: SlotRefEntry, path: &[(String, String)], name: &str, report: &mut ValidationReport) {
        let routes = self.routes.entry(slot).or_default();
        if let Some((previous, _)) = routes.iter().find(|(_, other)| !exclusive_cases(path, other)) {
            report.push(
                "inventory.slot.duplicate_route",
                format!("manifest.windows.{}", self.window_name),
                format!("{:?} slot {} routes clicks to both {previous} and {name}", slot.area, slot.index),
            );
        }
        routes.push((name.to_string(), path.to_vec()));
    }
}

fn validate_regions(window: &WindowEntry, paths: &CasePaths, claims: &mut SlotClaims, report: &mut ValidationReport) {
    for (name, region) in &window.regions {
        let path = paths.regions.get(name).map(Vec::as_slice).unwrap_or_default();
        let owner = format!("region `{name}`");
        claims.own(&owner, path, region.filled_slots(), report);
        validate_fill_slots(window, claims.window_name, name, region, report);
        if region.action.is_some() {
            for slot in &region.slots {
                claims.route(*slot, path, &owner, report);
            }
        }
    }
}

/// A region may fill no slot only when it routes the anvil input's slot, whose seed item the input keeps.
fn validate_fill_slots(
    window: &WindowEntry,
    window_name: &str,
    name: &str,
    region: &RegionEntry,
    report: &mut ValidationReport,
) {
    let Some(fill_slots) = &region.fill_slots else {
        return;
    };
    for slot in fill_slots {
        if !region.slots.contains(slot) {
            report.push(
                "inventory.slot.fill_not_routed",
                format!("manifest.windows.{window_name}.regions.{name}.fill_slots"),
                format!("region `{name}` fills {:?} slot {} that it does not route", slot.area, slot.index),
            );
        }
    }
    let routes_input = window.inputs.values().any(|input| region.slots.contains(&input.slot));
    if fill_slots.is_empty() && !routes_input {
        report.push(
            "inventory.slot.fill_empty",
            format!("manifest.windows.{window_name}.regions.{name}.fill_slots"),
            format!("region `{name}` fills no slot; omit `fill_slots` instead"),
        );
    }
}

fn validate_inputs(window: &WindowEntry, claims: &mut SlotClaims, report: &mut ValidationReport) {
    let window_name = claims.window_name;
    if window.inputs.len() > 1 {
        report.push(
            "inventory.input.too_many",
            format!("manifest.windows.{window_name}.inputs"),
            "anvil surfaces support at most one native text input",
        );
    }
    for (name, input) in &window.inputs {
        claims.own(&format!("anvil input `{name}`"), &[], &[input.slot], report);
        if claims.kind != ContainerKind::Anvil {
            report.push(
                "inventory.input.surface_mismatch",
                format!("manifest.windows.{window_name}.inputs.{name}"),
                "anvil input requires container `anvil`",
            );
        }
        let expected_slot = SlotRefEntry { area: SlotAreaEntry::Container, index: 0 };
        if input.slot != expected_slot {
            report.push(
                "inventory.input.slot_mismatch",
                format!("manifest.windows.{window_name}.inputs.{name}.slot"),
                "anvil input must own container slot 0",
            );
        }
    }
}

fn validate_collections(
    window: &WindowEntry,
    paths: &CasePaths,
    claims: &mut SlotClaims,
    report: &mut ValidationReport,
) {
    for (name, collection) in &window.collections {
        let path = paths.collections.get(name).map(Vec::as_slice).unwrap_or_default();
        claims.own(&format!("collection `{name}`"), path, &collection.slots, report);
        if !collection.action {
            continue;
        }
        for slot in &collection.slots {
            claims.route(*slot, path, &format!("collection `{name}`"), report);
        }
    }
}
