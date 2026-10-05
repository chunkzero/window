use std::collections::BTreeMap;

use super::ValidationReport;
use super::main_font::FontAdvances;
use crate::manifest::{Manifest, SwitchEntry};

pub(super) fn validate_static_segments(manifest: &Manifest, metrics: &FontAdvances, report: &mut ValidationReport) {
    for (name, window) in &manifest.windows {
        let location = format!("manifest.windows.{name}");
        validate_segment(&window.static_text, 0, metrics, report, format!("{location}.static"));
        validate_cases(&window.switches, metrics, report, &location);
    }
    for (name, hud) in &manifest.huds {
        let location = format!("manifest.huds.{name}");
        let expected = if hud.shader.is_some() { 0 } else { hud.surface.width as i32 };
        validate_segment(&hud.static_text, expected, metrics, report, format!("{location}.static"));
        validate_cases(&hud.switches, metrics, report, &location);
    }
}

/// Switch case segments are net-zero.
fn validate_cases(
    switches: &BTreeMap<String, SwitchEntry>,
    metrics: &FontAdvances,
    report: &mut ValidationReport,
    location: &str,
) {
    for (name, switch) in switches {
        for case in &switch.cases {
            let location = format!("{location}.switches.{name}.{}.static", case.value);
            validate_segment(&case.static_text, 0, metrics, report, location);
        }
    }
}

fn validate_segment(
    text: &str,
    expected: i32,
    metrics: &FontAdvances,
    report: &mut ValidationReport,
    location: String,
) {
    let mut actual = 0i64;
    for character in text.chars() {
        let Some(advance) = metrics.advances.get(&character) else {
            report.push(
                "segment.glyph.missing",
                &location,
                format!("glyph U+{:04X} is absent from the main font", character as u32),
            );
            continue;
        };
        actual += i64::from(*advance);
    }
    if actual != i64::from(expected) {
        report.push(
            "segment.advance.mismatch",
            location,
            format!("Minecraft cursor ends at {actual}px; expected {expected}px"),
        );
    }
}
