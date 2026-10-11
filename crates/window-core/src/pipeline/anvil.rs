//! Anvil input windows: a static title, a hole in the art so the client's rename box shows through,
//! and the `vanilla.anvil` texture replacements.

use std::collections::BTreeMap;

use crate::authoring::{Replacement, VanillaAnvil};
use crate::compose::{Composite, Texture};
use crate::geometry::{Rect, Size};
use crate::ir::{HandleKind, LaidOutWindow};
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

const BACKGROUND: (&str, Size) =
    ("assets/minecraft/textures/gui/container/anvil.png", Size { width: 256, height: 256 });

const ERROR_ICON: (&str, Size) =
    ("assets/minecraft/textures/gui/sprites/container/anvil/error.png", Size { width: 28, height: 21 });

/// Files for the `vanilla.anvil` options. Unless `background` says otherwise, windows with an anvil input
/// (`has_input`) blank the background and error icon, since Window windows draw over them.
pub(super) fn vanilla_files(
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    options: &VanillaAnvil,
    has_input: bool,
) -> Result<Vec<OutputFile>> {
    let mut files = Vec::new();
    match &options.field {
        Replacement::Keep => {}
        Replacement::Blank => {
            for path in FIELD_SPRITES {
                files.push(blank(path, TEXT_FIELD.size())?);
            }
        }
        Replacement::Sprite(sprite) => {
            files.extend(sprite_files(
                "vanilla.anvil.field",
                runtime_sprites,
                sprite,
                TEXT_FIELD.size(),
                &FIELD_SPRITES,
            )?);
        }
    }
    let background =
        options.background.clone().unwrap_or(if has_input { Replacement::Blank } else { Replacement::Keep });
    match background {
        Replacement::Keep => {}
        Replacement::Blank => {
            files.push(blank(BACKGROUND.0, BACKGROUND.1)?);
            files.push(blank(ERROR_ICON.0, ERROR_ICON.1)?);
        }
        Replacement::Sprite(sprite) => {
            let (path, size) = BACKGROUND;
            files.extend(sprite_files("vanilla.anvil.background", runtime_sprites, &sprite, size, &[path])?);
            files.push(blank(ERROR_ICON.0, ERROR_ICON.1)?);
        }
    }
    Ok(files)
}

pub(super) fn blank(path: &str, size: Size) -> Result<OutputFile> {
    Ok(OutputFile::binary(path, Texture::transparent(size.width, size.height).encode_png()?))
}

/// Emits catalog `sprite` at each of `paths`; `option` names the plugin option in errors.
pub(super) fn sprite_files(
    option: &str,
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    sprite: &str,
    size: Size,
    paths: &[&str],
) -> Result<Vec<OutputFile>> {
    let asset = runtime_sprites
        .get(sprite)
        .ok_or_else(|| Error::Validation(format!("`{option}` references unknown sprite `{sprite}`")))?;
    if asset.size != size {
        return Err(Error::Validation(format!(
            "`{option}` sprite `{sprite}` must be {}x{}, but is {}x{}",
            size.width, size.height, asset.size.width, asset.size.height
        )));
    }
    let output = asset.output.as_ref().ok_or_else(|| {
        Error::Validation(format!("`{option}` sprite `{sprite}` must be generated or bundled, not external"))
    })?;
    Ok(paths.iter().map(|path| OutputFile { path: (*path).into(), contents: output.contents.clone() }).collect())
}

/// Rejects controls that change the title of `w` when it has an anvil input, since each title change
/// reopens the anvil and races the player's typing. With `experimental` set, warns instead.
pub(super) fn check_title(w: &LaidOutWindow, experimental: bool, warnings: &mut Vec<String>) -> Result<()> {
    if w.inputs.is_empty() {
        return Ok(());
    }
    let slots = w.slots.iter().filter(|slot| slot.text.is_none()).map(|slot| &slot.name);
    let sprites = w.sprite_slots.iter().map(|slot| &slot.name);
    let collections = w.collections.iter().filter(|collection| collection.selected_sprite.is_some());
    // A switch whose cases only swap regions changes inventory items, not the title.
    let drawing = w.switches.iter().filter(|switch| {
        switch.cases.iter().any(|case| {
            !case.draws.is_empty()
                || !case.slots.is_empty()
                || !case.sprite_slots.is_empty()
                || !case.switches.is_empty()
                || !case.art.is_empty()
        })
    });
    let tints = w.handles.iter().filter(|(_, handle)| handle.kind == HandleKind::Tint).map(|(id, _)| id);
    let updates = slots
        .chain(sprites)
        .chain(collections.map(|collection| &collection.name))
        .chain(drawing.map(|switch| &switch.name))
        .chain(tints)
        .chain(w.art.iter().filter_map(|art| art.bar.as_ref()));
    for name in updates {
        if !experimental {
            return Err(Error::Validation(format!(
                "window `{}`: `{name}` changes the title of an anvil input window, which reopens the anvil while \
                 the player types; keep anvil windows static or set `experimental.anvilUpdates`",
                w.name
            )));
        }
        warnings.push(format!(
            "window `{}`: `{name}` reopens the anvil to change its title (`experimental.anvilUpdates`), which can \
             flicker and drop keystrokes",
            w.name
        ));
    }
    Ok(())
}

/// Clears the text field from `w`'s static, switch case, and art layer art when it has an anvil input, and warns
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
    layers.cases.iter_mut().flatten().chain(&mut layers.art).for_each(clear_field);
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
