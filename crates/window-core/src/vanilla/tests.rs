use super::*;

#[test]
fn table_is_sorted_and_covers_printable_ascii() {
    // Strictly ascending, no duplicates.
    for pair in ADVANCES.windows(2) {
        assert!(pair[0].0 < pair[1].0, "table not sorted at {pair:?}");
    }
    for pair in SMALL_TEXT_ADVANCES.windows(2) {
        assert!(pair[0].0 < pair[1].0, "small-text table not sorted at {pair:?}");
    }
    // Every printable ASCII codepoint is present.
    for code in 0x20u8..=0x7E {
        let c = code as char;
        assert!(advance(c).is_some(), "missing advance for {c:?}");
    }
    assert_eq!(ADVANCES.len(), (0x7E - 0x20 + 1) as usize);
}

#[test]
fn canonical_advances() {
    assert_eq!(advance(' '), Some(4));
    assert_eq!(advance('!'), Some(2));
    assert_eq!(advance('i'), Some(2));
    assert_eq!(advance('l'), Some(3));
    assert_eq!(advance('t'), Some(4));
    assert_eq!(advance('I'), Some(4));
    assert_eq!(advance('f'), Some(5));
    assert_eq!(advance('k'), Some(5));
    assert_eq!(advance('A'), Some(6));
    assert_eq!(advance('@'), Some(7));
    assert_eq!(advance('~'), Some(7));
    assert_eq!(advance('\u{1D0D}'), Some(6)); // ᴍ
    assert_eq!(advance('\u{0274}'), Some(6)); // ɴ
    assert_eq!(advance('\u{029C}'), Some(6)); // ʜ
    assert_eq!(advance('\u{1D21}'), Some(6)); // ᴡ
    assert_eq!(advance('\u{026A}'), Some(4)); // ɪ
    assert_eq!(advance('\u{01EB}'), Some(6)); // ǫ
    assert_eq!(advance('·'), Some(2));
    assert_eq!(advance('×'), Some(6));
    assert_eq!(advance('▲'), Some(6));
    assert_eq!(advance('▼'), Some(6));
    assert_eq!(advance('◆'), Some(8));
    assert_eq!(advance('●'), Some(5));
    assert_eq!(advance('\u{A731}'), Some(6)); // ꜱ
}

#[test]
fn canonical_glyph_widths() {
    assert_eq!(glyph_width(' '), Some(0));
    assert_eq!(glyph_width('!'), Some(1));
    assert_eq!(glyph_width('i'), Some(1));
    assert_eq!(glyph_width('l'), Some(2));
    assert_eq!(glyph_width('t'), Some(3));
    assert_eq!(glyph_width('I'), Some(3));
    assert_eq!(glyph_width('f'), Some(4));
    assert_eq!(glyph_width('k'), Some(4));
    assert_eq!(glyph_width('A'), Some(5));
    assert_eq!(glyph_width('@'), Some(6));
    assert_eq!(glyph_width('~'), Some(6));
    assert_eq!(glyph_width('\u{1D0D}'), Some(5)); // ᴍ
    assert_eq!(glyph_width('\u{0274}'), Some(5)); // ɴ
    assert_eq!(glyph_width('\u{029C}'), Some(5)); // ʜ
    assert_eq!(glyph_width('\u{1D21}'), Some(5)); // ᴡ
    assert_eq!(glyph_width('\u{026A}'), Some(3)); // ɪ
    assert_eq!(glyph_width('\u{01EB}'), Some(5)); // ǫ
    assert_eq!(glyph_width('\u{00D7}'), Some(5)); // ×
    assert_eq!(glyph_width('·'), Some(1));
    assert_eq!(glyph_width('▲'), Some(5));
    assert_eq!(glyph_width('▼'), Some(5));
    assert_eq!(glyph_width('◆'), Some(7));
    assert_eq!(glyph_width('●'), Some(4));
    assert_eq!(glyph_width('\u{A731}'), Some(5)); // ꜱ
}

#[test]
fn small_cap_advances_match_minecraft_26_1_2_bitmaps() {
    let expected = [
        ('\u{1D00}', 6), // ᴀ
        ('\u{0299}', 6), // ʙ
        ('\u{1D04}', 6), // ᴄ
        ('\u{1D05}', 6), // ᴅ
        ('\u{1D07}', 6), // ᴇ
        ('\u{A730}', 6), // ꜰ
        ('\u{0262}', 6), // ɢ
        ('\u{029C}', 6), // ʜ
        ('\u{026A}', 4), // ɪ
        ('\u{1D0A}', 6), // ᴊ
        ('\u{1D0B}', 6), // ᴋ
        ('\u{029F}', 6), // ʟ
        ('\u{1D0D}', 6), // ᴍ
        ('\u{0274}', 6), // ɴ
        ('\u{1D0F}', 6), // ᴏ
        ('\u{1D18}', 6), // ᴘ
        ('\u{01EB}', 6), // ǫ
        ('\u{0280}', 6), // ʀ
        ('\u{0455}', 6), // ѕ
        ('\u{1D1B}', 6), // ᴛ
        ('\u{1D1C}', 6), // ᴜ
        ('\u{1D20}', 6), // ᴠ
        ('\u{1D21}', 6), // ᴡ
        ('x', 6),        // small-caps X intentionally uses ASCII x
        ('\u{028F}', 6), // ʏ
        ('\u{1D22}', 6), // ᴢ
    ];
    for (character, wanted) in expected {
        assert_eq!(advance(character), Some(wanted), "U+{:04X}", character as u32);
    }
}

#[test]
fn unknown_chars_have_no_entry_but_fall_back() {
    assert_eq!(advance('√'), None);
    assert_eq!(advance('\u{1F600}'), None);
    assert_eq!(glyph_width('√'), None);
    assert_eq!(text_width("√"), FALLBACK_ADVANCE);
    assert_eq!(text_visible_width("√"), FALLBACK_GLYPH_WIDTH);
    assert_eq!(bold_text_visible_width("√"), FALLBACK_GLYPH_WIDTH + BOLD_ADVANCE);
}

#[test]
fn text_width_sums_advances() {
    // "Hi!" = H(6) + i(2) + !(2) = 10.
    assert_eq!(text_width("Hi!"), 10);
    // "Buy" = B(6) + u(6) + y(6) = 18.
    assert_eq!(text_width("Buy"), 18);
    // A space-padded string.
    assert_eq!(text_width(" l "), 4 + 3 + 4);
    assert_eq!(text_width("ᴍɪɴɪɴɢ ѕɪᴍ"), 6 + 4 + 6 + 4 + 6 + 6 + 4 + 6 + 4 + 6);
    assert_eq!(text_width(""), 0);
}

#[test]
fn visible_width_uses_ink_bounds() {
    assert_eq!(text_visible_width("Hi!"), 9);
    assert_eq!(text_visible_width("Hi! "), 9);
    assert_eq!(text_visible_width(" "), 0);
    assert_eq!(text_visible_width("A A"), 15);
    assert_eq!(text_visible_width(""), 0);
    assert_eq!(bold_text_visible_width("Hi!"), 12);
    assert_eq!(bold_text_visible_width("Hi! "), 12);
    assert_eq!(bold_text_visible_width(" "), 0);
}

#[test]
fn advances_iterator_matches_table() {
    let collected: Vec<_> = advances().collect();
    assert_eq!(collected.len(), ADVANCES.len() + SMALL_TEXT_ADVANCES.len());
    assert_eq!(collected.first(), Some(&(' ', 4)));
    assert_eq!(collected.last(), Some(&('\u{A731}', 6)));
}

#[test]
fn glyph_widths_iterator_matches_table() {
    let collected: Vec<_> = glyph_widths().collect();
    assert_eq!(collected.len(), ADVANCES.len() + SMALL_TEXT_ADVANCES.len());
    assert_eq!(collected.first(), Some(&(' ', 0)));
    assert_eq!(collected.last(), Some(&('\u{A731}', 5)));
}
