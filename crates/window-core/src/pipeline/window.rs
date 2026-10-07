//! Per-window compilation: static bake, text slots, and sprite slots.

use std::collections::BTreeMap;

use crate::compose::Composite;
use crate::geometry::{Point, Rect};
use crate::ir::{Align, LaidOutWindow, Rgb, SlotIr};
use crate::manifest::{SlotEntry, SlotLinesEntry, SpriteSlotEntry, SurfaceEntry, TextOverflow, WindowEntry};
use crate::surface::Surface;
use crate::{Result, bake};

use super::CompileContext;
use super::glyphs::{Layers, static_key, window_key_prefix};
use super::inventory::{compile_inventory, repeat_groups};
use super::sprites::check_sprite_fits;

pub(super) fn compile_window(ctx: &mut CompileContext<'_>, w: &LaidOutWindow, layers: &Layers) -> Result<WindowEntry> {
    ctx.warnings.extend(w.warnings.iter().cloned());
    let title_origin = w.surface.title_origin();
    let static_text = bake_static(ctx, w, &layers.base, title_origin)?;
    let switches =
        ctx.switch_entries(&w.name, &window_key_prefix(&w.name), &w.name, &w.switches, &layers.cases, title_origin)?;

    let mut slots = BTreeMap::new();
    for slot in &w.slots {
        let entry = slot_entry(ctx, slot, slot.rect.y - title_origin.y, None)?;
        slots.insert(slot.name.clone(), entry);
    }
    let sprite_slots = sprite_slots(ctx, w, title_origin.y)?;
    let inventory = compile_inventory(ctx, w, title_origin.y)?;

    Ok(WindowEntry {
        surface: surface_entry(&w.surface),
        static_text,
        slots,
        sprite_slots,
        buttons: inventory.buttons,
        items: inventory.items,
        collections: inventory.collections,
        inputs: inventory.inputs,
        slot_rects: inventory.slot_rects,
        groups: repeat_groups(w),
        switches,
        indexed: w.indexed.clone(),
    })
}

fn bake_static(
    ctx: &mut CompileContext<'_>,
    w: &LaidOutWindow,
    comp: &Composite,
    title_origin: Point,
) -> Result<String> {
    if !comp.has_content {
        return Ok(bake::bake_empty());
    }
    // Ascent places the composite's top edge: A = title_y + 7 - top_y.
    let ascent = title_origin.y + 7 - comp.bounds.y;
    let glyph = ctx.emit_static_glyph(comp, ascent, &w.name, &w.name, static_key(&w.name))?;
    Ok(bake::bake_static_with_advance(glyph.glyph, &comp.bounds, title_origin, glyph.advance))
}

fn sprite_slots(
    ctx: &mut CompileContext<'_>,
    w: &LaidOutWindow,
    title_y: i32,
) -> Result<BTreeMap<String, SpriteSlotEntry>> {
    let mut entries = BTreeMap::new();
    for sprite_slot in &w.sprite_slots {
        let k = sprite_slot.rect.y - title_y;
        let font = ctx.sprite_font(k);
        let entry = sprite_slot_entry(font, &sprite_slot.rect, sprite_slot.align, sprite_slot.sprite.clone());
        entries.insert(sprite_slot.name.clone(), SpriteSlotEntry { binding: sprite_slot.binding.clone(), ..entry });
        if let Some(sprite) = &sprite_slot.sprite {
            let subject = format!("sprite slot `{}`", sprite_slot.name);
            check_sprite_fits(ctx.runtime_sprites, &w.name, &subject, sprite, &sprite_slot.rect)?;
        }
    }
    Ok(entries)
}

/// Build a [`SlotEntry`] for `slot`, whose box top lies `k` pixels below the font baseline origin, registering the
/// shifted font of every position a line can take.
pub(super) fn slot_entry(
    ctx: &mut CompileContext<'_>,
    slot: &SlotIr,
    k: i32,
    shader_marker: Option<Rgb>,
) -> Result<SlotEntry> {
    let fit = slot.fit;
    let lines = if fit.lines > 1 {
        let steps = 0..(2 * fit.lines - 1) as i32;
        let fonts = steps.map(|s| ctx.text_font(slot, k + s * fit.line_height as i32 / 2)).collect::<Result<_>>()?;
        Some(SlotLinesEntry { count: fit.lines, line_height: fit.line_height, fonts })
    } else {
        None
    };
    Ok(SlotEntry {
        x: slot.rect.x,
        y: slot.rect.y,
        width: slot.rect.width,
        align: slot.align,
        font: ctx.text_font(slot, k)?,
        color: slot.color.to_hex(),
        shader_marker: shader_marker.map(Rgb::to_hex),
        shader_color: None,
        shadow: slot.shadow,
        bold: slot.bold,
        italic: slot.italic,
        underlined: slot.underlined,
        strikethrough: slot.strikethrough,
        obfuscated: slot.obfuscated,
        text: slot.text.clone(),
        binding: slot.binding.clone(),
        overflow: fit.ellipsis.then_some(TextOverflow::Ellipsis),
        lines,
    })
}

/// Build a [`SpriteSlotEntry`] covering `rect` drawn with `font`.
pub(super) fn sprite_slot_entry(font: String, rect: &Rect, align: Align, sprite: Option<String>) -> SpriteSlotEntry {
    SpriteSlotEntry { x: rect.x, y: rect.y, width: rect.width, height: rect.height, align, font, sprite, binding: None }
}

fn surface_entry(surface: &Surface) -> SurfaceEntry {
    let Surface::Container(kind) = surface;
    let size = surface.gui_size();
    let origin = surface.title_origin();
    SurfaceEntry {
        kind: "container".into(),
        container: kind.id().to_string(),
        size: [size.width, size.height],
        title_origin: [origin.x, origin.y],
    }
}
