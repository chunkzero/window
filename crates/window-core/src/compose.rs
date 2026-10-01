//! Static-layer compositing: expand bitmap/generated draws into a single
//! per-window RGBA composite image.
//!
//! The composite covers the union of every [`Draw`]'s destination rect (in GUI
//! space). Draws are painted back-to-front with straight-alpha "over" blending.
//! Bitmap stretching (9-slice edges/center) uses deterministic
//! nearest-neighbor sampling so identical inputs produce byte-identical output.

use std::collections::BTreeMap;

use image::{ImageEncoder, RgbaImage};

use crate::geometry::{Insets, Rect};
use crate::ir::{Draw, LaidOutWindow, TextureKey};
use crate::{Error, Result};

/// A decoded RGBA8 texture, row-major (`rgba.len() == width * height * 4`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major RGBA8 pixels.
    pub rgba: Vec<u8>,
}

impl Texture {
    /// Construct a fully transparent texture of the given size.
    pub fn transparent(width: u32, height: u32) -> Self {
        Self { width, height, rgba: vec![0; (width as usize) * (height as usize) * 4] }
    }

    /// Decode a PNG byte buffer into an RGBA8 texture.
    pub fn decode_png(bytes: &[u8]) -> Result<Texture> {
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .map_err(|e| Error::Texture { path: String::new(), message: e.to_string() })?;
        let rgba = img.to_rgba8();
        Ok(Texture { width: rgba.width(), height: rgba.height(), rgba: rgba.into_raw() })
    }

    /// Read the RGBA pixel at `(x, y)`; out-of-bounds reads return transparent.
    fn get(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let i = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]]
    }

    /// Alpha-over blend `src` onto the pixel at `(x, y)` (straight alpha).
    fn blend(&mut self, x: u32, y: u32, src: [u8; 4]) {
        if x >= self.width || y >= self.height || src[3] == 0 {
            return;
        }
        let i = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        let dst = [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]];
        let out = over(src, dst);
        self.rgba[i] = out[0];
        self.rgba[i + 1] = out[1];
        self.rgba[i + 2] = out[2];
        self.rgba[i + 3] = out[3];
    }

    /// Encode this texture as a PNG. Output is deterministic for identical input.
    pub fn encode_png(&self) -> Result<Vec<u8>> {
        // Round-trip through RgbaImage so the encoder sees a validated buffer.
        let img = RgbaImage::from_raw(self.width, self.height, self.rgba.clone()).ok_or_else(|| Error::Texture {
            path: String::new(),
            message: "texture buffer size does not match dimensions".into(),
        })?;
        let mut out = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut out);
        encoder
            .write_image(img.as_raw(), self.width, self.height, image::ExtendedColorType::Rgba8)
            .map_err(|e| Error::Texture { path: String::new(), message: e.to_string() })?;
        Ok(out)
    }
}

/// Straight-alpha "over" compositing of `src` onto `dst` (integer math).
///
/// `out_a = src_a + dst_a * (1 - src_a)`; color channels are the
/// alpha-weighted average, rounded to nearest. Fully transparent results have
/// zeroed color channels for deterministic, canonical output.
fn over(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let sa = src[3] as u32;
    let da = dst[3] as u32;
    // out_a = sa + da*(255 - sa)/255, rounded.
    let out_a = sa + (da * (255 - sa) + 127) / 255;
    if out_a == 0 {
        return [0, 0, 0, 0];
    }
    let mut out = [0u8; 4];
    for c in 0..3 {
        // numerator = src_c*sa*255 + dst_c*da*(255 - sa)
        let num = (src[c] as u32) * sa * 255 + (dst[c] as u32) * da * (255 - sa);
        let den = out_a * 255;
        out[c] = ((num + den / 2) / den) as u8;
    }
    out[3] = out_a as u8;
    out
}

/// The composite result for one window: the rendered image and the GUI-space
/// rect it occupies. `bounds` may have a negative origin.
#[derive(Clone, Debug)]
pub struct Composite {
    /// The rendered RGBA image; empty (0×0) when there were no draws.
    pub texture: Texture,
    /// GUI-space rect covered by the composite (its origin is the image's
    /// top-left in GUI space).
    pub bounds: Rect,
    /// Whether any draw was rendered (false ⇒ no static glyph for the window).
    pub has_content: bool,
}

/// Compose a window's static draws into a single image.
///
/// Returns a [`Composite`] whose `bounds` is the union of every draw's
/// destination. With no draws, `has_content` is false and the texture is empty.
pub fn compose_window(window: &LaidOutWindow, textures: &BTreeMap<String, Texture>) -> Result<Composite> {
    compose_draws(&window.draws, &window.name, textures)
}

/// Compose an explicit slice of draws (the [`compose_window`] core, exposed for
/// testing and the pipeline).
pub fn compose_draws(draws: &[Draw], window_name: &str, textures: &BTreeMap<String, Texture>) -> Result<Composite> {
    let Some(bounds) = union_bounds(draws) else {
        return Ok(Composite {
            texture: Texture::transparent(0, 0),
            bounds: Rect::new(0, 0, 0, 0),
            has_content: false,
        });
    };

    let mut canvas = Texture::transparent(bounds.width, bounds.height);
    for draw in draws {
        let dest = draw.dest();
        // Destination, translated into canvas-local coordinates.
        let local = Rect::new(dest.x - bounds.x, dest.y - bounds.y, dest.width, dest.height);
        match draw {
            Draw::Sprite { texture, .. } => {
                let texture = lookup(textures, texture, window_name)?;
                blit_sprite(&mut canvas, texture, &local);
            }
            Draw::NineSlice { texture, insets, .. } => {
                let texture = lookup(textures, texture, window_name)?;
                blit_nine_slice(&mut canvas, texture, insets, &local);
            }
            Draw::Generated { style, .. } => {
                let texture = crate::raster::render(style, dest.size())?;
                blit_sprite(&mut canvas, &texture, &local);
            }
        }
    }

    Ok(Composite { texture: canvas, bounds, has_content: true })
}

/// The union of every draw's destination rect, or `None` if there are no draws.
fn union_bounds(draws: &[Draw]) -> Option<Rect> {
    let mut iter = draws.iter();
    let first = *iter.next()?.dest();
    Some(iter.fold(first, |acc, d| acc.union(d.dest())))
}

/// Resolve a texture key to a decoded texture, erroring if missing.
fn lookup<'a>(textures: &'a BTreeMap<String, Texture>, key: &TextureKey, window: &str) -> Result<&'a Texture> {
    textures.get(&key.0).ok_or_else(|| Error::Texture {
        path: key.0.clone(),
        message: format!("referenced by window `{window}` but not provided"),
    })
}

/// 1:1 blit of `src` into `dest` (top-left aligned). Pixels of `src` beyond
/// `dest`'s size are clipped; `dest` larger than `src` leaves the remainder
/// untouched.
fn blit_sprite(canvas: &mut Texture, src: &Texture, dest: &Rect) {
    let w = dest.width.min(src.width);
    let h = dest.height.min(src.height);
    for sy in 0..h {
        for sx in 0..w {
            let px = src.get(sx, sy);
            let dx = dest.x + sx as i32;
            let dy = dest.y + sy as i32;
            if dx >= 0 && dy >= 0 {
                canvas.blend(dx as u32, dy as u32, px);
            }
        }
    }
}

/// 9-slice expansion of `src` over `dest`.
///
/// The source is split into 9 regions by `insets`: the four corners are copied
/// 1:1, the four edges are stretched along their long axis, and the center is
/// stretched on both axes. Stretching uses nearest-neighbor sampling.
///
/// Degenerate `dest` (smaller than the combined corner widths/heights) clamps:
/// the first corner is given as much room as fits, the opposite corner takes
/// the remainder, and edges/center collapse to zero width/height. No pixel is
/// written outside `dest`.
fn blit_nine_slice(canvas: &mut Texture, src: &Texture, insets: &Insets, dest: &Rect) {
    // Source column/row boundaries.
    let s_left = insets.left.min(src.width);
    let s_right = insets.right.min(src.width.saturating_sub(s_left));
    let s_top = insets.top.min(src.height);
    let s_bottom = insets.bottom.min(src.height.saturating_sub(s_top));
    let s_mid_w = src.width.saturating_sub(s_left + s_right);
    let s_mid_h = src.height.saturating_sub(s_top + s_bottom);

    // Destination column widths: corners take priority, center absorbs slack.
    let (d_left, d_right) = split_corners(dest.width, s_left, s_right);
    let d_mid_w = dest.width.saturating_sub(d_left + d_right);
    let (d_top, d_bottom) = split_corners(dest.height, s_top, s_bottom);
    let d_mid_h = dest.height.saturating_sub(d_top + d_bottom);

    // Column layout: (dest x offset, dest width, src x offset, src width).
    let cols = [
        (0, d_left, 0, s_left),
        (d_left, d_mid_w, s_left, s_mid_w),
        (d_left + d_mid_w, d_right, s_left + s_mid_w, s_right),
    ];
    let rows = [
        (0, d_top, 0, s_top),
        (d_top, d_mid_h, s_top, s_mid_h),
        (d_top + d_mid_h, d_bottom, s_top + s_mid_h, s_bottom),
    ];

    for (dy0, dh, sy0, sh) in rows {
        for &(dx0, dw, sx0, sw) in &cols {
            blit_region(canvas, src, dest, dx0, dy0, dw, dh, sx0, sy0, sw, sh);
        }
    }
}

/// Split `total` destination pixels between two corners of source size `a`/`b`.
/// Corners keep their native size when there's room; otherwise the first corner
/// is filled before the second (the center collapses to zero).
fn split_corners(total: u32, a: u32, b: u32) -> (u32, u32) {
    if a + b <= total {
        (a, b)
    } else if a >= total {
        (total, 0)
    } else {
        // a < total < a + b
        (a, total - a)
    }
}

/// Blit one nine-slice region, stretching `sw×sh` source pixels into `dw×dh`
/// destination pixels with nearest-neighbor sampling. No-op if any extent is 0.
#[allow(clippy::too_many_arguments)]
fn blit_region(
    canvas: &mut Texture,
    src: &Texture,
    dest: &Rect,
    dx0: u32,
    dy0: u32,
    dw: u32,
    dh: u32,
    sx0: u32,
    sy0: u32,
    sw: u32,
    sh: u32,
) {
    if dw == 0 || dh == 0 || sw == 0 || sh == 0 {
        return;
    }
    for ry in 0..dh {
        // Nearest-neighbor: map destination row to a source row.
        let sy = sy0 + nearest(ry, dh, sh);
        let dy = dest.y + (dy0 + ry) as i32;
        if dy < 0 {
            continue;
        }
        for rx in 0..dw {
            let sx = sx0 + nearest(rx, dw, sw);
            let dx = dest.x + (dx0 + rx) as i32;
            if dx < 0 {
                continue;
            }
            canvas.blend(dx as u32, dy as u32, src.get(sx, sy));
        }
    }
}

/// Map destination index `i` of `dst_len` onto a source index in `0..src_len`
/// via nearest-neighbor: `floor(i * src_len / dst_len)`.
fn nearest(i: u32, dst_len: u32, src_len: u32) -> u32 {
    debug_assert!(dst_len > 0 && src_len > 0);
    ((i as u64 * src_len as u64) / dst_len as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a solid-color texture.
    fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Texture {
        Texture { width, height, rgba: rgba.iter().copied().cycle().take((width * height * 4) as usize).collect() }
    }

    /// Build a texture from an explicit list of pixels (row-major).
    fn from_pixels(width: u32, height: u32, pixels: &[[u8; 4]]) -> Texture {
        assert_eq!(pixels.len() as u32, width * height);
        Texture { width, height, rgba: pixels.iter().flatten().copied().collect() }
    }

    fn pixel(t: &Texture, x: u32, y: u32) -> [u8; 4] {
        t.get(x, y)
    }

    #[test]
    fn sprite_blit_is_one_to_one() {
        let mut textures = BTreeMap::new();
        let src = from_pixels(2, 2, &[[10, 20, 30, 255], [40, 50, 60, 255], [70, 80, 90, 255], [100, 110, 120, 255]]);
        textures.insert("s.png".to_string(), src);
        let draws = vec![Draw::Sprite { texture: TextureKey("s.png".into()), dest: Rect::new(1, 1, 2, 2) }];
        let c = compose_draws(&draws, "w", &textures).unwrap();
        assert!(c.has_content);
        assert_eq!(c.bounds, Rect::new(1, 1, 2, 2));
        assert_eq!(pixel(&c.texture, 0, 0), [10, 20, 30, 255]);
        assert_eq!(pixel(&c.texture, 1, 0), [40, 50, 60, 255]);
        assert_eq!(pixel(&c.texture, 0, 1), [70, 80, 90, 255]);
        assert_eq!(pixel(&c.texture, 1, 1), [100, 110, 120, 255]);
    }

    #[test]
    fn alpha_over_blends_straight_alpha() {
        // Opaque red base, then 50% green on top.
        let opaque = over([0, 255, 0, 128], [255, 0, 0, 255]);
        // out_a = 128 + 255*(255-128)/255 = 128 + 127 = 255.
        assert_eq!(opaque[3], 255);
        // R: src_r=0, dst_r=255. num = 0 + 255*255*127; den = 255*255 -> 127.
        assert_eq!(opaque[0], 127);
        // G: src_g=255. num = 255*128*255; den = 255*255 -> 128.
        assert_eq!(opaque[1], 128);
        assert_eq!(opaque[2], 0);
    }

    #[test]
    fn over_onto_transparent_keeps_src() {
        let out = over([10, 20, 30, 200], [0, 0, 0, 0]);
        assert_eq!(out, [10, 20, 30, 200]);
    }

    #[test]
    fn fully_transparent_src_is_noop() {
        let mut textures = BTreeMap::new();
        textures.insert("base.png".to_string(), solid(2, 2, [1, 2, 3, 255]));
        textures.insert("clear.png".to_string(), solid(2, 2, [9, 9, 9, 0]));
        let draws = vec![
            Draw::Sprite { texture: TextureKey("base.png".into()), dest: Rect::new(0, 0, 2, 2) },
            Draw::Sprite { texture: TextureKey("clear.png".into()), dest: Rect::new(0, 0, 2, 2) },
        ];
        let c = compose_draws(&draws, "w", &textures).unwrap();
        assert_eq!(pixel(&c.texture, 0, 0), [1, 2, 3, 255]);
    }

    #[test]
    fn union_bounds_with_negative_origin() {
        let mut textures = BTreeMap::new();
        textures.insert("a.png".to_string(), solid(2, 2, [255, 0, 0, 255]));
        textures.insert("b.png".to_string(), solid(2, 2, [0, 255, 0, 255]));
        let draws = vec![
            Draw::Sprite { texture: TextureKey("a.png".into()), dest: Rect::new(-3, -1, 2, 2) },
            Draw::Sprite { texture: TextureKey("b.png".into()), dest: Rect::new(2, 3, 2, 2) },
        ];
        let c = compose_draws(&draws, "w", &textures).unwrap();
        // x from -3..4 -> width 7; y from -1..5 -> height 6.
        assert_eq!(c.bounds, Rect::new(-3, -1, 7, 6));
        // a.png lands at local (0,0)..(2,2).
        assert_eq!(pixel(&c.texture, 0, 0), [255, 0, 0, 255]);
        // b.png at dest (2,3) -> local (5,4).
        assert_eq!(pixel(&c.texture, 5, 4), [0, 255, 0, 255]);
        // empty corner is transparent.
        assert_eq!(pixel(&c.texture, 6, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn empty_draws_have_no_content() {
        let textures = BTreeMap::new();
        let c = compose_draws(&[], "w", &textures).unwrap();
        assert!(!c.has_content);
        assert_eq!(c.bounds, Rect::new(0, 0, 0, 0));
        assert_eq!(c.texture.width, 0);
        assert_eq!(c.texture.height, 0);
    }

    #[test]
    fn missing_texture_errors() {
        let textures = BTreeMap::new();
        let draws = vec![Draw::Sprite { texture: TextureKey("nope.png".into()), dest: Rect::new(0, 0, 2, 2) }];
        assert!(compose_draws(&draws, "w", &textures).is_err());
    }

    #[test]
    fn nine_slice_3x3_inset_stretched_larger() {
        // 3×3 source: distinct corner colors, edges, and a center.
        // Layout (1px insets all around):
        //   C T C
        //   L M R
        //   C B C
        let c = [255, 0, 0, 255]; // corners
        let t = [0, 255, 0, 255]; // top edge
        let l = [0, 0, 255, 255]; // left edge
        let m = [255, 255, 0, 255]; // center
        let r = [255, 0, 255, 255]; // right edge
        let b = [0, 255, 255, 255]; // bottom edge
        let src = from_pixels(3, 3, &[c, t, c, l, m, r, c, b, c]);
        let mut textures = BTreeMap::new();
        textures.insert("frame.png".to_string(), src);
        let draws = vec![Draw::NineSlice {
            texture: TextureKey("frame.png".into()),
            insets: Insets::uniform(1),
            dest: Rect::new(0, 0, 5, 5),
        }];
        let comp = compose_draws(&draws, "w", &textures).unwrap();
        let img = &comp.texture;
        // Corners stay 1:1 at the four extremes.
        assert_eq!(pixel(img, 0, 0), c);
        assert_eq!(pixel(img, 4, 0), c);
        assert_eq!(pixel(img, 0, 4), c);
        assert_eq!(pixel(img, 4, 4), c);
        // Top edge spans the middle 3 columns of row 0.
        assert_eq!(pixel(img, 1, 0), t);
        assert_eq!(pixel(img, 2, 0), t);
        assert_eq!(pixel(img, 3, 0), t);
        // Left edge spans middle rows of column 0.
        assert_eq!(pixel(img, 0, 1), l);
        assert_eq!(pixel(img, 0, 3), l);
        // Right edge.
        assert_eq!(pixel(img, 4, 2), r);
        // Bottom edge.
        assert_eq!(pixel(img, 2, 4), b);
        // Center fills the 3×3 interior.
        assert_eq!(pixel(img, 2, 2), m);
        assert_eq!(pixel(img, 1, 3), m);
    }

    #[test]
    fn nine_slice_degenerate_clamps_corners() {
        // Source corners are 2px; dest is only 3px wide → left corner 2, right 1.
        let src = solid(6, 6, [100, 100, 100, 255]);
        let mut textures = BTreeMap::new();
        textures.insert("f.png".to_string(), src);
        let draws = vec![Draw::NineSlice {
            texture: TextureKey("f.png".into()),
            insets: Insets::uniform(2),
            dest: Rect::new(0, 0, 3, 3),
        }];
        let comp = compose_draws(&draws, "w", &textures).unwrap();
        // No panic and image is the requested size; all pixels written.
        assert_eq!(comp.texture.width, 3);
        assert_eq!(comp.texture.height, 3);
        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(pixel(&comp.texture, x, y), [100, 100, 100, 255]);
            }
        }
    }

    #[test]
    fn png_round_trip_is_deterministic() {
        let t = from_pixels(2, 1, &[[1, 2, 3, 255], [4, 5, 6, 128]]);
        let a = t.encode_png().unwrap();
        let b = t.encode_png().unwrap();
        assert_eq!(a, b);
        let decoded = Texture::decode_png(&a).unwrap();
        assert_eq!(decoded, t);
    }
}
