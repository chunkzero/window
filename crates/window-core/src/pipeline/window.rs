//! Per-window compilation: static bake, text slots, and sprite slots.

use std::collections::BTreeMap;

use crate::compose::Composite;
use crate::font::shifted_suffix;
use crate::geometry::Point;
use crate::ir::{LaidOutWindow, Rgb, SlotIr, SpriteSlotIr};
use crate::manifest::{SlotEntry, SpriteSlotEntry, SurfaceEntry, WindowEntry};
use crate::surface::Surface;
use crate::{Result, bake};

use super::CompileContext;
use super::glyphs::static_key;
use super::inventory::{compile_inventory, repeat_groups};
use super::sprites::check_sprite_fits;

pub(super) fn compile_window(ctx: &mut CompileContext<'_>, w: &LaidOutWindow, comp: &Composite) -> Result<WindowEntry> {
    ctx.warnings.extend(w.warnings.iter().cloned());
    let title_origin = w.surface.title_origin();
    let static_text = bake_static(ctx, w, comp, title_origin)?;

    let mut slots = BTreeMap::new();
    for slot in &w.slots {
        let font = ctx.text_font(slot, slot.rect.y - title_origin.y)?;
        slots.insert(slot.name.clone(), slot_entry(slot, font, None));
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
        ctx.sprite_offsets.insert(k);
        entries.insert(sprite_slot.name.clone(), sprite_slot_entry(ctx.namespace, sprite_slot, k));
        if let Some(sprite) = &sprite_slot.sprite {
            let subject = format!("sprite slot `{}`", sprite_slot.name);
            check_sprite_fits(ctx.runtime_sprites, &w.name, &subject, sprite, &sprite_slot.rect)?;
        }
    }
    Ok(entries)
}

/// Build a [`SlotEntry`] for `slot` drawn with `font`.
pub(super) fn slot_entry(slot: &SlotIr, font: String, shader_marker: Option<Rgb>) -> SlotEntry {
    SlotEntry {
        x: slot.rect.x,
        y: slot.rect.y,
        width: slot.rect.width,
        align: slot.align,
        font,
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
    }
}

fn sprite_slot_entry(namespace: &str, slot: &SpriteSlotIr, k: i32) -> SpriteSlotEntry {
    SpriteSlotEntry {
        x: slot.rect.x,
        y: slot.rect.y,
        width: slot.rect.width,
        height: slot.rect.height,
        align: slot.align,
        font: format!("{namespace}:sprite_{}", shifted_suffix(k)),
        sprite: slot.sprite.clone(),
    }
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
