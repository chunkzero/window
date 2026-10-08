//! Window and HUD surface layers.

use std::collections::BTreeMap;

use super::glyphs::BitmapGlyph;
use super::schema::{DebugCursorContract, DebugLayer, DebugSurface, DebugTextStyle};
use crate::ir::Layer;
use crate::manifest::{HudEntry, Manifest, SlotEntry, SpriteSlotEntry, SwitchEntry, WindowEntry};

pub(super) fn window_surface(
    manifest: &Manifest,
    window: &WindowEntry,
    glyphs: &BTreeMap<char, BitmapGlyph>,
) -> DebugSurface {
    let origin = window.surface.title_origin;
    let mut layers: Vec<DebugLayer> =
        static_layer(manifest, &window.static_text, origin, glyphs, Some(0)).into_iter().collect();
    let sprites = &window.sprite_slots;
    let entries = Entries { slots: &window.slots, sprite_slots: sprites, switches: &window.switches };
    layers.extend(tree_layers(manifest, &window.layers, &entries, origin, glyphs, Some(0)));
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
    let entries = Entries { slots: &hud.slots, sprite_slots: &BTreeMap::new(), switches: &hud.switches };
    layers.extend(tree_layers(manifest, &hud.layers, &entries, origin, glyphs, independent_layer_net_advance));
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

/// A surface's runtime entries.
struct Entries<'a> {
    slots: &'a BTreeMap<String, SlotEntry>,
    sprite_slots: &'a BTreeMap<String, SpriteSlotEntry>,
    switches: &'a BTreeMap<String, SwitchEntry>,
}

/// The runtime layers in authored tree order: text and sprite slots, and each case of a switch with baked art.
fn tree_layers(
    manifest: &Manifest,
    order: &[Layer],
    entries: &Entries<'_>,
    origin: [i32; 2],
    glyphs: &BTreeMap<char, BitmapGlyph>,
    net_advance: Option<i32>,
) -> Vec<DebugLayer> {
    // Entries a derived switch owns name that switch's source.
    let mut sources: BTreeMap<&str, &str> = BTreeMap::new();
    for switch in entries.switches.values() {
        if let Some(source) = &switch.source {
            for case in &switch.cases {
                for name in case.slots.iter().chain(&case.sprite_slots) {
                    sources.insert(name, source);
                }
            }
        }
    }
    let source = |name: &str| sources.get(name).map(|source| source.to_string());
    let mut out = Vec::new();
    for layer in order {
        match layer {
            Layer::Slot(name) => {
                if let Some(slot) = entries.slots.get(name) {
                    out.push(DebugLayer {
                        source: slot.source.clone().or_else(|| source(name)),
                        ..text_layer(manifest, name, slot, origin[1], net_advance)
                    });
                }
            }
            Layer::SpriteSlot(name) => {
                if let Some(slot) = entries.sprite_slots.get(name) {
                    out.push(DebugLayer {
                        source: slot.source.clone().or_else(|| source(name)),
                        ..sprite_layer(name, slot, origin[1], net_advance)
                    });
                }
            }
            Layer::Switch(name) => {
                let Some(switch) = entries.switches.get(name) else {
                    continue;
                };
                for case in &switch.cases {
                    if let Some(layer) = static_layer(manifest, &case.static_text, origin, glyphs, Some(0)) {
                        out.push(DebugLayer {
                            id: format!("case:{name}/{}", case.value),
                            kind: "case_chrome".into(),
                            source: switch.source.clone(),
                            ..layer
                        });
                    }
                }
            }
            Layer::Collection(_) => {}
        }
    }
    out
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
                source: None,
            });
        }
        cursor = cursor.saturating_add(manifest.spacers.get(&(character as u32)).copied().unwrap_or(0));
    }
    None
}

fn text_layer(manifest: &Manifest, name: &str, slot: &SlotEntry, title_y: i32, net_advance: Option<i32>) -> DebugLayer {
    DebugLayer {
        id: format!("text:{name}"),
        kind: "text_slot".into(),
        bounds: [slot.x, slot.y, slot.width as i32, slot.height() as i32],
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
        source: None,
    }
}

fn text_advance(manifest: &Manifest, font: &str, text: &str, bold: bool) -> Option<u32> {
    let metrics = manifest.font_metrics.get(font)?;
    text.chars().try_fold(0u32, |total, character| {
        let advance = metrics.advances.get(&character)?;
        Some(total.saturating_add(*advance).saturating_add(if bold { metrics.bold_advance } else { 0 }))
    })
}

fn sprite_layer(name: &str, slot: &SpriteSlotEntry, title_y: i32, net_advance: Option<i32>) -> DebugLayer {
    DebugLayer {
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
        source: None,
    }
}
