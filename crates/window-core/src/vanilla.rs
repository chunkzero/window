//! Vanilla font metrics for the default font's ASCII range, used for label
//! measurement at build time and exported in the manifest for runtime
//! alignment.
//!
//! An advance is the cursor movement after drawing a character. For vanilla's
//! `ascii.png` bitmap glyphs, Minecraft computes that as detected glyph ink
//! width plus the 1px inter-glyph gap. The space character is special: vanilla
//! supplies it from a `space` provider with a 4px advance and no ink.
//!
//! The table covers all printable ASCII (`0x20`–`0x7E`) using the glyph pixel
//! widths baked into vanilla's `ascii.png`, plus the vanilla space-provider
//! override for `' '`. It also includes the mcutils-style Minecraft small-text
//! Unicode letters and common UI symbols that vanilla renders from
//! `nonlatin_european.png` and `accented.png`, so generated shifted fonts can
//! align that style exactly. These advances are measured from the rightmost
//! opaque cell pixels in Minecraft 26.1.2's bitmap sheets; getting even one
//! glyph wrong causes each net-zero title segment to leak cursor movement into
//! every segment after it.

/// The advance table: `(char, advance)` pairs covering printable ASCII,
/// sorted ascending by codepoint. `advance == glyph_width + 1`.
///
/// Glyph widths are the canonical vanilla `ascii.png` widths; advances add the
/// 1px inter-glyph gap, except `' '` which is a no-ink glyph with a 4px advance.
/// Narrow glyphs include `'!','.',',',':',';','i','|'`→2,
/// `'l','\'','`'`→3, and wide glyphs `'@','~'`→7. Most letters and digits are
/// 6.
const ADVANCES: &[(char, u32)] = &[
    (' ', 4),
    ('!', 2),
    ('"', 4),
    ('#', 6),
    ('$', 6),
    ('%', 6),
    ('&', 6),
    ('\'', 3),
    ('(', 4),
    (')', 4),
    ('*', 4),
    ('+', 6),
    (',', 2),
    ('-', 6),
    ('.', 2),
    ('/', 6),
    ('0', 6),
    ('1', 6),
    ('2', 6),
    ('3', 6),
    ('4', 6),
    ('5', 6),
    ('6', 6),
    ('7', 6),
    ('8', 6),
    ('9', 6),
    (':', 2),
    (';', 2),
    ('<', 5),
    ('=', 6),
    ('>', 5),
    ('?', 6),
    ('@', 7),
    ('A', 6),
    ('B', 6),
    ('C', 6),
    ('D', 6),
    ('E', 6),
    ('F', 6),
    ('G', 6),
    ('H', 6),
    ('I', 4),
    ('J', 6),
    ('K', 6),
    ('L', 6),
    ('M', 6),
    ('N', 6),
    ('O', 6),
    ('P', 6),
    ('Q', 6),
    ('R', 6),
    ('S', 6),
    ('T', 6),
    ('U', 6),
    ('V', 6),
    ('W', 6),
    ('X', 6),
    ('Y', 6),
    ('Z', 6),
    ('[', 4),
    ('\\', 6),
    (']', 4),
    ('^', 6),
    ('_', 6),
    ('`', 3),
    ('a', 6),
    ('b', 6),
    ('c', 6),
    ('d', 6),
    ('e', 6),
    ('f', 5),
    ('g', 6),
    ('h', 6),
    ('i', 2),
    ('j', 6),
    ('k', 5),
    ('l', 3),
    ('m', 6),
    ('n', 6),
    ('o', 6),
    ('p', 6),
    ('q', 6),
    ('r', 6),
    ('s', 6),
    ('t', 4),
    ('u', 6),
    ('v', 6),
    ('w', 6),
    ('x', 6),
    ('y', 6),
    ('z', 6),
    ('{', 4),
    ('|', 2),
    ('}', 4),
    ('~', 7),
];

/// Extra vanilla bitmap glyph metrics used by Minecraft's common small-text
/// converter and Window's standard UI status markers.
///
/// The small-cap glyphs use a 6px advance except for narrow `ɪ`, whose advance
/// is 4px in Minecraft 26.1.2. Bitmap advance is the rightmost opaque cell's
/// zero-based x-coordinate plus two (one to turn it into a width and one for
/// the inter-glyph gap), so transparent left padding still contributes to the
/// cursor. The symbol advances below come from the same built-in bitmap sheet.
const SMALL_TEXT_ADVANCES: &[(char, u32)] = &[
    ('\u{00B7}', 2), // ·
    ('\u{00D7}', 6), // ×
    ('\u{01EB}', 6), // ǫ
    ('\u{0262}', 6), // ɢ
    ('\u{026A}', 4), // ɪ
    ('\u{0274}', 6), // ɴ
    ('\u{0280}', 6), // ʀ
    ('\u{028F}', 6), // ʏ
    ('\u{0299}', 6), // ʙ
    ('\u{029C}', 6), // ʜ
    ('\u{029F}', 6), // ʟ
    ('\u{0455}', 6), // ѕ
    ('\u{1D00}', 6), // ᴀ
    ('\u{1D04}', 6), // ᴄ
    ('\u{1D05}', 6), // ᴅ
    ('\u{1D07}', 6), // ᴇ
    ('\u{1D0A}', 6), // ᴊ
    ('\u{1D0B}', 6), // ᴋ
    ('\u{1D0D}', 6), // ᴍ
    ('\u{1D0F}', 6), // ᴏ
    ('\u{1D18}', 6), // ᴘ
    ('\u{1D1B}', 6), // ᴛ
    ('\u{1D1C}', 6), // ᴜ
    ('\u{1D20}', 6), // ᴠ
    ('\u{1D21}', 6), // ᴡ
    ('\u{1D22}', 6), // ᴢ
    ('\u{25B2}', 6), // ▲
    ('\u{25BC}', 6), // ▼
    ('\u{25C6}', 8), // ◆
    ('\u{25CF}', 5), // ●
    ('\u{A730}', 6), // ꜰ
    ('\u{A731}', 6), // ꜱ
];

/// Minecraft's bitmap-font inter-glyph gap.
pub const INTER_GLYPH_GAP: u32 = 1;

/// Extra cursor advance added by Minecraft's bold decoration for default-font
/// glyphs.
pub const BOLD_ADVANCE: u32 = 1;

/// The advance width of `c` in pixels, or `None` if unknown (callers should
/// treat unknown characters as [`FALLBACK_ADVANCE`] and warn).
pub fn advance(c: char) -> Option<u32> {
    // The table is sorted by char, so a binary search keeps lookups cheap.
    let ascii = ADVANCES.binary_search_by(|&(probe, _)| probe.cmp(&c)).ok().map(|idx| ADVANCES[idx].1);
    if ascii.is_some() {
        return ascii;
    }
    SMALL_TEXT_ADVANCES.binary_search_by(|&(probe, _)| probe.cmp(&c)).ok().map(|idx| SMALL_TEXT_ADVANCES[idx].1)
}

/// Advance used for characters outside the table.
pub const FALLBACK_ADVANCE: u32 = 6;

/// Visible ink width used for characters outside the table.
pub const FALLBACK_GLYPH_WIDTH: u32 = FALLBACK_ADVANCE - INTER_GLYPH_GAP;

/// Visible ink width of `c` in pixels, or `None` if unknown.
pub fn glyph_width(c: char) -> Option<u32> {
    if c == ' ' {
        return Some(0);
    }
    advance(c).map(|advance| advance.saturating_sub(INTER_GLYPH_GAP))
}

/// Total advance width of `s` (unknown chars use [`FALLBACK_ADVANCE`]).
pub fn text_width(s: &str) -> u32 {
    s.chars().map(|c| advance(c).unwrap_or(FALLBACK_ADVANCE)).sum()
}

/// Total advance width of `s` when bold is active.
///
/// Minecraft's default glyphs add a 1px bold offset to cursor advance. Italic,
/// underline, strikethrough, and obfuscated do not change advance math.
pub fn bold_text_width(s: &str) -> u32 {
    s.chars().map(|c| advance(c).unwrap_or(FALLBACK_ADVANCE) + BOLD_ADVANCE).sum()
}

/// Visible ink width of `s`.
///
/// This differs from [`text_width`] by not counting the final inter-glyph gap
/// and by treating trailing spaces as no-ink. It is the width that should be
/// centered inside visual boxes while [`text_width`] remains the cursor advance
/// used for net-zero spacer math.
pub fn text_visible_width(s: &str) -> u32 {
    let mut cursor = 0;
    let mut visible = 0;
    for c in s.chars() {
        let advance = advance(c).unwrap_or(FALLBACK_ADVANCE);
        let glyph = glyph_width(c).unwrap_or(FALLBACK_GLYPH_WIDTH);
        if glyph > 0 {
            visible = visible.max(cursor + glyph);
        }
        cursor += advance;
    }
    visible
}

/// Visible ink width of `s` when bold is active.
pub fn bold_text_visible_width(s: &str) -> u32 {
    let mut cursor = 0;
    let mut visible = 0;
    for c in s.chars() {
        let advance = advance(c).unwrap_or(FALLBACK_ADVANCE) + BOLD_ADVANCE;
        let glyph = glyph_width(c).unwrap_or(FALLBACK_GLYPH_WIDTH);
        let glyph = if glyph > 0 { glyph + BOLD_ADVANCE } else { glyph };
        if glyph > 0 {
            visible = visible.max(cursor + glyph);
        }
        cursor += advance;
    }
    visible
}

/// The full table, for export into the manifest's `text_advances`.
///
/// Yields every printable-ASCII `(char, advance)` pair, sorted ascending by
/// character.
pub fn advances() -> impl Iterator<Item = (char, u32)> {
    ADVANCES.iter().copied().chain(SMALL_TEXT_ADVANCES.iter().copied())
}

/// The full visible-ink width table, for export into the manifest's
/// `text_glyph_widths`.
///
/// Yields every printable-ASCII `(char, glyph_width)` pair, sorted ascending by
/// character.
pub fn glyph_widths() -> impl Iterator<Item = (char, u32)> {
    advances().map(|(c, _)| (c, glyph_width(c).expect("advance table entry has a glyph width")))
}

#[cfg(test)]
mod tests;
