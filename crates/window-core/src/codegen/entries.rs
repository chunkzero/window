//! Kotlin constructor expressions for manifest entries, rendered with their closing paren at `level`.

use std::collections::BTreeMap;

use crate::ir::{ButtonState, ButtonTooltip};
use crate::manifest::{
    AnvilInputEntry, ButtonEntry, CollectionEntry, FontMetricsEntry, HoverEntry, HudShaderEntry, HudSurfaceEntry,
    ItemEntry, RepeatGroupEntry, SlotAreaEntry, SlotEntry, SlotLinesEntry, SlotRectEntry, SlotRefEntry, SpriteEntry,
    SpriteSlotEntry, SurfaceEntry, SwitchCaseEntry, SwitchEntry, TextOverflow,
};

use super::literals::{
    align_expr, char_map, float_expr, kt_string, optional_button_default_expr, optional_string_expr, string_list_expr,
    string_map,
};
use super::writer::{Call, indent, list_of};

pub(super) fn surface_entry_expr(surface: &SurfaceEntry, level: usize) -> String {
    Call::new("SurfaceEntry", level)
        .arg("kind", kt_string(&surface.kind))
        .arg("container", kt_string(&surface.container))
        .arg("size", format!("listOf({}, {})", surface.size[0], surface.size[1]))
        .arg("titleOrigin", format!("listOf({}, {})", surface.title_origin[0], surface.title_origin[1]))
        .finish()
}

pub(super) fn hud_surface_entry_expr(surface: &HudSurfaceEntry, level: usize) -> String {
    Call::new("HudSurfaceEntry", level)
        .arg("kind", kt_string(&surface.kind))
        .arg("channel", kt_string(&surface.channel))
        .arg("width", surface.width)
        .arg("height", surface.height)
        .finish()
}

pub(super) fn optional_hud_shader_entry_expr(shader: Option<&HudShaderEntry>, level: usize) -> String {
    let Some(shader) = shader else {
        return "null".into();
    };
    Call::new("HudShaderEntry", level)
        .arg("staticMarker", kt_string(&shader.static_marker))
        .arg("sourceBottom", shader.source_bottom)
        .arg("originX", float_expr(shader.origin_x))
        .arg("originY", float_expr(shader.origin_y))
        .arg("anchorX", float_expr(shader.anchor_x))
        .arg("anchorY", float_expr(shader.anchor_y))
        .arg("offsetX", shader.offset_x)
        .arg("offsetY", shader.offset_y)
        .finish()
}

pub(super) fn slot_entry_expr(slot: &SlotEntry, level: usize) -> String {
    let call = Call::new("SlotEntry", level)
        .arg("x", slot.x)
        .arg("y", slot.y)
        .arg("width", slot.width)
        .arg("align", align_expr(slot.align))
        .arg("font", kt_string(&slot.font))
        .arg("color", kt_string(&slot.color))
        .arg("shadow", slot.shadow)
        .arg("bold", slot.bold)
        .arg("italic", slot.italic)
        .arg("underlined", slot.underlined)
        .arg("strikethrough", slot.strikethrough)
        .arg("obfuscated", slot.obfuscated)
        .arg("text", optional_string_expr(slot.text.as_ref()))
        .arg("shaderMarker", optional_string_expr(slot.shader_marker.as_ref()))
        .arg("shaderColor", optional_string_expr(slot.shader_color.as_ref()));
    let call = with_binding(call, slot.binding.as_ref());
    let call = match slot.overflow {
        Some(TextOverflow::Ellipsis) => call.arg("overflow", "TextOverflow.ELLIPSIS"),
        None => call,
    };
    match &slot.lines {
        Some(lines) => call.arg("lines", slot_lines_entry_expr(lines, level + 1)),
        None => call,
    }
    .finish()
}

fn slot_lines_entry_expr(lines: &SlotLinesEntry, level: usize) -> String {
    Call::new("SlotLinesEntry", level)
        .arg("count", lines.count)
        .arg("lineHeight", lines.line_height)
        .arg("fonts", string_list_expr(&lines.fonts))
        .finish()
}

pub(super) fn sprite_entry_expr(sprite: &SpriteEntry, level: usize) -> String {
    Call::new("SpriteEntry", level)
        .arg("width", sprite.width)
        .arg("height", sprite.height)
        .arg("xOffset", sprite.x_offset)
        .arg("glyphWidth", sprite.glyph_width)
        .arg("advance", sprite.advance)
        .arg("glyph", kt_string(&sprite.glyph))
        .finish()
}

pub(super) fn sprite_slot_entry_expr(slot: &SpriteSlotEntry, level: usize) -> String {
    let call = Call::new("SpriteSlotEntry", level)
        .arg("x", slot.x)
        .arg("y", slot.y)
        .arg("width", slot.width)
        .arg("height", slot.height)
        .arg("align", align_expr(slot.align))
        .arg("font", kt_string(&slot.font))
        .arg("sprite", optional_string_expr(slot.sprite.as_ref()));
    with_binding(call, slot.binding.as_ref()).finish()
}

/// Adds the `binding` argument only to slots that share a binding across switch cases.
fn with_binding(call: Call, binding: Option<&String>) -> Call {
    match binding {
        Some(binding) => call.arg("binding", kt_string(binding)),
        None => call,
    }
}

/// Renders font metrics, referencing `advances_expr`/`glyph_widths_expr` when a table equals the vanilla one.
pub(super) fn font_metrics_expr(
    metrics: &FontMetricsEntry,
    advances_expr: &str,
    glyph_widths_expr: &str,
    level: usize,
) -> String {
    let table = |values: &BTreeMap<char, u32>, vanilla: BTreeMap<char, u32>, vanilla_expr: &str| {
        if values.is_empty() {
            "emptyMap()".to_string()
        } else if *values == vanilla {
            vanilla_expr.to_string()
        } else {
            // Custom tables keep a leading indent after `=`; the generated output depends on it.
            format!("{}{}", indent(level + 1), char_map(values, level + 1))
        }
    };
    Call::new("FontMetricsEntry", level)
        .arg("advances", table(&metrics.advances, crate::vanilla::advances().collect(), advances_expr))
        .arg("glyphWidths", table(&metrics.glyph_widths, crate::vanilla::glyph_widths().collect(), glyph_widths_expr))
        .arg("boldAdvance", metrics.bold_advance)
        .finish()
}

pub(super) fn button_entry_expr(button: &ButtonEntry, level: usize) -> String {
    let inner = level + 1;
    let fill_slots = button.fill_slots.as_ref().map_or_else(|| "null".into(), |slots| slot_ref_list_expr(slots, inner));
    let call = Call::new("ButtonEntry", level)
        .arg("x", button.x)
        .arg("y", button.y)
        .arg("width", button.width)
        .arg("height", button.height)
        .arg("slots", slot_ref_list_expr(&button.slots, inner))
        .arg("fillSlots", fill_slots)
        .arg("default", optional_button_default_expr(button.default))
        .arg("action", button.action)
        .arg("tooltip", optional_tooltip_expr(button.tooltip.as_ref(), inner))
        .arg("states", string_map(&button.states, inner, button_state_expr))
        .arg("spriteFont", optional_string_expr(button.sprite_font.as_ref()));
    match &button.hover {
        Some(hover) => call.arg("hover", hover_entry_expr(hover, inner)).finish(),
        None => call.finish(),
    }
}

fn hover_entry_expr(hover: &HoverEntry, level: usize) -> String {
    Call::new("HoverEntry", level)
        .arg("glyph", kt_string(&hover.glyph))
        .arg("advance", hover.advance)
        .arg("x", hover.x)
        .arg("y", hover.y)
        .finish()
}

fn button_state_expr(state: &ButtonState, level: usize) -> String {
    Call::new("ButtonState", level)
        .arg("itemModel", optional_string_expr(state.item_model.as_ref()))
        .arg("sprite", optional_string_expr(state.sprite.as_ref()))
        .arg("tooltip", optional_tooltip_expr(state.tooltip.as_ref(), level + 1))
        .finish()
}

fn optional_tooltip_expr(tooltip: Option<&ButtonTooltip>, level: usize) -> String {
    let Some(tooltip) = tooltip else {
        return "null".into();
    };
    Call::new("TooltipEntry", level)
        .arg("title", kt_string(&tooltip.title))
        .arg("lines", string_list_expr(&tooltip.lines))
        .finish()
}

pub(super) fn item_entry_expr(item: &ItemEntry, level: usize) -> String {
    Call::new("ItemEntry", level).arg("slots", slot_ref_list_expr(&item.slots, level + 1)).finish()
}

pub(super) fn collection_entry_expr(collection: &CollectionEntry, level: usize) -> String {
    let mut call = Call::new("CollectionEntry", level)
        .arg("slots", slot_ref_list_expr(&collection.slots, level + 1))
        .arg("action", collection.action);
    if !collection.selection.is_empty() {
        let cells = collection.selection.iter().map(|cell| sprite_slot_entry_expr(cell, level + 2));
        call = call.arg("selection", list_of(cells, level + 1));
    }
    call.finish()
}

pub(super) fn anvil_input_entry_expr(input: &AnvilInputEntry, level: usize) -> String {
    Call::new("AnvilInputEntry", level)
        .arg("slot", slot_ref_expr(&input.slot, level + 1))
        .arg("initial", kt_string(&input.initial))
        .arg("itemModel", optional_string_expr(input.item_model.as_ref()))
        .finish()
}

pub(super) fn slot_rect_entry_expr(slot_rect: &SlotRectEntry, level: usize) -> String {
    Call::new("SlotRectEntry", level).arg("slots", slot_ref_list_expr(&slot_rect.slots, level + 1)).finish()
}

pub(super) fn repeat_group_entry_expr(group: &RepeatGroupEntry, level: usize) -> String {
    let inner = level + 1;
    let string_lists = |values: &BTreeMap<String, Vec<String>>| {
        string_map(values, inner, |strings: &Vec<String>, _| string_list_expr(strings))
    };
    Call::new("RepeatGroupEntry", level)
        .arg("count", group.count)
        .arg("slots", string_lists(&group.slots))
        .arg("spriteSlots", string_lists(&group.sprite_slots))
        .arg("items", string_lists(&group.items))
        .arg("buttons", string_list_expr(&group.buttons))
        .finish()
}

pub(super) fn switch_entry_expr(switch: &SwitchEntry, level: usize) -> String {
    let cases = switch.cases.iter().map(|case| switch_case_entry_expr(case, level + 2));
    Call::new("SwitchEntry", level).arg("cases", list_of(cases, level + 1)).finish()
}

fn switch_case_entry_expr(case: &SwitchCaseEntry, level: usize) -> String {
    Call::new("SwitchCaseEntry", level)
        .arg("value", kt_string(&case.value))
        .arg("static", kt_string(&case.static_text))
        .arg("slots", string_list_expr(&case.slots))
        .arg("spriteSlots", string_list_expr(&case.sprite_slots))
        .finish()
}

fn slot_ref_list_expr(slots: &[SlotRefEntry], level: usize) -> String {
    list_of(slots.iter().map(|slot| slot_ref_expr(slot, level + 1)), level)
}

fn slot_ref_expr(slot: &SlotRefEntry, level: usize) -> String {
    let area = match slot.area {
        SlotAreaEntry::Container => "SlotAreaEntry.CONTAINER",
        SlotAreaEntry::Player => "SlotAreaEntry.PLAYER",
    };
    Call::new("SlotRefEntry", level).arg("area", area).arg("index", slot.index).finish()
}
