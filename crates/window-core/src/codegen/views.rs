use std::collections::BTreeMap;

use super::INDENT;
use crate::ir::{Align, ButtonDefault, ButtonState, ButtonTooltip};
use crate::manifest::{
    AnvilInputEntry, ButtonEntry, CollectionEntry, FontMetricsEntry, HudShaderEntry, HudSurfaceEntry, ItemEntry,
    RepeatGroupEntry, SlotAreaEntry, SlotEntry, SlotRectEntry, SlotRefEntry, SpriteEntry, SpriteSlotEntry,
    SurfaceEntry,
};

pub(super) fn surface_entry_expr(surface: &SurfaceEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!(
        "SurfaceEntry(\n{ii}kind = {},\n{ii}container = {},\n{ii}size = listOf({}, {}),\n{ii}titleOrigin = listOf({}, {}),\n{i})",
        kt_string(&surface.kind),
        kt_string(&surface.container),
        surface.size[0],
        surface.size[1],
        surface.title_origin[0],
        surface.title_origin[1],
    )
}

pub(super) fn hud_surface_entry_expr(surface: &HudSurfaceEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!(
        "HudSurfaceEntry(\n{ii}kind = {},\n{ii}channel = {},\n{ii}width = {},\n{ii}height = {},\n{i})",
        kt_string(&surface.kind),
        kt_string(&surface.channel),
        surface.width,
        surface.height,
    )
}

pub(super) fn slot_map_expr(slots: &BTreeMap<String, SlotEntry>, level: usize) -> String {
    if slots.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, slot) in slots {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&slot_entry_expr(slot, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn sprite_map_expr(sprites: &BTreeMap<String, SpriteEntry>, level: usize) -> String {
    if sprites.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, sprite) in sprites {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&sprite_entry_expr(sprite, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn sprite_entry_expr(sprite: &SpriteEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!(
        "SpriteEntry(\n{ii}width = {},\n{ii}height = {},\n{ii}xOffset = {},\n{ii}glyphWidth = {},\n{ii}advance = {},\n{ii}glyph = {},\n{i})",
        sprite.width,
        sprite.height,
        sprite.x_offset,
        sprite.glyph_width,
        sprite.advance,
        kt_string(&sprite.glyph),
    )
}

pub(super) fn sprite_slot_map_expr(slots: &BTreeMap<String, SpriteSlotEntry>, level: usize) -> String {
    if slots.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, slot) in slots {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&sprite_slot_entry_expr(slot, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn sprite_slot_entry_expr(slot: &SpriteSlotEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!(
        "SpriteSlotEntry(\n{ii}x = {},\n{ii}y = {},\n{ii}width = {},\n{ii}height = {},\n{ii}align = {},\n{ii}font = {},\n{ii}sprite = {},\n{i})",
        slot.x,
        slot.y,
        slot.width,
        slot.height,
        align_expr(slot.align),
        kt_string(&slot.font),
        optional_string_expr(slot.sprite.as_ref()),
    )
}

fn slot_entry_expr(slot: &SlotEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::new();
    out.push_str("SlotEntry(\n");
    out.push_str(&ii);
    out.push_str("x = ");
    out.push_str(&slot.x.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("y = ");
    out.push_str(&slot.y.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("width = ");
    out.push_str(&slot.width.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("align = ");
    out.push_str(align_expr(slot.align));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("font = ");
    out.push_str(&kt_string(&slot.font));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("color = ");
    out.push_str(&kt_string(&slot.color));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("shadow = ");
    out.push_str(bool_expr(slot.shadow));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("bold = ");
    out.push_str(bool_expr(slot.bold));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("italic = ");
    out.push_str(bool_expr(slot.italic));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("underlined = ");
    out.push_str(bool_expr(slot.underlined));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("strikethrough = ");
    out.push_str(bool_expr(slot.strikethrough));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("obfuscated = ");
    out.push_str(bool_expr(slot.obfuscated));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("text = ");
    out.push_str(&optional_string_expr(slot.text.as_ref()));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("shaderMarker = ");
    out.push_str(&optional_string_expr(slot.shader_marker.as_ref()));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("shaderColor = ");
    out.push_str(&optional_string_expr(slot.shader_color.as_ref()));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn button_map_expr(buttons: &BTreeMap<String, ButtonEntry>, level: usize) -> String {
    if buttons.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, button) in buttons {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&button_entry_expr(button, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn button_entry_expr(button: &ButtonEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::new();
    out.push_str("ButtonEntry(\n");
    out.push_str(&ii);
    out.push_str("x = ");
    out.push_str(&button.x.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("y = ");
    out.push_str(&button.y.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("width = ");
    out.push_str(&button.width.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("height = ");
    out.push_str(&button.height.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("slots = ");
    out.push_str(&slot_ref_list_expr(&button.slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("fillSlots = ");
    match &button.fill_slots {
        Some(fill_slots) => out.push_str(&slot_ref_list_expr(fill_slots, level + 1)),
        None => out.push_str("null"),
    }
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("default = ");
    out.push_str(optional_button_default_expr(button.default));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("action = ");
    out.push_str(bool_expr(button.action));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("tooltip = ");
    out.push_str(&optional_tooltip_expr(button.tooltip.as_ref(), level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("states = ");
    out.push_str(&button_state_map_expr(&button.states, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("spriteFont = ");
    out.push_str(&optional_string_expr(button.sprite_font.as_ref()));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn item_map_expr(items: &BTreeMap<String, ItemEntry>, level: usize) -> String {
    if items.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, item) in items {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&item_entry_expr(item, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn item_entry_expr(item: &ItemEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!("ItemEntry(\n{ii}slots = {},\n{i})", slot_ref_list_expr(&item.slots, level + 1))
}

pub(super) fn collection_map_expr(collections: &BTreeMap<String, CollectionEntry>, level: usize) -> String {
    if collections.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, collection) in collections {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&collection_entry_expr(collection, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn collection_entry_expr(collection: &CollectionEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("CollectionEntry(\n");
    out.push_str(&ii);
    out.push_str("slots = ");
    out.push_str(&slot_ref_list_expr(&collection.slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("action = ");
    out.push_str(bool_expr(collection.action));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn anvil_input_map_expr(inputs: &BTreeMap<String, AnvilInputEntry>, level: usize) -> String {
    if inputs.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, input) in inputs {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&anvil_input_entry_expr(input, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn anvil_input_entry_expr(input: &AnvilInputEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!(
        "AnvilInputEntry(\n{ii}slot = {},\n{ii}initial = {},\n{ii}itemModel = {},\n{i})",
        slot_ref_expr(&input.slot, level + 1),
        kt_string(&input.initial),
        optional_string_expr(input.item_model.as_ref()),
    )
}

pub(super) fn slot_rect_map_expr(slot_rects: &BTreeMap<String, SlotRectEntry>, level: usize) -> String {
    if slot_rects.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, slot_rect) in slot_rects {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&slot_rect_entry_expr(slot_rect, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn slot_rect_entry_expr(slot_rect: &SlotRectEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    format!("SlotRectEntry(\n{ii}slots = {},\n{i})", slot_ref_list_expr(&slot_rect.slots, level + 1))
}

pub(super) fn repeat_group_map_expr(groups: &BTreeMap<String, RepeatGroupEntry>, level: usize) -> String {
    if groups.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, group) in groups {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&repeat_group_entry_expr(group, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn repeat_group_entry_expr(group: &RepeatGroupEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("RepeatGroupEntry(\n");
    out.push_str(&ii);
    out.push_str("count = ");
    out.push_str(&group.count.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("slots = ");
    out.push_str(&string_vec_map_expr(&group.slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("spriteSlots = ");
    out.push_str(&string_vec_map_expr(&group.sprite_slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("items = ");
    out.push_str(&string_vec_map_expr(&group.items, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("buttons = ");
    out.push_str(&string_list_expr(&group.buttons));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

fn string_vec_map_expr(values: &BTreeMap<String, Vec<String>>, level: usize) -> String {
    if values.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, strings) in values {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&string_list_expr(strings));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn slot_ref_list_expr(slots: &[SlotRefEntry], level: usize) -> String {
    if slots.is_empty() {
        return "emptyList()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("listOf(\n");
    for slot in slots {
        out.push_str(&ii);
        out.push_str(&slot_ref_expr(slot, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn slot_ref_expr(slot: &SlotRefEntry, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let area = match slot.area {
        SlotAreaEntry::Container => "SlotAreaEntry.CONTAINER",
        SlotAreaEntry::Player => "SlotAreaEntry.PLAYER",
    };
    format!("SlotRefEntry(\n{ii}area = {area},\n{ii}index = {},\n{i})", slot.index)
}

fn button_state_map_expr(states: &BTreeMap<String, ButtonState>, level: usize) -> String {
    if states.is_empty() {
        return "emptyMap()".into();
    }
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::from("mapOf(\n");
    for (name, state) in states {
        out.push_str(&ii);
        out.push_str(&kt_string(name));
        out.push_str(" to ");
        out.push_str(&button_state_expr(state, level + 1));
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push(')');
    out
}

fn button_state_expr(state: &ButtonState, level: usize) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::new();
    out.push_str("ButtonState(\n");
    out.push_str(&ii);
    out.push_str("itemModel = ");
    out.push_str(&optional_string_expr(state.item_model.as_ref()));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("sprite = ");
    out.push_str(&optional_string_expr(state.sprite.as_ref()));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("tooltip = ");
    out.push_str(&optional_tooltip_expr(state.tooltip.as_ref(), level + 1));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

fn optional_tooltip_expr(tooltip: Option<&ButtonTooltip>, level: usize) -> String {
    let Some(tooltip) = tooltip else {
        return "null".into();
    };
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::new();
    out.push_str("ButtonTooltip(\n");
    out.push_str(&ii);
    out.push_str("title = ");
    out.push_str(&kt_string(&tooltip.title));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("lines = ");
    out.push_str(&string_list_expr(&tooltip.lines));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn optional_hud_shader_entry_expr(shader: Option<&HudShaderEntry>, level: usize) -> String {
    let Some(shader) = shader else {
        return "null".into();
    };
    let i = indent(level);
    let ii = indent(level + 1);
    let mut out = String::new();
    out.push_str("HudShaderEntry(\n");
    out.push_str(&ii);
    out.push_str("staticMarker = ");
    out.push_str(&kt_string(&shader.static_marker));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("sourceBottom = ");
    out.push_str(&shader.source_bottom.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("originX = ");
    out.push_str(&float_expr(shader.origin_x));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("originY = ");
    out.push_str(&float_expr(shader.origin_y));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("anchorX = ");
    out.push_str(&float_expr(shader.anchor_x));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("anchorY = ");
    out.push_str(&float_expr(shader.anchor_y));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("offsetX = ");
    out.push_str(&shader.offset_x.to_string());
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("offsetY = ");
    out.push_str(&shader.offset_y.to_string());
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn font_metrics_expr(
    metrics: &FontMetricsEntry,
    advances_expr: &str,
    glyph_widths_expr: &str,
    level: usize,
) -> String {
    let i = indent(level);
    let ii = indent(level + 1);
    let advances = if metrics.advances.is_empty() {
        "emptyMap()".to_string()
    } else if metrics.advances == crate::vanilla::advances().collect() {
        advances_expr.to_string()
    } else {
        let mut out = String::new();
        render_char_int_map(&mut out, &metrics.advances, level + 1);
        trim_trailing_newline(out)
    };
    let glyph_widths = if metrics.glyph_widths.is_empty() {
        "emptyMap()".to_string()
    } else if metrics.glyph_widths == crate::vanilla::glyph_widths().collect() {
        glyph_widths_expr.to_string()
    } else {
        let mut out = String::new();
        render_char_int_map(&mut out, &metrics.glyph_widths, level + 1);
        trim_trailing_newline(out)
    };
    format!(
        "FontMetricsEntry(\n{ii}advances = {advances},\n{ii}glyphWidths = {glyph_widths},\n{ii}boldAdvance = {},\n{i})",
        metrics.bold_advance
    )
}

pub(super) fn render_int_int_map(out: &mut String, values: &BTreeMap<u32, i32>, level: usize) {
    let i = indent(level);
    let ii = indent(level + 1);
    if values.is_empty() {
        out.push_str(&i);
        out.push_str("emptyMap()\n");
        return;
    }
    out.push_str(&i);
    out.push_str("mapOf(\n");
    for (key, value) in values {
        out.push_str(&ii);
        out.push_str(&key.to_string());
        out.push_str(" to ");
        out.push_str(&value.to_string());
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push_str(")\n");
}

pub(super) fn render_char_int_map(out: &mut String, values: &BTreeMap<char, u32>, level: usize) {
    let i = indent(level);
    let ii = indent(level + 1);
    if values.is_empty() {
        out.push_str(&i);
        out.push_str("emptyMap()\n");
        return;
    }
    out.push_str(&i);
    out.push_str("mapOf(\n");
    for (key, value) in values {
        out.push_str(&ii);
        out.push_str(&kt_string(&key.to_string()));
        out.push_str(" to ");
        out.push_str(&value.to_string());
        out.push_str(",\n");
    }
    out.push_str(&i);
    out.push_str(")\n");
}

fn align_expr(align: Align) -> &'static str {
    match align {
        Align::Left => "Align.LEFT",
        Align::Center => "Align.CENTER",
        Align::Right => "Align.RIGHT",
    }
}

fn optional_button_default_expr(default: Option<ButtonDefault>) -> &'static str {
    match default {
        Some(ButtonDefault::Close) => "ButtonDefault.CLOSE",
        None => "null",
    }
}

fn optional_string_expr(value: Option<&String>) -> String {
    value.map_or_else(|| "null".into(), |value| kt_string(value))
}

fn string_list_expr(values: &[String]) -> String {
    if values.is_empty() {
        return "emptyList()".into();
    }
    let body = values.iter().map(|value| kt_string(value)).collect::<Vec<_>>().join(", ");
    format!("listOf({body})")
}

fn bool_expr(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn float_expr(value: f32) -> String {
    let mut out = value.to_string();
    if !out.contains('.') && !out.contains('e') && !out.contains('E') {
        out.push_str(".0");
    }
    out
}

pub(super) fn kt_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_ascii_graphic() || ch == ' ' => out.push(ch),
            ch => {
                let mut units = [0u16; 2];
                for unit in ch.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04X}"));
                }
            }
        }
    }
    out.push('"');
    out
}

pub(super) fn indent(level: usize) -> String {
    INDENT.repeat(level)
}

fn trim_trailing_newline(mut value: String) -> String {
    if value.ends_with('\n') {
        value.pop();
    }
    value
}
