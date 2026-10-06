//! Pack-level objects: `WindowPackData`, `WindowSpacers`, `WindowFonts`, and `WindowSprites`.

use crate::manifest::{FontMetricsEntry, Manifest};
use crate::pipeline::OutputFile;

use super::entries::{font_metrics_expr, sprite_entry_expr};
use super::literals::{char_map, kt_string, string_map};
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
