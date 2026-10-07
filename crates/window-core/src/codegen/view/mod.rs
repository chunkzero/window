//! Typed abstract view classes (`*View` for windows, `*Hud` for HUDs) that bind runtime members.

use crate::Result;
use crate::manifest::{HudEntry, WindowEntry};
use crate::pipeline::OutputFile;

use super::{KotlinTarget, naming};

mod class;
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
    let members = Members::of_window(&class_name, window)?;
    Ok(OutputFile::text(
        format!("{class_name}.kt"),
        class::render(package_name, &WINDOW, target, name, &class_name, &members),
    ))
}

pub(super) fn generate_hud(name: &str, hud: &HudEntry, package_name: &str, target: KotlinTarget) -> Result<OutputFile> {
    let class_name = naming::hud_class_name(name);
    let mut members = Members::new(&class_name, naming::HUD_RESERVED_MEMBERS);
    members.indexed(&hud.indexed, &hud.switches)?;
    for (slot, entry) in &hud.slots {
        if entry.text.is_none() {
            members.value(ValueKind::Slot, slot)?;
        }
    }
    members.switches(&hud.switches)?;
    Ok(OutputFile::text(
        format!("{class_name}.kt"),
        class::render(package_name, &HUD, target, name, &class_name, &members.finish()),
    ))
}
