//! Transparent replacements for vanilla HUD GUI sprites (`vanilla.hud`).

use std::collections::BTreeSet;

use crate::authoring::HudGroup;
use crate::compose::Texture;
use crate::{Error, Result};

use super::OutputFile;

/// First resource-pack format with `gui/sprites` (Minecraft 1.20.2).
const SPRITE_PACK_FORMAT: u32 = 18;

const HUD_DIR: &str = "assets/minecraft/textures/gui/sprites/hud/";

/// Sprite paths under the HUD sprite directory, each with its pixel size. Taken from the Minecraft 26.1.2
/// client; a path an older client lacks is harmless.
type Sprites = &'static [(&'static str, u32, u32)];

const HOTBAR: Sprites = &[
    ("hotbar", 182, 22),
    ("hotbar_attack_indicator_background", 18, 18),
    ("hotbar_attack_indicator_progress", 18, 18),
    ("hotbar_offhand_left", 29, 24),
    ("hotbar_offhand_right", 29, 24),
    ("hotbar_selection", 24, 23),
];

const HEALTH: Sprites = &[
    ("heart/absorbing_full", 9, 9),
    ("heart/absorbing_full_blinking", 9, 9),
    ("heart/absorbing_half", 9, 9),
    ("heart/absorbing_half_blinking", 9, 9),
    ("heart/absorbing_hardcore_full", 9, 9),
    ("heart/absorbing_hardcore_full_blinking", 9, 9),
    ("heart/absorbing_hardcore_half", 9, 9),
    ("heart/absorbing_hardcore_half_blinking", 9, 9),
    ("heart/container", 9, 9),
    ("heart/container_blinking", 9, 9),
    ("heart/container_hardcore", 9, 9),
    ("heart/container_hardcore_blinking", 9, 9),
    ("heart/frozen_full", 9, 9),
    ("heart/frozen_full_blinking", 9, 9),
    ("heart/frozen_half", 9, 9),
    ("heart/frozen_half_blinking", 9, 9),
    ("heart/frozen_hardcore_full", 9, 9),
    ("heart/frozen_hardcore_full_blinking", 9, 9),
    ("heart/frozen_hardcore_half", 9, 9),
    ("heart/frozen_hardcore_half_blinking", 9, 9),
    ("heart/full", 9, 9),
    ("heart/full_blinking", 9, 9),
    ("heart/half", 9, 9),
    ("heart/half_blinking", 9, 9),
    ("heart/hardcore_full", 9, 9),
    ("heart/hardcore_full_blinking", 9, 9),
    ("heart/hardcore_half", 9, 9),
    ("heart/hardcore_half_blinking", 9, 9),
    ("heart/poisoned_full", 9, 9),
    ("heart/poisoned_full_blinking", 9, 9),
    ("heart/poisoned_half", 9, 9),
    ("heart/poisoned_half_blinking", 9, 9),
    ("heart/poisoned_hardcore_full", 9, 9),
    ("heart/poisoned_hardcore_full_blinking", 9, 9),
    ("heart/poisoned_hardcore_half", 9, 9),
    ("heart/poisoned_hardcore_half_blinking", 9, 9),
    ("heart/vehicle_container", 9, 9),
    ("heart/vehicle_full", 9, 9),
    ("heart/vehicle_half", 9, 9),
    ("heart/withered_full", 9, 9),
    ("heart/withered_full_blinking", 9, 9),
    ("heart/withered_half", 9, 9),
    ("heart/withered_half_blinking", 9, 9),
    ("heart/withered_hardcore_full", 9, 9),
    ("heart/withered_hardcore_full_blinking", 9, 9),
    ("heart/withered_hardcore_half", 9, 9),
    ("heart/withered_hardcore_half_blinking", 9, 9),
];

const FOOD: Sprites = &[
    ("food_empty", 9, 9),
    ("food_empty_hunger", 9, 9),
    ("food_full", 9, 9),
    ("food_full_hunger", 9, 9),
    ("food_half", 9, 9),
    ("food_half_hunger", 9, 9),
];

const ARMOR: Sprites = &[("armor_empty", 9, 9), ("armor_full", 9, 9), ("armor_half", 9, 9)];

const AIR: Sprites = &[("air", 9, 9), ("air_bursting", 9, 9), ("air_empty", 9, 9)];

/// The experience bar and the jump and locator bars that take its place.
const EXPERIENCE: Sprites = &[
    ("experience_bar_background", 182, 5),
    ("experience_bar_progress", 182, 5),
    ("jump_bar_background", 182, 5),
    ("jump_bar_cooldown", 182, 5),
    ("jump_bar_progress", 182, 5),
    ("locator_bar_arrow_down", 7, 10),
    ("locator_bar_arrow_up", 7, 10),
    ("locator_bar_background", 12, 5),
    ("locator_bar_dot/bowtie", 9, 9),
    ("locator_bar_dot/default_0", 9, 9),
    ("locator_bar_dot/default_1", 9, 9),
    ("locator_bar_dot/default_2", 9, 9),
    ("locator_bar_dot/default_3", 9, 9),
];

const CROSSHAIR: Sprites = &[
    ("crosshair", 15, 15),
    ("crosshair_attack_indicator_background", 16, 4),
    ("crosshair_attack_indicator_full", 16, 16),
    ("crosshair_attack_indicator_progress", 16, 4),
];

fn sprites(group: HudGroup) -> Sprites {
    match group {
        HudGroup::Hotbar => HOTBAR,
        HudGroup::Health => HEALTH,
        HudGroup::Food => FOOD,
        HudGroup::Armor => ARMOR,
        HudGroup::Air => AIR,
        HudGroup::Experience => EXPERIENCE,
        HudGroup::Crosshair => CROSSHAIR,
    }
}

/// Emits transparent textures for every sprite of the `blank` groups. Fails when `pack_format` is known
/// to predate `gui/sprites`, since the client would ignore the files.
pub(super) fn blank_sprites(blank: &BTreeSet<HudGroup>, pack_format: Option<u32>) -> Result<Vec<OutputFile>> {
    if blank.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(format) = pack_format.filter(|&format| format < SPRITE_PACK_FORMAT) {
        return Err(Error::Validation(format!(
            "`vanilla.hud` blanks GUI sprites, which need pack format {SPRITE_PACK_FORMAT} or newer, but the pack \
             format is {format}"
        )));
    }
    blank
        .iter()
        .flat_map(|&group| sprites(group))
        .map(|&(name, width, height)| {
            Ok(OutputFile::binary(format!("{HUD_DIR}{name}.png"), Texture::transparent(width, height).encode_png()?))
        })
        .collect()
}
