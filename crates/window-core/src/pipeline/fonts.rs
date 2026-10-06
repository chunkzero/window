//! Font provider documents, font metrics, and the hitbox item.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::Texture;
use crate::font::{
    bitmap_provider_file, main_font, provider_font, shifted_font, shifted_suffix, space_provider, text_font,
    text_font_suffix,
};
use crate::manifest::FontMetricsEntry;
use crate::{Error, Result, vanilla};

use super::glyphs::sprite_glyph;
use super::sprites::RuntimeSpriteAsset;
use super::{CompileContext, OutputFile};

/// Emit the main font (space provider, then static bitmap providers in push
/// order), the hitbox item, one shifted label font per text offset, one
/// document per text font offset plus each used text font's sheet, and one
/// sprite font per sprite offset.
pub(super) fn emit_fonts(ctx: &mut CompileContext<'_>) -> Result<()> {
    let namespace = ctx.namespace;
    let main = main_font(space_provider(), std::mem::take(&mut ctx.bitmap_providers));
    ctx.files.push(OutputFile::text(format!("assets/{namespace}/font/ui.json"), to_json(&main)?));
    emit_hitbox_item(namespace, &mut ctx.files)?;

    for k in &ctx.shift_offsets {
        ctx.files.push(OutputFile::text(
            format!("assets/{namespace}/font/{}.json", shifted_suffix(*k)),
            to_json(&shifted_font(*k)?)?,
        ));
    }
    let mut sheets = BTreeSet::new();
    for &(name, k) in &ctx.text_font_offsets {
        let font = &ctx.text_fonts[name];
        let file = format!("{namespace}:font/text/{name}.png");
        ctx.files.push(OutputFile::text(
            format!("assets/{namespace}/font/{}.json", text_font_suffix(name, k)),
            to_json(&text_font(&file, &font.chars, k)?)?,
        ));
        if sheets.insert(name) {
            ctx.files.push(OutputFile::binary(
                format!("assets/{namespace}/textures/font/text/{name}.png"),
                font.texture.encode_png()?,
            ));
        }
    }
    for k in &ctx.sprite_offsets {
        ctx.files.push(OutputFile::text(
            format!("assets/{namespace}/font/sprite_{}.json", shifted_suffix(*k)),
            to_json(&sprite_font(*k, ctx.runtime_sprites, &ctx.codepoints)?)?,
        ));
    }
    Ok(())
}

fn sprite_font(
    k: i32,
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    codepoints: &BTreeMap<String, u32>,
) -> Result<serde_json::Value> {
    let ascent = 7 - k;
    let mut providers = Vec::with_capacity(runtime_sprites.len());
    for (name, asset) in runtime_sprites {
        providers.push(bitmap_provider_file(
            &asset.file,
            &format!("sprite `{name}` at y offset {k}"),
            asset.size.height,
            ascent,
            sprite_glyph(name, codepoints),
        )?);
    }
    Ok(provider_font(providers))
}

pub(super) fn font_metrics(
    ctx: &CompileContext<'_>,
    text_advances: &BTreeMap<char, u32>,
    text_glyph_widths: &BTreeMap<char, u32>,
) -> BTreeMap<String, FontMetricsEntry> {
    let mut metrics = BTreeMap::new();
    let entry = FontMetricsEntry {
        advances: text_advances.clone(),
        glyph_widths: text_glyph_widths.clone(),
        bold_advance: vanilla::BOLD_ADVANCE,
    };
    metrics.insert("minecraft:default".into(), entry.clone());
    let namespace = ctx.namespace;
    for k in &ctx.shift_offsets {
        metrics.insert(format!("{namespace}:{}", shifted_suffix(*k)), entry.clone());
    }
    for &(name, k) in &ctx.text_font_offsets {
        metrics.insert(format!("{namespace}:{}", text_font_suffix(name, k)), ctx.text_fonts[name].metrics());
    }
    metrics
}

fn emit_hitbox_item(namespace: &str, files: &mut Vec<OutputFile>) -> Result<()> {
    files.push(OutputFile::text(
        format!("assets/{namespace}/items/gui/hitbox.json"),
        to_json(&serde_json::json!({
            "model": {
                "type": "minecraft:model",
                "model": format!("{namespace}:gui/hitbox"),
            },
        }))?,
    ));
    files.push(OutputFile::text(
        format!("assets/{namespace}/models/gui/hitbox.json"),
        to_json(&serde_json::json!({
            "parent": "minecraft:item/generated",
            "textures": {
                "layer0": "minecraft:item/barrier",
            },
            "display": {
                "gui": {
                    "scale": [0, 0, 0],
                },
            },
        }))?,
    ));
    files.push(OutputFile::binary(
        format!("assets/{namespace}/textures/gui/hitbox.png"),
        Texture { width: 1, height: 1, rgba: vec![0, 0, 0, 0] }.encode_png()?,
    ));
    Ok(())
}

/// Serialize a JSON value deterministically (pretty, trailing newline).
fn to_json(value: &serde_json::Value) -> Result<String> {
    let mut json = serde_json::to_string_pretty(value).map_err(|e| Error::Font(e.to_string()))?;
    json.push('\n');
    Ok(json)
}
