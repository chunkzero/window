//! Font provider documents, font metrics, and the hitbox item.

use std::collections::{BTreeMap, BTreeSet};

use crate::compose::Texture;
use crate::font::{
    bitmap_provider_file, main_font, provider_font, shifted_font, shifted_suffix, space_provider, text_font,
    text_font_suffix,
};
use crate::manifest::FontMetricsEntry;
use crate::size::Part;
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
    ctx.push(Part::MainFont, OutputFile::text(format!("assets/{namespace}/font/ui.json"), to_json(&main)?));
    for file in hitbox_item(namespace)? {
        ctx.push(Part::Hitbox, file);
    }

    let mut emitted = Vec::new();
    for k in &ctx.shift_offsets {
        let suffix = shifted_suffix(*k);
        let file = OutputFile::text(format!("assets/{namespace}/font/{suffix}.json"), to_json(&shifted_font(*k)?)?);
        emitted.push((Part::TextRowFont(suffix), file));
    }
    let mut sheets = BTreeSet::new();
    for &(name, k) in &ctx.text_font_offsets {
        let font = &ctx.text_fonts[name];
        let file = format!("{namespace}:font/text/{name}.png");
        let suffix = text_font_suffix(name, k);
        let document = OutputFile::text(
            format!("assets/{namespace}/font/{suffix}.json"),
            to_json(&text_font(&file, &font.chars, k)?)?,
        );
        emitted.push((Part::TextRowFont(suffix), document));
        if sheets.insert(name) {
            let sheet = OutputFile::binary(
                format!("assets/{namespace}/textures/font/text/{name}.png"),
                font.texture.encode_png()?,
            );
            emitted.push((Part::TextSheet, sheet));
        }
    }
    for (k, shown) in &ctx.sprite_fonts {
        let suffix = format!("sprite_{}", shifted_suffix(*k));
        let document = OutputFile::text(
            format!("assets/{namespace}/font/{suffix}.json"),
            to_json(&sprite_font(*k, shown, ctx.runtime_sprites, &ctx.codepoints)?)?,
        );
        emitted.push((Part::SpriteFont(suffix), document));
    }
    for (part, file) in emitted {
        ctx.push(part, file);
    }
    Ok(())
}

fn sprite_font(
    k: i32,
    shown: &BTreeSet<String>,
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    codepoints: &BTreeMap<String, u32>,
) -> Result<serde_json::Value> {
    let ascent = 7 - k;
    let mut providers = Vec::with_capacity(shown.len());
    for (name, asset) in runtime_sprites.iter().filter(|(name, _)| shown.contains(*name)) {
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

fn hitbox_item(namespace: &str) -> Result<[OutputFile; 3]> {
    Ok([
        OutputFile::text(
            format!("assets/{namespace}/items/gui/hitbox.json"),
            to_json(&serde_json::json!({
                "model": {
                    "type": "minecraft:model",
                    "model": format!("{namespace}:gui/hitbox"),
                },
            }))?,
        ),
        OutputFile::text(
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
        ),
        OutputFile::binary(
            format!("assets/{namespace}/textures/gui/hitbox.png"),
            Texture { width: 1, height: 1, rgba: vec![0, 0, 0, 0] }.encode_png()?,
        ),
    ])
}

/// Serialize a JSON value deterministically (compact, trailing newline). The font documents are dominated by
/// `chars` grids, so indentation alone is a tenth of their bytes.
pub(super) fn to_json(value: &serde_json::Value) -> Result<String> {
    let mut json = serde_json::to_string(value).map_err(|e| Error::Font(e.to_string()))?;
    json.push('\n');
    Ok(json)
}
