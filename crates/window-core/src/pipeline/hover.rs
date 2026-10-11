//! Hover outlines (`coreShaders.hover`) and the vanilla slot highlight (`vanilla.slotHighlight`).
//!
//! An outlined region carries a hover glyph the runtime puts in its hitbox item's tooltip title, so the client draws
//! the glyph exactly while a slot of the region is hovered. The core text shader then moves it onto the region.

use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::Replacement;
use crate::compose::Texture;
use crate::font::{bitmap_provider, spacers_for};
use crate::geometry::Size;
use crate::hud::{HOVER_GLYPH_ID, HOVER_SIZE_MARK};
use crate::ir::{LaidOutWindow, RegionIr};
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

/// The narrowest outline whose rows hold a corner and a size pixel at each end.
const MIN_WIDTH: u32 = 4;

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

/// Whether `region` is outlined while hovered: as it says, else when it takes clicks.
fn outlined(region: &RegionIr) -> bool {
    region.hover.unwrap_or(region.action.is_some())
}

/// The codepoint allocation key of each distinct outline size among `windows`.
pub(super) fn glyph_keys(windows: &[&LaidOutWindow]) -> impl Iterator<Item = String> {
    windows
        .iter()
        .flat_map(|w| &w.regions)
        .filter(|region| outlined(region))
        .map(|region| glyph_key(region.rect.size()))
        .collect::<BTreeSet<_>>()
        .into_iter()
}

fn glyph_key(size: Size) -> String {
    format!("hover/{}x{}", size.width, size.height)
}

impl CompileContext<'_> {
    /// The hover outline of `region` of `w`, emitting its glyph the first time its size is seen; `None` when the
    /// region is not outlined or `coreShaders.hover` is off.
    pub(super) fn hover_entry(&mut self, w: &LaidOutWindow, region: &RegionIr) -> Result<Option<HoverEntry>> {
        if !self.hover || !outlined(region) {
            return Ok(None);
        }
        let (rect, size) = (region.rect, region.rect.size());
        let err = |message: String| {
            Error::Validation(format!("window `{}`: {} {message}; set `hover` to false on it", w.name, region.source))
        };
        if !(MIN_WIDTH..=MAX_GLYPH_SIZE).contains(&size.width) || !(1..=MAX_GLYPH_SIZE).contains(&size.height) {
            return Err(err(format!(
                "is {}x{} px, but a hover outline is {MIN_WIDTH} to {MAX_GLYPH_SIZE} px wide and 1 to \
                 {MAX_GLYPH_SIZE} px tall",
                size.width, size.height
            )));
        }
        // The runtime sends each offset plus 128 as one color channel.
        let gui = w.surface.gui_size();
        let (x, y) = (rect.x - (gui.width / 2) as i32, rect.y - (gui.height / 2) as i32);
        if !(-128..=127).contains(&x) || !(-128..=127).contains(&y) {
            return Err(err("starts more than 128 px from the GUI center, too far for a hover outline".into()));
        }

        let key = glyph_key(size);
        let glyph = char::from_u32(self.codepoints[&key]).expect("hover codepoint is a valid char");
        if self.hover_glyphs.insert(glyph) {
            let texture = outline_texture(size);
            // The text shader assumes ascent 0 when it finds the tooltip from the glyph's position.
            self.bitmap_providers.push(bitmap_provider(self.namespace, &key, size.height, 0, glyph)?);
            let path = format!("assets/{}/textures/font/{key}.png", self.namespace);
            self.push(Part::Hover, OutputFile::binary(path, texture.encode_png()?));
        }
        // A bitmap glyph advances by its width plus 1.
        let text = format!("{glyph}{}", spacers_for(-(size.width as i32 + 1)));
        Ok(Some(HoverEntry { text, x, y }))
    }

    /// Emits the transparent sprites of the tooltip style `<namespace>:hover`, which hides the tooltip box of an
    /// outlined region without a tooltip.
    pub(super) fn emit_hover_tooltip_style(&mut self) -> Result<()> {
        for sprite in ["background", "frame"] {
            let path = format!("assets/{}/textures/gui/sprites/tooltip/hover_{sprite}.png", self.namespace);
            self.push(Part::Hover, blank(&path, Size { width: 1, height: 1 })?);
        }
        Ok(())
    }
}

/// The 1px outline of a `size` region. The text shader paints every opaque pixel white, so their colors hold data:
/// each corner is [`HOVER_GLYPH_ID`], and the pixel beside it in its row holds the size minus one in red and green
/// and [`HOVER_SIZE_MARK`] in blue. Every vertex of the glyph's quad reads both from its own corner.
fn outline_texture(size: Size) -> Texture {
    let Size { width, height } = size;
    let mut texture = Texture::transparent(width, height);
    let mut set = |x: u32, y: u32, rgb: [u8; 3]| {
        let i = ((y * width + x) * 4) as usize;
        texture.rgba[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 0xff]);
    };
    for y in 0..height {
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                set(x, y, [0xff; 3]);
            }
        }
    }
    let id = [HOVER_GLYPH_ID.r, HOVER_GLYPH_ID.g, HOVER_GLYPH_ID.b];
    let size_pixel = [(width - 1) as u8, (height - 1) as u8, HOVER_SIZE_MARK];
    for y in [0, height - 1] {
        set(0, y, id);
        set(1, y, size_pixel);
        set(width - 2, y, size_pixel);
        set(width - 1, y, id);
    }
    texture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_texture_holds_the_id_and_size_at_every_corner() {
        let texture = outline_texture(Size { width: 6, height: 3 });
        let id = [HOVER_GLYPH_ID.r, HOVER_GLYPH_ID.g, HOVER_GLYPH_ID.b, 0xff];
        for y in [0, 2] {
            assert_eq!(texture.get(0, y), id);
            assert_eq!(texture.get(5, y), id);
            assert_eq!(texture.get(1, y), [5, 2, HOVER_SIZE_MARK, 0xff]);
            assert_eq!(texture.get(4, y), [5, 2, HOVER_SIZE_MARK, 0xff]);
        }
        assert_eq!(texture.get(0, 1), [0xff; 4], "the edges between corners are opaque");
        assert_eq!(texture.get(2, 1)[3], 0, "the interior is transparent");
    }
}
