//! Native anvil text field: a hole in each anvil input window's art so the client's rename box
//! shows through, and optional themed vanilla field sprites.

use std::collections::BTreeMap;

use crate::compose::Composite;
use crate::geometry::Rect;
use crate::ir::LaidOutWindow;
use crate::surface::ContainerKind;
use crate::{Error, Result};

use super::OutputFile;
use super::sprites::RuntimeSpriteAsset;

const TEXT_FIELD: Rect = ContainerKind::ANVIL_TEXT_FIELD;

const FIELD_SPRITES: [&str; 2] = [
    "assets/minecraft/textures/gui/sprites/container/anvil/text_field.png",
    "assets/minecraft/textures/gui/sprites/container/anvil/text_field_disabled.png",
];

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

/// Warns that `w`'s anvil input uses the unstable drawn field.
pub(super) fn warn_drawn_input(w: &LaidOutWindow, warnings: &mut Vec<String>) {
    if !w.inputs.is_empty() {
        warnings.push(format!(
            "window `{}`: unstable_drawn_anvil_input is unstable; text drawn over the anvil field only updates by \
             reopening the anvil, which can flicker while the player types",
            w.name
        ));
    }
}

/// Clears the text field from `w`'s static art when it has an anvil input, and warns about
/// slots that would still draw over the native rename box.
pub(super) fn open_field(w: &LaidOutWindow, comp: &mut Composite, warnings: &mut Vec<String>) {
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
