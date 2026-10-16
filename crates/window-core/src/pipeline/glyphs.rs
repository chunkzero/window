//! Static composites and stable glyph codepoint allocation.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::{Composite, RasterCache, Texture, compose_draws_cached};
use crate::font::{allocate, bitmap_provider};
use crate::geometry::{Point, Rect, Size};
use crate::ir::{ArtIr, Draw, LaidOutFullscreen, LaidOutHud, LaidOutWindow, SwitchIr};
use crate::size::{Pane, PaneKind, Part};
use crate::{Error, Result, bake};

use super::bars::clip_keys;
use super::metrics::bitmap_metrics;
use super::sprites::RuntimeSpriteAsset;
use super::{CompileContext, OutputFile};

/// A surface's static composite, one composite per switch case, indexed `[switch][case]`, and one per art layer.
pub(super) struct Layers {
    pub(super) base: Composite,
    pub(super) cases: Vec<Vec<Composite>>,
    pub(super) art: Vec<Composite>,
}

impl Layers {
    /// Every composite with content.
    pub(super) fn composites(&self) -> impl Iterator<Item = &Composite> {
        let all = std::iter::once(&self.base).chain(self.cases.iter().flatten()).chain(&self.art);
        all.filter(|comp| comp.has_content)
    }
}

/// Static layer composites, index-aligned with the name-sorted windows, HUDs, and full-screen windows.
pub(super) struct Composites {
    pub(super) windows: Vec<Layers>,
    pub(super) huds: Vec<Layers>,
    pub(super) fullscreens: Vec<Layers>,
}

pub(super) fn compose_layers(
    windows: &[&LaidOutWindow],
    huds: &[&LaidOutHud],
    fullscreens: &[&LaidOutFullscreen],
    textures: &BTreeMap<String, Texture>,
) -> Result<Composites> {
    let mut cache = RasterCache::default();
    let windows = windows
        .iter()
        .map(|w| layers(&w.name, &w.draws, &w.switches, &w.art, textures, &mut cache))
        .collect::<Result<_>>()?;
    let huds = huds
        .iter()
        .map(|h| layers(&h.name, &h.draws, &h.switches, &h.art, textures, &mut cache))
        .collect::<Result<_>>()?;
    let fullscreens = fullscreens
        .iter()
        .map(|f| layers(&f.name, &f.draws, &f.switches, &f.art, textures, &mut cache))
        .collect::<Result<_>>()?;
    Ok(Composites { windows, huds, fullscreens })
}

fn layers(
    name: &str,
    draws: &[Draw],
    switches: &[SwitchIr],
    art: &[ArtIr],
    textures: &BTreeMap<String, Texture>,
    cache: &mut RasterCache,
) -> Result<Layers> {
    let cases = switches
        .iter()
        .map(|switch| {
            switch.cases.iter().map(|case| compose_draws_cached(&case.draws, name, textures, cache)).collect()
        })
        .collect::<Result<_>>()?;
    let art = art.iter().map(|art| compose_draws_cached(&art.draws, name, textures, cache)).collect::<Result<_>>()?;
    Ok(Layers { base: compose_draws_cached(draws, name, textures, cache)?, cases, art })
}

/// Allocate codepoints across the whole build: one key per static, case, or art layer composite
/// with content, one per glyph of a bar's fill, one per runtime sprite when any window uses sprites, the hover
/// outline keys `hover`, and the built-in full-screen glyphs and control outlines and tooltip panels when there are
/// full-screen windows.
pub(super) fn allocate_codepoints(
    windows: &[&LaidOutWindow],
    huds: &[&LaidOutHud],
    fullscreens: &[&LaidOutFullscreen],
    composites: &Composites,
    runtime_sprites: Option<&BTreeMap<String, RuntimeSpriteAsset>>,
    hover: impl Iterator<Item = String>,
) -> Result<BTreeMap<String, u32>> {
    let mut keys: BTreeSet<String> = hover.collect();
    for (w, layers) in windows.iter().zip(&composites.windows) {
        layer_keys(&mut keys, &window_key_prefix(&w.name), &w.switches, &w.art, layers, false);
    }
    for (h, layers) in huds.iter().zip(&composites.huds) {
        layer_keys(&mut keys, &hud_key_prefix(&h.name), &h.switches, &h.art, layers, false);
    }
    for (f, layers) in fullscreens.iter().zip(&composites.fullscreens) {
        layer_keys(&mut keys, &fullscreen_key_prefix(&f.name), &f.switches, &f.art, layers, true);
        for (_, key, outline) in super::fullscreen::outlines(f) {
            composite_keys(&mut keys, key, &outline, true);
        }
        for (key, panel) in super::fullscreen::tooltip_panels(f) {
            composite_keys(&mut keys, key, &panel, true);
        }
    }
    if !fullscreens.is_empty() {
        keys.extend([FULLSCREEN_CURSOR_KEY.to_string(), FULLSCREEN_BACKDROP_KEY.to_string()]);
    }
    for sprite in runtime_sprites.into_iter().flat_map(BTreeMap::keys) {
        keys.insert(sprite_key(sprite));
    }
    allocate(&keys)
}

/// Inserts the keys of every composite with content; `tiled` composites get one key per [`tiles`] glyph.
fn layer_keys(
    keys: &mut BTreeSet<String>,
    prefix: &str,
    switches: &[SwitchIr],
    art: &[ArtIr],
    layers: &Layers,
    tiled: bool,
) {
    if layers.base.has_content {
        composite_keys(keys, format!("{prefix}/static"), &layers.base, tiled);
    }
    for (switch, cases) in switches.iter().zip(&layers.cases) {
        for (case, comp) in switch.cases.iter().zip(cases) {
            if comp.has_content {
                composite_keys(keys, case_key(prefix, &switch.name, &case.value), comp, tiled);
            }
        }
    }
    for (art, comp) in art.iter().zip(&layers.art) {
        let key = art_key(prefix, &art.name);
        if art.bar.is_some() {
            keys.extend(clip_keys(&key, &comp.texture));
        } else if comp.has_content {
            composite_keys(keys, key, comp, tiled);
        }
    }
}

fn composite_keys(keys: &mut BTreeSet<String>, key: String, comp: &Composite, tiled: bool) {
    if tiled {
        keys.extend(tiles(comp).iter().map(|tile| tile_key(&key, tile)));
    } else {
        keys.insert(key);
    }
}

pub(super) fn window_key_prefix(window: &str) -> String {
    format!("window/{window}")
}

pub(super) fn hud_key_prefix(hud: &str) -> String {
    format!("hud/{hud}")
}

pub(super) fn fullscreen_key_prefix(fullscreen: &str) -> String {
    format!("fullscreen/{fullscreen}")
}

/// The codepoint allocation key of the built-in full-screen cursor glyph.
pub(super) const FULLSCREEN_CURSOR_KEY: &str = "fullscreen/cursor";

/// The codepoint allocation key of the built-in full-screen backdrop glyph.
pub(super) const FULLSCREEN_BACKDROP_KEY: &str = "fullscreen/backdrop";

/// The codepoint allocation key for one switch case's composite under a surface key prefix.
pub(super) fn case_key(prefix: &str, switch: &str, case: &str) -> String {
    format!("{prefix}/switch/{switch}/{case}")
}

/// The codepoint allocation key for the composite of art layer `art` under a surface key prefix.
pub(super) fn art_key(prefix: &str, art: &str) -> String {
    format!("{prefix}/{art}")
}

/// The codepoint allocation key for a window's static composite.
pub(super) fn static_key(window: &str) -> String {
    format!("{}/static", window_key_prefix(window))
}

pub(super) fn hud_static_key(hud: &str) -> String {
    format!("{}/static", hud_key_prefix(hud))
}

fn sprite_key(sprite: &str) -> String {
    format!("sprite/{sprite}")
}

pub(super) fn sprite_glyph(sprite: &str, codepoints: &BTreeMap<String, u32>) -> char {
    char::from_u32(codepoints[&sprite_key(sprite)]).expect("sprite codepoint is a valid char")
}

/// An emitted static composite glyph and its alpha-trimmed advance.
pub(super) struct StaticGlyph {
    pub(super) glyph: char,
    pub(super) advance: u32,
}

/// The side of the client's font atlas pages. A larger bitmap glyph is never placed on one, so the client does not
/// draw it and advances the cursor by an amount the baked segment does not compensate for.
pub(super) const MAX_GLYPH_SIZE: u32 = 256;

impl CompileContext<'_> {
    /// Emits `comp` and returns its text, net-zero from `origin`. Window and HUD composites are one glyph, which must
    /// fit [`MAX_GLYPH_SIZE`]; a full-screen composite is split into tiles of at most that size, each glyph placed at
    /// its own ascent and the empty tiles skipped.
    pub(super) fn bake_composite(
        &mut self,
        comp: &Composite,
        origin: Point,
        pane: Pane<'_>,
        file_base: &str,
        key: String,
        part: Part,
    ) -> Result<String> {
        if pane.kind != PaneKind::Fullscreen {
            let ascent = origin.y + 7 - comp.bounds.y;
            let glyph = self.emit_static_glyph(comp, ascent, pane, file_base, key, part)?;
            return Ok(bake::bake_static_with_advance(glyph.glyph, &comp.bounds, origin, glyph.advance));
        }
        let mut text = String::new();
        for tile in tiles(comp) {
            let ascent = origin.y + 7 - tile.comp.bounds.y;
            let file = format!("{file_base}_{}_{}", tile.col, tile.row);
            let glyph = self.emit_static_glyph(&tile.comp, ascent, pane, &file, tile_key(&key, &tile), part.clone())?;
            text.push_str(&bake::bake_static_with_advance(glyph.glyph, &tile.comp.bounds, origin, glyph.advance));
        }
        Ok(text)
    }

    /// Emit `comp` as a bitmap glyph texture and main-font provider at `ascent`.
    ///
    /// Fails when the glyph exceeds [`MAX_GLYPH_SIZE`] in either direction.
    pub(super) fn emit_static_glyph(
        &mut self,
        comp: &Composite,
        ascent: i32,
        pane: Pane<'_>,
        file_base: &str,
        key: String,
        part: Part,
    ) -> Result<StaticGlyph> {
        let (width, height) = (comp.texture.width, comp.texture.height.max(ascent.max(0) as u32));
        if width > MAX_GLYPH_SIZE || height > MAX_GLYPH_SIZE {
            return Err(Error::Font(format!(
                "{} `{}`: art `{file_base}` bakes into one {}x{} px font glyph, but the client's font atlas pages are \
                 {MAX_GLYPH_SIZE}x{MAX_GLYPH_SIZE} px, so a larger glyph is not drawn and misplaces everything after \
                 it; keep the art within {MAX_GLYPH_SIZE} px each way (text may extend past it) or split it across \
                 panes",
                pane.kind.noun(),
                pane.name,
                width,
                height,
            )));
        }
        let texture = provider_texture(comp, ascent);
        let advance = bitmap_metrics(&texture, Size::new(texture.width, texture.height)).advance;
        let glyph = char::from_u32(self.codepoints[&key]).expect("glyph codepoint is a valid char");
        let provider = bitmap_provider(self.namespace, file_base, texture.height, ascent, glyph)?;
        self.bitmap_providers.push(provider);
        let file = OutputFile::binary(
            format!("assets/{}/textures/font/{file_base}.png", self.namespace),
            texture.encode_png()?,
        );
        self.push(part, file);
        Ok(StaticGlyph { glyph, advance })
    }
}

/// A piece of a composite no larger than [`MAX_GLYPH_SIZE`], `col` pieces right and `row` pieces down from its
/// top-left.
struct Tile {
    col: u32,
    row: u32,
    comp: Composite,
}

/// The pieces of `comp` that draw anything, in row-major order.
fn tiles(comp: &Composite) -> Vec<Tile> {
    let texture = &comp.texture;
    let mut tiles = Vec::new();
    for row in 0..texture.height.div_ceil(MAX_GLYPH_SIZE) {
        for col in 0..texture.width.div_ceil(MAX_GLYPH_SIZE) {
            let (x, y) = (col * MAX_GLYPH_SIZE, row * MAX_GLYPH_SIZE);
            let piece =
                crop(texture, x, y, MAX_GLYPH_SIZE.min(texture.width - x), MAX_GLYPH_SIZE.min(texture.height - y));
            if piece.rgba.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 0) {
                continue;
            }
            let bounds = Rect::new(comp.bounds.x + x as i32, comp.bounds.y + y as i32, piece.width, piece.height);
            tiles.push(Tile { col, row, comp: Composite { texture: piece, bounds, has_content: true } });
        }
    }
    tiles
}

/// The `width`x`height` pixels of `texture` from `(x, y)`.
fn crop(texture: &Texture, x: u32, y: u32, width: u32, height: u32) -> Texture {
    let mut out = Texture::transparent(width, height);
    let row = width as usize * 4;
    for dy in 0..height as usize {
        let from = ((y as usize + dy) * texture.width as usize + x as usize) * 4;
        out.rgba[dy * row..(dy + 1) * row].copy_from_slice(&texture.rgba[from..from + row]);
    }
    out
}

/// The codepoint allocation key of `tile` of the composite under `key`.
fn tile_key(key: &str, tile: &Tile) -> String {
    format!("{key}/{}/{}", tile.col, tile.row)
}

/// Pad the composite downward so its height covers `ascent`.
fn provider_texture(comp: &Composite, ascent: i32) -> Texture {
    if ascent <= comp.texture.height as i32 {
        return comp.texture.clone();
    }
    let height = ascent as u32;
    let mut texture = Texture::transparent(comp.texture.width, height);
    let row_len = (comp.texture.width * 4) as usize;
    for y in 0..comp.texture.height as usize {
        let start = y * row_len;
        texture.rgba[start..start + row_len].copy_from_slice(&comp.texture.rgba[start..start + row_len]);
    }
    texture
}
