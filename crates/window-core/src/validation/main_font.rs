use std::collections::BTreeMap;

use serde_json::Value;

use super::ValidationReport;
use super::bitmap::collect_bitmap_advances;
use super::json::{font_path, parse_json};
use crate::manifest::Manifest;

#[derive(Clone, Debug, Default)]
pub(super) struct FontAdvances {
    pub(super) advances: BTreeMap<char, i32>,
}

pub(super) fn validate_main_font(
    manifest: &Manifest,
    files: &BTreeMap<&str, &[u8]>,
    report: &mut ValidationReport,
) -> FontAdvances {
    let path = font_path(&manifest.font);
    let Some(bytes) = files.get(path.as_str()) else {
        return FontAdvances::default();
    };
    let Some(document) = parse_json(bytes, &path, report) else {
        return FontAdvances::default();
    };
    let Some(providers) = document.get("providers").and_then(Value::as_array) else {
        report.push("font.providers.missing", &path, "font document must contain a providers array");
        return FontAdvances::default();
    };

    let mut metrics = FontAdvances::default();
    for (index, provider) in providers.iter().enumerate() {
        let location = format!("{path}#providers[{index}]");
        match provider.get("type").and_then(Value::as_str) {
            Some("space") => collect_space_advances(provider, &location, &mut metrics, report),
            Some("bitmap") => collect_bitmap_advances(provider, &location, files, &mut metrics, report),
            Some(kind) => {
                report.push("font.provider.unsupported", location, format!("unsupported provider type `{kind}`"))
            }
            None => report.push("font.provider.type_missing", location, "provider has no string `type`"),
        }
    }
    validate_spacers(manifest, &metrics, report);
    metrics
}

fn validate_spacers(manifest: &Manifest, metrics: &FontAdvances, report: &mut ValidationReport) {
    for (codepoint, expected) in &manifest.spacers {
        let Some(character) = char::from_u32(*codepoint) else {
            report.push(
                "spacer.codepoint.invalid",
                "manifest.spacers",
                format!("U+{codepoint:04X} is not a Unicode scalar value"),
            );
            continue;
        };
        match metrics.advances.get(&character) {
            Some(actual) if actual == expected => {}
            Some(actual) => report.push(
                "spacer.advance.mismatch",
                "manifest.spacers",
                format!("U+{codepoint:04X} has manifest advance {expected}, provider advance {actual}"),
            ),
            None => report.push(
                "spacer.provider.missing",
                "manifest.spacers",
                format!("U+{codepoint:04X} is absent from the main font"),
            ),
        }
    }
}

fn collect_space_advances(provider: &Value, location: &str, metrics: &mut FontAdvances, report: &mut ValidationReport) {
    let Some(advances) = provider.get("advances").and_then(Value::as_object) else {
        report.push("font.space.advances_missing", location, "space provider has no advances object");
        return;
    };
    for (text, value) in advances {
        let mut chars = text.chars();
        let Some(character) = chars.next() else {
            report.push("font.character.invalid", location, "empty character key in advances");
            continue;
        };
        if chars.next().is_some() {
            report.push("font.character.invalid", location, format!("advance key `{text}` is not one character"));
            continue;
        }
        let Some(advance) = value.as_i64().and_then(|value| i32::try_from(value).ok()) else {
            report.push("font.advance.invalid", location, format!("advance for `{text}` is not an i32"));
            continue;
        };
        insert_advance(metrics, character, advance, location, report);
    }
}

pub(super) fn insert_advance(
    metrics: &mut FontAdvances,
    character: char,
    advance: i32,
    location: &str,
    report: &mut ValidationReport,
) {
    if let Some(previous) = metrics.advances.insert(character, advance)
        && previous != advance
    {
        report.push(
            "font.glyph.duplicate",
            location,
            format!("glyph `{character}` has conflicting advances {previous} and {advance}"),
        );
    }
}
