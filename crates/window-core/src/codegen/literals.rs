//! Kotlin literal expressions for primitive manifest values.

use std::collections::BTreeMap;

use crate::ir::{Align, ButtonDefault};

use super::writer::map_of;

/// Renders a `mapOf` keyed by Kotlin string literals, rendering each value at `level + 1`.
pub(super) fn string_map<V>(values: &BTreeMap<String, V>, level: usize, value: impl Fn(&V, usize) -> String) -> String {
    map_of(values.iter().map(|(name, entry)| (kt_string(name), value(entry, level + 1))), level)
}

pub(super) fn char_map(values: &BTreeMap<char, u32>, level: usize) -> String {
    map_of(values.iter().map(|(key, value)| (kt_string(&key.to_string()), value)), level)
}

pub(super) fn align_expr(align: Align) -> &'static str {
    match align {
        Align::Left => "Align.LEFT",
        Align::Center => "Align.CENTER",
        Align::Right => "Align.RIGHT",
    }
}

pub(super) fn optional_button_default_expr(default: Option<ButtonDefault>) -> &'static str {
    match default {
        Some(ButtonDefault::Close) => "ButtonDefault.CLOSE",
        None => "null",
    }
}

pub(super) fn optional_string_expr(value: Option<&String>) -> String {
    value.map_or_else(|| "null".into(), |value| kt_string(value))
}

pub(super) fn string_list_expr(values: &[String]) -> String {
    if values.is_empty() {
        return "emptyList()".into();
    }
    let body = values.iter().map(|value| kt_string(value)).collect::<Vec<_>>().join(", ");
    format!("listOf({body})")
}

pub(super) fn float_expr(value: f32) -> String {
    let mut out = value.to_string();
    if !out.contains('.') && !out.contains('e') && !out.contains('E') {
        out.push_str(".0");
    }
    out
}

pub(super) fn kt_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_ascii_graphic() || ch == ' ' => out.push(ch),
            ch => {
                let mut units = [0u16; 2];
                for unit in ch.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04X}"));
                }
            }
        }
    }
    out.push('"');
    out
}
