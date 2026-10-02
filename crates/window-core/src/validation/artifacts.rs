use std::collections::{BTreeMap, BTreeSet};

use super::ValidationReport;
use super::json::font_path;
use crate::manifest::Manifest;
use crate::pipeline::OutputFile;

pub(super) fn index_files<'a>(
    output: &'a [OutputFile],
    files: &mut BTreeMap<&'a str, &'a [u8]>,
    report: &mut ValidationReport,
) {
    let mut previous: Option<&str> = None;
    let mut seen = BTreeSet::new();
    for file in output {
        if file.path.starts_with('/')
            || file.path.split('/').any(|part| part.is_empty() || part == "..")
            || file.path.contains('\\')
        {
            report.push(
                "artifact.path.invalid",
                &file.path,
                "path must be normalized, relative, and use forward slashes",
            );
        }
        if let Some(previous) = previous
            && previous > file.path.as_str()
        {
            report.push(
                "artifact.order.unstable",
                &file.path,
                format!("artifact sorts before preceding path `{previous}`"),
            );
        }
        previous = Some(&file.path);
        if !seen.insert(file.path.as_str()) {
            report.push("artifact.path.duplicate", &file.path, "path is emitted more than once");
        }
        files.insert(file.path.as_str(), file.contents.as_slice());
    }
}

pub(super) fn validate_manifest_paths(
    manifest: &Manifest,
    files: &BTreeMap<&str, &[u8]>,
    report: &mut ValidationReport,
) {
    let main = font_path(&manifest.font);
    if !files.contains_key(main.as_str()) {
        report.push("font.main.missing", main, format!("manifest font `{}` was not emitted", manifest.font));
    }
    for (name, window) in &manifest.windows {
        for (slot_name, slot) in &window.slots {
            validate_font_reference(
                &slot.font,
                files,
                report,
                format!("manifest.windows.{name}.slots.{slot_name}.font"),
            );
        }
        for (slot_name, slot) in &window.sprite_slots {
            validate_font_reference(
                &slot.font,
                files,
                report,
                format!("manifest.windows.{name}.sprite_slots.{slot_name}.font"),
            );
        }
        for (button_name, button) in &window.buttons {
            if let Some(font) = &button.sprite_font {
                validate_font_reference(
                    font,
                    files,
                    report,
                    format!("manifest.windows.{name}.buttons.{button_name}.sprite_font"),
                );
            }
        }
    }
    for (name, hud) in &manifest.huds {
        for (slot_name, slot) in &hud.slots {
            validate_font_reference(&slot.font, files, report, format!("manifest.huds.{name}.slots.{slot_name}.font"));
        }
    }
}

fn validate_font_reference(font: &str, files: &BTreeMap<&str, &[u8]>, report: &mut ValidationReport, location: String) {
    if font.starts_with("minecraft:") {
        return;
    }
    let path = font_path(font);
    if !files.contains_key(path.as_str()) {
        report.push("font.reference.missing", location, format!("font `{font}` requires emitted `{path}`"));
    }
}
