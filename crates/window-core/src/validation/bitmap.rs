use std::collections::BTreeMap;

use serde_json::Value;

use super::ValidationReport;
use super::json::texture_path;
use super::main_font::{FontAdvances, insert_advance};
use crate::compose::Texture;

pub(super) fn collect_bitmap_advances(
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
