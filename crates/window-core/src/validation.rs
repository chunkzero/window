//! Validation of compiled resource-pack artifacts against the runtime manifest.
//!
//! Compiler unit tests prove individual algorithms. This module validates the
//! seam Minecraft actually consumes: emitted JSON/PNG files, bitmap-provider
//! metrics, baked cursor movement, shifted-font ascent, and inventory ownership.
//! It is intentionally usable by tests and build hosts without filesystem I/O.

use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;

use crate::pipeline::CompileOutput;
use artifacts::{index_files, validate_manifest_paths};
use debug_check::validate_debug_descriptor;
use fonts::validate_shifted_fonts;
use inventory::validate_inventory;
use main_font::validate_main_font;
use segments::validate_static_segments;
use sprites::validate_sprite_fonts;

mod artifacts;
mod bitmap;
mod debug_check;
mod fonts;
mod groups;
mod inventory;
mod json;
mod main_font;
mod segments;
mod sprites;
#[cfg(test)]
mod tests;

/// One stable, machine-classifiable validation finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ValidationIssue {
    /// Stable identifier suitable for CI filtering.
    pub code: &'static str,
    /// Emitted artifact or manifest location, when applicable.
    pub location: String,
    /// Human-readable explanation.
    pub message: String,
}

/// Stable description of one validation area.
///
/// The catalog is public so build hosts and CI frontends can display what
/// Window validates even before a compiler run has produced a report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ValidationCheckDefinition {
    /// Stable identifier suitable for report consumers.
    pub id: &'static str,
    /// Human-readable description of the contract covered by this check.
    pub description: &'static str,
}

/// Outcome of one validation area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// No findings were produced for this area.
    Passed,
    /// One or more findings were produced for this area.
    Failed,
}

/// Inspectable result for one validation area.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ValidationCheck {
    /// Stable identifier from [`validation_catalog`].
    pub id: &'static str,
    /// Human-readable description of the validated contract.
    pub description: &'static str,
    /// Pass/fail status for this compiler result.
    pub status: ValidationStatus,
    /// Number of findings attributed to this area.
    pub finding_count: usize,
}

const VALIDATION_CATALOG: &[ValidationCheckDefinition] = &[
    ValidationCheckDefinition {
        id: "artifact_set",
        description: "output paths are normalized, unique, and deterministically ordered",
    },
    ValidationCheckDefinition {
        id: "debug_descriptor",
        description: "pack debug descriptor exactly matches compiled semantics and generated assets",
    },
    ValidationCheckDefinition {
        id: "manifest_references",
        description: "manifest font references resolve to emitted resource-pack artifacts",
    },
    ValidationCheckDefinition {
        id: "font_providers",
        description: "font JSON, PNG grids, Minecraft limits, glyph advances, and spacers agree",
    },
    ValidationCheckDefinition {
        id: "cursor_segments",
        description: "static window and HUD segments end at their required cursor positions",
    },
    ValidationCheckDefinition {
        id: "shifted_fonts",
        description: "dynamic text fonts use the ascent required by each authored y offset",
    },
    ValidationCheckDefinition {
        id: "runtime_sprites",
        description: "sprite catalogs, glyph metrics, advances, textures, and ascents agree",
    },
    ValidationCheckDefinition {
        id: "inventory_contract",
        description: "container/player slots are in bounds, exclusively owned, and grouped correctly",
    },
];

/// Return the stable catalog of compiled-artifact checks.
pub fn validation_catalog() -> &'static [ValidationCheckDefinition] {
    VALIDATION_CATALOG
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.location.is_empty() {
            write!(f, "[{}] {}", self.code, self.message)
        } else {
            write!(f, "[{}] {}: {}", self.code, self.location, self.message)
        }
    }
}

/// Complete validation result. Validation accumulates independent findings so
/// one run explains the whole broken artifact set.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ValidationReport {
    /// Report schema version. Incremented only for incompatible JSON changes.
    pub schema_version: u32,
    /// Whether the compiled artifact set satisfies every validation area.
    pub valid: bool,
    /// Named pass/fail results for every area in [`validation_catalog`].
    pub checks: Vec<ValidationCheck>,
    /// All errors, deterministically ordered by validation phase.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Whether no contract violations were found.
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    /// Panic with every finding. Intended for concise test assertions.
    #[track_caller]
    pub fn assert_valid(&self) {
        assert!(self.is_valid(), "{self}");
    }

    /// Serialize this report as stable, human-readable JSON.
    pub fn to_json_pretty(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    fn finish(mut self) -> Self {
        self.schema_version = 1;
        self.valid = self.issues.is_empty();
        self.checks = VALIDATION_CATALOG
            .iter()
            .map(|definition| {
                let finding_count =
                    self.issues.iter().filter(|issue| validation_area(issue.code) == definition.id).count();
                ValidationCheck {
                    id: definition.id,
                    description: definition.description,
                    status: if finding_count == 0 { ValidationStatus::Passed } else { ValidationStatus::Failed },
                    finding_count,
                }
            })
            .collect();
        self
    }

    fn push(&mut self, code: &'static str, location: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ValidationIssue { code, location: location.into(), message: message.into() });
    }
}

fn validation_area(code: &str) -> &'static str {
    if code.starts_with("debug.descriptor.") {
        "debug_descriptor"
    } else if code.starts_with("artifact.path.") || code == "artifact.order.unstable" {
        "artifact_set"
    } else if matches!(code, "font.main.missing" | "font.reference.missing") {
        "manifest_references"
    } else if code.starts_with("segment.") {
        "cursor_segments"
    } else if code.starts_with("font.shifted.") {
        "shifted_fonts"
    } else if code.starts_with("font.sprite.") || code.starts_with("sprite.") {
        "runtime_sprites"
    } else if code.starts_with("inventory.") || code.starts_with("group.") {
        "inventory_contract"
    } else {
        "font_providers"
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            return f.write_str("compiled Window artifacts are valid");
        }
        writeln!(f, "{} compiled-artifact validation error(s):", self.issues.len())?;
        for issue in &self.issues {
            writeln!(f, "- {issue}")?;
        }
        Ok(())
    }
}

/// Validate one in-memory compiler result exactly as it will be emitted to a
/// resource pack.
pub fn validate_compile_output(output: &CompileOutput) -> ValidationReport {
    validate_compile_output_with_pack(output, &BTreeMap::new())
}

/// Validate compiler output together with the pre-existing pack files visible
/// to the compiler. This resolves bitmap providers that intentionally reuse a
/// resource texture instead of emitting a new PNG.
pub fn validate_compile_output_with_pack(
    output: &CompileOutput,
    pack_files: &BTreeMap<String, Vec<u8>>,
) -> ValidationReport {
    validate(output, pack_files, true)
}

/// [`validate_compile_output_with_pack`] for output whose debug descriptor was
/// just built from its files: the descriptor must exist and parse, but is not
/// rebuilt for comparison.
pub(crate) fn validate_compiled(output: &CompileOutput, pack_files: &BTreeMap<String, Vec<u8>>) -> ValidationReport {
    validate(output, pack_files, false)
}

fn validate(
    output: &CompileOutput,
    pack_files: &BTreeMap<String, Vec<u8>>,
    rebuild_descriptor: bool,
) -> ValidationReport {
    let mut report = ValidationReport::default();
    let mut files: BTreeMap<&str, &[u8]> =
        pack_files.iter().map(|(path, contents)| (path.as_str(), contents.as_slice())).collect();
    index_files(&output.files, &mut files, &mut report);
    validate_manifest_paths(&output.manifest, &files, &mut report);
    let main_metrics = validate_main_font(&output.manifest, &files, &mut report);
    validate_static_segments(&output.manifest, &main_metrics, &mut report);
    validate_shifted_fonts(&output.manifest, &files, &mut report);
    validate_sprite_fonts(&output.manifest, &files, &mut report);
    validate_inventory(&output.manifest, &mut report);
    validate_debug_descriptor(output, rebuild_descriptor, &mut report);
    report.finish()
}
