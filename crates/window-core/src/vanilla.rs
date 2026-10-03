//! Vanilla font metrics for every character the shifted label fonts provide,
//! used for label measurement at build time and exported in the manifest for
//! runtime alignment.
//!
//! An advance is the cursor movement after drawing a character. For vanilla's
//! `ascii.png` bitmap glyphs, Minecraft computes that as detected glyph ink
//! width plus the 1px inter-glyph gap. The space character is special: vanilla
//! supplies it from a `space` provider with a 4px advance and no ink.
//!
//! The tables cover all printable ASCII (`0x20`–`0x7E`), the remaining
//! `ascii.png` cells, the mcutils-style Minecraft small-text Unicode letters and
//! common UI symbols that vanilla renders from `nonlatin_european.png` and
//! `accented.png`, and the vanilla space-provider entries (`' '`, `U+200C`).
//! Every character the shifted fonts provide has exactly one entry here.
//! Bitmap advances are measured from the rightmost opaque cell pixels in the
//! identical bitmap sheets shipped by Minecraft 26.1.2 and 26.2; getting even
//! one glyph wrong causes each net-zero title segment to leak cursor movement
//! into every segment after it.

/// The advance table: `(char, advance)` pairs covering printable ASCII,
/// sorted ascending by codepoint. `advance == glyph_width + 1`.
///
/// Glyph widths are the canonical vanilla `ascii.png` widths; advances add the
/// 1px inter-glyph gap, except `' '` which is a no-ink glyph with a 4px advance.
/// Narrow glyphs include `'!','\'','.',',',':',';','i','|'`→2,
/// `'l','`'`→3, and wide glyphs `'@','~'`→7. Most letters and digits are
/// 6.
const ADVANCES: &[(char, u32)] = &[
    (' ', 4),
    ('!', 2),
    ('"', 4),
    ('#', 6),
    ('$', 6),
    ('%', 6),
    ('&', 6),
    ('\'', 2),
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

/// Metrics for every non-ASCII character the shifted fonts provide, sorted
/// ascending by codepoint: the extra `ascii.png` cells, the small-caps letters
/// and UI markers from `nonlatin_european.png` and `accented.png`, and the
/// zero-advance `U+200C` from the space provider.
///
/// Bitmap advance is the rightmost opaque cell column's zero-based
/// x-coordinate plus two (one to turn it into a width and one for the
/// inter-glyph gap), so transparent left padding still contributes to the
/// cursor.
const EXTRA_ADVANCES: &[(char, u32)] = &[
    ('\u{00A3}', 6), // £
    ('\u{00AA}', 5), // ª
    ('\u{00AB}', 7), // «
    ('\u{00AC}', 6), // ¬
    ('\u{00B0}', 5), // °
    ('\u{00B1}', 6), // ±
    ('\u{00B2}', 5), // ²
    ('\u{00B7}', 2), // ·
    ('\u{00BA}', 5), // º
    ('\u{00BB}', 7), // »
    ('\u{00D7}', 6), // ×
    ('\u{00F7}', 6), // ÷
    ('\u{0192}', 6), // ƒ
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
    ('\u{200C}', 0), // zero-width non-joiner
    ('\u{207F}', 5), // ⁿ
    ('\u{2205}', 8), // ∅
    ('\u{2208}', 6), // ∈
    ('\u{2219}', 6), // ∙
    ('\u{221A}', 7), // √
    ('\u{2248}', 7), // ≈
    ('\u{2261}', 7), // ≡
    ('\u{2264}', 6), // ≤
    ('\u{2265}', 6), // ≥
    ('\u{2320}', 8), // ⌠
    ('\u{2321}', 5), // ⌡
    ('\u{2500}', 9), // ─
    ('\u{2502}', 6), // │
    ('\u{250C}', 9), // ┌
    ('\u{2510}', 6), // ┐
    ('\u{2514}', 9), // └
    ('\u{2518}', 6), // ┘
    ('\u{251C}', 9), // ├
    ('\u{2524}', 6), // ┤
    ('\u{252C}', 9), // ┬
    ('\u{2534}', 9), // ┴
    ('\u{253C}', 9), // ┼
    ('\u{2550}', 9), // ═
    ('\u{2551}', 8), // ║
    ('\u{2552}', 9), // ╒
    ('\u{2553}', 9), // ╓
    ('\u{2554}', 9), // ╔
    ('\u{2555}', 6), // ╕
    ('\u{2556}', 8), // ╖
    ('\u{2557}', 8), // ╗
    ('\u{2558}', 9), // ╘
    ('\u{2559}', 9), // ╙
    ('\u{255A}', 9), // ╚
    ('\u{255B}', 6), // ╛
    ('\u{255C}', 8), // ╜
    ('\u{255D}', 8), // ╝
    ('\u{255E}', 9), // ╞
    ('\u{255F}', 9), // ╟
    ('\u{2560}', 9), // ╠
    ('\u{2561}', 6), // ╡
    ('\u{2562}', 8), // ╢
    ('\u{2563}', 8), // ╣
    ('\u{2564}', 9), // ╤
    ('\u{2565}', 9), // ╥
    ('\u{2566}', 9), // ╦
    ('\u{2567}', 9), // ╧
    ('\u{2568}', 9), // ╨
    ('\u{2569}', 9), // ╩
    ('\u{256A}', 9), // ╪
    ('\u{256B}', 9), // ╫
    ('\u{256C}', 9), // ╬
    ('\u{2580}', 9), // ▀
    ('\u{2584}', 9), // ▄
    ('\u{2588}', 9), // █
    ('\u{258C}', 5), // ▌
    ('\u{2590}', 9), // ▐
    ('\u{2591}', 8), // ░
    ('\u{2592}', 9), // ▒
    ('\u{2593}', 9), // ▓
    ('\u{25A0}', 6), // ■
    ('\u{25B2}', 6), // ▲
    ('\u{25BC}', 6), // ▼
    ('\u{25C6}', 6), // ◆
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
    EXTRA_ADVANCES.binary_search_by(|&(probe, _)| probe.cmp(&c)).ok().map(|idx| EXTRA_ADVANCES[idx].1)
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
/// Yields every `(char, advance)` pair, sorted ascending by character.
pub fn advances() -> impl Iterator<Item = (char, u32)> {
    ADVANCES.iter().copied().chain(EXTRA_ADVANCES.iter().copied())
}

/// The full visible-ink width table, for export into the manifest's
/// `text_glyph_widths`.
///
/// Yields every `(char, glyph_width)` pair, sorted ascending by character.
pub fn glyph_widths() -> impl Iterator<Item = (char, u32)> {
    advances().map(|(c, _)| (c, glyph_width(c).expect("advance table entry has a glyph width")))
}

#[cfg(test)]
mod tests;
