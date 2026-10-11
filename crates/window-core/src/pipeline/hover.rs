//! Region hovers (`coreShaders.hover`) and the vanilla slot highlight (`vanilla.slotHighlight`).
//!
//! A region that draws an outline or art while hovered carries a hover glyph the runtime puts in its hitbox item's
//! tooltip title, so the client draws the glyph exactly while a slot of the region is hovered. The core text shader
//! then moves it onto the region.

use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::Replacement;
use crate::compose::{Texture, compose_draws};
use crate::font::{bitmap_provider, spacers_for};
use crate::geometry::{Rect, Size};
use crate::hud::{HOVER_GLYPH_ID, HOVER_SIZE_MARK};
use crate::ir::{Draw, HoverIr, LaidOutWindow, RegionIr};
use crate::manifest::HoverEntry;
use crate::size::Part;
use crate::{Error, Result};

use super::anvil::{blank, sprite_files};
use super::glyphs::MAX_GLYPH_SIZE;
use super::sprites::RuntimeSpriteAsset;
use super::{CompileContext, OutputFile};

/// First resource-pack format with the slot highlight sprites (Minecraft 1.21.2).
const SLOT_HIGHLIGHT_PACK_FORMAT: u32 = 42;

const SLOT_HIGHLIGHT_SIZE: Size = Size { width: 24, height: 24 };

/// Vanilla draws the back sprite under the hovered slot's item and the front sprite over it.
const SLOT_HIGHLIGHT_BACK: &str = "assets/minecraft/textures/gui/sprites/container/slot_highlight_back.png";
const SLOT_HIGHLIGHT_FRONT: &str = "assets/minecraft/textures/gui/sprites/container/slot_highlight_front.png";

/// The narrowest hover glyph whose data rows hold an id and a size pixel at each end.
const MIN_WIDTH: u32 = 4;

/// The tallest hover art: its glyph adds a data row above and below it.
const MAX_HEIGHT: u32 = MAX_GLYPH_SIZE - 2;

/// Files for `vanilla.slotHighlight`: a sprite replaces the front highlight and the back one is blanked.
pub(super) fn slot_highlight_files(
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    highlight: &Replacement,
    pack_format: Option<u32>,
) -> Result<Vec<OutputFile>> {
    if *highlight == Replacement::Keep {
        return Ok(Vec::new());
    }
    if let Some(format) = pack_format.filter(|&format| format < SLOT_HIGHLIGHT_PACK_FORMAT) {
        return Err(Error::Validation(format!(
            "`vanilla.slotHighlight` replaces GUI sprites that need pack format {SLOT_HIGHLIGHT_PACK_FORMAT} or \
             newer, but the pack format is {format}"
        )));
    }
    let mut files = vec![blank(SLOT_HIGHLIGHT_BACK, SLOT_HIGHLIGHT_SIZE)?];
    match highlight {
        Replacement::Sprite(sprite) => files.extend(sprite_files(
            "vanilla.slotHighlight",
            runtime_sprites,
            sprite,
            SLOT_HIGHLIGHT_SIZE,
            &[SLOT_HIGHLIGHT_FRONT],
        )?),
        _ => files.push(blank(SLOT_HIGHLIGHT_FRONT, SLOT_HIGHLIGHT_SIZE)?),
    }
    Ok(files)
}

/// Rejects a region that sets `hover` while `coreShaders.hover` is off.
pub(super) fn check_disabled(windows: &[&LaidOutWindow]) -> Result<()> {
    for w in windows {
        if let Some(region) = w.regions.iter().find(|region| region.hover.is_some()) {
            return Err(Error::Validation(format!(
                "window `{}`: {} sets `hover`, but `coreShaders.hover` is off; set `coreShaders: {{ hover: true }}` in \
                 the Window plugin options or remove `hover`",
                w.name, region.source
            )));
        }
    }
    Ok(())
}

/// What a hovered region draws: authored art, or else the generated outline.
struct Shown<'a> {
    /// Where it is drawn, in GUI space.
    rect: Rect,
    /// The art's interned name and its draw over `rect`.
    art: Option<(&'a str, &'a Draw)>,
}

impl Shown<'_> {
    /// What `region` draws while hovered: as it says, else the outline when it takes clicks.
    fn of(region: &RegionIr) -> Option<Shown<'_>> {
        match &region.hover {
            Some(HoverIr::None) => None,
            Some(HoverIr::Art { name, draw }) => Some(Shown { rect: *draw.dest(), art: Some((name, draw)) }),
            Some(HoverIr::Outline) => Some(Shown { rect: region.rect, art: None }),
            None => region.action.is_some().then_some(Shown { rect: region.rect, art: None }),
        }
    }

    /// The codepoint allocation key of its glyph, which regions drawing the same pixels share.
    fn glyph_key(&self) -> String {
        let Size { width, height } = self.rect.size();
        match self.art {
            Some((name, _)) => format!("hover/{name}/{width}x{height}"),
            None => format!("hover/{width}x{height}"),
        }
    }
}

/// The codepoint allocation key of each distinct hover glyph among `windows`.
pub(super) fn glyph_keys(windows: &[&LaidOutWindow]) -> impl Iterator<Item = String> {
    windows
        .iter()
        .flat_map(|w| &w.regions)
        .filter_map(Shown::of)
        .map(|shown| shown.glyph_key())
        .collect::<BTreeSet<_>>()
        .into_iter()
}

impl CompileContext<'_> {
    /// The hover of `region` of `w`, emitting its glyph the first time it is seen; `None` when the region draws
    /// nothing while hovered or `coreShaders.hover` is off.
    pub(super) fn hover_entry(&mut self, w: &LaidOutWindow, region: &RegionIr) -> Result<Option<HoverEntry>> {
        let Some(shown) = Shown::of(region).filter(|_| self.hover) else {
            return Ok(None);
        };
        let (rect, size) = (shown.rect, shown.rect.size());
        let (what, fix) = match shown.art {
            Some(_) => ("hover art", "resize the art or set `hover` to false on it"),
            None => ("hover outline", "set `hover` to false on it"),
        };
        let err =
            |message: String| Error::Validation(format!("window `{}`: {} {message}; {fix}", w.name, region.source));
        if !(MIN_WIDTH..=MAX_GLYPH_SIZE).contains(&size.width) || !(1..=MAX_HEIGHT).contains(&size.height) {
            return Err(err(format!(
                "has a {}x{} px {what}, which must be {MIN_WIDTH} to {MAX_GLYPH_SIZE} px wide and 1 to {MAX_HEIGHT} \
                 px tall",
                size.width, size.height
            )));
        }
        // The runtime sends each offset plus 128 as one color channel.
        let gui = w.surface.gui_size();
        let (x, y) = (rect.x - (gui.width / 2) as i32, rect.y - (gui.height / 2) as i32);
        if !(-128..=127).contains(&x) || !(-128..=127).contains(&y) {
            return Err(err(format!("has a {what} that starts more than 128 px from the GUI center")));
        }

        let key = shown.glyph_key();
        let glyph = char::from_u32(self.codepoints[&key]).expect("hover codepoint is a valid char");
        if self.hover_glyphs.insert(glyph) {
            let art = match shown.art {
                Some((_, draw)) => compose_draws(std::slice::from_ref(draw), &w.name, self.textures)?.texture,
                None => outline_texture(size),
            };
            let texture = glyph_texture(&art);
            // The text shader assumes ascent 0 when it finds the tooltip from the glyph's position.
            self.bitmap_providers.push(bitmap_provider(self.namespace, &key, texture.height, 0, glyph)?);
            let path = format!("assets/{}/textures/font/{key}.png", self.namespace);
            self.push(Part::Hover, OutputFile::binary(path, texture.encode_png()?));
        }
        // A bitmap glyph advances by its width plus 1; the data rows are opaque at both ends.
        let text = format!("{glyph}{}", spacers_for(-(size.width as i32 + 1)));
        Ok(Some(HoverEntry { text, x, y }))
    }

    /// Emits the transparent sprites of the tooltip style `<namespace>:hover`, which hides the tooltip box of a
    /// region that draws a hover without a tooltip.
    pub(super) fn emit_hover_tooltip_style(&mut self) -> Result<()> {
        for sprite in ["background", "frame"] {
            let path = format!("assets/{}/textures/gui/sprites/tooltip/hover_{sprite}.png", self.namespace);
            self.push(Part::Hover, blank(&path, Size { width: 1, height: 1 })?);
        }
        Ok(())
    }
}

/// The 1px white outline of a `size` region.
fn outline_texture(size: Size) -> Texture {
    let Size { width, height } = size;
    let mut texture = Texture::transparent(width, height);
    for y in 0..height {
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                texture.blend(x, y, [0xff; 4]);
            }
        }
    }
    texture
}

/// The hover glyph of `art`: the art between two data rows, which the text shader reads and does not draw. Each end
/// of a data row is [`HOVER_GLYPH_ID`], and the pixel beside it holds the art's size minus one in red and green and
/// [`HOVER_SIZE_MARK`] in blue. Every vertex of the glyph's quad reads both from its own corner.
fn glyph_texture(art: &Texture) -> Texture {
    let (width, height) = (art.width, art.height);
    let mut texture = Texture::transparent(width, height + 2);
    let row = width as usize * 4;
    texture.rgba[row..row + art.rgba.len()].copy_from_slice(&art.rgba);
    let id = [HOVER_GLYPH_ID.r, HOVER_GLYPH_ID.g, HOVER_GLYPH_ID.b, 0xff];
    let size_pixel = [(width - 1) as u8, (height - 1) as u8, HOVER_SIZE_MARK, 0xff];
    for y in [0, height + 1] {
        for (x, pixel) in [(0, id), (1, size_pixel), (width - 2, size_pixel), (width - 1, id)] {
            texture.blend(x, y, pixel);
        }
    }
    texture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_texture_holds_the_id_and_size_in_rows_outside_the_art() {
        let mut art = Texture::transparent(6, 3);
        art.blend(0, 0, [1, 2, 3, 4]);
        art.blend(5, 2, [5, 6, 7, 8]);
        let texture = glyph_texture(&art);
        assert_eq!((texture.width, texture.height), (6, 5));
        let id = [HOVER_GLYPH_ID.r, HOVER_GLYPH_ID.g, HOVER_GLYPH_ID.b, 0xff];
        for y in [0, 4] {
            assert_eq!(texture.get(0, y), id);
            assert_eq!(texture.get(5, y), id);
            assert_eq!(texture.get(1, y), [5, 2, HOVER_SIZE_MARK, 0xff]);
            assert_eq!(texture.get(4, y), [5, 2, HOVER_SIZE_MARK, 0xff]);
            assert_eq!(texture.get(2, y)[3], 0, "the rest of a data row is transparent");
        }
        assert_eq!(texture.get(0, 1), [1, 2, 3, 4], "the art keeps its colors and alpha");
        assert_eq!(texture.get(5, 3), [5, 6, 7, 8]);
        assert_eq!(texture.get(1, 1)[3], 0);
    }

    #[test]
    fn outline_texture_is_a_white_border() {
        let texture = outline_texture(Size { width: 6, height: 3 });
        assert_eq!(texture.get(0, 0), [0xff; 4]);
        assert_eq!(texture.get(5, 1), [0xff; 4]);
        assert_eq!(texture.get(3, 2), [0xff; 4]);
        assert_eq!(texture.get(2, 1)[3], 0, "the interior is transparent");
    }
}
