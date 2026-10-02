//! Bitmap glyph measurements matching Minecraft's alpha-trimmed advances.

use crate::compose::Texture;
use crate::geometry::Size;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GlyphMetrics {
    pub(super) x_offset: u32,
    pub(super) glyph_width: u32,
    pub(super) advance: u32,
}

/// Measure `texture`'s opaque columns as Minecraft does when it renders the
/// bitmap at `rendered_size`.
pub(super) fn bitmap_metrics(texture: &Texture, rendered_size: Size) -> GlyphMetrics {
    let Some((left, right)) = opaque_x_bounds(texture) else {
        return GlyphMetrics { x_offset: 0, glyph_width: 0, advance: 0 };
    };
    let source_height = texture.height.max(1);
    let x_offset = scale_boundary(left, source_height, rendered_size.height);
    let right_edge = scale_boundary(right + 1, source_height, rendered_size.height);
    GlyphMetrics { x_offset, glyph_width: right_edge.saturating_sub(x_offset), advance: right_edge.saturating_add(1) }
}

fn scale_boundary(value: u32, source_height: u32, rendered_height: u32) -> u32 {
    let value = u128::from(value);
    let source_height = u128::from(source_height.max(1));
    let rendered_height = u128::from(rendered_height);
    ((value * rendered_height * 2 + source_height) / (source_height * 2)).min(u128::from(u32::MAX)) as u32
}

fn opaque_x_bounds(texture: &Texture) -> Option<(u32, u32)> {
    let mut left: Option<u32> = None;
    let mut right: Option<u32> = None;
    for y in 0..texture.height {
        for x in 0..texture.width {
            let alpha_index = ((y * texture.width + x) * 4 + 3) as usize;
            if texture.rgba.get(alpha_index).copied().unwrap_or(0) != 0 {
                left = Some(left.map_or(x, |current| current.min(x)));
                right = Some(right.map_or(x, |current| current.max(x)));
            }
        }
    }
    left.zip(right)
}
