//! Static composites and stable glyph codepoint allocation.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::{Composite, Texture, compose_draws, compose_window};
use crate::font::{allocate, bitmap_provider};
use crate::geometry::Size;
use crate::ir::{LaidOutHud, LaidOutWindow};
use crate::{Error, Result};

use super::metrics::bitmap_metrics;
use super::sprites::RuntimeSpriteAsset;
use super::{CompileContext, OutputFile};

/// Static layer composites, index-aligned with the name-sorted windows and HUDs.
pub(super) struct Composites {
    pub(super) windows: Vec<Composite>,
    pub(super) huds: Vec<Composite>,
}

pub(super) fn compose_layers(
    windows: &[&LaidOutWindow],
    huds: &[&LaidOutHud],
    textures: &BTreeMap<String, Texture>,
) -> Result<Composites> {
    let windows = windows.iter().map(|w| compose_window(w, textures)).collect::<Result<_>>()?;
    let huds = huds.iter().map(|h| compose_draws(&h.draws, &h.name, textures)).collect::<Result<_>>()?;
    Ok(Composites { windows, huds })
}

/// Allocate codepoints across the whole build: one key per static composite
/// with content, plus one per runtime sprite when any window uses sprites.
pub(super) fn allocate_codepoints(
    windows: &[&LaidOutWindow],
    huds: &[&LaidOutHud],
    composites: &Composites,
    runtime_sprites: Option<&BTreeMap<String, RuntimeSpriteAsset>>,
) -> Result<BTreeMap<String, u32>> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for (w, comp) in windows.iter().zip(&composites.windows) {
        if comp.has_content {
            keys.insert(static_key(&w.name));
        }
    }
    for (h, comp) in huds.iter().zip(&composites.huds) {
        if comp.has_content {
            keys.insert(hud_static_key(&h.name));
        }
    }
    for sprite in runtime_sprites.into_iter().flat_map(BTreeMap::keys) {
        keys.insert(sprite_key(sprite));
    }
    allocate(&keys)
}

/// The codepoint allocation key for a window's static composite.
pub(super) fn static_key(window: &str) -> String {
    format!("window/{window}/static")
}

pub(super) fn hud_static_key(hud: &str) -> String {
    format!("hud/{hud}/static")
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
