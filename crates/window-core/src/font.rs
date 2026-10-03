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

mod vanilla_grids;

use vanilla_grids::{VANILLA_ASCII_CHARS, vanilla_small_text_accented_rows, vanilla_window_text_nonlatin_rows};

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
/// touches it. Returns each key's assigned codepoint, or [`Error::Font`] if
/// there are more keys than the [`GLYPH_SPAN`] codepoints in the range.
pub fn allocate(keys: &BTreeSet<String>) -> Result<BTreeMap<String, u32>> {
    allocate_with(keys, |k| xxh3_64(k.as_bytes()))
}

/// [`allocate`] with an injectable hash function (for testing collisions).
pub fn allocate_with(keys: &BTreeSet<String>, hash: impl Fn(&str) -> u64) -> Result<BTreeMap<String, u32>> {
    if keys.len() > GLYPH_SPAN as usize {
        return Err(Error::Font(format!(
            "{} glyphs requested, but only {GLYPH_SPAN} private-use codepoints are available",
            keys.len()
        )));
    }
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
    Ok(out)
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
    Ok(provider_font(shifted_providers(k)?))
}

/// The font id suffix of text font `name` at vertical offset `k`: `("small_caps", -1) -> "small_caps/ym1"`.
pub fn text_font_suffix(name: &str, k: i32) -> String {
    format!("{name}/{}", shifted_suffix(k))
}

/// Build the document of a text font whose glyph sheet is `file`, drawn at vertical offset `k`.
///
/// The sheet's cells share the vanilla ASCII cell geometry, and its provider comes first, so its glyphs replace the
/// shifted vanilla glyphs that follow.
pub fn text_font(file: &str, chars: &[String], k: i32) -> Result<Value> {
    let vanilla = shifted_providers(k)?;
    let sheet = vanilla_bitmap_provider(file, 8, 7 - k, chars.iter().map(String::as_str).collect());
    Ok(provider_font(std::iter::once(sheet).chain(vanilla).collect()))
}

/// The space provider and the vanilla sheets shifted to vertical offset `k`.
fn shifted_providers(k: i32) -> Result<Vec<Value>> {
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
    Ok(vec![
        vanilla_space_provider(),
        vanilla_bitmap_provider("minecraft:font/ascii.png", 8, ascent, VANILLA_ASCII_CHARS.to_vec()),
        vanilla_bitmap_provider("minecraft:font/nonlatin_european.png", 8, ascent, vanilla_window_text_nonlatin_rows()),
        vanilla_bitmap_provider("minecraft:font/accented.png", 12, accented_ascent, vanilla_small_text_accented_rows()),
    ])
}

#[cfg(test)]
mod tests;
