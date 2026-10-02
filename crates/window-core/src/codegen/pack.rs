//! Pack-level objects: `WindowPack`, `WindowSpacers`, `WindowFonts`, and `WindowSprites`.

use crate::manifest::{FontMetricsEntry, Manifest};
use crate::pipeline::OutputFile;

use super::entries::{font_metrics_expr, sprite_entry_expr};
use super::literals::{char_map, kt_string, string_map};
use super::writer::{Call, KotlinWriter, map_of};

pub(super) fn generate_pack(manifest: &Manifest, package_name: &str) -> OutputFile {
    let imports = ["dev.oglass.window.Windows", "dev.oglass.window.manifest.WindowManifest"];
    let mut w = KotlinWriter::file(package_name, imports);
    w.doc("Compiled Window pack definition.");
    w.open("public object WindowPack {");
    let definition = Call::new("WindowManifest", 2)
        .arg("version", manifest.version)
        .arg("namespace", kt_string(&manifest.namespace))
        .arg("font", kt_string(&manifest.font))
        .arg("spacers", "WindowSpacers.values")
        .arg("textAdvances", "WindowFonts.advances")
        .arg("textGlyphWidths", "WindowFonts.glyphWidths")
        .arg("fontMetrics", "WindowFonts.metrics")
        .arg("sprites", "WindowSprites.all")
        .arg("windows", "WindowDefinitions.all")
        .arg("huds", "WindowHudDefinitions.all")
        .finish();
    w.property("public val definition: WindowManifest", definition);
    w.blank();
    w.line("public fun windows(): Windows = Windows.load(definition)");
    w.close("}");
    OutputFile { path: "WindowPack.kt".into(), contents: w.finish().into_bytes() }
}

pub(super) fn generate_spacers(manifest: &Manifest, package_name: &str) -> OutputFile {
    let mut w = KotlinWriter::file(package_name, std::iter::empty::<&str>());
    w.doc("Generated spacer glyph advances for the Window pack font.");
    w.open("public object WindowSpacers {");
    w.property("val values: Map<Int, Int>", map_of(&manifest.spacers, 2));
    w.close("}");
    OutputFile { path: "WindowSpacers.kt".into(), contents: w.finish().into_bytes() }
}

pub(super) fn generate_fonts(manifest: &Manifest, package_name: &str) -> OutputFile {
    let vanilla = FontMetricsEntry {
        advances: manifest.text_advances.clone(),
        glyph_widths: manifest.text_glyph_widths.clone(),
        bold_advance: crate::vanilla::BOLD_ADVANCE,
    };
    let metrics = manifest.font_metrics.iter().map(|(font, metrics)| {
        let expr = if metrics == &vanilla {
            "vanillaMetrics".to_string()
        } else {
            font_metrics_expr(metrics, "advances", "glyphWidths", 4)
        };
        (kt_string(font), expr)
    });

    let mut w = KotlinWriter::file(package_name, ["dev.oglass.window.manifest.FontMetricsEntry"]);
    w.doc("Generated font metrics used for runtime text measurement.");
    w.open("public object WindowFonts {");
    w.property("val advances: Map<String, Int>", char_map(&manifest.text_advances, 2));
    w.property("val glyphWidths: Map<String, Int>", char_map(&manifest.text_glyph_widths, 2));
    w.property(
        "private val vanillaMetrics: FontMetricsEntry",
        font_metrics_expr(&vanilla, "advances", "glyphWidths", 2),
    );
    w.property("val metrics: Map<String, FontMetricsEntry>", map_of(metrics, 2));
    w.close("}");
    OutputFile { path: "WindowFonts.kt".into(), contents: w.finish().into_bytes() }
}

pub(super) fn generate_sprites(manifest: &Manifest, package_name: &str) -> OutputFile {
    let mut w = KotlinWriter::file(package_name, ["dev.oglass.window.manifest.SpriteEntry"]);
    w.doc("Generated runtime sprite catalog for this Window pack.");
    w.open("public object WindowSprites {");
    w.property("val all: Map<String, SpriteEntry>", string_map(&manifest.sprites, 2, sprite_entry_expr));
    w.close("}");
    OutputFile { path: "WindowSprites.kt".into(), contents: w.finish().into_bytes() }
}
