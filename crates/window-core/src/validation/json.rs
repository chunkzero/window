use std::collections::BTreeSet;

use serde_json::Value;

use super::ValidationReport;

pub(super) fn parse_json(bytes: &[u8], path: &str, report: &mut ValidationReport) -> Option<Value> {
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            report.push("artifact.json.invalid", path, error.to_string());
            None
        }
    }
}

pub(super) fn providers(document: &Value) -> &[Value] {
    document.get("providers").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

pub(super) fn one_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let character = chars.next()?;
    chars.next().is_none().then_some(character)
}

/// Add every non-NUL character in a bitmap provider's `chars` grid to `glyphs`.
pub(super) fn extend_bitmap_glyphs(provider: &Value, glyphs: &mut BTreeSet<char>) {
    if let Some(rows) = provider.get("chars").and_then(Value::as_array) {
        for row in rows.iter().filter_map(Value::as_str) {
            glyphs.extend(row.chars().filter(|character| *character != '\0'));
        }
    }
}

pub(super) fn font_path(font: &str) -> String {
    let (namespace, path) = split_resource_id(font);
    format!("assets/{namespace}/font/{path}.json")
}

pub(super) fn texture_path(texture: &str) -> String {
    let (namespace, path) = split_resource_id(texture);
    format!("assets/{namespace}/textures/{path}")
}

fn split_resource_id(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}
