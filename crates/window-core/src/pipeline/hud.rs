//! Per-HUD compilation and generated HUD shader files.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::Composite;
use crate::font::{MAX_TEXT_ROW, MIN_ASCENT, MIN_TEXT_ROW};
use crate::geometry::Point;
use crate::hud::{SegmentMarkers, TintedSegment, tinted_segments};
use crate::ir::{HudShader, LaidOutHud, Rgb, tint_palettes};
use crate::manifest::{HudEntry, HudShaderEntry, HudSurfaceEntry};
use crate::size::{Pane, Part};
use crate::{Result, bake};

use super::glyphs::{Layers, MAX_GLYPH_SIZE, hud_key_prefix, hud_static_key};
use super::window::{slot_entry, slot_rows};
use super::{CompileContext, OutputFile};

pub(super) fn compile_hud(
    ctx: &mut CompileContext<'_>,
    h: &LaidOutHud,
    layers: &Layers,
    markers: &SegmentMarkers,
) -> Result<HudEntry> {
    ctx.warnings.extend(h.warnings.iter().cloned());
    let row_offset = row_offset(ctx, h, layers);
    // The pane row the text baseline origin sits on.
    let origin = Point::new(0, -row_offset);
    let static_text = bake_static(ctx, h, &layers.base, origin)?;
    let file_base = format!("hud_{}", h.name);
    let prefix = hud_key_prefix(&h.name);
    let switches = ctx.switch_entries(pane(h), &prefix, &file_base, &h.switches, &layers.cases, origin)?;
    let mut art = ctx.art_entries(pane(h), &prefix, &file_base, &h.art, &layers.art, origin)?;

    let mut slots = BTreeMap::new();
    for slot in &h.slots {
        let marker = h.shader.map(|_| markers.slot_marker(&h.name, &slot.name));
        let entry = slot_entry(ctx, slot, slot.rect.y - origin.y, marker)?;
        slots.insert(slot.name.clone(), entry);
    }
    if h.shader.is_some() {
        for (segment, palette) in tinted_segments(h) {
            let tint_markers = match segment {
                TintedSegment::Art(name) => art.get_mut(name).map(|entry| &mut entry.tint_markers),
                TintedSegment::Slot(name) => slots.get_mut(name).map(|entry| &mut entry.tint_markers),
            };
            let tint_markers = tint_markers.expect("a tinted segment has an entry");
            for (value, _) in palette {
                let marker = markers.tint_marker(&h.name, &segment, &value);
                tint_markers.insert(value, marker.to_hex());
            }
        }
    }

    Ok(HudEntry {
        surface: HudSurfaceEntry {
            kind: "hud".into(),
            channel: h.channel.id().into(),
            width: h.width,
            height: h.height,
        },
        static_text,
        slots,
        shader: h.shader.map(|shader| shader_entry(shader, markers.static_marker(&h.name), row_offset)),
        switches,
        art,
        tints: tint_palettes(&h.handles),
        layers: h.layers.clone(),
        handles: h.handles.clone(),
    })
}

fn pane(h: &LaidOutHud) -> Pane<'_> {
    Pane { hud: true, name: &h.name }
}

/// The rows a shader-placed HUD is baked below its source origin, which its shader rule subtracts again, so the
/// offset moves nothing on screen. It is the offset whose text rows need the fewest text-row fonts that are not
/// registered yet, and the smallest such offset; `0` for any other HUD, and for a HUD with underlined, strikethrough
/// or italic text, which Minecraft positions from the unshifted baseline or the glyph ascent.
fn row_offset(ctx: &CompileContext<'_>, h: &LaidOutHud, layers: &Layers) -> i32 {
    if h.shader.is_none() || h.slots.iter().any(|slot| slot.italic || slot.underlined || slot.strikethrough) {
        return 0;
    }
    let rows: BTreeSet<(Option<&str>, i32)> = h
        .slots
        .iter()
        .flat_map(|slot| slot_rows(slot, slot.rect.y).map(move |row| (slot.font.as_deref(), row)))
        .collect();
    let (Some(top), Some(bottom)) = (rows.iter().map(|&(_, row)| row).min(), rows.iter().map(|&(_, row)| row).max())
    else {
        return 0;
    };
    let registered: BTreeSet<(Option<&str>, i32)> = ctx
        .shift_offsets
        .iter()
        .map(|&row| (None, row))
        .chain(ctx.text_font_offsets.iter().map(|&(font, row)| (Some(font), row)))
        .collect();

    // Text rows and art ascents must stay within what fonts and 256 px glyph pages accept.
    let (mut lowest, mut highest) = (MIN_TEXT_ROW - top, MAX_TEXT_ROW - bottom);
    for comp in layers.composites() {
        lowest = lowest.max(7 - comp.bounds.y - MAX_GLYPH_SIZE as i32);
        highest = highest.min(7 - comp.bounds.y - MIN_ASCENT);
    }
    if lowest > highest {
        return 0;
    }
    let new_fonts =
        |offset: i32| rows.iter().filter(|&&(font, row)| !registered.contains(&(font, row + offset))).count();
    rows.iter()
        .flat_map(|&(font, row)| registered.iter().filter(move |taken| taken.0 == font).map(move |taken| taken.1 - row))
        .chain(std::iter::once(0))
        .map(|offset| offset.clamp(lowest, highest))
        .min_by_key(|&offset| (new_fonts(offset), offset.abs(), offset))
        .unwrap_or(0)
}

/// Shader HUDs bake net-zero from `origin`; fallback HUDs pad to their fixed width.
fn bake_static(ctx: &mut CompileContext<'_>, h: &LaidOutHud, comp: &Composite, origin: Point) -> Result<String> {
    if !comp.has_content {
        return Ok(if h.shader.is_some() { bake::bake_empty() } else { bake::bake_fixed_width_empty(h.width) });
    }
    let ascent = origin.y + 7 - comp.bounds.y;
    let file_stem = format!("hud_{}", h.name);
    let glyph =
        ctx.emit_static_glyph(comp, ascent, pane(h), &file_stem, hud_static_key(&h.name), Part::static_art(pane(h)))?;
    Ok(if h.shader.is_some() {
        bake::bake_static_with_advance(glyph.glyph, &comp.bounds, origin, glyph.advance)
    } else {
        bake::bake_fixed_width_static_with_advance(glyph.glyph, &comp.bounds, h.width, glyph.advance)
    })
}

fn shader_entry(shader: HudShader, static_marker: Rgb, row_offset: i32) -> HudShaderEntry {
    HudShaderEntry {
        static_marker: static_marker.to_hex(),
        source_bottom: shader.source_bottom,
        row_offset,
        origin_x: shader.origin_x,
        origin_y: shader.origin_y,
        anchor_x: shader.anchor_x,
        anchor_y: shader.anchor_y,
        offset_x: shader.offset_x,
        offset_y: shader.offset_y,
    }
}

/// Emits the shaders that relocate `huds`, where `row_offsets` holds each HUD's baked row offset in the same order.
pub(super) fn emit_shader_files(
    ctx: &mut CompileContext<'_>,
    pack_format: Option<u32>,
    huds: &[&LaidOutHud],
    row_offsets: &[i32],
) -> Result<()> {
    let output = crate::hud::emit(pack_format, huds, row_offsets)?;
    for file in output.files {
        ctx.push(Part::Shader, OutputFile::text(file.path, file.contents));
    }
    ctx.warnings.extend(output.warnings);
    Ok(())
}
