//! Runtime theme sprites: decoded assets, manifest entries, and fit checks.

use std::collections::BTreeMap;

use crate::authoring::ParsedProject;
use crate::authoring::art::is_inline;
use crate::compose::Texture;
use crate::geometry::{Rect, Size};
use crate::manifest::SpriteEntry;
use crate::model::{GeneratedStyle, SpriteDef};
use crate::{Error, Result};

use super::glyphs::sprite_glyph;
use super::metrics::{GlyphMetrics, bitmap_metrics};
use super::{CompileContext, OutputFile};

#[derive(Clone, Debug)]
pub(super) struct RuntimeSpriteAsset {
    pub(super) size: Size,
    pub(super) x_offset: u32,
    pub(super) glyph_width: u32,
    pub(super) advance: u32,
    pub(super) file: String,
    pub(super) output: Option<OutputFile>,
}

impl RuntimeSpriteAsset {
    fn new(size: Size, metrics: GlyphMetrics, file: String, output: Option<OutputFile>) -> Self {
        Self {
            size,
            x_offset: metrics.x_offset,
            glyph_width: metrics.glyph_width,
            advance: metrics.advance,
            file,
            output,
        }
    }
}

pub(super) fn runtime_sprite_assets(
    project: &ParsedProject,
    textures: &BTreeMap<String, Texture>,
    namespace: &str,
) -> Result<BTreeMap<String, RuntimeSpriteAsset>> {
    let mut sprites = BTreeMap::new();
    for (name, sprite) in &project.theme.sprites {
        // Inline art drawn only into static art needs no runtime glyph.
        if is_inline(name) && !project.theme.runtime_art.contains(name) {
            continue;
        }
        let asset = match sprite {
            SpriteDef::Texture { texture, size } if is_resource_texture_id(texture) => {
                resource_sprite(name, texture, *size, textures)?
            }
            SpriteDef::Texture { texture, size } => pack_texture_sprite(name, texture, *size, textures, namespace)?,
            SpriteDef::Generated { size, style } => {
                let texture = render_sprite(style, *size, name)?;
                bundled_sprite(name, &texture, *size, namespace)?
            }
        };
        sprites.insert(name.clone(), asset);
    }
    Ok(sprites)
}

/// A sprite referencing a texture id in another pack namespace; its PNG is not
/// re-emitted.
fn resource_sprite(
    name: &str,
    texture: &str,
    size: Option<Size>,
    textures: &BTreeMap<String, Texture>,
) -> Result<RuntimeSpriteAsset> {
    let size = size.ok_or_else(|| {
        Error::Validation(format!(
            "sprite `{name}` uses external texture `{texture}` and must set `width` and `height`"
        ))
    })?;
    let metrics = textures
        .get(&texture_source_path(texture))
        .map(|decoded| bitmap_metrics(decoded, size))
        .unwrap_or(GlyphMetrics { x_offset: 0, glyph_width: size.width, advance: size.width + 1 });
    Ok(RuntimeSpriteAsset::new(size, metrics, resource_texture_file(texture), None))
}

fn pack_texture_sprite(
    name: &str,
    texture: &str,
    size: Option<Size>,
    textures: &BTreeMap<String, Texture>,
    namespace: &str,
) -> Result<RuntimeSpriteAsset> {
    let decoded = textures.get(texture).ok_or_else(|| Error::Texture {
        path: texture.to_string(),
        message: format!("referenced by runtime sprite `{name}` but not provided"),
    })?;
    let size = size.unwrap_or_else(|| Size::new(decoded.width, decoded.height));
    bundled_sprite(name, decoded, size, namespace)
}

/// A sprite whose PNG is emitted under this namespace's font sprite textures.
fn bundled_sprite(name: &str, texture: &Texture, size: Size, namespace: &str) -> Result<RuntimeSpriteAsset> {
    let file_stem = sprite_file_stem(name);
    let output =
        OutputFile::binary(format!("assets/{namespace}/textures/font/sprites/{file_stem}.png"), texture.encode_png()?);
    Ok(RuntimeSpriteAsset::new(
        size,
        bitmap_metrics(texture, size),
        format!("{namespace}:font/sprites/{file_stem}.png"),
        Some(output),
    ))
}

fn render_sprite(style: &GeneratedStyle, size: Size, name: &str) -> Result<Texture> {
    crate::raster::render(style, size)
        .map_err(|error| Error::Texture { path: name.to_string(), message: error.to_string() })
}

fn is_resource_texture_id(texture: &str) -> bool {
    texture.contains(':')
}

fn resource_texture_file(texture: &str) -> String {
    if texture.ends_with(".png") { texture.to_string() } else { format!("{texture}.png") }
}

/// The pack-source path of a `namespace:path` texture id; other textures are already source paths.
pub(crate) fn texture_source_path(texture: &str) -> String {
    let Some((namespace, path)) = texture.split_once(':') else {
        return texture.to_string();
    };
    let path = if path.ends_with(".png") { path.to_string() } else { format!("{path}.png") };
    format!("assets/{namespace}/textures/{path}")
}

fn sprite_file_stem(name: &str) -> String {
    name.replace('_', "/")
}

/// Emit every bundled sprite PNG and build the manifest sprite table.
pub(super) fn sprite_entries(ctx: &mut CompileContext<'_>) -> BTreeMap<String, SpriteEntry> {
    let mut entries = BTreeMap::new();
    for (name, asset) in ctx.runtime_sprites {
        if let Some(output) = &asset.output {
            ctx.files.push(output.clone());
        }
        let glyph = sprite_glyph(name, &ctx.codepoints);
        entries.insert(
            name.clone(),
            SpriteEntry {
                width: asset.size.width,
                height: asset.size.height,
                x_offset: asset.x_offset,
                glyph_width: asset.glyph_width,
                advance: asset.advance,
                glyph: glyph.to_string(),
            },
        );
    }
    entries
}

/// Require `sprite` to exist and fit inside `rect`; `subject` names the region
/// in error messages.
pub(super) fn check_sprite_fits(
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    window: &str,
    subject: &str,
    sprite: &str,
    rect: &Rect,
) -> Result<()> {
    let asset = runtime_sprites.get(sprite).ok_or_else(|| {
        Error::Validation(format!("window `{window}`: {subject} references unknown sprite `{sprite}`"))
    })?;
    if asset.size.width > rect.width || asset.size.height > rect.height {
        return Err(Error::Validation(format!("window `{window}`: sprite `{sprite}` does not fit {subject}")));
    }
    Ok(())
}
