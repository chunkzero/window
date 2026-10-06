//! Static composites and stable glyph codepoint allocation.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::{Composite, RasterCache, Texture, compose_draws_cached};
use crate::font::{allocate, bitmap_provider};
use crate::geometry::Size;
use crate::ir::{Draw, LaidOutHud, LaidOutWindow, SwitchIr};
use crate::{Error, Result};

use super::metrics::bitmap_metrics;
use super::sprites::RuntimeSpriteAsset;
use super::{CompileContext, OutputFile};

/// A surface's static composite plus one composite per switch case, indexed `[switch][case]`.
pub(super) struct Layers {
    pub(super) base: Composite,
    pub(super) cases: Vec<Vec<Composite>>,
}

/// Static layer composites, index-aligned with the name-sorted windows and HUDs.
pub(super) struct Composites {
    pub(super) windows: Vec<Layers>,
    pub(super) huds: Vec<Layers>,
}

pub(super) fn compose_layers(
    windows: &[&LaidOutWindow],
    huds: &[&LaidOutHud],
    textures: &BTreeMap<String, Texture>,
) -> Result<Composites> {
    let mut cache = RasterCache::default();
    let windows =
        windows.iter().map(|w| layers(&w.name, &w.draws, &w.switches, textures, &mut cache)).collect::<Result<_>>()?;
    let huds =
        huds.iter().map(|h| layers(&h.name, &h.draws, &h.switches, textures, &mut cache)).collect::<Result<_>>()?;
    Ok(Composites { windows, huds })
}

fn layers(
    name: &str,
    draws: &[Draw],
    switches: &[SwitchIr],
    textures: &BTreeMap<String, Texture>,
    cache: &mut RasterCache,
) -> Result<Layers> {
    let cases = switches
        .iter()
        .map(|switch| {
            switch.cases.iter().map(|case| compose_draws_cached(&case.draws, name, textures, cache)).collect()
        })
        .collect::<Result<_>>()?;
    Ok(Layers { base: compose_draws_cached(draws, name, textures, cache)?, cases })
}

/// Allocate codepoints across the whole build: one key per static or case composite
/// with content, plus one per runtime sprite when any window uses sprites.
pub(super) fn allocate_codepoints(
    windows: &[&LaidOutWindow],
    huds: &[&LaidOutHud],
    composites: &Composites,
    runtime_sprites: Option<&BTreeMap<String, RuntimeSpriteAsset>>,
) -> Result<BTreeMap<String, u32>> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for (w, layers) in windows.iter().zip(&composites.windows) {
        layer_keys(&mut keys, &window_key_prefix(&w.name), &w.switches, layers);
    }
    for (h, layers) in huds.iter().zip(&composites.huds) {
        layer_keys(&mut keys, &hud_key_prefix(&h.name), &h.switches, layers);
    }
    for sprite in runtime_sprites.into_iter().flat_map(BTreeMap::keys) {
        keys.insert(sprite_key(sprite));
    }
    allocate(&keys)
}

fn layer_keys(keys: &mut BTreeSet<String>, prefix: &str, switches: &[SwitchIr], layers: &Layers) {
    if layers.base.has_content {
        keys.insert(format!("{prefix}/static"));
    }
    for (switch, cases) in switches.iter().zip(&layers.cases) {
        for (case, comp) in switch.cases.iter().zip(cases) {
            if comp.has_content {
                keys.insert(case_key(prefix, &switch.name, &case.value));
            }
        }
    }
}

pub(super) fn window_key_prefix(window: &str) -> String {
    format!("window/{window}")
}

pub(super) fn hud_key_prefix(hud: &str) -> String {
    format!("hud/{hud}")
}

/// The codepoint allocation key for one switch case's composite under a surface key prefix.
pub(super) fn case_key(prefix: &str, switch: &str, case: &str) -> String {
    format!("{prefix}/switch/{switch}/{case}")
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

impl CompileContext<'_> {
    /// Emit `comp` as a bitmap glyph texture and main-font provider at `ascent`.
    pub(super) fn emit_static_glyph(
        &mut self,
        comp: &Composite,
        ascent: i32,
        name: &str,
        file_base: &str,
        key: String,
    ) -> Result<StaticGlyph> {
        let texture = provider_texture(comp, ascent, name)?;
        let advance = bitmap_metrics(&texture, Size::new(texture.width, texture.height)).advance;
        let glyph = char::from_u32(self.codepoints[&key]).expect("glyph codepoint is a valid char");
        let provider = bitmap_provider(self.namespace, file_base, texture.height, ascent, glyph)?;
        self.bitmap_providers.push(provider);
        self.files.push(OutputFile {
            path: format!("assets/{}/textures/font/{file_base}.png", self.namespace),
            contents: texture.encode_png()?,
        });
        Ok(StaticGlyph { glyph, advance })
    }
}

/// Pad the composite downward so its height covers `ascent`.
fn provider_texture(comp: &Composite, ascent: i32, name: &str) -> Result<Texture> {
    if ascent <= comp.texture.height as i32 {
        return Ok(comp.texture.clone());
    }
    let height = ascent as u32;
    if height > 512 {
        return Err(Error::Font(format!(
            "window `{name}`: generated visual bleed requires bitmap height {height}, \
             exceeding the 512px provider limit"
        )));
    }

    let mut texture = Texture::transparent(comp.texture.width, height);
    let row_len = (comp.texture.width * 4) as usize;
    for y in 0..comp.texture.height as usize {
        let start = y * row_len;
        texture.rgba[start..start + row_len].copy_from_slice(&comp.texture.rgba[start..start + row_len]);
    }
    Ok(texture)
}
