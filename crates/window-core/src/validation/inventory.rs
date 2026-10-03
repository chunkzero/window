use std::collections::BTreeMap;

use super::ValidationReport;
use super::groups::validate_groups;
use crate::manifest::{ButtonEntry, Manifest, SlotAreaEntry, SlotRefEntry, WindowEntry};
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
        validate_buttons(window, &mut claims, report);
        for (name, item) in &window.items {
            claims.own(&format!("item `{name}`"), &item.slots, report);
        }
        validate_inputs(window, &mut claims, report);
        validate_collections(window, &mut claims, report);
        for (name, slot_rect) in &window.slot_rects {
            claims.own(&format!("slot rect `{name}`"), &slot_rect.slots, report);
        }
        validate_groups(window_name, window, report);
    }
}

/// Slot ownership tracks the slot a control fills; routing is a separate,
/// also-exclusive map so a repeater cell can route a slot it no longer fills.
struct SlotClaims<'a> {
    window_name: &'a str,
    kind: ContainerKind,
    owners: BTreeMap<SlotRefEntry, String>,
    routes: BTreeMap<SlotRefEntry, String>,
}

impl SlotClaims<'_> {
    fn own(&mut self, owner: &str, slots: &[SlotRefEntry], report: &mut ValidationReport) {
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
            if let Some(previous) = self.owners.insert(*slot, owner.to_string()) {
                report.push(
                    "inventory.slot.duplicate_owner",
                    format!("manifest.windows.{}", self.window_name),
                    format!("{:?} slot {} is owned by both {previous} and {owner}", slot.area, slot.index),
                );
            }
        }
    }

    /// Route clicks on `slot` to `name`. `describe` renders `(previous, current)` router names for the
    /// duplicate-route finding.
    fn route(
        &mut self,
        slot: SlotRefEntry,
        name: &str,
        describe: impl FnOnce(&str, &str) -> String,
        report: &mut ValidationReport,
    ) {
        if let Some(previous) = self.routes.insert(slot, name.to_string()) {
            report.push(
                "inventory.slot.duplicate_route",
                format!("manifest.windows.{}", self.window_name),
                format!("{:?} slot {} routes clicks to both {}", slot.area, slot.index, describe(&previous, name)),
            );
        }
    }
}

fn validate_buttons(window: &WindowEntry, claims: &mut SlotClaims, report: &mut ValidationReport) {
    for (name, button) in &window.buttons {
        claims.own(&format!("button `{name}`"), button.filled_slots(), report);
        validate_fill_slots(window, claims.window_name, name, button, report);
        for slot in &button.slots {
            claims.route(*slot, name, |previous, name| format!("button `{previous}` and button `{name}`"), report);
        }
    }
}

/// A button may fill no slot only when it routes the anvil input's slot, whose seed item the input keeps.
fn validate_fill_slots(
    window: &WindowEntry,
    window_name: &str,
    name: &str,
    button: &ButtonEntry,
    report: &mut ValidationReport,
) {
    let Some(fill_slots) = &button.fill_slots else {
        return;
    };
    for slot in fill_slots {
        if !button.slots.contains(slot) {
            report.push(
                "inventory.slot.fill_not_routed",
                format!("manifest.windows.{window_name}.buttons.{name}.fill_slots"),
                format!("button `{name}` fills {:?} slot {} that it does not route", slot.area, slot.index),
            );
        }
    }
    let routes_input = window.inputs.values().any(|input| button.slots.contains(&input.slot));
    if fill_slots.is_empty() && !routes_input {
        report.push(
            "inventory.slot.fill_empty",
            format!("manifest.windows.{window_name}.buttons.{name}.fill_slots"),
            format!("button `{name}` fills no slot; omit `fill_slots` instead"),
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
        claims.own(&format!("anvil input `{name}`"), &[input.slot], report);
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

fn validate_collections(window: &WindowEntry, claims: &mut SlotClaims, report: &mut ValidationReport) {
    for (name, collection) in &window.collections {
        claims.own(&format!("collection `{name}`"), &collection.slots, report);
        if !collection.action {
            continue;
        }
        for slot in &collection.slots {
            claims.route(*slot, name, |previous, name| format!("`{previous}` and collection `{name}`"), report);
        }
    }
}
