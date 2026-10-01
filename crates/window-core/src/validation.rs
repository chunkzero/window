//! Validation of compiled resource-pack artifacts against the runtime manifest.
//!
//! Compiler unit tests prove individual algorithms. This module validates the
//! seam Minecraft actually consumes: emitted JSON/PNG files, bitmap-provider
//! metrics, baked cursor movement, shifted-font ascent, and inventory ownership.
//! It is intentionally usable by tests and build hosts without filesystem I/O.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Serialize;
use serde_json::Value;

use crate::compose::Texture;
use crate::manifest::{Manifest, SlotAreaEntry, SlotRefEntry, WindowEntry};
use crate::pipeline::{CompileOutput, OutputFile};
use crate::surface::ContainerKind;

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
    validate_debug_descriptor(output, &mut report);
    report.finish()
}

fn validate_debug_descriptor(output: &CompileOutput, report: &mut ValidationReport) {
    let path = crate::debug::output_path(&output.manifest.namespace);
    let Some(emitted) = output.files.iter().find(|file| file.path == path) else {
        report.push("debug.descriptor.missing", path, "compiler output did not emit the active-pack debug descriptor");
        return;
    };

    if let Err(error) = crate::debug::DebugDescriptor::from_json(&emitted.contents) {
        report.push("debug.descriptor.invalid", &emitted.path, error.to_string());
    }

    let expected = match crate::debug::DebugDescriptor::build(&output.manifest, &output.files)
        .and_then(|descriptor| descriptor.to_json_bytes())
    {
        Ok(expected) => expected,
        Err(error) => {
            report.push(
                "debug.descriptor.rebuild_failed",
                &emitted.path,
                format!("could not rebuild compiler expectation: {error}"),
            );
            return;
        }
    };
    if emitted.contents != expected {
        report.push(
            "debug.descriptor.mismatch",
            &emitted.path,
            format!(
                "emitted descriptor is not the canonical {}-byte descriptor rebuilt from the manifest and generated assets",
                expected.len()
            ),
        );
    }
}

#[derive(Clone, Debug, Default)]
struct FontAdvances {
    advances: BTreeMap<char, i32>,
}

fn index_files<'a>(output: &'a [OutputFile], files: &mut BTreeMap<&'a str, &'a [u8]>, report: &mut ValidationReport) {
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

fn validate_manifest_paths(manifest: &Manifest, files: &BTreeMap<&str, &[u8]>, report: &mut ValidationReport) {
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

fn validate_main_font(
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
    metrics
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

fn collect_bitmap_advances(
    provider: &Value,
    location: &str,
    files: &BTreeMap<&str, &[u8]>,
    metrics: &mut FontAdvances,
    report: &mut ValidationReport,
) {
    let Some(file) = provider.get("file").and_then(Value::as_str) else {
        report.push("font.bitmap.file_missing", location, "bitmap provider has no file");
        return;
    };
    let Some(height) = provider.get("height").and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok())
    else {
        report.push("font.bitmap.height_invalid", location, "bitmap provider height is not a u32");
        return;
    };
    let ascent = provider.get("ascent").and_then(Value::as_i64);
    if height > 512 || ascent.is_none_or(|value| value <= -32768 || value > i64::from(height)) {
        report.push(
            "font.bitmap.limits",
            location,
            format!("invalid Minecraft bitmap metrics height={height}, ascent={ascent:?}"),
        );
    }
    let Some(rows) = bitmap_rows(provider, location, report) else {
        return;
    };
    if file.starts_with("minecraft:") {
        return;
    }
    let texture_path = texture_path(file);
    let Some(bytes) = files.get(texture_path.as_str()) else {
        report.push(
            "font.bitmap.texture_missing",
            location,
            format!("bitmap file `{file}` requires emitted `{texture_path}`"),
        );
        return;
    };
    let Ok(texture) = Texture::decode_png(bytes) else {
        report.push("font.bitmap.texture_invalid", &texture_path, "file is not a decodable PNG");
        return;
    };
    collect_bitmap_cells(&texture, &rows, height, location, metrics, report);
}

fn bitmap_rows(provider: &Value, location: &str, report: &mut ValidationReport) -> Option<Vec<Vec<char>>> {
    let Some(values) = provider.get("chars").and_then(Value::as_array) else {
        report.push("font.bitmap.chars_missing", location, "bitmap provider has no chars array");
        return None;
    };
    let mut rows: Vec<Vec<char>> = Vec::with_capacity(values.len());
    for value in values {
        let Some(row) = value.as_str() else {
            report.push("font.bitmap.chars_invalid", location, "bitmap chars rows must be strings");
            return None;
        };
        rows.push(row.chars().collect());
    }
    if rows.is_empty() || rows[0].is_empty() {
        report.push("font.bitmap.chars_empty", location, "bitmap chars grid must not be empty");
        return None;
    }
    let columns = rows[0].len();
    if rows.iter().any(|row| row.len() != columns) {
        report.push("font.bitmap.chars_ragged", location, "all bitmap chars rows must have equal length");
        return None;
    }
    Some(rows)
}

fn collect_bitmap_cells(
    texture: &Texture,
    rows: &[Vec<char>],
    rendered_height: u32,
    location: &str,
    metrics: &mut FontAdvances,
    report: &mut ValidationReport,
) {
    let row_count = rows.len() as u32;
    let column_count = rows[0].len() as u32;
    if !texture.width.is_multiple_of(column_count) || !texture.height.is_multiple_of(row_count) {
        report.push(
            "font.bitmap.grid_mismatch",
            location,
            format!(
                "texture {}x{} is not divisible by chars grid {}x{}",
                texture.width, texture.height, column_count, row_count
            ),
        );
        return;
    }
    let cell_width = texture.width / column_count;
    let cell_height = texture.height / row_count;
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, character) in row.iter().copied().enumerate() {
            if character == '\0' {
                continue;
            }
            let right = cell_opaque_right(
                texture,
                column_index as u32 * cell_width,
                row_index as u32 * cell_height,
                cell_width,
                cell_height,
            );
            let advance =
                right.map_or(0, |right| scale_boundary(right + 1, cell_height, rendered_height).saturating_add(1));
            let Ok(advance) = i32::try_from(advance) else {
                report.push("font.advance.overflow", location, format!("bitmap advance for `{character}` exceeds i32"));
                continue;
            };
            insert_advance(metrics, character, advance, location, report);
        }
    }
}

fn cell_opaque_right(texture: &Texture, origin_x: u32, origin_y: u32, width: u32, height: u32) -> Option<u32> {
    let mut right = None;
    for y in origin_y..origin_y + height {
        for local_x in 0..width {
            let x = origin_x + local_x;
            let alpha = texture.rgba[((y * texture.width + x) * 4 + 3) as usize];
            if alpha != 0 {
                right = Some(right.map_or(local_x, |current: u32| current.max(local_x)));
            }
        }
    }
    right
}

fn scale_boundary(value: u32, source_height: u32, rendered_height: u32) -> u32 {
    let value = u128::from(value);
    let source_height = u128::from(source_height.max(1));
    let rendered_height = u128::from(rendered_height);
    ((value * rendered_height * 2 + source_height) / (source_height * 2)).min(u128::from(u32::MAX)) as u32
}

fn insert_advance(
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

fn validate_static_segments(manifest: &Manifest, metrics: &FontAdvances, report: &mut ValidationReport) {
    for (name, window) in &manifest.windows {
        validate_segment(&window.static_text, 0, metrics, report, format!("manifest.windows.{name}.static"));
    }
    for (name, hud) in &manifest.huds {
        let expected = if hud.shader.is_some() { 0 } else { hud.surface.width as i32 };
        validate_segment(&hud.static_text, expected, metrics, report, format!("manifest.huds.{name}.static"));
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

fn validate_shifted_fonts(manifest: &Manifest, files: &BTreeMap<&str, &[u8]>, report: &mut ValidationReport) {
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
    for (font, offset) in expected {
        let path = font_path(font);
        let Some(bytes) = files.get(path.as_str()) else {
            continue;
        };
        let Some(document) = parse_json(bytes, &path, report) else {
            continue;
        };
        let Some(provider) = providers(&document).iter().find(|provider| {
            provider.get("type").and_then(Value::as_str) == Some("bitmap")
                && provider.get("file").and_then(Value::as_str) == Some("minecraft:font/ascii.png")
        }) else {
            report.push("font.shifted.ascii_missing", &path, "shifted text font has no vanilla ASCII provider");
            continue;
        };
        let actual = provider.get("ascent").and_then(Value::as_i64);
        let wanted = i64::from(7 - offset);
        if actual != Some(wanted) {
            report.push(
                "font.shifted.ascent_mismatch",
                &path,
                format!("ASCII ascent is {actual:?}; expected {wanted} for offset {offset}"),
            );
        }

        let mut provided = BTreeSet::new();
        for provider in providers(&document) {
            match provider.get("type").and_then(Value::as_str) {
                Some("space") => {
                    if let Some(advances) = provider.get("advances").and_then(Value::as_object) {
                        for character in advances.keys().filter_map(|text| one_char(text)) {
                            provided.insert(character);
                        }
                    }
                }
                Some("bitmap") => {
                    if let Some(rows) = provider.get("chars").and_then(Value::as_array) {
                        for row in rows.iter().filter_map(Value::as_str) {
                            provided.extend(row.chars().filter(|character| *character != '\0'));
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(metrics) = manifest.font_metrics.get(font) {
            for character in metrics.advances.keys() {
                if !provided.contains(character) {
                    report.push(
                        "font.shifted.glyph_missing",
                        &path,
                        format!(
                            "manifest metrics declare U+{:04X}, but the shifted font has no provider for it",
                            *character as u32
                        ),
                    );
                }
            }
        }
    }
}

fn validate_sprite_fonts(manifest: &Manifest, files: &BTreeMap<&str, &[u8]>, report: &mut ValidationReport) {
    let expected_glyphs: BTreeSet<char> =
        manifest.sprites.values().filter_map(|sprite| one_char(&sprite.glyph)).collect();
    for (window_name, window) in &manifest.windows {
        for (slot_name, slot) in &window.sprite_slots {
            let expected_ascent = i64::from(7 - (slot.y - window.surface.title_origin[1]));
            validate_sprite_font(
                &slot.font,
                expected_ascent,
                &format!("{window_name}.{slot_name}"),
                &expected_glyphs,
                files,
                report,
            );
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
        for (button_name, button) in &window.buttons {
            if let Some(font) = &button.sprite_font {
                let expected_ascent = i64::from(7 - (button.y - window.surface.title_origin[1]));
                validate_sprite_font(
                    font,
                    expected_ascent,
                    &format!("{window_name}.{button_name}"),
                    &expected_glyphs,
                    files,
                    report,
                );
            }
            for (state_name, state) in &button.states {
                if let Some(sprite) = &state.sprite
                    && !manifest.sprites.contains_key(sprite)
                {
                    report.push(
                        "sprite.reference.unknown",
                        format!("manifest.windows.{window_name}.buttons.{button_name}.states.{state_name}.sprite"),
                        format!("unknown sprite `{sprite}`"),
                    );
                }
            }
        }
    }
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

fn validate_sprite_font(
    font: &str,
    expected_ascent: i64,
    owner: &str,
    expected_glyphs: &BTreeSet<char>,
    files: &BTreeMap<&str, &[u8]>,
    report: &mut ValidationReport,
) {
    let path = font_path(font);
    let Some(bytes) = files.get(path.as_str()) else {
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
        if let Some(rows) = provider.get("chars").and_then(Value::as_array) {
            for row in rows.iter().filter_map(Value::as_str) {
                glyphs.extend(row.chars().filter(|character| *character != '\0'));
            }
        }
    }
    if glyphs != *expected_glyphs {
        report.push(
            "font.sprite.catalog_mismatch",
            &path,
            format!("font contains {} sprite glyph(s); manifest requires {}", glyphs.len(), expected_glyphs.len()),
        );
    }
}

fn validate_inventory(manifest: &Manifest, report: &mut ValidationReport) {
    for (window_name, window) in &manifest.windows {
        let Some(kind) = ContainerKind::parse(&window.surface.container) else {
            report.push(
                "inventory.container.unknown",
                format!("manifest.windows.{window_name}.surface.container"),
                format!("unsupported container `{}`", window.surface.container),
            );
            continue;
        };
        let mut owners: BTreeMap<SlotRefEntry, String> = BTreeMap::new();
        let mut routes: BTreeMap<SlotRefEntry, String> = BTreeMap::new();
        for (name, button) in &window.buttons {
            // Ownership tracks the slot a control *fills*; routing is a separate,
            // also-exclusive map so a repeater cell can route a slot it no longer
            // fills.
            validate_owned_slots(
                window_name,
                kind,
                &format!("button `{name}`"),
                button.filled_slots(),
                &mut owners,
                report,
            );
            if let Some(fill_slots) = &button.fill_slots {
                for slot in fill_slots {
                    if !button.slots.contains(slot) {
                        report.push(
                            "inventory.slot.fill_not_routed",
                            format!("manifest.windows.{window_name}.buttons.{name}.fill_slots"),
                            format!("button `{name}` fills {:?} slot {} that it does not route", slot.area, slot.index),
                        );
                    }
                }
                if fill_slots.is_empty() {
                    report.push(
                        "inventory.slot.fill_empty",
                        format!("manifest.windows.{window_name}.buttons.{name}.fill_slots"),
                        format!("button `{name}` fills no slot; omit `fill_slots` instead"),
                    );
                }
            }
            for slot in &button.slots {
                if let Some(previous) = routes.insert(*slot, name.clone()) {
                    report.push(
                        "inventory.slot.duplicate_route",
                        format!("manifest.windows.{window_name}"),
                        format!(
                            "{:?} slot {} routes clicks to both button `{previous}` and \
                             button `{name}`",
                            slot.area, slot.index
                        ),
                    );
                }
            }
        }
        for (name, item) in &window.items {
            validate_owned_slots(window_name, kind, &format!("item `{name}`"), &item.slots, &mut owners, report);
        }
        if window.inputs.len() > 1 {
            report.push(
                "inventory.input.too_many",
                format!("manifest.windows.{window_name}.inputs"),
                "anvil surfaces support at most one native text input",
            );
        }
        for (name, input) in &window.inputs {
            validate_owned_slots(
                window_name,
                kind,
                &format!("anvil input `{name}`"),
                &[input.slot],
                &mut owners,
                report,
            );
            if kind != ContainerKind::Anvil {
                report.push(
                    "inventory.input.surface_mismatch",
                    format!("manifest.windows.{window_name}.inputs.{name}"),
                    "anvil input requires container `anvil`",
                );
            }
            let expected_slot = SlotRefEntry { area: SlotAreaEntry::Container, index: 0 };
            if input.slot != expected_slot {
                report.push(
                    "inventory.input.slot_mismatch",
                    format!("manifest.windows.{window_name}.inputs.{name}.slot"),
                    "anvil input must own container slot 0",
                );
            }
        }
        for (name, collection) in &window.collections {
            validate_owned_slots(
                window_name,
                kind,
                &format!("collection `{name}`"),
                &collection.slots,
                &mut owners,
                report,
            );
            if !collection.action {
                continue;
            }
            for slot in &collection.slots {
                if let Some(previous) = routes.insert(*slot, name.clone()) {
                    report.push(
                        "inventory.slot.duplicate_route",
                        format!("manifest.windows.{window_name}"),
                        format!(
                            "{:?} slot {} routes clicks to both `{previous}` and collection \
                             `{name}`",
                            slot.area, slot.index
                        ),
                    );
                }
            }
        }
        for (name, slot_rect) in &window.slot_rects {
            validate_owned_slots(
                window_name,
                kind,
                &format!("slot rect `{name}`"),
                &slot_rect.slots,
                &mut owners,
                report,
            );
        }
        validate_groups(window_name, window, report);
    }
}

fn validate_owned_slots(
    window_name: &str,
    kind: ContainerKind,
    owner: &str,
    slots: &[SlotRefEntry],
    owners: &mut BTreeMap<SlotRefEntry, String>,
    report: &mut ValidationReport,
) {
    for slot in slots {
        let valid = match slot.area {
            SlotAreaEntry::Container => slot.index < kind.slot_count(),
            SlotAreaEntry::Player => slot.index < 36,
        };
        if !valid {
            report.push(
                "inventory.slot.out_of_bounds",
                format!("manifest.windows.{window_name}"),
                format!("{owner} owns invalid {:?} slot {}", slot.area, slot.index),
            );
        }
        if let Some(previous) = owners.insert(*slot, owner.to_string()) {
            report.push(
                "inventory.slot.duplicate_owner",
                format!("manifest.windows.{window_name}"),
                format!("{:?} slot {} is owned by both {previous} and {owner}", slot.area, slot.index),
            );
        }
    }
}

fn validate_groups(window_name: &str, window: &WindowEntry, report: &mut ValidationReport) {
    for (group_name, group) in &window.groups {
        for (field, names) in &group.slots {
            validate_group_names(window_name, group_name, field, group.count, names, &window.slots, "slot", report);
        }
        for (field, names) in &group.sprite_slots {
            validate_group_names(
                window_name,
                group_name,
                field,
                group.count,
                names,
                &window.sprite_slots,
                "sprite slot",
                report,
            );
        }
        for (field, names) in &group.items {
            validate_group_names(window_name, group_name, field, group.count, names, &window.items, "item", report);
        }
        if !group.buttons.is_empty() {
            validate_group_names(
                window_name,
                group_name,
                "buttons",
                group.count,
                &group.buttons,
                &window.buttons,
                "button",
                report,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_group_names<T>(
    window_name: &str,
    group_name: &str,
    field: &str,
    count: u32,
    names: &[String],
    entries: &BTreeMap<String, T>,
    kind: &str,
    report: &mut ValidationReport,
) {
    let location = format!("manifest.windows.{window_name}.groups.{group_name}.{field}");
    if names.len() != count as usize {
        report.push(
            "group.count.mismatch",
            &location,
            format!("contains {} names but group count is {count}", names.len()),
        );
    }
    for name in names {
        if !entries.contains_key(name) {
            report.push("group.control.missing", &location, format!("references missing {kind} `{name}`"));
        }
    }
}

fn parse_json(bytes: &[u8], path: &str, report: &mut ValidationReport) -> Option<Value> {
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            report.push("artifact.json.invalid", path, error.to_string());
            None
        }
    }
}

fn providers(document: &Value) -> &[Value] {
    document.get("providers").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn one_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let character = chars.next()?;
    chars.next().is_none().then_some(character)
}

fn font_path(font: &str) -> String {
    let (namespace, path) = split_resource_id(font);
    format!("assets/{namespace}/font/{path}.json")
}

fn texture_path(texture: &str) -> String {
    let (namespace, path) = split_resource_id(texture);
    format!("assets/{namespace}/textures/{path}")
}

fn split_resource_id(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::pipeline::{CompileInput, compile_project_json};

    fn compile(project: &str) -> CompileOutput {
        compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap()
    }

    fn project() -> &'static str {
        r##"{
          "windows": [{
            "name": "validation",
            "container": "generic_9x3",
            "children": [
              {"type":"panel","frame":"panel","width":32,"height":16,"x":8,"y":0},
              {"type":"slot","name":"title","width":30,"x":9,"y":5},
              {"type":"button","name":"buy","width":16,"height":16,"x":8,"y":18}
            ]
          }],
          "theme": {"frames": {"panel": {
            "kind":"panel", "fill":"#123456", "border_color":"#abcdef",
            "border_width":1, "radius":0, "inset_depth":0
          }}}
        }"##
    }

    #[test]
    fn validates_real_compiler_output() {
        let report = validate_compile_output(&compile(project()));
        report.assert_valid();
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.checks.len(), validation_catalog().len());
        assert!(report.checks.iter().all(|check| check.status == ValidationStatus::Passed));
    }

    #[test]
    fn exports_named_checks_and_findings_as_json() {
        let mut output = compile(project());
        output.files.push(output.files[0].clone());
        let report = validate_compile_output(&output);
        let document: Value = serde_json::from_slice(&report.to_json_pretty().unwrap()).unwrap();

        assert_eq!(document["schema_version"], 1);
        assert_eq!(document["valid"], false);
        let artifact_check =
            document["checks"].as_array().unwrap().iter().find(|check| check["id"] == "artifact_set").unwrap();
        assert_eq!(artifact_check["status"], "failed");
        assert!(artifact_check["finding_count"].as_u64().unwrap() > 0);
        assert_eq!(document["issues"][0]["code"], "artifact.order.unstable");
    }

    #[test]
    fn reports_all_duplicate_and_unsorted_paths() {
        let mut output = compile(project());
        let duplicate = output.files[0].clone();
        output.files.push(duplicate);
        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "artifact.path.duplicate"));
        assert!(report.issues.iter().any(|issue| issue.code == "artifact.order.unstable"));
    }

    #[test]
    fn catches_corrupt_font_json_without_panicking() {
        let mut output = compile(project());
        let main = output.files.iter_mut().find(|file| file.path.ends_with("/font/ui.json")).unwrap();
        main.contents = b"not json".to_vec();
        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "artifact.json.invalid"));
    }

    #[test]
    fn catches_debug_descriptor_drift_from_compiled_definition() {
        let mut output = compile(project());
        output.manifest.windows.get_mut("validation").unwrap().slots.get_mut("title").unwrap().x += 1;

        let report = validate_compile_output(&output);

        assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.mismatch"));
    }

    #[test]
    fn catches_debug_descriptor_drift_from_generated_asset_bytes() {
        let mut output = compile(project());
        let model = output.files.iter_mut().find(|file| file.path.ends_with("/models/gui/hitbox.json")).unwrap();
        model.contents.push(b'\n');

        let report = validate_compile_output(&output);

        assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.mismatch"));
    }

    #[test]
    fn catches_invalid_debug_descriptor_schema() {
        let mut output = compile(project());
        let descriptor = output.files.iter_mut().find(|file| file.path.ends_with("/window/debug.json")).unwrap();
        let mut document: Value = serde_json::from_slice(&descriptor.contents).unwrap();
        document["schema_version"] = Value::from(999);
        descriptor.contents = serde_json::to_vec(&document).unwrap();

        let report = validate_compile_output(&output);

        assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.invalid"));
        assert!(report.issues.iter().any(|issue| issue.code == "debug.descriptor.mismatch"));
    }

    #[test]
    fn catches_actual_minecraft_cursor_drift() {
        let mut output = compile(project());
        let main = output.files.iter_mut().find(|file| file.path.ends_with("/font/ui.json")).unwrap();
        let mut document: Value = serde_json::from_slice(&main.contents).unwrap();
        let advances = document["providers"][0]["advances"].as_object_mut().unwrap();
        let key = advances.keys().next().unwrap().clone();
        *advances.get_mut(&key).unwrap() = Value::from(-999);
        main.contents = serde_json::to_vec(&document).unwrap();
        let report = validate_compile_output(&output);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| { issue.code == "spacer.advance.mismatch" || issue.code == "segment.advance.mismatch" })
        );
    }

    #[test]
    fn catches_shifted_font_ascent_drift() {
        let mut output = compile(project());
        let shifted = output.files.iter_mut().find(|file| file.path.contains("/font/ym1.json")).unwrap();
        let mut document: Value = serde_json::from_slice(&shifted.contents).unwrap();
        document["providers"][1]["ascent"] = Value::from(0);
        shifted.contents = serde_json::to_vec(&document).unwrap();
        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "font.shifted.ascent_mismatch"));
    }

    #[test]
    fn catches_shifted_font_provider_metric_drift() {
        let mut output = compile(project());
        let shifted = output.files.iter_mut().find(|file| file.path.contains("/font/ym1.json")).unwrap();
        let mut document: Value = serde_json::from_slice(&shifted.contents).unwrap();
        document["providers"][2]["chars"][56] = Value::from("\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
        shifted.contents = serde_json::to_vec(&document).unwrap();
        let report = validate_compile_output(&output);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "font.shifted.glyph_missing" && issue.message.contains("U+25C6"))
        );
    }

    #[test]
    fn catches_fill_slots_that_are_not_routed() {
        let mut output = compile(project());
        let window = output.manifest.windows.get_mut("validation").unwrap();
        let buy = window.buttons.get_mut("buy").unwrap();
        buy.fill_slots = Some(vec![SlotRefEntry { area: SlotAreaEntry::Container, index: 8 }]);
        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.fill_not_routed"), "{report}");
    }

    #[test]
    fn catches_two_buttons_routing_one_slot() {
        let mut output = compile(project());
        let window = output.manifest.windows.get_mut("validation").unwrap();
        let buy = window.buttons.get("buy").unwrap().clone();
        // A second button routes `buy`'s slots but fills none of them, so the
        // ownership map stays clean and only the routing map catches it.
        window.buttons.insert(
            "shadow".into(),
            crate::manifest::ButtonEntry { fill_slots: Some(buy.slots[..1].to_vec()), ..buy.clone() },
        );
        window.buttons.get_mut("buy").unwrap().fill_slots = Some(buy.slots[1..].to_vec());
        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.duplicate_route"), "{report}");
    }

    #[test]
    fn catches_inventory_ownership_and_bounds_corruption() {
        let mut output = compile(project());
        let window = output.manifest.windows.get_mut("validation").unwrap();
        let buy = window.buttons.get_mut("buy").unwrap();
        buy.slots.push(SlotRefEntry { area: SlotAreaEntry::Container, index: 99 });
        window.items.insert("duplicate".into(), crate::manifest::ItemEntry { slots: buy.slots.clone() });
        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.out_of_bounds"));
        assert!(report.issues.iter().any(|issue| issue.code == "inventory.slot.duplicate_owner"));
    }

    #[test]
    fn catches_corrupt_anvil_input_slot() {
        let mut output = compile(
            r#"{
              "windows": [{
                "name": "search",
                "container": "anvil",
                "children": [{ "type": "anvil_input", "name": "query" }]
              }]
            }"#,
        );
        output.manifest.windows.get_mut("search").unwrap().inputs.get_mut("query").unwrap().slot.index = 1;

        let report = validate_compile_output(&output);
        assert!(report.issues.iter().any(|issue| issue.code == "inventory.input.slot_mismatch"));
    }
}
