//! Bitmap text fonts: theme glyph sheets and the bundled small-caps font.
//!
//! A text font draws the characters its sheet maps and falls back to the vanilla glyphs for everything else, so its
//! metrics are the vanilla tables overlaid with the sheet's measured glyphs. Cells are [`CELL_HEIGHT`] pixels tall and
//! sit on the text line like vanilla's `ascii.png` cells, so glyph art carries its own vertical placement.

use std::collections::BTreeMap;

use crate::compose::Texture;
use crate::manifest::FontMetricsEntry;
use crate::model::FontDef;
use crate::{Error, Result, vanilla};

/// The bundled font `small_caps: true` selects. A theme font with this name replaces it.
pub const SMALL_CAPS: &str = "small_caps";

const SMALL_CAPS_PNG: &[u8] = include_bytes!("text_font/small_caps.png");
const SMALL_CAPS_CHARS: [&str; 6] = [
    "ABCDEFGHIJKLMNOP",
    "QRSTUVWXYZ012345",
    "abcdefghijklmnop",
    "qrstuvwxyz6789.,",
    ":;!?'\"-+=/%()[]<",
    ">*#_\0\0\0\0\0\0\0\0\0\0\0\0",
];

/// The height of every glyph cell: rows above the baseline, then one row below it, as in vanilla's `ascii.png`.
pub const CELL_HEIGHT: u32 = 8;

/// Text fonts by name.
pub type TextFonts = BTreeMap<String, TextFont>;

/// A decoded glyph sheet and its measured advances.
#[derive(Clone, Debug)]
pub struct TextFont {
    /// The glyph sheet.
    pub texture: Texture,
    /// One string per sheet row; `\0` marks an empty cell.
    pub chars: Vec<String>,
    advances: BTreeMap<char, u32>,
}

impl TextFont {
    /// Splits `texture` into the cells `chars` describes and measures each glyph as Minecraft does.
    pub fn new(name: &str, texture: Texture, chars: Vec<String>) -> Result<Self> {
        let invalid = |message: String| Error::Validation(format!("font `{name}`: {message}"));
        let columns = chars.first().map_or(0, |row| row.chars().count()) as u32;
        if columns == 0 || chars.iter().any(|row| row.chars().count() as u32 != columns) {
            return Err(invalid("`chars` rows must be non-empty and equally long".into()));
        }
        let rows = chars.len() as u32;
        if !texture.width.is_multiple_of(columns) || texture.height != rows * CELL_HEIGHT {
            return Err(invalid(format!(
                "{}x{} texture does not divide into {columns}x{rows} cells {CELL_HEIGHT}px tall",
                texture.width, texture.height
            )));
        }
        let cell_width = texture.width / columns;

        let mut advances = BTreeMap::new();
        for (row, line) in (0..).zip(&chars) {
            for (column, c) in (0..).zip(line.chars()) {
                if c == '\0' {
                    continue;
                }
                let right = opaque_right(&texture, column * cell_width, row * CELL_HEIGHT, cell_width, CELL_HEIGHT)
                    .ok_or_else(|| {
                        invalid(format!("glyph `{c}` has no opaque pixels; use `\\u0000` for empty cells"))
                    })?;
                advances.insert(c, right + 1 + vanilla::INTER_GLYPH_GAP);
            }
        }
        Ok(Self { texture, chars, advances })
    }

    /// Cursor advance of `c`, or `None` when neither the sheet nor the vanilla glyphs draw it.
    pub fn advance(&self, c: char) -> Option<u32> {
        self.advances.get(&c).copied().or_else(|| vanilla::advance(c))
    }

    /// Visible ink width of `c`, or `None` when neither the sheet nor the vanilla glyphs draw it.
    pub fn glyph_width(&self, c: char) -> Option<u32> {
        match self.advances.get(&c) {
            Some(advance) => Some(advance - vanilla::INTER_GLYPH_GAP),
            None => vanilla::glyph_width(c),
        }
    }

    /// The runtime metrics of every character the font draws.
    pub fn metrics(&self) -> FontMetricsEntry {
        let chars = vanilla::advances().map(|(c, _)| c).chain(self.advances.keys().copied());
        let (advances, glyph_widths) = chars
            .map(|c| {
                let advance = self.advance(c).expect("font draws every listed character");
                let width = self.glyph_width(c).expect("font draws every listed character");
                ((c, advance), (c, width))
            })
            .unzip();
        FontMetricsEntry { advances, glyph_widths, bold_advance: vanilla::BOLD_ADVANCE }
    }
}

/// The theme's fonts, plus the bundled small-caps font unless the theme defines its own.
pub fn resolve(fonts: &BTreeMap<String, FontDef>, textures: &BTreeMap<String, Texture>) -> Result<TextFonts> {
    let mut resolved = TextFonts::new();
    if !fonts.contains_key(SMALL_CAPS) {
        resolved.insert(SMALL_CAPS.into(), small_caps());
    }
    for (name, font) in fonts {
        let texture = textures.get(&font.texture).ok_or_else(|| Error::Texture {
            path: font.texture.clone(),
            message: format!("referenced by font `{name}` but not provided"),
        })?;
        resolved.insert(name.clone(), TextFont::new(name, texture.clone(), font.chars.clone())?);
    }
    Ok(resolved)
}

/// The bundled small-caps font: letters of either case as 5px small capitals, 3px-wide digits, and common symbols,
/// drawn centered on vanilla capital height.
pub fn small_caps() -> TextFont {
    let texture = Texture::decode_png(SMALL_CAPS_PNG).expect("bundled small-caps sheet decodes");
    let chars = SMALL_CAPS_CHARS.iter().map(|row| row.to_string()).collect();
    TextFont::new(SMALL_CAPS, texture, chars).expect("bundled small-caps sheet is valid")
}

/// The rightmost opaque column within a cell, relative to the cell's left edge.
fn opaque_right(texture: &Texture, x: u32, y: u32, width: u32, height: u32) -> Option<u32> {
    (0..width).rev().find(|&local_x| {
        (0..height).any(|local_y| {
            let index = (((y + local_y) * texture.width + x + local_x) * 4 + 3) as usize;
            texture.rgba.get(index).is_some_and(|&alpha| alpha != 0)
        })
    })
}

#[cfg(test)]
mod tests;
