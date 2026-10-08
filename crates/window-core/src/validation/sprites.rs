use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::ValidationReport;
use super::json::{extend_bitmap_glyphs, font_path, one_char, parse_json, providers};
use crate::manifest::{Manifest, WindowEntry};

pub(super) fn validate_sprite_fonts(manifest: &Manifest, files: &BTreeMap<&str, &[u8]>, report: &mut ValidationReport) {
    let expected_glyphs: BTreeSet<char> =
        manifest.sprites.values().filter_map(|sprite| one_char(&sprite.glyph)).collect();
    for (window_name, window) in &manifest.windows {
        let context = WindowSprites { manifest, window_name, window, expected_glyphs: &expected_glyphs, files };
        context.validate_slots(report);
    }
    validate_sprite_catalog(manifest, report);
}

struct WindowSprites<'a> {
    manifest: &'a Manifest,
    window_name: &'a str,
    window: &'a WindowEntry,
    expected_glyphs: &'a BTreeSet<char>,
    files: &'a BTreeMap<&'a str, &'a [u8]>,
}

impl WindowSprites<'_> {
    fn validate_slots(&self, report: &mut ValidationReport) {
        let Self { manifest, window_name, window, .. } = self;
        for (slot_name, slot) in &window.sprite_slots {
            let expected_ascent = i64::from(7 - (slot.y - window.surface.title_origin[1]));
            self.validate_font(&slot.font, expected_ascent, &format!("{window_name}.{slot_name}"), report);
            if let Some(sprite) = &slot.sprite
                && !manifest.sprites.contains_key(sprite)
            {
                report.push(
                    "sprite.reference.unknown",
                    format!("manifest.windows.{window_name}.sprite_slots.{slot_name}.sprite"),
                    format!("unknown sprite `{sprite}`"),
                );
            }
        }
    }

    fn validate_font(&self, font: &str, expected_ascent: i64, owner: &str, report: &mut ValidationReport) {
        let path = font_path(font);
        let Some(bytes) = self.files.get(path.as_str()) else {
            return;
        };
        let Some(document) = parse_json(bytes, &path, report) else {
            return;
        };
        let mut glyphs = BTreeSet::new();
        for provider in providers(&document) {
            if provider.get("type").and_then(Value::as_str) != Some("bitmap") {
                continue;
            }
            if provider.get("ascent").and_then(Value::as_i64) != Some(expected_ascent) {
                report.push(
                    "font.sprite.ascent_mismatch",
                    &path,
                    format!("sprite provider ascent must be {expected_ascent} for `{owner}`"),
                );
            }
            extend_bitmap_glyphs(provider, &mut glyphs);
        }
        if glyphs != *self.expected_glyphs {
            report.push(
                "font.sprite.catalog_mismatch",
                &path,
                format!(
                    "font contains {} sprite glyph(s); manifest requires {}",
                    glyphs.len(),
                    self.expected_glyphs.len()
                ),
            );
        }
    }
}

fn validate_sprite_catalog(manifest: &Manifest, report: &mut ValidationReport) {
    for (name, sprite) in &manifest.sprites {
        if one_char(&sprite.glyph).is_none() {
            report.push(
                "sprite.glyph.invalid",
                format!("manifest.sprites.{name}.glyph"),
                "sprite glyph must contain exactly one character",
            );
        }
        if sprite.x_offset.saturating_add(sprite.glyph_width) > sprite.width {
            report.push(
                "sprite.metrics.invalid",
                format!("manifest.sprites.{name}"),
                "x_offset + glyph_width exceeds rendered width",
            );
        }
        if sprite.glyph_width > 0 && sprite.advance != sprite.x_offset + sprite.glyph_width + 1 {
            report.push(
                "sprite.advance.mismatch",
                format!("manifest.sprites.{name}.advance"),
                "advance must equal the visible right edge plus Minecraft's one-pixel gap",
            );
        }
    }
}
