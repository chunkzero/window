//! Per-full-screen-window compilation and the full-screen canvas contract: its include and built-in glyphs.

use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::HudGroup;
use crate::compose::Texture;
use crate::font::bitmap_provider;
use crate::fullscreen::{
    self, BACKDROP_LAYER, CURSOR_LAYER, MAX_COLORS, MAX_CONTENT_LAYER, Marker, MarkerKind, OVERLAY_LAYER, STATIC_LAYER,
};
use crate::geometry::{Point, Rect, Size};
use crate::ir::{LaidOutFullscreen, Layer, Rgb, tint_palette, tint_palettes};
use crate::manifest::{
    ControlEntry, CursorEntry, FullscreenCanvasEntry, FullscreenEntry, FullscreenSurfaceEntry, SlotEntry,
};
use crate::size::{Pane, PaneKind, Part};
use crate::{Error, Result, bake};

use super::glyphs::{FULLSCREEN_BACKDROP_KEY, FULLSCREEN_CURSOR_KEY, Layers, fullscreen_key_prefix};
use super::metrics::bitmap_metrics;
use super::window::slot_entry;
use super::{CompileContext, OutputFile};

const WHITE: Rgb = Rgb::new(0xff, 0xff, 0xff);

/// What the full-screen windows compiled so far put in the include: the color table and every control rect.
pub(super) struct Contract {
    colors: Vec<Rgb>,
    controls: Vec<Rect>,
}

impl Default for Contract {
    fn default() -> Self {
        Self { colors: vec![WHITE], controls: Vec::new() }
    }
}

impl Contract {
    /// The marker color of a `kind` run at `layer` drawn in `color`, adding `color` to the color table.
    fn marker(&mut self, kind: MarkerKind, layer: u8, color: Rgb) -> Result<String> {
        let index = match self.colors.iter().position(|known| *known == color) {
            Some(index) => index,
            None => {
                if self.colors.len() == MAX_COLORS {
                    return Err(Error::Validation(format!(
                        "full-screen windows use more than {MAX_COLORS} distinct text, tint, and backdrop colors"
                    )));
                }
                self.colors.push(color);
                self.colors.len() - 1
            }
        };
        Ok(Marker { kind, layer, color: index as u16 }.rgb().to_hex())
    }
}

pub(super) fn compile_fullscreen(
    ctx: &mut CompileContext<'_>,
    f: &LaidOutFullscreen,
    layers: &Layers,
    contract: &mut Contract,
) -> Result<FullscreenEntry> {
    ctx.warnings.extend(f.warnings.iter().cloned());
    let pane = Pane { kind: PaneKind::Fullscreen, name: &f.name };
    let origin = Point::new(0, 0);
    let file_base = format!("fullscreen/window/{}", f.name);
    let prefix = fullscreen_key_prefix(&f.name);
    let base = &layers.base;
    let static_text = if base.has_content {
        let key = format!("{prefix}/static");
        ctx.bake_composite(base, origin, pane, &file_base, key, Part::static_art(pane))?
    } else {
        bake::bake_empty()
    };
    let switches = ctx.switch_entries(pane, &prefix, &file_base, &f.switches, &layers.cases, origin)?;
    let mut art = ctx.art_entries(pane, &prefix, &file_base, &f.art, &layers.art, origin)?;
    let mut slots = BTreeMap::new();
    for slot in &f.slots {
        if slot.shadow {
            return Err(Error::Validation(format!(
                "fullscreen `{}` slot `{}` sets `shadow`, but full-screen text is drawn without shadows: a shadow \
                 recolors the glyph and breaks marker detection",
                f.name, slot.name
            )));
        }
        slots.insert(slot.name.clone(), slot_entry(ctx, slot, slot.rect.y, None)?);
    }

    let most = usize::from(MAX_CONTENT_LAYER - STATIC_LAYER);
    if f.layers.len() > most {
        return Err(Error::Validation(format!(
            "fullscreen `{}` has {} runtime layers (text, switches, and art layers), but at most {most} fit a marker",
            f.name,
            f.layers.len()
        )));
    }
    let static_marker = contract.marker(MarkerKind::Content, STATIC_LAYER, WHITE)?;
    let mut markers = BTreeMap::new();
    for (index, layer) in f.layers.iter().enumerate() {
        let depth = STATIC_LAYER + 1 + index as u8;
        let (name, color, tint) = match layer {
            Layer::Slot(name) => {
                let slot = f.slots.iter().find(|slot| &slot.name == name).expect("a slot layer has a slot");
                (name, slot.color, slot.tint.as_deref())
            }
            Layer::Art(name) => {
                let ir = f.art.iter().find(|art| &art.name == name).expect("an art layer has art");
                (name, WHITE, ir.tint.as_deref())
            }
            Layer::Switch(name) | Layer::SpriteSlot(name) | Layer::Collection(name) => (name, WHITE, None),
        };
        markers.insert(name.clone(), contract.marker(MarkerKind::Content, depth, color)?);
        let Some(tint) = tint else {
            continue;
        };
        let tint_markers = match layer {
            Layer::Slot(_) => slots.get_mut(name).map(|slot: &mut SlotEntry| &mut slot.tint_markers),
            _ => art.get_mut(name).map(|art| &mut art.tint_markers),
        };
        let tint_markers = tint_markers.expect("a tinted layer has an entry");
        for (value, color) in tint_palette(&f.handles, tint) {
            tint_markers.insert(value, contract.marker(MarkerKind::Content, depth, color)?);
        }
    }

    let mut controls = BTreeMap::new();
    for region in &f.regions {
        let rect = region.rect;
        let entry = ControlEntry {
            index: contract.controls.len() as u32,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            action: region.action.clone(),
            default_action: region.default_action.clone(),
            hover: region.hover.unwrap_or(region.action.is_some()),
            source: Some(region.source.clone()),
        };
        contract.controls.push(rect);
        controls.insert(region.name.clone(), entry);
    }
    let backdrop_marker =
        f.backdrop.map(|color| contract.marker(MarkerKind::Backdrop, BACKDROP_LAYER, color)).transpose()?;

    Ok(FullscreenEntry {
        surface: FullscreenSurfaceEntry {
            kind: "fullscreen".into(),
            mode: f.mode,
            size: [fullscreen::CANVAS.width, fullscreen::CANVAS.height],
        },
        static_text,
        static_marker,
        slots,
        controls,
        switches,
        art,
        tints: tint_palettes(&f.handles),
        layers: f.layers.clone(),
        markers,
        backdrop_marker,
        handles: f.handles.clone(),
    })
}

/// Emits the built-in cursor and backdrop glyphs and the include, and returns the canvas entry.
pub(super) fn emit_contract(ctx: &mut CompileContext<'_>, mut contract: Contract) -> Result<FullscreenCanvasEntry> {
    let cursor_texture = fullscreen::cursor_texture();
    let cursor_size = Size::new(cursor_texture.width, cursor_texture.height);
    let cursor = builtin_glyph(ctx, FULLSCREEN_CURSOR_KEY, "cursor", &cursor_texture)?;
    let backdrop = builtin_glyph(ctx, FULLSCREEN_BACKDROP_KEY, "backdrop", &fullscreen::backdrop_texture())?;
    let cursor_marker = contract.marker(MarkerKind::Cursor, CURSOR_LAYER, WHITE)?;
    let overlay_marker = contract.marker(MarkerKind::Content, OVERLAY_LAYER, WHITE)?;
    let include = fullscreen::include(&contract.colors, &contract.controls, cursor_size);
    ctx.push(Part::Fullscreen, OutputFile::text(fullscreen::INCLUDE_PATH, include));
    Ok(FullscreenCanvasEntry {
        width: fullscreen::CANVAS.width,
        height: fullscreen::CANVAS.height,
        degrees_per_pixel: fullscreen::DEGREES_PER_PIXEL,
        edge_yaw: fullscreen::EDGE_YAW,
        edge_pitch: fullscreen::EDGE_PITCH,
        snap_epsilon: fullscreen::SNAP_EPSILON,
        hit_margin: fullscreen::HIT_MARGIN,
        display_scale: fullscreen::DISPLAY_SCALE,
        marker_alpha: u32::from(fullscreen::MARKER_ALPHA),
        cursor: CursorEntry {
            text: cursor,
            marker: cursor_marker,
            width: cursor_size.width,
            height: cursor_size.height,
        },
        backdrop,
        overlay_marker,
        hud_hidden_world_age: fullscreen::HUD_HIDDEN_WORLD_AGE,
        hud_strip_height: fullscreen::HUD_STRIP_HEIGHT,
        hud_strip_half_width: fullscreen::HUD_STRIP_HALF_WIDTH,
        hud_max_health: fullscreen::HUD_MAX_HEALTH,
    })
}

/// Emits the crosshair sprite override the HUD-hiding fragment shader discards, unless `vanilla.hud` already blanks
/// the crosshair.
pub(super) fn emit_crosshair_marker(ctx: &mut CompileContext<'_>, blank_hud: &BTreeSet<HudGroup>) -> Result<()> {
    if !blank_hud.contains(&HudGroup::Crosshair) {
        let texture = fullscreen::crosshair_marker_texture();
        ctx.push(Part::Fullscreen, OutputFile::binary(fullscreen::CROSSHAIR_SPRITE_PATH, texture.encode_png()?));
    }
    Ok(())
}

/// Emits `texture` as the main-font glyph of `key`, stored as the built-in texture `name` apart from authored
/// full-screen textures, with its top on the line's top, and returns its net-zero text.
fn builtin_glyph(ctx: &mut CompileContext<'_>, key: &str, name: &str, texture: &Texture) -> Result<String> {
    let glyph = char::from_u32(ctx.codepoints[key]).expect("glyph codepoint is a valid char");
    let file = format!("fullscreen/builtin/{name}");
    ctx.bitmap_providers.push(bitmap_provider(ctx.namespace, &file, texture.height, 7, glyph)?);
    let path = format!("assets/{}/textures/font/{file}.png", ctx.namespace);
    ctx.push(Part::Fullscreen, OutputFile::binary(path, texture.encode_png()?));
    let advance = bitmap_metrics(texture, Size::new(texture.width, texture.height)).advance;
    let bounds = Rect::new(0, 0, texture.width, texture.height);
    Ok(bake::bake_static_with_advance(glyph, &bounds, Point::new(0, 0), advance))
}
