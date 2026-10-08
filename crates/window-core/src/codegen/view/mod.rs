//! Typed abstract view classes (`*View` for windows, `*Hud` for HUDs) whose members the surface's handles declare.

use std::collections::BTreeMap;

use crate::Result;
use crate::manifest::{HudEntry, WindowEntry};
use crate::pipeline::OutputFile;

use super::{KotlinTarget, naming};

mod class;
mod handles;

use class::ViewBase;
use handles::HandleMember;

const WINDOW: ViewBase = ViewBase { prefix: "Window", noun: "window", definitions: "WindowDefinitions", hosted: true };
const HUD: ViewBase = ViewBase { prefix: "Hud", noun: "HUD", definitions: "WindowHudDefinitions", hosted: false };

pub(super) fn generate_window(
    name: &str,
    window: &WindowEntry,
    package_name: &str,
    target: KotlinTarget,
) -> Result<OutputFile> {
    let class_name = naming::class_name(name);
    let cells = |id: &str| cell_count(id, window);
    let handles = HandleMember::all(&class_name, &[], &window.handles, cells)?;
    Ok(OutputFile::text(
        format!("{class_name}.kt"),
        class::render(package_name, &WINDOW, target, name, &class_name, &handles),
    ))
}

pub(super) fn generate_hud(name: &str, hud: &HudEntry, package_name: &str, target: KotlinTarget) -> Result<OutputFile> {
    let class_name = naming::hud_class_name(name);
    let handles = HandleMember::all(&class_name, naming::HUD_RESERVED_MEMBERS, &hud.handles, |_| None)?;
    Ok(OutputFile::text(
        format!("{class_name}.kt"),
        class::render(package_name, &HUD, target, name, &class_name, &handles),
    ))
}

/// The cell count shared by every collection the collection handle `id` binds; `None` when they differ.
fn cell_count(id: &str, window: &WindowEntry) -> Option<u32> {
    let handle = window.handles.get(id)?;
    let collections = &window.collections;
    let mut counts = handle.uses.iter().filter_map(|use_| collections.get(&use_.entry)).map(|c| c.slots.len() as u32);
    let first = counts.next()?;
    counts.all(|count| count == first).then_some(first)
}

/// Member names claimed so far, by the source each was claimed for.
type Taken = BTreeMap<String, String>;
