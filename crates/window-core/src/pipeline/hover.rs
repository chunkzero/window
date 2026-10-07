//! Hover outlines: glyphs that ride in a button's tooltip title and that the core text shader moves onto the
//! button, so a button shows hover art exactly while the client draws its tooltip.

use std::collections::BTreeSet;

use crate::compose::Texture;
use crate::font::bitmap_provider;
use crate::geometry::Size;
use crate::hud::HOVER_GLYPH_ID;
use crate::ir::{ButtonIr, LaidOutWindow, Rgb};
use crate::manifest::HoverEntry;
use crate::surface::Surface;
use crate::{Error, Result};

use super::{CompileContext, OutputFile};

const HOVER_COLOR: Rgb = Rgb { r: 0xff, g: 0xff, b: 0xff };

/// Vanilla's slot highlight, drawn as a back and a front sprite around the hovered slot.
const SLOT_HIGHLIGHT_SPRITES: [&str; 2] = [
    "assets/minecraft/textures/gui/sprites/container/slot_highlight_back.png",
    "assets/minecraft/textures/gui/sprites/container/slot_highlight_front.png",
];

/// Transparent replacements for vanilla's slot highlight. They apply to every container, not only Window's.
pub(super) fn hidden_slot_highlight() -> Result<Vec<OutputFile>> {
    let png = Texture::transparent(24, 24).encode_png()?;
    Ok(SLOT_HIGHLIGHT_SPRITES.iter().map(|path| OutputFile::binary(*path, png.clone())).collect())
}

/// The codepoint allocation keys of every distinct hover outline size.
pub(super) fn hover_keys(windows: &[&LaidOutWindow]) -> BTreeSet<String> {
    windows
        .iter()
        .flat_map(|w| &w.buttons)
        .map(|button| hover_key(Size::new(button.rect.width, button.rect.height)))
        .collect()
}

fn hover_key(size: Size) -> String {
    format!("hover/{}x{}", size.width, size.height)
}

impl CompileContext<'_> {
    /// The hover outline entry for `button`, emitting its glyph the first time its size is seen.
    pub(super) fn hover_entry(&mut self, window: &str, surface: Surface, button: &ButtonIr) -> Result<HoverEntry> {
        let rect = button.rect;
        if !(4..=256).contains(&rect.width) || !(1..=256).contains(&rect.height) {
            return Err(Error::Validation(format!(
                "window `{window}`: button `{}` is {}x{}, but hover outlines need 4..256 by 1..256 pixels",
                button.name, rect.width, rect.height
            )));
        }
        let gui = surface.gui_size();
        let x = rect.x - (gui.width / 2) as i32;
        let y = rect.y - (gui.height / 2) as i32;
        if !(-128..=127).contains(&x) || !(-128..=127).contains(&y) {
            return Err(Error::Validation(format!(
                "window `{window}`: button `{}` is too far from the screen center for a hover outline",
                button.name
            )));
        }

        let key = hover_key(Size::new(rect.width, rect.height));
        let glyph = char::from_u32(self.codepoints[&key]).expect("hover codepoint is a valid char");
        if self.hover_glyphs.insert(glyph) {
            let file = format!("hover/{}x{}", rect.width, rect.height);
            let texture = outline_texture(rect.width, rect.height);
            // Ascent 0 puts the glyph's top 7px below the text origin, which the shader relies on to find the tooltip.
            self.bitmap_providers.push(bitmap_provider(self.namespace, &file, texture.height, 0, glyph)?);
            self.files.push(OutputFile::binary(
                format!("assets/{}/textures/font/{file}.png", self.namespace),
                texture.encode_png()?,
            ));
        }
        Ok(HoverEntry { glyph: glyph.to_string(), advance: rect.width + 1, x, y })
    }
}

/// A 1px outline of a `width`x`height` button, framed by one data row above and below.
///
/// Each data row holds the hover id at both ends and the content size one pixel inward, so every corner vertex of
/// the glyph quad can read them; the shader crops the rows off before drawing.
fn outline_texture(width: u32, height: u32) -> Texture {
    let mut texture = Texture::transparent(width, height + 2);
    for y in 1..=height {
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 1 || y == height {
                set_pixel(&mut texture, x, y, HOVER_COLOR);
            }
        }
    }
    let size = Rgb { r: (width - 1) as u8, g: (height - 1) as u8, b: 0 };
    for y in [0, height + 1] {
        set_pixel(&mut texture, 0, y, HOVER_GLYPH_ID);
        set_pixel(&mut texture, 1, y, size);
        set_pixel(&mut texture, width - 2, y, size);
        set_pixel(&mut texture, width - 1, y, HOVER_GLYPH_ID);
    }
    texture
}

fn set_pixel(texture: &mut Texture, x: u32, y: u32, color: Rgb) {
    let i = ((y * texture.width + x) * 4) as usize;
    texture.rgba[i..i + 4].copy_from_slice(&[color.r, color.g, color.b, 0xff]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_texture_carries_id_and_size_at_every_corner() {
        let texture = outline_texture(6, 3);
        let pixel = |x: u32, y: u32| {
            let i = ((y * texture.width + x) * 4) as usize;
            Rgb { r: texture.rgba[i], g: texture.rgba[i + 1], b: texture.rgba[i + 2] }
        };
        for y in [0, 4] {
            assert_eq!(pixel(0, y), HOVER_GLYPH_ID);
            assert_eq!(pixel(5, y), HOVER_GLYPH_ID);
            assert_eq!(pixel(1, y), Rgb { r: 5, g: 2, b: 0 });
            assert_eq!(pixel(4, y), Rgb { r: 5, g: 2, b: 0 });
        }
        assert_eq!(pixel(0, 1), HOVER_COLOR);
        assert_eq!(texture.rgba[((2 * 6 + 2) * 4 + 3) as usize], 0, "the outline interior is transparent");
    }
}
