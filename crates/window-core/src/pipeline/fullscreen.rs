//! Per-full-screen-window compilation and the full-screen canvas contract: its include and built-in glyphs.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::authoring::HudGroup;
use crate::compose::{Composite, Texture};
use crate::font::bitmap_provider;
use crate::fullscreen::{
    self, BACKDROP_LAYER, CURSOR_LAYER, MAX_COLORS, MAX_CONTENT_LAYER, Marker, MarkerKind, OVERLAY_LAYER, STATIC_LAYER,
    TOOLTIP_LAYER, TOOLTIP_TEXT_LAYER,
};
use crate::geometry::{Point, Rect, Size};
use crate::ir::{FullscreenMode, LaidOutFullscreen, Layer, RegionIr, Rgb, tint_palette, tint_palettes};
use crate::manifest::{
    ControlEntry, ControlTooltipEntry, CursorEntry, FullscreenCanvasEntry, FullscreenEntry, FullscreenSurfaceEntry,
    SlotEntry,
};
use crate::size::{Pane, PaneKind, Part};
use crate::{Error, Result, bake};

use super::fonts::to_json;
use super::glyphs::{FULLSCREEN_BACKDROP_KEY, FULLSCREEN_CURSOR_KEY, Layers, fullscreen_key_prefix};
use super::metrics::bitmap_metrics;
use super::window::slot_entry;
use super::{CompileContext, OutputFile};

mod horse;

const WHITE: Rgb = Rgb::new(0xff, 0xff, 0xff);

/// The item model, under the pack namespace, that draws nothing: the runtime fills the hotbar with it while a
/// full-screen window is open, which also hides the hand.
const BLANK_ITEM: &str = "fullscreen/blank";

/// What the full-screen windows compiled so far put in the include: the color table and every control rect.
pub(super) struct Contract {
    colors: Vec<Rgb>,
    controls: Vec<Rect>,
    /// Whether a horse-mode window was compiled.
    horse: bool,
}

impl Default for Contract {
    fn default() -> Self {
        Self { colors: vec![WHITE], controls: Vec::new(), horse: false }
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
    contract.horse |= f.mode == FullscreenMode::Horse;
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

    let mut outline_texts = BTreeMap::new();
    for (index, key, outline) in outlines(f) {
        let file = format!("{file_base}_hover_{index}");
        let text = ctx.bake_composite(&outline, origin, pane, &file, key, Part::Hover)?;
        outline_texts.insert(index, text);
    }
    let mut tooltips = BTreeMap::new();
    if !f.tooltips.is_empty() {
        let marker = contract.marker(MarkerKind::Content, TOOLTIP_LAYER, WHITE)?;
        let mut panels: HashMap<Rect, String> = HashMap::new();
        for (tooltip, (key, panel)) in f.tooltips.iter().zip(tooltip_panels(f)) {
            let panel = match panels.entry(tooltip.panel) {
                Entry::Occupied(baked) => baked.get().clone(),
                Entry::Vacant(entry) => {
                    let Rect { x, y, width, height } = tooltip.panel;
                    let file = format!("{file_base}_tooltip_{x}_{y}_{width}x{height}");
                    entry.insert(ctx.bake_composite(&panel, origin, pane, &file, key, Part::Hover)?).clone()
                }
            };
            let mut lines = Vec::with_capacity(tooltip.lines.len());
            for line in &tooltip.lines {
                let mut entry = slot_entry(ctx, line, line.rect.y, None)?;
                entry.shader_marker = Some(contract.marker(MarkerKind::Content, TOOLTIP_TEXT_LAYER, line.color)?);
                lines.push(entry);
            }
            tooltips.insert(tooltip.region, ControlTooltipEntry { panel, marker: marker.clone(), lines });
        }
    }
    let mut controls = BTreeMap::new();
    for (index, region) in f.regions.iter().enumerate() {
        let rect = region.rect;
        let entry = ControlEntry {
            index: contract.controls.len() as u32,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            action: region.action.clone(),
            default_action: region.default_action.clone(),
            hover: outlined(region),
            outline: outline_texts.remove(&index).filter(|text| !text.is_empty()),
            tooltip: tooltips.remove(&index),
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

/// Emits the built-in cursor and backdrop glyphs, the include, and with a horse-mode window the horse's files, and
/// returns the canvas entry.
pub(super) fn emit_contract(ctx: &mut CompileContext<'_>, mut contract: Contract) -> Result<FullscreenCanvasEntry> {
    let cursor_texture = fullscreen::cursor_texture();
    let cursor_size = Size::new(cursor_texture.width, cursor_texture.height);
    let cursor = builtin_glyph(ctx, FULLSCREEN_CURSOR_KEY, "cursor", &cursor_texture)?;
    let backdrop = builtin_glyph(ctx, FULLSCREEN_BACKDROP_KEY, "backdrop", &fullscreen::backdrop_texture())?;
    let cursor_marker = contract.marker(MarkerKind::Cursor, CURSOR_LAYER, WHITE)?;
    let overlay_marker = contract.marker(MarkerKind::Content, OVERLAY_LAYER, WHITE)?;
    let include = fullscreen::include(&contract.colors, &contract.controls, cursor_size);
    ctx.push(Part::Fullscreen, OutputFile::text(fullscreen::INCLUDE_PATH, include));
    let blank = to_json(&serde_json::json!({ "model": { "type": "minecraft:empty" } }))?;
    ctx.push(Part::Fullscreen, OutputFile::text(format!("assets/{}/items/{BLANK_ITEM}.json", ctx.namespace), blank));
    let horse = if contract.horse { Some(horse::emit(ctx, &cursor_texture)?) } else { None };
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
        prompt_world_age: fullscreen::PROMPT_WORLD_AGE,
        hud_strip_height: fullscreen::HUD_STRIP_HEIGHT,
        hud_strip_half_width: fullscreen::HUD_STRIP_HALF_WIDTH,
        hud_max_health: fullscreen::HUD_MAX_HEALTH,
        horse,
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

/// Whether `region` is outlined while hovered: as it says, else when it takes clicks.
fn outlined(region: &RegionIr) -> bool {
    region.hover.unwrap_or(region.action.is_some())
}

/// The tooltip panel of each tooltip of `f`, in order, with its codepoint allocation key, which panels with the same
/// rect share: a vanilla-styled box with cut corners, a purple border, and a dark translucent fill.
pub(super) fn tooltip_panels(f: &LaidOutFullscreen) -> impl Iterator<Item = (String, Composite)> + '_ {
    let prefix = fullscreen_key_prefix(&f.name);
    f.tooltips.iter().map(move |tooltip| {
        let Rect { width, height, .. } = tooltip.panel;
        let mut texture = Texture::transparent(width, height);
        for y in 0..height {
            let shade = |top: u8, bottom: u8| {
                let t = f64::from(y) / f64::from(height.max(2) - 1);
                (f64::from(top) + (f64::from(bottom) - f64::from(top)) * t).round() as u8
            };
            for x in 0..width {
                let rim = x == 0 || y == 0 || x == width - 1 || y == height - 1;
                let corner = (x == 0 || x == width - 1) && (y == 0 || y == height - 1);
                let border = !rim && (x == 1 || y == 1 || x == width - 2 || y == height - 2);
                if corner {
                    continue;
                }
                let rgba =
                    if border { [shade(0x24, 0x18), 0x00, shade(0x5b, 0x33), 0xf0] } else { [0x10, 0x00, 0x10, 0xf0] };
                let at = ((y * width + x) * 4) as usize;
                texture.rgba[at..at + 4].copy_from_slice(&rgba);
            }
        }
        let Rect { x, y, .. } = tooltip.panel;
        let key = format!("{prefix}/tooltip/{x}_{y}_{width}x{height}");
        (key, Composite { texture, bounds: tooltip.panel, has_content: true })
    })
}

/// The hover outline of each outlined region of `f`, with the region's index and its codepoint allocation key: a 1px
/// white border just inside the region's rect.
pub(super) fn outlines(f: &LaidOutFullscreen) -> impl Iterator<Item = (usize, String, Composite)> + '_ {
    let prefix = fullscreen_key_prefix(&f.name);
    f.regions.iter().enumerate().filter(|(_, region)| outlined(region)).map(move |(index, region)| {
        let Rect { width, height, .. } = region.rect;
        let mut texture = Texture::transparent(width, height);
        for y in 0..height {
            for x in 0..width {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    let at = ((y * width + x) * 4) as usize;
                    texture.rgba[at..at + 4].fill(0xff);
                }
            }
        }
        let has_content = width > 0 && height > 0;
        (index, format!("{prefix}/hover/{index}"), Composite { texture, bounds: region.rect, has_content })
    })
}
