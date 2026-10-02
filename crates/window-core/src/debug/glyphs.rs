//! Font-JSON walking and bitmap glyph measurement.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::compose::Texture;
use crate::manifest::Manifest;
use crate::pipeline::OutputFile;
use crate::{Error, Result};

/// Measured metrics of one bitmap glyph in the main Window font.
#[derive(Clone, Debug)]
pub(super) struct BitmapGlyph {
    pub(super) file: String,
    pub(super) ascent: i32,
    pub(super) height: u32,
    pub(super) advance: u32,
    pub(super) width: u32,
}

/// A decoded `bitmap` font provider.
struct BitmapProvider<'a> {
    file: &'a str,
    ascent: i32,
    height: u32,
    texture: Texture,
    rows: &'a [Value],
}

impl<'a> BitmapProvider<'a> {
    fn parse(provider: &'a Value, files: &BTreeMap<&str, &[u8]>) -> Result<Self> {
        let file =
            provider["file"].as_str().ok_or_else(|| Error::Manifest("bitmap provider is missing file".into()))?;
        let height = provider["height"]
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| Error::Manifest("bitmap provider has invalid height".into()))?;
        let ascent = provider["ascent"]
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| Error::Manifest("bitmap provider has invalid ascent".into()))?;
        let texture_path = texture_output_path(file);
        let texture_bytes = files
            .get(texture_path.as_str())
            .ok_or_else(|| Error::Manifest(format!("bitmap provider resource `{file}` was not emitted")))?;
        let texture = Texture::decode_png(texture_bytes)
            .map_err(|error| Error::Texture { path: texture_path, message: error.to_string() })?;
        let rows =
            provider["chars"].as_array().ok_or_else(|| Error::Manifest("bitmap provider has invalid chars".into()))?;
        Ok(Self { file, ascent, height, texture, rows })
    }

    fn measure_into(&self, out: &mut BTreeMap<char, BitmapGlyph>) -> Result<()> {
        let row_count = u32::try_from(self.rows.len()).unwrap_or(u32::MAX).max(1);
        let cell_height = self.texture.height / row_count;
        for (row_index, row) in self.rows.iter().enumerate() {
            let row = row.as_str().ok_or_else(|| Error::Manifest("bitmap provider row is not a string".into()))?;
            let columns = u32::try_from(row.chars().count()).unwrap_or(u32::MAX).max(1);
            let cell_width = self.texture.width / columns;
            for (column_index, character) in row.chars().enumerate() {
                if character == '\0' {
                    continue;
                }
                let right = opaque_right(
                    &self.texture,
                    column_index as u32 * cell_width,
                    row_index as u32 * cell_height,
                    cell_width,
                    cell_height,
                );
                let visible = right.map_or(0, |right| right + 1);
                let scaled = scale_boundary(visible, cell_height.max(1), self.height);
                out.insert(
                    character,
                    BitmapGlyph {
                        file: self.file.to_string(),
                        ascent: self.ascent,
                        height: self.height,
                        advance: if visible == 0 { 0 } else { scaled + 1 },
                        width: scale_boundary(cell_width, cell_height.max(1), self.height),
                    },
                );
            }
        }
        Ok(())
    }
}

pub(super) fn main_bitmap_glyphs(
    manifest: &Manifest,
    files: &BTreeMap<&str, &[u8]>,
) -> Result<BTreeMap<char, BitmapGlyph>> {
    let path = font_output_path(&manifest.font);
    let bytes =
        files.get(path.as_str()).ok_or_else(|| Error::Manifest(format!("debug descriptor requires `{path}`")))?;
    let document: Value = serde_json::from_slice(bytes).map_err(|error| Error::Manifest(error.to_string()))?;
    let mut out = BTreeMap::new();
    for provider in document["providers"].as_array().into_iter().flatten() {
        if provider["type"].as_str() == Some("bitmap") {
            BitmapProvider::parse(provider, files)?.measure_into(&mut out)?;
        }
    }
    Ok(out)
}

fn opaque_right(texture: &Texture, x: u32, y: u32, width: u32, height: u32) -> Option<u32> {
    let mut right = None;
    for local_y in 0..height {
        for local_x in 0..width {
            let index = (((y + local_y) * texture.width + x + local_x) * 4 + 3) as usize;
            if texture.rgba.get(index).copied().unwrap_or(0) != 0 {
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

fn font_documents<'a>(files: &'a [&OutputFile]) -> impl Iterator<Item = Result<Value>> + 'a {
    files.iter().filter(|file| file.path.contains("/font/") && file.path.ends_with(".json")).map(|file| {
        serde_json::from_slice(&file.contents).map_err(|error| Error::Manifest(format!("{}: {error}", file.path)))
    })
}

pub(super) fn referenced_bitmap_resources(files: &[&OutputFile]) -> Result<BTreeSet<String>> {
    let mut referenced = BTreeSet::new();
    for document in font_documents(files) {
        let document = document?;
        for provider in document["providers"].as_array().into_iter().flatten() {
            if provider["type"].as_str() == Some("bitmap")
                && let Some(resource) = provider["file"].as_str()
            {
                referenced.insert(resource.to_string());
            }
        }
    }
    Ok(referenced)
}

pub(super) fn bitmap_resources_by_glyph(files: &[&OutputFile]) -> Result<BTreeMap<char, String>> {
    let mut resources = BTreeMap::new();
    for document in font_documents(files) {
        let document = document?;
        for provider in document["providers"].as_array().into_iter().flatten() {
            let Some(resource) = provider["file"].as_str() else {
                continue;
            };
            for row in provider["chars"].as_array().into_iter().flatten() {
                for glyph in row.as_str().into_iter().flat_map(str::chars) {
                    if glyph != '\0' {
                        resources.entry(glyph).or_insert_with(|| resource.to_string());
                    }
                }
            }
        }
    }
    Ok(resources)
}

fn font_output_path(font: &str) -> String {
    let (namespace, path) = font.split_once(':').unwrap_or(("minecraft", font));
    format!("assets/{namespace}/font/{path}.json")
}

fn texture_output_path(resource: &str) -> String {
    let (namespace, path) = resource.split_once(':').unwrap_or(("minecraft", resource));
    format!("assets/{namespace}/textures/{path}")
}
