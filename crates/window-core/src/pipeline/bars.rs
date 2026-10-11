//! Bar fills: an art layer cut into power-of-two clips, each a glyph, that the runtime lays side by side to draw any
//! number of the fill's columns. Clips with the same pixels share a glyph, so a fill whose columns repeat costs one
//! glyph per clip width.

use std::collections::{BTreeMap, HashMap};

use crate::Result;
use crate::compose::{Composite, Texture};
use crate::geometry::{Point, Rect};
use crate::manifest::BarEntry;
use crate::size::{Pane, Part};

use super::CompileContext;

/// The columns `start..start + width` of `texture`.
pub(super) fn columns(texture: &Texture, start: u32, width: u32) -> Texture {
    let mut out = Texture::transparent(width, texture.height);
    let row = width as usize * 4;
    for y in 0..texture.height as usize {
        let from = (y * texture.width as usize + start as usize) * 4;
        out.rgba[y * row..(y + 1) * row].copy_from_slice(&texture.rgba[from..from + row]);
    }
    out
}

/// The clips of `fill`: `levels[k][i]` is the clip of `2^k` columns starting at column `i << (k + 1)`, as the index
/// of the first clip of its level with the same pixels, or `None` when it is fully transparent.
fn clip_levels(fill: &Texture) -> Vec<Vec<Option<usize>>> {
    let mut levels = Vec::new();
    let mut size = 1;
    while size <= fill.width {
        let mut first: HashMap<Vec<u8>, usize> = HashMap::new();
        let starts = (0..=fill.width - size).step_by(2 * size as usize);
        let clips = starts.enumerate().map(|(index, start)| {
            let clip = columns(fill, start, size);
            let drawn = clip.rgba.as_chunks::<4>().0.iter().any(|pixel| pixel[3] != 0);
            drawn.then(|| *first.entry(clip.rgba).or_insert(index))
        });
        levels.push(clips.collect());
        size *= 2;
    }
    levels
}

/// The codepoint allocation key of the clip at `index` of `level`, under the key of its art layer.
fn clip_key(art_key: &str, level: usize, index: usize) -> String {
    format!("{art_key}/{level}/{index}")
}

/// The codepoint allocation keys of the glyphs of bar fill `fill`, under the key of its art layer.
pub(super) fn clip_keys(art_key: &str, fill: &Texture) -> Vec<String> {
    let levels = clip_levels(fill);
    let distinct = levels.iter().enumerate().flat_map(|(level, clips)| {
        clips.iter().enumerate().filter(|&(index, clip)| *clip == Some(index)).map(move |(index, _)| (level, index))
    });
    distinct.map(|(level, index)| clip_key(art_key, level, index)).collect()
}

impl CompileContext<'_> {
    /// Emits a glyph per distinct clip of bar `name`'s fill `comp`, and returns how the runtime composes them from
    /// `origin`. `art_key` and `file_base` name the fill's art layer among the glyph keys and textures.
    pub(super) fn bar_entry(
        &mut self,
        name: &str,
        comp: &Composite,
        pane: Pane<'_>,
        art_key: &str,
        file_base: &str,
        origin: Point,
    ) -> Result<BarEntry> {
        let ascent = origin.y + 7 - comp.bounds.y;
        let mut entry = BarEntry {
            name: name.into(),
            x: comp.bounds.x - origin.x,
            width: comp.texture.width,
            levels: Vec::new(),
            advances: BTreeMap::new(),
        };
        for (level, clips) in clip_levels(&comp.texture).iter().enumerate() {
            let size = 1 << level;
            let mut glyphs: Vec<char> = Vec::with_capacity(clips.len());
            for (index, clip) in clips.iter().enumerate() {
                let glyph = match *clip {
                    None => ' ',
                    Some(first) if first < index => glyphs[first],
                    Some(_) => {
                        let start = (index as u32) << (level + 1);
                        let bounds = Rect::new(comp.bounds.x + start as i32, comp.bounds.y, size, comp.bounds.height);
                        let clip =
                            Composite { texture: columns(&comp.texture, start, size), bounds, has_content: true };
                        let file = format!("{file_base}_{level}_{index}");
                        let key = clip_key(art_key, level, index);
                        let glyph = self.emit_static_glyph(&clip, ascent, pane, &file, key, Part::art_layer(pane))?;
                        entry.advances.insert(glyph.glyph, glyph.advance);
                        glyph.glyph
                    }
                };
                glyphs.push(glyph);
            }
            entry.levels.push(glyphs.into_iter().collect());
        }
        Ok(entry)
    }
}
