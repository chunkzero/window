//! Per-HUD compilation and generated HUD shader files.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::Composite;
use crate::geometry::Point;
use crate::hud::SegmentMarkers;
use crate::ir::{HudShader, LaidOutHud, Rgb};
use crate::manifest::{HudEntry, HudShaderEntry, HudSurfaceEntry};
use crate::size::{Pane, Part};
use crate::{Result, bake};

use super::glyphs::{Layers, MAX_GLYPH_SIZE, hud_key_prefix, hud_static_key};
use super::window::{slot_entry, slot_rows};
use super::{CompileContext, OutputFile};

/// The highest row a text-row font can draw at: its ascent `7 - row` may not exceed the 8 px glyph height.
const MIN_TEXT_ROW: i32 = -1;

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
    let switches =
        ctx.switch_entries(pane(h), &hud_key_prefix(&h.name), &file_base, &h.switches, &layers.cases, origin)?;

    let mut slots = BTreeMap::new();
    for slot in &h.slots {
        let marker = h.shader.map(|_| markers.slot_marker(&h.name, &slot.name));
        let entry = slot_entry(ctx, slot, slot.rect.y - origin.y, marker)?;
        slots.insert(slot.name.clone(), entry);
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
        layers: h.layers.clone(),
        handles: h.handles.clone(),
    })
}

fn pane(h: &LaidOutHud) -> Pane<'_> {
    Pane { hud: true, name: &h.name }
}

/// The rows a shader-placed HUD is baked below its source origin, which its shader rule subtracts again, so the
/// offset moves nothing on screen. It is the offset whose text rows need the fewest text-row fonts that are not
/// registered yet, and the smallest such offset; `0` for any other HUD.
fn row_offset(ctx: &CompileContext<'_>, h: &LaidOutHud, layers: &Layers) -> i32 {
    if h.shader.is_none() {
        return 0;
    }
    let rows: BTreeSet<(Option<&str>, i32)> = h
        .slots
        .iter()
        .flat_map(|slot| slot_rows(slot, slot.rect.y).map(move |row| (slot.font.as_deref(), row)))
        .collect();
    let Some(top) = rows.iter().map(|&(_, row)| row).min() else {
        return 0;
    };
    let registered: BTreeSet<(Option<&str>, i32)> = ctx
        .shift_offsets
        .iter()
        .map(|&row| (None, row))
        .chain(ctx.text_font_offsets.iter().map(|&(font, row)| (Some(font), row)))
        .collect();

    // Text may not rise above `MIN_TEXT_ROW`, and art may not be padded past the glyph size limit.
    let art = std::iter::once(&layers.base).chain(layers.cases.iter().flatten()).filter(|comp| comp.has_content);
    let lowest = art
        .map(|comp| 7 - comp.bounds.y - MAX_GLYPH_SIZE as i32)
        .chain(std::iter::once(MIN_TEXT_ROW - top))
        .max()
        .unwrap_or(0);
    let new_fonts =
        |offset: i32| rows.iter().filter(|&&(font, row)| !registered.contains(&(font, row + offset))).count();
    rows.iter()
        .flat_map(|&(font, row)| registered.iter().filter(move |taken| taken.0 == font).map(move |taken| taken.1 - row))
        .chain(std::iter::once(0))
        .map(|offset| offset.max(lowest))
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
