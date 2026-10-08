//! Pack-level objects: `WindowPackData`, `WindowSpacers`, `WindowFonts`, `WindowSprites`, `WindowSprite`, and
//! `WindowColors`.

use std::collections::BTreeMap;

use crate::authoring::art::is_inline;
use crate::ir::Rgb;
use crate::manifest::{FontMetricsEntry, Manifest};
use crate::pipeline::OutputFile;
use crate::{Error, Result};

use super::entries::{font_metrics_expr, sprite_entry_expr};
use super::literals::{char_map, kt_string, string_map};
use super::naming;
use super::writer::{Call, KotlinWriter, map_of};

pub(super) fn generate_pack_data(manifest: &Manifest, package_name: &str) -> OutputFile {
    let mut w = KotlinWriter::file(package_name, ["com.chunkzero.window.manifest.WindowManifest"]);
    w.doc("Compiled Window pack manifest.");
    w.open("internal object WindowPackData {");
    let manifest = Call::new("WindowManifest", 2)
        .arg("version", manifest.version)
        .arg("namespace", kt_string(&manifest.namespace))
        .arg("font", kt_string(&manifest.font))
        .arg("spacers", "WindowSpacers.values")
        .arg("textAdvances", "WindowFonts.advances")
        .arg("textGlyphWidths", "WindowFonts.glyphWidths")
        .arg("fontMetrics", "WindowFonts.metrics")
        .arg("sprites", "WindowSprites.all")
        .arg("windows", "WindowEntries.all")
        .arg("huds", "WindowHudEntries.all")
        .finish();
    w.property("val manifest: WindowManifest", manifest);
    w.close("}");
    OutputFile::text("WindowPackData.kt", w.finish())
}

pub(super) fn generate_spacers(manifest: &Manifest, package_name: &str) -> OutputFile {
    let mut w = KotlinWriter::file(package_name, std::iter::empty::<&str>());
    w.doc("Generated spacer glyph advances for the Window pack font.");
    w.open("internal object WindowSpacers {");
    w.property("val values: Map<Int, Int>", map_of(&manifest.spacers, 2));
    w.close("}");
    OutputFile::text("WindowSpacers.kt", w.finish())
}

pub(super) fn generate_fonts(manifest: &Manifest, package_name: &str) -> OutputFile {
    let vanilla = FontMetricsEntry {
        advances: manifest.text_advances.clone(),
        glyph_widths: manifest.text_glyph_widths.clone(),
        bold_advance: crate::vanilla::BOLD_ADVANCE,
    };
    // Fonts with identical metrics share one table, keeping the object initializer within the JVM method size limit.
    let mut shared: Vec<&FontMetricsEntry> = Vec::new();
    let metrics: Vec<(String, String)> = manifest
        .font_metrics
        .iter()
        .map(|(font, metrics)| {
            let expr = if metrics == &vanilla {
                "vanillaMetrics".to_string()
            } else {
                let index = shared.iter().position(|m| *m == metrics).unwrap_or_else(|| {
                    shared.push(metrics);
                    shared.len() - 1
                });
                format!("fontMetrics{index}")
            };
            (kt_string(font), expr)
        })
        .collect();

    let mut w = KotlinWriter::file(package_name, ["com.chunkzero.window.manifest.FontMetricsEntry"]);
    w.doc("Generated font metrics used for runtime text measurement.");
    w.open("internal object WindowFonts {");
    w.property("val advances: Map<String, Int>", char_map(&manifest.text_advances, 2));
    w.property("val glyphWidths: Map<String, Int>", char_map(&manifest.text_glyph_widths, 2));
    w.property(
        "private val vanillaMetrics: FontMetricsEntry",
        font_metrics_expr(&vanilla, "advances", "glyphWidths", 2),
    );
    for (index, metrics) in shared.iter().enumerate() {
        w.property(
            format!("private val fontMetrics{index}: FontMetricsEntry"),
            font_metrics_expr(metrics, "advances", "glyphWidths", 2),
        );
    }
    w.property("val metrics: Map<String, FontMetricsEntry>", map_of(metrics, 2));
    w.close("}");
    OutputFile::text("WindowFonts.kt", w.finish())
}

pub(super) fn generate_sprites(manifest: &Manifest, package_name: &str) -> OutputFile {
    let mut w = KotlinWriter::file(package_name, ["com.chunkzero.window.manifest.SpriteEntry"]);
    w.doc("Generated runtime sprite catalog for this Window pack.");
    w.open("internal object WindowSprites {");
    w.property("val all: Map<String, SpriteEntry>", string_map(&manifest.sprites, 2, sprite_entry_expr));
    w.close("}");
    OutputFile::text("WindowSprites.kt", w.finish())
}

/// The runtime sprites as a `WindowSprite` enum for sprite slot members, or `None` when the pack has neither sprites
/// nor dynamic sprite slots.
pub(super) fn generate_sprite_ids(manifest: &Manifest, package_name: &str) -> Option<OutputFile> {
    let dynamic_slots = manifest.windows.values().flat_map(|window| window.sprite_slots.values());
    if manifest.sprites.is_empty() && dynamic_slots.filter(|slot| slot.sprite.is_none()).count() == 0 {
        return None;
    }
    let mut w = KotlinWriter::file(package_name, std::iter::empty::<&str>());
    w.doc("Runtime sprites of this Window pack, returned by sprite slot members.");
    w.open("public enum class WindowSprite(public val id: String) {");
    // Inline art drawn by fixed sprite slots has no constant; Kotlin cannot select it.
    for name in manifest.sprites.keys().filter(|name| !is_inline(name)) {
        w.line(format_args!("{}({}),", name.to_uppercase(), kt_string(name)));
    }
    w.close("}");
    Some(OutputFile::text("WindowSprite.kt", w.finish()))
}

/// The theme palette as `TextColor` constants, or `None` when the theme declares no colors.
pub(super) fn generate_colors(manifest: &Manifest, package_name: &str) -> Result<Option<OutputFile>> {
    if manifest.colors.is_empty() {
        return Ok(None);
    }
    let mut w = KotlinWriter::file(package_name, ["net.kyori.adventure.text.format.TextColor"]);
    w.doc("Theme palette colors of this Window pack.");
    w.open("public object WindowColors {");
    let mut taken = BTreeMap::new();
    for (name, value) in &manifest.colors {
        let member = naming::definition_member(name);
        if !naming::is_valid_identifier(&member) {
            return Err(Error::Validation(format!("color `{name}` maps to invalid Kotlin identifier `{member}`")));
        }
        if let Some(existing) = taken.insert(member.clone(), name) {
            return Err(Error::Validation(format!(
                "colors `{existing}` and `{name}` both map to `WindowColors.{member}`"
            )));
        }
        let Rgb { r, g, b } = Rgb::parse_hex(value).ok_or_else(|| {
            Error::Validation(format!("color `{name}` has invalid value `{value}`; expected #rrggbb"))
        })?;
        w.line(format_args!("public val {member}: TextColor = TextColor.color(0x{r:02x}{g:02x}{b:02x})"));
    }
    w.close("}");
    Ok(Some(OutputFile::text("WindowColors.kt", w.finish())))
}
