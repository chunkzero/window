use super::*;

use crate::font::spacer_table;

/// Net advance of a baked string: spacer advances plus the glyph's
/// `width + 1`, given the spacer table for decoding.
fn net_advance(s: &str, glyph: char, glyph_width: i32) -> i32 {
    let table = spacer_table();
    s.chars().map(|c| if c == glyph { glyph_width + 1 } else { table[&(c as u32)] }).sum()
}

#[test]
fn baked_string_is_net_zero() {
    let glyph = '\u{E000}';
    for (x, w) in [(8, 16), (0, 176), (40, 9), (-3, 5)] {
        let bounds = Rect::new(x, 6, w as u32, 8);
        let s = bake_static(glyph, &bounds, Point::new(8, 6));
        assert_eq!(net_advance(&s, glyph, w), 0, "x={x} w={w}");
    }
}

#[test]
fn exact_string_for_known_case() {
    // bounds.x == title_origin.x → no leading spacers. width 16 → trailing
    // must undo 17 (= 16 + 1): spacers_for(-17) = -16, -1.
    let glyph = '\u{E000}';
    let bounds = Rect::new(8, 6, 16, 8);
    let s = bake_static(glyph, &bounds, Point::new(8, 6));
    let table = spacer_table();
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], glyph);
    // Remaining chars sum to -17.
    let trailing: i32 = chars[1..].iter().map(|&c| table[&(c as u32)]).sum();
    assert_eq!(trailing, -17);
    assert_eq!(chars.len(), 3); // glyph + (-16) + (-1)
}

#[test]
fn empty_is_blank() {
    assert_eq!(bake_empty(), "");
}

#[test]
fn fixed_width_static_claims_line_width() {
    let glyph = '\u{E000}';
    let bounds = Rect::new(10, 0, 20, 8);
    let s = bake_fixed_width_static(glyph, &bounds, 100);
    assert_eq!(net_advance(&s, glyph, 20), 100);
}

#[test]
fn measured_advance_handles_transparent_right_padding() {
    let glyph = '\u{E000}';
    let bounds = Rect::new(8, 6, 16, 8);
    let s = bake_static_with_advance(glyph, &bounds, Point::new(8, 6), 10);
    let table = spacer_table();
    let net: i32 = s.chars().map(|character| if character == glyph { 10 } else { table[&(character as u32)] }).sum();
    assert_eq!(net, 0);
}
