//! Window and HUD surface layers.

use std::collections::BTreeMap;

use super::glyphs::BitmapGlyph;
use super::schema::{DebugCursorContract, DebugLayer, DebugSurface, DebugTextStyle};
use crate::manifest::{HudEntry, Manifest, SlotEntry, SpriteSlotEntry, WindowEntry};

pub(super) fn window_surface(
    manifest: &Manifest,
    window: &WindowEntry,
    glyphs: &BTreeMap<char, BitmapGlyph>,
) -> DebugSurface {
    let origin = window.surface.title_origin;
    let mut layers: Vec<DebugLayer> =
        static_layer(manifest, &window.static_text, origin, glyphs, Some(0)).into_iter().collect();
    layers.extend(text_layers(manifest, &window.slots, origin[1], Some(0)));
    layers.extend(sprite_layers(&window.sprite_slots, origin[1], Some(0)));
    DebugSurface {
        kind: window.surface.kind.clone(),
        target: window.surface.container.clone(),
        bounds: [0, 0, window.surface.size[0] as i32, window.surface.size[1] as i32],
        cursor_origin: origin,
        cursor_contract: DebugCursorContract {
            convention: "independent_net_zero_segments".into(),
            independent_layer_net_advance: Some(0),
            composed_advance: 0,
        },
        layers,
    }
}

pub(super) fn hud_surface(manifest: &Manifest, hud: &HudEntry, glyphs: &BTreeMap<char, BitmapGlyph>) -> DebugSurface {
    let origin = [0, 0];
    let width = hud.surface.width as i32;
    let independent_layer_net_advance = hud.shader.as_ref().map(|_| 0);
    let mut layers: Vec<DebugLayer> =
        static_layer(manifest, &hud.static_text, origin, glyphs, Some(width)).into_iter().collect();
    layers.extend(text_layers(manifest, &hud.slots, 0, independent_layer_net_advance));
    DebugSurface {
        kind: hud.surface.kind.clone(),
        target: hud.surface.channel.clone(),
        bounds: [0, 0, hud.surface.width as i32, hud.surface.height as i32],
        cursor_origin: origin,
        cursor_contract: DebugCursorContract {
            convention: if hud.shader.is_some() {
                "fixed_width_with_independent_overlays".into()
            } else {
                "fixed_width_shared_cursor".into()
            },
            independent_layer_net_advance,
            composed_advance: width,
        },
        layers,
    }
}

fn static_layer(
    manifest: &Manifest,
    text: &str,
    origin: [i32; 2],
    glyphs: &BTreeMap<char, BitmapGlyph>,
    net_advance: Option<i32>,
) -> Option<DebugLayer> {
    let mut cursor = 0i32;
    for character in text.chars() {
        if let Some(glyph) = glyphs.get(&character) {
            return Some(DebugLayer {
                id: "static".into(),
                kind: "static_chrome".into(),
                bounds: [origin[0] + cursor, origin[1] + 7 - glyph.ascent, glyph.width as i32, glyph.height as i32],
                font: manifest.font.clone(),
                glyph: Some(character.to_string()),
                content: None,
                ascent: Some(glyph.ascent),
                content_advance: Some(glyph.advance),
                net_advance,
                resource: Some(glyph.file.clone()),
                align: None,
                style: None,
            });
        }
        cursor = cursor.saturating_add(manifest.spacers.get(&(character as u32)).copied().unwrap_or(0));
    }
    None
}

fn text_layers(
    manifest: &Manifest,
    slots: &BTreeMap<String, SlotEntry>,
    title_y: i32,
    net_advance: Option<i32>,
) -> Vec<DebugLayer> {
    slots
        .iter()
        .map(|(name, slot)| DebugLayer {
            id: format!("text:{name}"),
            kind: "text_slot".into(),
            bounds: [slot.x, slot.y, slot.width as i32, 8],
            font: slot.font.clone(),
            glyph: None,
            content: slot.text.clone(),
            ascent: Some(7 - (slot.y - title_y)),
            content_advance: slot.text.as_deref().and_then(|text| text_advance(manifest, &slot.font, text, slot.bold)),
            net_advance,
            resource: None,
            align: Some(slot.align),
            style: Some(DebugTextStyle {
                color: slot.color.clone(),
                shadow: slot.shadow,
                bold: slot.bold,
                italic: slot.italic,
                underlined: slot.underlined,
                strikethrough: slot.strikethrough,
                obfuscated: slot.obfuscated,
            }),
        })
        .collect()
}

fn text_advance(manifest: &Manifest, font: &str, text: &str, bold: bool) -> Option<u32> {
    let metrics = manifest.font_metrics.get(font)?;
    text.chars().try_fold(0u32, |total, character| {
        let advance = metrics.advances.get(&character)?;
        Some(total.saturating_add(*advance).saturating_add(if bold { metrics.bold_advance } else { 0 }))
    })
}

fn sprite_layers(slots: &BTreeMap<String, SpriteSlotEntry>, title_y: i32, net_advance: Option<i32>) -> Vec<DebugLayer> {
    slots
        .iter()
        .map(|(name, slot)| DebugLayer {
            id: format!("sprite:{name}"),
            kind: "sprite_slot".into(),
            bounds: [slot.x, slot.y, slot.width as i32, slot.height as i32],
            font: slot.font.clone(),
            glyph: None,
            content: None,
            ascent: Some(7 - (slot.y - title_y)),
            content_advance: None,
            net_advance,
            resource: None,
            align: Some(slot.align),
            style: None,
        })
        .collect()
}
