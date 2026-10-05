//! `WindowDefinitions` and `WindowHudDefinitions` objects holding every compiled UI entry.

use crate::manifest::{HudEntry, Manifest, WindowEntry};
use crate::pipeline::OutputFile;

use super::entries::{
    anvil_input_entry_expr, button_entry_expr, collection_entry_expr, hud_surface_entry_expr, item_entry_expr,
    optional_hud_shader_entry_expr, repeat_group_entry_expr, slot_entry_expr, slot_rect_entry_expr,
    sprite_slot_entry_expr, surface_entry_expr,
};
use super::literals::{kt_string, string_map};
use super::writer::{Call, KotlinWriter, indent, multiline_call};

const MANIFEST_PACKAGE: &str = "com.chunkzero.window.manifest";

pub(super) fn generate_window_definitions(manifest: &Manifest, package_name: &str) -> OutputFile {
    let imports = window_imports(manifest).into_iter().map(|name| format!("{MANIFEST_PACKAGE}.{name}"));
    let mut w = KotlinWriter::file(package_name, imports);
    w.doc("Generated window definitions for this Window pack.");
    w.open("public object WindowDefinitions {");
    let windows = manifest
        .windows
        .iter()
        .map(|(name, window)| format!("{} to\n{}{}", kt_string(name), indent(4), window_entry_expr(window, 4)));
    w.property("val all: Map<String, WindowEntry>", multiline_call("mapOf", "emptyMap()", windows, 2));
    w.close("}");
    OutputFile { path: "WindowDefinitions.kt".into(), contents: w.finish().into_bytes() }
}

/// Manifest classes referenced by the window entries, in import order.
fn window_imports(manifest: &Manifest) -> Vec<&'static str> {
    let any = |test: &dyn Fn(&WindowEntry) -> bool| manifest.windows.values().any(test);
    let has_slots = any(&|w| !w.slots.is_empty());
    let has_sprite_slots = any(&|w| {
        !w.sprite_slots.is_empty() || w.collections.values().any(|collection| !collection.selection.is_empty())
    });
    let has_tooltip = any(&|w| {
        w.buttons.values().any(|b| b.tooltip.is_some() || b.states.values().any(|state| state.tooltip.is_some()))
    });
    let has_slot_refs = any(&|w| {
        w.buttons.values().any(|b| !b.slots.is_empty())
            || w.items.values().any(|item| !item.slots.is_empty())
            || w.collections.values().any(|collection| !collection.slots.is_empty())
            || w.slot_rects.values().any(|slot_rect| !slot_rect.slots.is_empty())
            || !w.inputs.is_empty()
    });
    [
        (has_slots || has_sprite_slots, "Align"),
        (any(&|w| w.buttons.values().any(|b| b.default.is_some())), "ButtonDefault"),
        (any(&|w| !w.buttons.is_empty()), "ButtonEntry"),
        (any(&|w| w.buttons.values().any(|b| !b.states.is_empty())), "ButtonState"),
        (has_tooltip, "ButtonTooltip"),
        (any(&|w| !w.collections.is_empty()), "CollectionEntry"),
        (any(&|w| !w.inputs.is_empty()), "AnvilInputEntry"),
        (any(&|w| !w.items.is_empty()), "ItemEntry"),
        (any(&|w| !w.groups.is_empty()), "RepeatGroupEntry"),
        (has_slot_refs, "SlotAreaEntry"),
        (has_slots, "SlotEntry"),
        (any(&|w| !w.slot_rects.is_empty()), "SlotRectEntry"),
        (has_slot_refs, "SlotRefEntry"),
        (has_sprite_slots, "SpriteSlotEntry"),
        (true, "SurfaceEntry"),
        (true, "WindowEntry"),
    ]
    .into_iter()
    .filter_map(|(used, name)| used.then_some(name))
    .collect()
}

fn window_entry_expr(window: &WindowEntry, level: usize) -> String {
    let inner = level + 1;
    Call::new("WindowEntry", level)
        .arg("surface", surface_entry_expr(&window.surface, inner))
        .arg("static", kt_string(&window.static_text))
        .arg("slots", string_map(&window.slots, inner, slot_entry_expr))
        .arg("spriteSlots", string_map(&window.sprite_slots, inner, sprite_slot_entry_expr))
        .arg("buttons", string_map(&window.buttons, inner, button_entry_expr))
        .arg("items", string_map(&window.items, inner, item_entry_expr))
        .arg("collections", string_map(&window.collections, inner, collection_entry_expr))
        .arg("inputs", string_map(&window.inputs, inner, anvil_input_entry_expr))
        .arg("slotRects", string_map(&window.slot_rects, inner, slot_rect_entry_expr))
        .arg("groups", string_map(&window.groups, inner, repeat_group_entry_expr))
        .finish()
}

pub(super) fn generate_hud_definitions(manifest: &Manifest, package_name: &str) -> OutputFile {
    let imports = ["Align", "HudEntry", "HudShaderEntry", "HudSurfaceEntry", "SlotEntry"];
    let mut w = KotlinWriter::file(package_name, imports.map(|name| format!("{MANIFEST_PACKAGE}.{name}")));
    w.doc("Generated HUD definitions for this Window pack.");
    w.open("public object WindowHudDefinitions {");
    w.property("val all: Map<String, HudEntry>", string_map(&manifest.huds, 2, hud_entry_expr));
    w.close("}");
    OutputFile { path: "WindowHudDefinitions.kt".into(), contents: w.finish().into_bytes() }
}

fn hud_entry_expr(hud: &HudEntry, level: usize) -> String {
    let inner = level + 1;
    Call::new("HudEntry", level)
        .arg("surface", hud_surface_entry_expr(&hud.surface, inner))
        .arg("static", kt_string(&hud.static_text))
        .arg("slots", string_map(&hud.slots, inner, slot_entry_expr))
        .arg("shader", optional_hud_shader_entry_expr(hud.shader.as_ref(), inner))
        .finish()
}
