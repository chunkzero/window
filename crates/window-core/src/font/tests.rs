use super::*;

#[test]
fn spacer_table_has_22_entries_with_expected_advances() {
    let t = spacer_table();
    assert_eq!(t.len(), 22);
    assert_eq!(t[&0xF0000], -1024);
    assert_eq!(t[&0xF000A], -1);
    assert_eq!(t[&0xF000B], 1);
    assert_eq!(t[&0xF0015], 1024);
}

#[test]
fn spacers_for_zero_is_empty() {
    assert_eq!(spacers_for(0), "");
}

#[test]
fn spacers_for_minus_nine_uses_8_then_1() {
    let s = spacers_for(-9);
    let table = spacer_table();
    let cps: Vec<u32> = s.chars().map(|c| c as u32).collect();
    assert_eq!(cps.len(), 2);
    assert_eq!(table[&cps[0]], -8);
    assert_eq!(table[&cps[1]], -1);
}

#[test]
fn spacers_for_round_trips_over_a_sweep() {
    let table = spacer_table();
    for dx in -2000..=2000 {
        let s = spacers_for(dx);
        let sum: i32 = s.chars().map(|c| table[&(c as u32)]).sum();
        assert_eq!(sum, dx, "round-trip failed for dx={dx}");
    }
}

#[test]
fn allocate_is_deterministic() {
    let keys: BTreeSet<String> = ["a", "b", "c", "window/shop/static"].iter().map(|s| s.to_string()).collect();
    let a = allocate(&keys).unwrap();
    let b = allocate(&keys).unwrap();
    assert_eq!(a, b);
    // Every codepoint is within the glyph range and disjoint from spacers.
    for &cp in a.values() {
        assert!((GLYPH_BASE..GLYPH_END).contains(&cp));
        assert!(!(SPACER_BASE..SPACER_BASE + SPACER_COUNT).contains(&cp));
    }
}

#[test]
fn allocate_resolves_collisions_by_sorted_probe() {
    // Force a collision: all keys hash to the same slot. Sorted order is
    // a < b < c, so a keeps the slot, b takes +1, c takes +2.
    let keys: BTreeSet<String> = ["b", "a", "c"].iter().map(|s| s.to_string()).collect();
    let out = allocate_with(&keys, |_| 5).unwrap();
    let base = GLYPH_BASE + (5 % GLYPH_SPAN);
    assert_eq!(out["a"], base);
    assert_eq!(out["b"], base + 1);
    assert_eq!(out["c"], base + 2);
}

#[test]
fn allocate_does_not_move_existing_non_colliding_keys() {
    let mut keys: BTreeSet<String> = ["x", "y"].iter().map(|s| s.to_string()).collect();
    let before = allocate(&keys).unwrap();
    keys.insert("z_unrelated_key".to_string());
    let after = allocate(&keys).unwrap();
    // x and y keep their hashed (non-colliding) slots when z is added.
    assert_eq!(before["x"], after["x"]);
    assert_eq!(before["y"], after["y"]);
}

#[test]
fn allocate_rejects_more_keys_than_codepoints() {
    let keys: BTreeSet<String> = (0..=GLYPH_SPAN).map(|i| i.to_string()).collect();
    assert!(matches!(allocate_with(&keys, |_| 0), Err(Error::Font(_))));
}

#[test]
fn space_provider_shape() {
    let v = space_provider();
    assert_eq!(v["type"], json!("space"));
    let advances = v["advances"].as_object().unwrap();
    assert_eq!(advances.len(), 22);
}

#[test]
fn bitmap_provider_calibration_ascent() {
    // glyph top at gui y=0, title y=6 → ascent 13, height H.
    let v = bitmap_provider("window", "shop", 16, 13, '\u{E000}').unwrap();
    assert_eq!(v["type"], json!("bitmap"));
    assert_eq!(v["file"], json!("window:font/shop.png"));
    assert_eq!(v["height"], json!(16));
    assert_eq!(v["ascent"], json!(13));
    assert_eq!(v["chars"], json!(["\u{E000}"]));
}

#[test]
fn bitmap_provider_rejects_bad_limits() {
    assert!(bitmap_provider("window", "w", 600, 10, 'a').is_err());
    assert!(bitmap_provider("window", "w", 16, 17, 'a').is_err());
}

#[test]
fn shifted_suffix_formats_minus() {
    assert_eq!(shifted_suffix(0), "y0");
    assert_eq!(shifted_suffix(12), "y12");
    assert_eq!(shifted_suffix(-4), "ym4");
}

#[test]
fn shifted_font_ascent_and_chars() {
    let v = shifted_font(0).unwrap();
    let space = &v["providers"][0];
    assert_eq!(space["type"], json!("space"));
    assert_eq!(space["advances"][" "], json!(4));
    assert_eq!(space["advances"]["\u{200c}"], json!(0));

    let p = &v["providers"][1];
    assert_eq!(p["file"], json!("minecraft:font/ascii.png"));
    assert_eq!(p["height"], json!(8));
    assert_eq!(p["ascent"], json!(7));
    let chars = p["chars"].as_array().unwrap();
    assert_eq!(chars.len(), 16);
    for row in chars {
        assert_eq!(row.as_str().unwrap().chars().count(), 16);
    }
    let nonlatin = &v["providers"][2];
    assert_eq!(nonlatin["file"], json!("minecraft:font/nonlatin_european.png"));
    assert_eq!(nonlatin["height"], json!(8));
    assert_eq!(nonlatin["ascent"], json!(7));
    let rows = nonlatin["chars"].as_array().unwrap();
    assert_eq!(rows.len(), 67);
    assert!(rows[0].as_str().unwrap().contains('\u{00B7}'));
    assert_eq!(rows[0].as_str().unwrap().chars().nth(7), Some('\u{00D7}'));
    assert!(rows[37].as_str().unwrap().contains('\u{1D00}'));
    assert!(rows[38].as_str().unwrap().contains('\u{1D0D}'));
    assert!(rows[38].as_str().unwrap().contains('\u{A731}'));
    assert!(rows[29].as_str().unwrap().contains('\u{25B2}'));
    assert!(rows[29].as_str().unwrap().contains('\u{25BC}'));
    assert!(rows[29].as_str().unwrap().contains('\u{25CF}'));
    assert!(rows[56].as_str().unwrap().contains('\u{25C6}'));

    let accented = &v["providers"][3];
    assert_eq!(accented["file"], json!("minecraft:font/accented.png"));
    assert_eq!(accented["height"], json!(12));
    assert_eq!(accented["ascent"], json!(10));
    let rows = accented["chars"].as_array().unwrap();
    assert_eq!(rows.len(), 75);
    assert!(rows[16].as_str().unwrap().contains('\u{01EB}'));

    // k = -1 → ascent 8 (the upper bound).
    assert_eq!(shifted_font(-1).unwrap()["providers"][1]["ascent"], json!(8));
}

#[test]
fn shifted_font_characters_match_vanilla_metrics() {
    let font = shifted_font(0).unwrap();
    let mut provided = BTreeSet::new();
    for provider in font["providers"].as_array().unwrap() {
        if provider["type"] == json!("space") {
            for (key, advance) in provider["advances"].as_object().unwrap() {
                let c = key.chars().next().unwrap();
                assert_eq!(crate::vanilla::advance(c).map(u64::from), advance.as_u64(), "U+{:04X}", c as u32);
                provided.insert(c);
            }
        } else {
            for row in provider["chars"].as_array().unwrap() {
                provided.extend(row.as_str().unwrap().chars().filter(|&c| c != '\0'));
            }
        }
    }
    let measured: BTreeSet<char> = crate::vanilla::advances().map(|(c, _)| c).collect();
    assert_eq!(provided, measured);
}
