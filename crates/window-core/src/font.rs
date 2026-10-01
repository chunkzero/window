//! Font generation: the spacer table, glyph codepoint allocation, and the
//! `space`/`bitmap` font-provider JSON values for the main and shifted fonts.
//!
//! All emitted JSON is deterministic. Maps are built as [`serde_json::Map`]
//! (insertion-ordered) with keys inserted in a fixed, documented order, so
//! identical inputs always serialize byte-identically.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};
use xxhash_rust::xxh3::xxh3_64;

use crate::{Error, Result};

/// First spacer codepoint (`U+F0000`). The 22-entry table runs through
/// `U+F0015`.
pub const SPACER_BASE: u32 = 0xF0000;

/// Number of spacer codepoints (22: eleven negative powers, eleven positive).
pub const SPACER_COUNT: u32 = 22;

/// First glyph codepoint (`U+E000`) of the bitmap allocation range.
pub const GLYPH_BASE: u32 = 0xE000;

/// Exclusive upper bound of the bitmap allocation range (`U+F900`).
pub const GLYPH_END: u32 = 0xF900;

/// Size of the glyph allocation range (BMP private-use codepoints).
pub const GLYPH_SPAN: u32 = GLYPH_END - GLYPH_BASE;

/// The spacer advance magnitudes, ascending: `1, 2, 4, … 1024`.
const SPACER_MAGNITUDES: [i32; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

/// The spacer table: codepoint → horizontal advance in pixels.
///
/// Indices 0–10 hold advances `-1024, -512, …, -1` and indices 11–21 hold
/// `+1, +2, …, +1024`, at codepoints `U+F0000..=U+F0015`.
pub fn spacer_table() -> BTreeMap<u32, i32> {
    let mut table = BTreeMap::new();
    // Negative powers, most negative first (index 0 = -1024 … index 10 = -1).
    for (i, mag) in SPACER_MAGNITUDES.iter().rev().enumerate() {
        table.insert(SPACER_BASE + i as u32, -mag);
    }
    // Positive powers, smallest first (index 11 = +1 … index 21 = +1024).
    for (i, mag) in SPACER_MAGNITUDES.iter().enumerate() {
        table.insert(SPACER_BASE + 11 + i as u32, *mag);
    }
    table
}

/// The spacer codepoint that advances by exactly `signed_mag` (a signed power
/// of two in `±1..=±1024`), or `None` if not a representable magnitude.
fn spacer_codepoint(signed_mag: i32) -> Option<u32> {
    let mag = signed_mag.unsigned_abs() as i32;
    let idx = SPACER_MAGNITUDES.iter().position(|&m| m == mag)?;
    if signed_mag < 0 { Some(SPACER_BASE + (10 - idx) as u32) } else { Some(SPACER_BASE + 11 + idx as u32) }
}

/// Decompose `dx` into a string of spacer characters whose advances sum to
/// `dx`.
///
/// Uses a greedy power-of-two decomposition over `±1024 … ±1`; magnitudes
/// beyond `±1024` are covered by repeating the `±1024` spacer. Returns the
/// empty string for `dx == 0`.
pub fn spacers_for(dx: i32) -> String {
    if dx == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut remaining = dx;
    let sign = if dx < 0 { -1 } else { 1 };
    // Greedy from the largest magnitude down; repeat 1024 as needed.
    for &mag in SPACER_MAGNITUDES.iter().rev() {
        while sign * remaining >= mag {
            let step = sign * mag;
            let cp = spacer_codepoint(step).expect("power-of-two magnitude is in table");
            out.push(char::from_u32(cp).expect("spacer codepoint is a valid char"));
            remaining -= step;
            if remaining == 0 {
                return out;
            }
        }
    }
    debug_assert_eq!(remaining, 0, "power-of-two decomposition must be exact");
    out
}

/// Allocate a stable codepoint for each key in `keys`.
///
/// The codepoint of `key` is `GLYPH_BASE + (xxh3_64(key) % GLYPH_SPAN)`.
/// Collisions within one build are resolved deterministically: colliding keys
/// are sorted, the lowest-sorted keeps the hashed slot, and the rest probe
/// forward (wrapping within `[GLYPH_BASE, GLYPH_END)`) for the next free slot.
/// The spacer range is disjoint from the glyph range, so allocation never
/// touches it. Returns each key's assigned codepoint.
pub fn allocate(keys: &BTreeSet<String>) -> BTreeMap<String, u32> {
    allocate_with(keys, |k| xxh3_64(k.as_bytes()))
}

/// [`allocate`] with an injectable hash function (for testing collisions).
pub fn allocate_with(keys: &BTreeSet<String>, hash: impl Fn(&str) -> u64) -> BTreeMap<String, u32> {
    // `keys` is a BTreeSet, so iteration is already sorted: the first key to
    // claim a slot is the lowest-sorted, satisfying the stability rule.
    let mut used: BTreeSet<u32> = BTreeSet::new();
    let mut out = BTreeMap::new();
    for key in keys {
        let start = GLYPH_BASE + (hash(key) % GLYPH_SPAN as u64) as u32;
        let mut cp = start;
        // Probe forward, wrapping within the glyph range, until a free slot.
        while used.contains(&cp) {
            cp += 1;
            if cp >= GLYPH_END {
                cp = GLYPH_BASE;
            }
        }
        used.insert(cp);
        out.insert(key.clone(), cp);
    }
    out
}

/// Build the `space` font provider value carrying the full spacer table.
///
/// Advances are keyed by the actual spacer characters, inserted in codepoint
/// order for deterministic output.
pub fn space_provider() -> Value {
    let mut advances = Map::new();
    for (cp, adv) in spacer_table() {
        let ch = char::from_u32(cp).expect("spacer codepoint is a valid char");
        advances.insert(ch.to_string(), json!(adv));
    }
    let mut provider = Map::new();
    provider.insert("type".into(), json!("space"));
    provider.insert("advances".into(), Value::Object(advances));
    Value::Object(provider)
}

/// Build the vanilla default-font space provider.
///
/// Shifted label fonts use a custom bitmap provider that references
/// `minecraft:font/ascii.png` directly. Vanilla's own default font puts a
/// `space` provider before that bitmap provider; without this provider, the
/// blank space cell in `ascii.png` advances by only 1px.
fn vanilla_space_provider() -> Value {
    let mut advances = Map::new();
    advances.insert(" ".into(), json!(4));
    advances.insert("\u{200c}".into(), json!(0));
    let mut provider = Map::new();
    provider.insert("type".into(), json!("space"));
    provider.insert("advances".into(), Value::Object(advances));
    Value::Object(provider)
}

fn vanilla_bitmap_provider(file: &str, height: u32, ascent: i32, chars: Vec<&str>) -> Value {
    let mut provider = Map::new();
    provider.insert("type".into(), json!("bitmap"));
    provider.insert("file".into(), json!(file));
    provider.insert("height".into(), json!(height));
    provider.insert("ascent".into(), json!(ascent));
    provider.insert("chars".into(), Value::Array(chars.into_iter().map(|row| json!(row)).collect()));
    Value::Object(provider)
}

/// Sparse copy of Minecraft's `nonlatin_european.png` bitmap grid containing
/// the mcutils-style small caps and Window's standard UI markers. Keeping the
/// provider grid and [`crate::vanilla`] metrics in lockstep is required for
/// net-zero title segments.
fn vanilla_window_text_nonlatin_rows() -> Vec<&'static str> {
    let mut rows = vec![EMPTY_BITMAP_ROW; 67];
    rows[0] = "\u{0000}\u{0000}\u{0000}\u{00B7}\u{0000}\u{0000}\u{0000}\u{00D7}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows[8] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0455}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows[9] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{026A}\u{0000}\u{0000}\u{0000}";
    rows[29] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{25B2}\u{0000}\u{25BC}\u{0000}\u{25CF}\u{0000}\u{0000}";
    rows[37] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{1D00}\u{0299}\u{1D04}\u{1D05}\u{1D07}\u{A730}\u{0262}\u{029C}\u{1D0A}";
    rows[38] = "\u{1D0B}\u{029F}\u{1D0D}\u{0274}\u{1D0F}\u{1D18}\u{0000}\u{0280}\u{A731}\u{1D1B}\u{1D1C}\u{1D20}\u{1D21}\u{028F}\u{1D22}\u{0000}";
    rows[56] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{25C6}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows
}

/// Sparse copy of Minecraft's `accented.png` bitmap grid containing the
/// mcutils-style small-cap Q (`U+01EB`).
fn vanilla_small_text_accented_rows() -> Vec<&'static str> {
    let mut rows = vec![EMPTY_BITMAP_ROW; 75];
    rows[16] = "\u{0000}\u{0000}\u{01EB}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows
}

/// Build a `bitmap` font provider for a window's composite glyph.
///
/// `namespace` is the resource namespace, `window` the window name (the texture
/// is `<namespace>:font/<window>.png`), `height` the composite pixel height,
/// `ascent` the vertical placement, and `glyph` the single codepoint character.
///
/// Validates the provider limits: `ascent <= height` and `height <= 512`
/// (otherwise [`Error::Font`]).
pub fn bitmap_provider(namespace: &str, window: &str, height: u32, ascent: i32, glyph: char) -> Result<Value> {
    bitmap_provider_file(&format!("{namespace}:font/{window}.png"), window, height, ascent, glyph)
}

/// Build a `bitmap` font provider for an arbitrary texture file id.
pub fn bitmap_provider_file(file: &str, debug_name: &str, height: u32, ascent: i32, glyph: char) -> Result<Value> {
    if height > 512 {
        return Err(Error::Font(format!("`{debug_name}`: bitmap height {height} exceeds the 512px provider limit")));
    }
    if ascent > height as i32 {
        return Err(Error::Font(format!("`{debug_name}`: ascent {ascent} exceeds height {height}")));
    }
    if ascent <= -32768 {
        return Err(Error::Font(format!(
            "`{debug_name}`: ascent {ascent} is outside the valid range (-32768, height]"
        )));
    }
    let mut provider = Map::new();
    provider.insert("type".into(), json!("bitmap"));
    provider.insert("file".into(), json!(file));
    provider.insert("height".into(), json!(height));
    provider.insert("ascent".into(), json!(ascent));
    provider.insert("chars".into(), json!([glyph.to_string()]));
    Ok(Value::Object(provider))
}

/// Assemble the main font document (`assets/<ns>/font/ui.json`).
///
/// Providers are the single `space` provider followed by `bitmaps` in the order
/// given (the caller passes them sorted by window name).
pub fn main_font(space: Value, bitmaps: Vec<Value>) -> Value {
    let mut providers = Vec::with_capacity(1 + bitmaps.len());
    providers.push(space);
    providers.extend(bitmaps);
    let mut doc = Map::new();
    doc.insert("providers".into(), Value::Array(providers));
    Value::Object(doc)
}

/// Assemble a font document from prebuilt providers.
pub fn provider_font(providers: Vec<Value>) -> Value {
    let mut doc = Map::new();
    doc.insert("providers".into(), Value::Array(providers));
    Value::Object(doc)
}

/// The font id suffix for a vertical text offset `k` (`y<k>`, with `m`
/// replacing the minus sign for negatives): `0 -> "y0"`, `-4 -> "ym4"`.
pub fn shifted_suffix(k: i32) -> String {
    if k < 0 { format!("ym{}", -k) } else { format!("y{k}") }
}

/// Build a shifted vanilla default-font document for vertical offset `k`.
///
/// The document mirrors the vanilla default font's ASCII path and adds sparse
/// shifted copies of the built-in sheets that carry the mcutils-style
/// Minecraft small text glyphs and Window's common UI markers/separators (`×`,
/// `▲`, `▼`, `◆`, `●`, `·`). We still ship no vanilla art; all bitmap providers reference
/// Minecraft's own textures with adjusted ascent values.
/// Validates the vanilla ASCII/non-Latin ascent bound `-32768 < 7 - k <= 8`
/// (otherwise [`Error::Font`]).
pub fn shifted_font(k: i32) -> Result<Value> {
    let ascent = 7 - k;
    if ascent > 8 || ascent <= -32768 {
        return Err(Error::Font(format!("shifted font y{k}: ascent {ascent} is outside the valid range (-32768, 8]")));
    }
    let accented_ascent = 10 - k;
    if accented_ascent > 12 || accented_ascent <= -32768 {
        return Err(Error::Font(format!(
            "shifted font y{k}: accented ascent {accented_ascent} is outside the valid range (-32768, 12]"
        )));
    }
    let mut doc = Map::new();
    doc.insert(
        "providers".into(),
        Value::Array(vec![
            vanilla_space_provider(),
            vanilla_bitmap_provider("minecraft:font/ascii.png", 8, ascent, VANILLA_ASCII_CHARS.to_vec()),
            vanilla_bitmap_provider(
                "minecraft:font/nonlatin_european.png",
                8,
                ascent,
                vanilla_window_text_nonlatin_rows(),
            ),
            vanilla_bitmap_provider(
                "minecraft:font/accented.png",
                12,
                accented_ascent,
                vanilla_small_text_accented_rows(),
            ),
        ]),
    );
    Ok(Value::Object(doc))
}

/// The canonical 16-row × 16-column character grid for vanilla `ascii.png`,
/// reproduced from the vanilla `minecraft:font/default.json` `ascii` provider.
///
/// Rows map to the glyph rows of the texture; positions without a glyph use the
/// null character (`U+0000`), as vanilla does for its unused cells.
const VANILLA_ASCII_CHARS: [&str; 16] = [
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}",
    " !\"#$%&'()*+,-./",
    "0123456789:;<=>?",
    "@ABCDEFGHIJKLMNO",
    "PQRSTUVWXYZ[\\]^_",
    "`abcdefghijklmno",
    "pqrstuvwxyz{|}~\u{0000}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{00A3}\u{0000}\u{0000}\u{0192}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{00AA}\u{00BA}\u{0000}\u{0000}\u{00AC}\u{0000}\u{0000}\u{0000}\u{00AB}\u{00BB}",
    "\u{2591}\u{2592}\u{2593}\u{2502}\u{2524}\u{2561}\u{2562}\u{2556}\u{2555}\u{2563}\u{2551}\u{2557}\u{255D}\u{255C}\u{255B}\u{2510}",
    "\u{2514}\u{2534}\u{252C}\u{251C}\u{2500}\u{253C}\u{255E}\u{255F}\u{255A}\u{2554}\u{2569}\u{2566}\u{2560}\u{2550}\u{256C}\u{2567}",
    "\u{2568}\u{2564}\u{2565}\u{2559}\u{2558}\u{2552}\u{2553}\u{256B}\u{256A}\u{2518}\u{250C}\u{2588}\u{2584}\u{258C}\u{2590}\u{2580}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{2205}\u{2208}\u{0000}",
    "\u{2261}\u{00B1}\u{2265}\u{2264}\u{2320}\u{2321}\u{00F7}\u{2248}\u{00B0}\u{2219}\u{0000}\u{221A}\u{207F}\u{00B2}\u{25A0}\u{0000}",
];

const EMPTY_BITMAP_ROW: &str = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";

#[cfg(test)]
mod tests {
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
        let a = allocate(&keys);
        let b = allocate(&keys);
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
        let out = allocate_with(&keys, |_| 5);
        let base = GLYPH_BASE + (5 % GLYPH_SPAN);
        assert_eq!(out["a"], base);
        assert_eq!(out["b"], base + 1);
        assert_eq!(out["c"], base + 2);
    }

    #[test]
    fn allocate_does_not_move_existing_non_colliding_keys() {
        let mut keys: BTreeSet<String> = ["x", "y"].iter().map(|s| s.to_string()).collect();
        let before = allocate(&keys);
        keys.insert("z_unrelated_key".to_string());
        let after = allocate(&keys);
        // x and y keep their hashed (non-colliding) slots when z is added.
        assert_eq!(before["x"], after["x"]);
        assert_eq!(before["y"], after["y"]);
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
}
