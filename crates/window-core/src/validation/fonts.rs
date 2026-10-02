use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::ValidationReport;
use super::json::{extend_bitmap_glyphs, font_path, one_char, parse_json, providers};
use crate::manifest::Manifest;

pub(super) fn validate_shifted_fonts(
    manifest: &Manifest,
    files: &BTreeMap<&str, &[u8]>,
    report: &mut ValidationReport,
) {
    for (font, offset) in shifted_font_offsets(manifest) {
        let path = font_path(font);
        let Some(bytes) = files.get(path.as_str()) else {
            continue;
        };
        let Some(document) = parse_json(bytes, &path, report) else {
            continue;
        };
        if validate_ascii_ascent(&document, &path, offset, report) {
            validate_declared_glyphs(manifest, font, &document, &path, report);
        }
    }
}

fn shifted_font_offsets(manifest: &Manifest) -> BTreeMap<&str, i32> {
    let mut expected: BTreeMap<&str, i32> = BTreeMap::new();
    for window in manifest.windows.values() {
        for slot in window.slots.values() {
            expected.insert(&slot.font, slot.y - window.surface.title_origin[1]);
        }
    }
    for hud in manifest.huds.values() {
        for slot in hud.slots.values() {
            expected.insert(&slot.font, slot.y);
        }
    }
    expected
}

/// Returns whether the font has the vanilla ASCII provider.
fn validate_ascii_ascent(document: &Value, path: &str, offset: i32, report: &mut ValidationReport) -> bool {
    let Some(provider) = providers(document).iter().find(|provider| {
        provider.get("type").and_then(Value::as_str) == Some("bitmap")
            && provider.get("file").and_then(Value::as_str) == Some("minecraft:font/ascii.png")
    }) else {
        report.push("font.shifted.ascii_missing", path, "shifted text font has no vanilla ASCII provider");
        return false;
    };
    let actual = provider.get("ascent").and_then(Value::as_i64);
    let wanted = i64::from(7 - offset);
    if actual != Some(wanted) {
        report.push(
            "font.shifted.ascent_mismatch",
            path,
            format!("ASCII ascent is {actual:?}; expected {wanted} for offset {offset}"),
        );
    }
    true
}

fn validate_declared_glyphs(
    manifest: &Manifest,
    font: &str,
    document: &Value,
    path: &str,
    report: &mut ValidationReport,
) {
    let Some(metrics) = manifest.font_metrics.get(font) else {
        return;
    };
    let provided = provided_glyphs(document);
    for character in metrics.advances.keys() {
        if !provided.contains(character) {
            report.push(
                "font.shifted.glyph_missing",
                path,
                format!(
                    "manifest metrics declare U+{:04X}, but the shifted font has no provider for it",
                    *character as u32
                ),
            );
        }
    }
}

fn provided_glyphs(document: &Value) -> BTreeSet<char> {
    let mut provided = BTreeSet::new();
    for provider in providers(document) {
        match provider.get("type").and_then(Value::as_str) {
            Some("space") => {
                if let Some(advances) = provider.get("advances").and_then(Value::as_object) {
                    provided.extend(advances.keys().filter_map(|text| one_char(text)));
                }
            }
            Some("bitmap") => extend_bitmap_glyphs(provider, &mut provided),
            _ => {}
        }
    }
    provided
}
