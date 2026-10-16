//! The pack files of horse-mode full-screen windows; see [`crate::fullscreen::horse`].

use crate::Result;
use crate::compose::Texture;
use crate::fullscreen::horse;
use crate::manifest::HorseEntry;
use crate::size::Part;

use super::super::fonts::to_json;
use super::super::{CompileContext, OutputFile};

/// Emits the cursor armour, drawing `cursor`, and the entity shader that recognises it, and returns the horse entry.
pub(super) fn emit(ctx: &mut CompileContext<'_>, cursor: &Texture) -> Result<HorseEntry> {
    let armor = format!("{}:{}", ctx.namespace, horse::ARMOR_ASSET);
    let equipment = to_json(&serde_json::json!({ "layers": { "horse_body": [{ "texture": armor }] } }))?;
    ctx.push(Part::Fullscreen, OutputFile::text(horse::equipment_path(ctx.namespace), equipment));
    let texture = horse::armor_texture(cursor).encode_png()?;
    ctx.push(Part::Fullscreen, OutputFile::binary(horse::texture_path(ctx.namespace), texture));
    let shader = crate::hud::horse_entity_shader();
    ctx.push(Part::Shader, OutputFile::text(shader.path, shader.contents));
    Ok(HorseEntry { offset: horse::OFFSET, armor })
}
