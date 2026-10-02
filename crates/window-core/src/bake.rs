//! Baking: a laid-out window's static layer → its net-zero codepoint string
//! (leading spacers, the static glyph, trailing spacers back to the title
//! origin).
//!
//! The vanilla client advances the cursor by `glyph_width + 1` after a bitmap
//! glyph, so the trailing spacers must account for the extra `+1`. After the
//! whole string the cursor is back at `title_origin.x` (net-zero), letting
//! the static chrome and slot text segments compose in any order.

use crate::font::spacers_for;
use crate::geometry::{Point, Rect};

/// Bake the net-zero static string for a window's composite glyph.
///
/// `glyph` is the allocated codepoint character, `bounds` the composite's
/// GUI-space rect, and `title_origin` the surface's title cursor origin. The
/// returned string is `spacers(leading) + glyph + spacers(trailing)` where
/// `leading = bounds.x - title_origin.x` and
/// `trailing = -(leading + bounds.width + 1)`.
pub fn bake_static(glyph: char, bounds: &Rect, title_origin: Point) -> String {
    bake_static_with_advance(glyph, bounds, title_origin, bounds.width.saturating_add(1))
}

/// Bake a net-zero static string using the advance Minecraft measured from the
/// actual provider PNG.
///
/// Unlike [`bake_static`], this remains correct when the composite has fully
/// transparent columns on its right edge.
pub fn bake_static_with_advance(glyph: char, bounds: &Rect, title_origin: Point, glyph_advance: u32) -> String {
    bake_glyph(glyph, glyph_advance, bounds.x - title_origin.x, 0)
}

/// Bake the empty static string for a window with no static draws.
///
/// Returns `""`: there is no glyph and nothing to compensate.
pub fn bake_empty() -> String {
    String::new()
}

/// Bake a HUD static string that claims exactly `line_width` pixels.
///
/// Vanilla actionbar and bossbar names are centered from the component's
/// claimed advance, so HUD statics intentionally end at the right edge of a
/// fixed-width line instead of returning to their origin. Dynamic HUD slot
/// overlays are then appended from that right edge.
pub fn bake_fixed_width_static(glyph: char, bounds: &Rect, line_width: u32) -> String {
    bake_fixed_width_static_with_advance(glyph, bounds, line_width, bounds.width.saturating_add(1))
}

/// Bake a fixed-width HUD string using the advance measured from the provider
/// PNG rather than its nominal bounds width.
pub fn bake_fixed_width_static_with_advance(glyph: char, bounds: &Rect, line_width: u32, glyph_advance: u32) -> String {
    bake_glyph(glyph, glyph_advance, bounds.x, line_width as i32)
}

fn bake_glyph(glyph: char, glyph_advance: u32, leading: i32, target_advance: i32) -> String {
    let mut s = String::new();
    s.push_str(&spacers_for(leading));
    s.push(glyph);
    let advance = leading + glyph_advance as i32;
    let trailing = target_advance - advance;
    s.push_str(&spacers_for(trailing));
    s
}

/// Bake an empty fixed-width HUD line.
pub fn bake_fixed_width_empty(line_width: u32) -> String {
    spacers_for(line_width as i32)
}

#[cfg(test)]
mod tests;
