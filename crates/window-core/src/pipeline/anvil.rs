//! Anvil input windows: a static title, a hole in the art so the client's rename box shows through,
//! hidden vanilla anvil art, and optional themed vanilla field sprites.

use std::collections::BTreeMap;

use crate::compose::{Composite, Texture};
use crate::geometry::Rect;
use crate::ir::LaidOutWindow;
use crate::surface::ContainerKind;
use crate::{Error, Result};

use super::OutputFile;
use super::glyphs::Layers;
use super::sprites::RuntimeSpriteAsset;

const TEXT_FIELD: Rect = ContainerKind::ANVIL_TEXT_FIELD;

const FIELD_SPRITES: [&str; 2] = [
    "assets/minecraft/textures/gui/sprites/container/anvil/text_field.png",
    "assets/minecraft/textures/gui/sprites/container/anvil/text_field_disabled.png",
];

/// Vanilla anvil art Window windows draw over, with its size: the background and the
/// missing-result error icon.
const VANILLA_ART: [(&str, u32, u32); 2] = [
    ("assets/minecraft/textures/gui/container/anvil.png", 256, 256),
    ("assets/minecraft/textures/gui/sprites/container/anvil/error.png", 28, 21),
];

/// Transparent replacements for vanilla's anvil art, so anvil windows only show Window art.
pub(super) fn hidden_vanilla_art() -> Result<Vec<OutputFile>> {
    VANILLA_ART
        .iter()
        .map(|&(path, width, height)| Ok(OutputFile::binary(path, Texture::transparent(width, height).encode_png()?)))
        .collect()
}

/// Emits theme `sprite` as vanilla's enabled and disabled anvil text-field sprites.
pub(super) fn field_sprites(
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    sprite: &str,
) -> Result<Vec<OutputFile>> {
    let asset = runtime_sprites
        .get(sprite)
        .ok_or_else(|| Error::Validation(format!("anvil_field_sprite references unknown sprite `{sprite}`")))?;
    if asset.size != TEXT_FIELD.size() {
        return Err(Error::Validation(format!(
            "anvil_field_sprite `{sprite}` must be {}x{}",
            TEXT_FIELD.width, TEXT_FIELD.height
        )));
    }
    let output = asset.output.as_ref().ok_or_else(|| {
        Error::Validation(format!("anvil_field_sprite `{sprite}` must be generated or bundled, not external"))
    })?;
    Ok(FIELD_SPRITES
        .iter()
        .map(|path| OutputFile { path: (*path).into(), contents: output.contents.clone() })
        .collect())
}

/// Rejects controls that change the title of `w` when it has an anvil input, since each title change
/// reopens the anvil and races the player's typing. With `experimental` set, warns instead.
pub(super) fn check_title(w: &LaidOutWindow, experimental: bool, warnings: &mut Vec<String>) -> Result<()> {
    if w.inputs.is_empty() {
        return Ok(());
    }
    let slots = w.slots.iter().filter(|slot| slot.text.is_none()).map(|slot| &slot.name);
    let sprites = w.sprite_slots.iter().filter(|slot| slot.sprite.is_none()).map(|slot| &slot.name);
    let collections = w.collections.iter().filter(|collection| collection.selected_sprite.is_some());
    // A switch whose cases only swap regions changes inventory items, not the title.
    let drawing = w.switches.iter().filter(|switch| {
        switch.cases.iter().any(|case| {
            !case.draws.is_empty()
                || !case.slots.is_empty()
                || !case.sprite_slots.is_empty()
                || !case.switches.is_empty()
        })
    });
    let updates = slots
        .chain(sprites)
        .chain(collections.map(|collection| &collection.name))
        .chain(drawing.map(|switch| &switch.name));
    for name in updates {
        if !experimental {
            return Err(Error::Validation(format!(
                "window `{}`: `{name}` changes the title of an anvil input window, which reopens the anvil while \
                 the player types; keep anvil windows static or set experimental_anvil_updates",
                w.name
            )));
        }
        warnings.push(format!(
            "window `{}`: `{name}` reopens the anvil to change its title (experimental_anvil_updates), which can \
             flicker and drop keystrokes",
            w.name
        ));
    }
    Ok(())
}

/// Clears the text field from `w`'s static and switch case art when it has an anvil input, and warns
/// about slots that would still draw over the native rename box.
pub(super) fn open_field(w: &LaidOutWindow, layers: &mut Layers, warnings: &mut Vec<String>) {
    if w.inputs.is_empty() {
        return;
    }
    let slots = w.slots.iter().map(|slot| (&slot.name, &slot.rect));
    let sprite_slots = w.sprite_slots.iter().map(|slot| (&slot.name, &slot.rect));
    for (name, rect) in slots.chain(sprite_slots) {
        if rect.intersects(&TEXT_FIELD) {
            warnings.push(format!("window `{}`: `{name}` draws over the native anvil text field", w.name));
        }
    }
    clear_field(&mut layers.base);
    layers.cases.iter_mut().flatten().for_each(clear_field);
}

fn clear_field(comp: &mut Composite) {
    if !comp.has_content {
        return;
    }
    let bounds = comp.bounds;
    let width = comp.texture.width as usize;
    for y in TEXT_FIELD.y.max(bounds.y)..TEXT_FIELD.bottom().min(bounds.bottom()) {
        for x in TEXT_FIELD.x.max(bounds.x)..TEXT_FIELD.right().min(bounds.right()) {
            let i = ((y - bounds.y) as usize * width + (x - bounds.x) as usize) * 4;
            comp.texture.rgba[i..i + 4].fill(0);
        }
    }
}
