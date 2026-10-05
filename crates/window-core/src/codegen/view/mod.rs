//! Typed abstract view classes (`*View` for windows, `*Hud` for HUDs) that bind runtime members.

use crate::Result;
use crate::manifest::{HudEntry, WindowEntry};
use crate::pipeline::OutputFile;

use super::naming;

mod class;
mod members;

use class::ViewBase;
use members::{Members, ValueKind};

const WINDOW: ViewBase = ViewBase { prefix: "Window", noun: "window" };
const HUD: ViewBase = ViewBase { prefix: "Hud", noun: "HUD" };

pub(super) fn generate_window(name: &str, window: &WindowEntry, package_name: &str) -> Result<OutputFile> {
    let class_name = naming::class_name(name);
    let members = Members::of_window(&class_name, window)?;
    Ok(OutputFile {
        path: format!("{class_name}.kt"),
        contents: class::render(package_name, &WINDOW, name, &class_name, &members).into_bytes(),
    })
}

pub(super) fn generate_hud(name: &str, hud: &HudEntry, package_name: &str) -> Result<OutputFile> {
    let class_name = naming::hud_class_name(name);
    let mut members = Members::new(&class_name);
    for (slot, entry) in &hud.slots {
        if entry.text.is_none() {
            members.value(ValueKind::Slot, slot)?;
        }
    }
    members.switches(&hud.switches)?;
    Ok(OutputFile {
        path: format!("{class_name}.kt"),
        contents: class::render(package_name, &HUD, name, &class_name, &members.finish()).into_bytes(),
    })
}
