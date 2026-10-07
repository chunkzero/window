//! Per-HUD compilation and generated HUD shader files.

use std::collections::BTreeMap;

use crate::compose::Composite;
use crate::geometry::Point;
use crate::hud::SegmentMarkers;
use crate::ir::{HudShader, LaidOutHud, Rgb};
use crate::manifest::{HudEntry, HudShaderEntry, HudSurfaceEntry};
use crate::{Result, bake};

use super::glyphs::{Layers, hud_key_prefix, hud_static_key};
use super::window::slot_entry;
use super::{CompileContext, OutputFile};

pub(super) fn compile_hud(
    ctx: &mut CompileContext<'_>,
    h: &LaidOutHud,
    layers: &Layers,
    markers: &SegmentMarkers,
) -> Result<HudEntry> {
    ctx.warnings.extend(h.warnings.iter().cloned());
    let static_text = bake_static(ctx, h, &layers.base)?;
    let file_base = format!("hud_{}", h.name);
    let switches = ctx.switch_entries(
        &h.name,
        &hud_key_prefix(&h.name),
        &file_base,
        &h.switches,
        &layers.cases,
        Point::new(0, 0),
    )?;

    let mut slots = BTreeMap::new();
    for slot in &h.slots {
        let marker = h.shader.map(|_| markers.slot_marker(&h.name, &slot.name));
        let entry = slot_entry(ctx, slot, slot.rect.y, marker)?;
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
        shader: h.shader.map(|shader| shader_entry(shader, markers.static_marker(&h.name))),
        switches,
        indexed: h.indexed.clone(),
    })
}

/// Shader HUDs bake at the origin; fallback HUDs pad to their fixed width.
fn bake_static(ctx: &mut CompileContext<'_>, h: &LaidOutHud, comp: &Composite) -> Result<String> {
    if !comp.has_content {
        return Ok(if h.shader.is_some() { bake::bake_empty() } else { bake::bake_fixed_width_empty(h.width) });
    }
    let ascent = 7 - comp.bounds.y;
    let file_stem = format!("hud_{}", h.name);
    let glyph = ctx.emit_static_glyph(comp, ascent, &h.name, &file_stem, hud_static_key(&h.name))?;
    Ok(if h.shader.is_some() {
        bake::bake_static_with_advance(glyph.glyph, &comp.bounds, Point::new(0, 0), glyph.advance)
    } else {
        bake::bake_fixed_width_static_with_advance(glyph.glyph, &comp.bounds, h.width, glyph.advance)
    })
}

fn shader_entry(shader: HudShader, static_marker: Rgb) -> HudShaderEntry {
    HudShaderEntry {
        static_marker: static_marker.to_hex(),
        source_bottom: shader.source_bottom,
        origin_x: shader.origin_x,
        origin_y: shader.origin_y,
        anchor_x: shader.anchor_x,
        anchor_y: shader.anchor_y,
        offset_x: shader.offset_x,
        offset_y: shader.offset_y,
    }
}

pub(super) fn emit_shader_files(
    ctx: &mut CompileContext<'_>,
    pack_format: Option<u32>,
    huds: &[&LaidOutHud],
) -> Result<()> {
    let output = crate::hud::emit(pack_format, huds)?;
    ctx.files.extend(output.files.into_iter().map(|file| OutputFile::text(file.path, file.contents)));
    ctx.warnings.extend(output.warnings);
    Ok(())
}
