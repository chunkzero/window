//! Horse mode: a full-screen window with the view locked.
//!
//! The client's camera spectates an invisible client-only bat at the player's eye, so the view never turns. The
//! player rides an invisible, saddled, client-only horse at [`OFFSET`] from the camera. Every client tick, the client
//! turns the horse it steers to the player's yaw and half their pitch (`AbstractHorse.tickRidden`). The horse wears
//! armour whose texture starts with [`ARMOR_SIGNATURE`]. The generated `core/entity` vertex shader recognises that
//! armour, recovers the yaw and pitch from the corners of the head's top face, and draws that face as the cursor
//! with the include's mapping. It hides every other vertex of the armour. The server maps the player's own rotation
//! the same way, so the drawn cursor and the hit-test agree.

use crate::compose::Texture;

/// Where the horse stands relative to the camera, in blocks, with the camera facing yaw and pitch 0 (+Z). It stays in
/// view at every FOV, so it is not frustum-culled, and below the camera's line of sight, so clicks reach the
/// interaction entity around the eye instead of the horse.
pub const OFFSET: [f64; 3] = [0.0, -1.75, 3.0];

/// Texel (0, 0) of the armour texture, as RGBA. The entity shader treats every armour texture that starts with it as
/// the cursor armour, so it is reserved pack-wide.
pub const ARMOR_SIGNATURE: [u8; 4] = [253, 1, 253, 255];

/// The armour's equipment asset, under the pack namespace.
pub const ARMOR_ASSET: &str = "fullscreen/horse_cursor";

/// The column of the armour texture where the cursor starts, right of the signature texel.
pub const ARMOR_CURSOR_X: u32 = 8;

/// The armour texture: [`ARMOR_SIGNATURE`] at (0, 0) and `cursor` from ([`ARMOR_CURSOR_X`], 0).
pub(crate) fn armor_texture(cursor: &Texture) -> Texture {
    let mut texture = Texture::transparent(ARMOR_CURSOR_X + cursor.width, cursor.height.max(1));
    texture.rgba[..4].copy_from_slice(&ARMOR_SIGNATURE);
    let row = cursor.width as usize * 4;
    for y in 0..cursor.height as usize {
        let at = (y * texture.width as usize + ARMOR_CURSOR_X as usize) * 4;
        texture.rgba[at..at + row].copy_from_slice(&cursor.rgba[y * row..(y + 1) * row]);
    }
    texture
}

/// The equipment asset's path relative to the pack root.
pub(crate) fn equipment_path(namespace: &str) -> String {
    format!("assets/{namespace}/equipment/{ARMOR_ASSET}.json")
}

/// The armour texture's path relative to the pack root, where the equipment asset's `horse_body` layer finds it.
pub(crate) fn texture_path(namespace: &str) -> String {
    format!("assets/{namespace}/textures/entity/equipment/horse_body/{ARMOR_ASSET}.png")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fullscreen::cursor_texture;

    #[test]
    fn armor_texture_starts_with_the_signature_then_holds_the_cursor() {
        let cursor = cursor_texture();
        let texture = armor_texture(&cursor);
        assert_eq!((texture.width, texture.height), (ARMOR_CURSOR_X + cursor.width, cursor.height));
        assert_eq!(texture.get(0, 0), ARMOR_SIGNATURE);
        for (x, y) in [(0, 0), (1, 1), (7, 7), (4, 11)] {
            assert_eq!(texture.get(ARMOR_CURSOR_X + x, y), cursor.get(x, y), "({x}, {y})");
        }
    }
}
