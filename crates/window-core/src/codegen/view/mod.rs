//! Typed abstract view classes (`*View` for windows, `*Hud` for HUDs) that bind runtime members.

use std::collections::BTreeMap;

use crate::Result;
use crate::manifest::{HudEntry, WindowEntry};
use crate::pipeline::OutputFile;

use super::{KotlinTarget, naming};

mod class;
mod handles;
mod members;

use class::ViewBase;
use members::{Members, ValueKind};

const WINDOW: ViewBase = ViewBase { prefix: "Window", noun: "window", definitions: "WindowDefinitions", hosted: true };
const HUD: ViewBase = ViewBase { prefix: "Hud", noun: "HUD", definitions: "WindowHudDefinitions", hosted: false };

pub(super) fn generate_window(
    name: &str,
    window: &WindowEntry,
    package_name: &str,
    target: KotlinTarget,
) -> Result<OutputFile> {
    let class_name = naming::class_name(name);
    let (handles, members) = Members::of_window(&class_name, window)?;
    let view = class::View { handles: &handles, members: &members, buttons: &window.buttons };
    Ok(OutputFile::text(
        format!("{class_name}.kt"),
        class::render(package_name, &WINDOW, target, name, &class_name, &view),
    ))
}

pub(super) fn generate_hud(name: &str, hud: &HudEntry, package_name: &str, target: KotlinTarget) -> Result<OutputFile> {
    let class_name = naming::hud_class_name(name);
    let mut members = Members::new(&class_name, naming::HUD_RESERVED_MEMBERS);
    let handles = members.handles(&hud.handles, &BTreeMap::new())?;
    let handled = handles::covered(&handles);
    members.indexed(&hud.indexed, &hud.switches)?;
    for (slot, entry) in &hud.slots {
        if entry.text.is_none() && !handled.contains(slot) {
            members.value(ValueKind::Slot, entry.binding.as_ref().unwrap_or(slot))?;
        }
    }
    members.switches(&hud.switches)?;
    let members = members.finish();
    let view = class::View { handles: &handles, members: &members, buttons: &BTreeMap::new() };
    Ok(OutputFile::text(
        format!("{class_name}.kt"),
        class::render(package_name, &HUD, target, name, &class_name, &view),
    ))
}
