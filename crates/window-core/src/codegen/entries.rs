//! Kotlin constructor expressions for manifest entries, rendered with their closing paren at `level`.

use std::collections::BTreeMap;

use crate::ir::{FullscreenMode, Hitbox, Layer, Tooltip};
use crate::manifest::{
    AnvilInputEntry, ArtEntry, BarEntry, CollectionEntry, ControlEntry, FontMetricsEntry, FullscreenCanvasEntry,
    FullscreenSurfaceEntry, HoverEntry, HudShaderEntry, HudSurfaceEntry, ItemEntry, RegionEntry, SlotAreaEntry,
    SlotEntry, SlotLinesEntry, SlotRefEntry, SpriteEntry, SpriteSlotEntry, SurfaceEntry, SwitchCaseEntry, SwitchEntry,
    TextOverflow,
};

use super::literals::{
    align_expr, char_map, float_expr, kt_string, optional_string_expr, string_list_expr, string_map,
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

pub(super) fn fullscreen_surface_entry_expr(surface: &FullscreenSurfaceEntry, level: usize) -> String {
    let mode = match surface.mode {
        FullscreenMode::Unlocked => "FullscreenMode.UNLOCKED",
        FullscreenMode::Horse => "FullscreenMode.HORSE",
    };
    Call::new("FullscreenSurfaceEntry", level)
        .arg("kind", kt_string(&surface.kind))
        .arg("mode", mode)
        .arg("size", format!("listOf({}, {})", surface.size[0], surface.size[1]))
        .finish()
}

pub(super) fn control_entry_expr(control: &ControlEntry, level: usize) -> String {
    Call::new("ControlEntry", level)
        .arg("index", control.index)
        .arg("x", control.x)
        .arg("y", control.y)
        .arg("width", control.width)
        .arg("height", control.height)
        .arg("action", optional_string_expr(control.action.as_ref()))
        .arg("defaultAction", optional_string_expr(control.default_action.as_ref()))
        .arg("hover", control.hover)
        .arg("source", optional_string_expr(control.source.as_ref()))
        .finish()
}

pub(super) fn fullscreen_canvas_entry_expr(canvas: &FullscreenCanvasEntry, level: usize) -> String {
    let cursor = Call::new("CursorEntry", level + 1)
        .arg("text", kt_string(&canvas.cursor.text))
        .arg("marker", kt_string(&canvas.cursor.marker))
        .arg("width", canvas.cursor.width)
        .arg("height", canvas.cursor.height)
        .finish();
    Call::new("FullscreenCanvasEntry", level)
        .arg("width", canvas.width)
        .arg("height", canvas.height)
        .arg("degreesPerPixel", format!("{:?}", canvas.degrees_per_pixel))
        .arg("edgeYaw", format!("{:?}", canvas.edge_yaw))
        .arg("edgePitch", format!("{:?}", canvas.edge_pitch))
        .arg("snapEpsilon", format!("{:?}", canvas.snap_epsilon))
        .arg("hitMargin", canvas.hit_margin)
        .arg("displayScale", format!("{:?}", canvas.display_scale))
        .arg("markerAlpha", canvas.marker_alpha)
        .arg("cursor", cursor)
        .arg("backdrop", kt_string(&canvas.backdrop))
        .arg("overlayMarker", kt_string(&canvas.overlay_marker))
        .arg("hudHiddenWorldAge", format!("{}L", canvas.hud_hidden_world_age))
        .arg("hudStripHeight", canvas.hud_strip_height)
        .arg("hudStripHalfWidth", canvas.hud_strip_half_width)
        .arg("hudMaxHealth", canvas.hud_max_health)
        .finish()
}

pub(super) fn optional_hud_shader_entry_expr(shader: Option<&HudShaderEntry>, level: usize) -> String {
    let Some(shader) = shader else {
        return "null".into();
    };
    let call = Call::new("HudShaderEntry", level)
        .arg("staticMarker", kt_string(&shader.static_marker))
        .arg("sourceBottom", shader.source_bottom)
        .arg("originX", float_expr(shader.origin_x))
        .arg("originY", float_expr(shader.origin_y))
        .arg("anchorX", float_expr(shader.anchor_x))
        .arg("anchorY", float_expr(shader.anchor_y))
        .arg("offsetX", shader.offset_x)
        .arg("offsetY", shader.offset_y);
    if shader.row_offset == 0 { call } else { call.arg("rowOffset", shader.row_offset) }.finish()
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
    let call = with_source(with_tint(call, slot.tint.as_ref(), &slot.tint_markers, level), slot.source.as_ref());
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
    let mut call = Call::new("SpriteSlotEntry", level)
        .arg("x", slot.x)
        .arg("y", slot.y)
        .arg("width", slot.width)
        .arg("height", slot.height)
        .arg("align", align_expr(slot.align))
        .arg("font", kt_string(&slot.font))
        .arg("sprite", optional_string_expr(slot.sprite.as_ref()));
    if !slot.only.is_empty() {
        call = call.arg("only", string_list_expr(&slot.only));
    }
    with_source(with_tint(call, slot.tint.as_ref(), &BTreeMap::new(), level), slot.source.as_ref()).finish()
}

pub(super) fn art_entry_expr(art: &ArtEntry, level: usize) -> String {
    let call = Call::new("ArtEntry", level).arg("static", kt_string(&art.static_text));
    let call = match &art.bar {
        Some(bar) => call.arg("bar", bar_entry_expr(bar, level + 1)),
        None => call,
    };
    with_tint(call, art.tint.as_ref(), &art.tint_markers, level).finish()
}

fn bar_entry_expr(bar: &BarEntry, level: usize) -> String {
    Call::new("BarEntry", level)
        .arg("name", kt_string(&bar.name))
        .arg("x", bar.x)
        .arg("width", bar.width)
        .arg("levels", string_list_expr(&bar.levels))
        .arg("advances", char_map(&bar.advances, level + 1))
        .finish()
}

/// A `mapOf` of strings.
pub(super) fn strings_map(values: &BTreeMap<String, String>, level: usize) -> String {
    string_map(values, level, |value, _| kt_string(value))
}

/// Adds the `tint` and `tintMarkers` arguments of an entry at `level` that a tint colors.
fn with_tint(call: Call, tint: Option<&String>, markers: &BTreeMap<String, String>, level: usize) -> Call {
    let Some(tint) = tint else {
        return call;
    };
    let call = call.arg("tint", kt_string(tint));
    if markers.is_empty() { call } else { call.arg("tintMarkers", strings_map(markers, level + 1)) }
}

/// Adds the `source` argument only to slots an authored `debug_name` labels.
fn with_source(call: Call, source: Option<&String>) -> Call {
    match source {
        Some(source) => call.arg("source", kt_string(source)),
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

pub(super) fn region_entry_expr(region: &RegionEntry, level: usize) -> String {
    let inner = level + 1;
    let fill_slots = region.fill_slots.as_ref().map_or_else(|| "null".into(), |slots| slot_ref_list_expr(slots, inner));
    let call = Call::new("RegionEntry", level)
        .arg("x", region.x)
        .arg("y", region.y)
        .arg("width", region.width)
        .arg("height", region.height)
        .arg("slots", slot_ref_list_expr(&region.slots, inner))
        .arg("fillSlots", fill_slots)
        .arg("action", optional_string_expr(region.action.as_ref()))
        .arg("defaultAction", optional_string_expr(region.default_action.as_ref()))
        .arg("hitbox", optional_hitbox_expr(region.hitbox.as_ref(), inner))
        .arg("source", optional_string_expr(region.source.as_ref()));
    match &region.hover {
        Some(hover) => call.arg("hover", hover_entry_expr(hover, inner)).finish(),
        None => call.finish(),
    }
}

fn hover_entry_expr(hover: &HoverEntry, level: usize) -> String {
    Call::new("HoverEntry", level).arg("text", kt_string(&hover.text)).arg("x", hover.x).arg("y", hover.y).finish()
}

fn optional_hitbox_expr(hitbox: Option<&Hitbox>, level: usize) -> String {
    let Some(hitbox) = hitbox else {
        return "null".into();
    };
    Call::new("HitboxEntry", level)
        .arg("itemModel", optional_string_expr(hitbox.item_model.as_ref()))
        .arg("tooltip", optional_tooltip_expr(hitbox.tooltip.as_ref(), level + 1))
        .finish()
}

fn optional_tooltip_expr(tooltip: Option<&Tooltip>, level: usize) -> String {
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

pub(super) fn switch_entry_expr(switch: &SwitchEntry, level: usize) -> String {
    let cases = switch.cases.iter().map(|case| switch_case_entry_expr(case, level + 2));
    Call::new("SwitchEntry", level)
        .arg("cases", list_of(cases, level + 1))
        .arg("source", optional_string_expr(switch.source.as_ref()))
        .finish()
}

fn switch_case_entry_expr(case: &SwitchCaseEntry, level: usize) -> String {
    let mut call = Call::new("SwitchCaseEntry", level)
        .arg("value", kt_string(&case.value))
        .arg("static", kt_string(&case.static_text))
        .arg("slots", string_list_expr(&case.slots))
        .arg("spriteSlots", string_list_expr(&case.sprite_slots))
        .arg("regions", string_list_expr(&case.regions))
        .arg("switches", string_list_expr(&case.switches));
    if !case.items.is_empty() {
        call = call.arg("items", string_list_expr(&case.items));
    }
    if !case.collections.is_empty() {
        call = call.arg("collections", string_list_expr(&case.collections));
    }
    if !case.art.is_empty() {
        call = call.arg("art", string_list_expr(&case.art));
    }
    call.finish()
}

/// A `listOf` of layers, each `LayerEntry(LayerKind.X, "name")`.
pub(super) fn layers_expr(layers: &[Layer], level: usize) -> String {
    let layer = |layer: &Layer| {
        let (kind, name) = match layer {
            Layer::Slot(name) => ("SLOT", name),
            Layer::SpriteSlot(name) => ("SPRITE_SLOT", name),
            Layer::Switch(name) => ("SWITCH", name),
            Layer::Collection(name) => ("COLLECTION", name),
            Layer::Art(name) => ("ART", name),
        };
        format!("LayerEntry(LayerKind.{kind}, {})", kt_string(name))
    };
    list_of(layers.iter().map(layer), level)
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
