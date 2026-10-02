use std::collections::BTreeMap;

use serde::Deserialize;

use super::insets::InsetsDto;
use super::parse::validate_name;
use crate::geometry::Size;
use crate::ir::Rgb;
use crate::model::{Frame, GeneratedKind, GeneratedStyle, SpriteDef, Theme};
use crate::{Error, Result};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ThemeDto {
    #[serde(default)]
    frames: BTreeMap<String, FrameDto>,
    #[serde(default)]
    sprites: BTreeMap<String, SpriteDto>,
}

#[derive(Debug, Deserialize)]
struct FrameDto {
    texture: Option<String>,
    insets: Option<InsetsDto>,
    #[serde(flatten)]
    generated: GeneratedStyleDto,
}

#[derive(Debug, Deserialize)]
struct SpriteDto {
    texture: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(flatten)]
    generated: GeneratedStyleDto,
}

#[derive(Debug, Default, Deserialize)]
struct GeneratedStyleDto {
    kind: Option<String>,
    fill: Option<String>,
    border_color: Option<String>,
    border_width: Option<u32>,
    radius: Option<u32>,
    inset_depth: Option<u32>,
    highlight_color: Option<String>,
    shadow_color: Option<String>,
    accent_color: Option<String>,
    stripe_color: Option<String>,
    stripe_shadow_color: Option<String>,
    stripe_width: Option<u32>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl ThemeDto {
    /// Adds this document's frames and sprites to `theme`; names are global across documents.
    pub(super) fn merge_into(self, theme: &mut Theme) -> Result<()> {
        for (name, frame) in self.frames {
            validate_name(&name, "frame")?;
            if theme.frames.contains_key(&name) {
                return Err(Error::Validation(format!("duplicate frame name `{name}`")));
            }
            if theme.sprites.contains_key(&name) {
                return Err(Error::Validation(format!("frame name `{name}` collides with a sprite of the same name")));
            }
            theme.frames.insert(name, frame.into_frame()?);
        }

        for (name, sprite) in self.sprites {
            validate_name(&name, "sprite")?;
            if theme.sprites.contains_key(&name) {
                return Err(Error::Validation(format!("duplicate sprite name `{name}`")));
            }
            if theme.frames.contains_key(&name) {
                return Err(Error::Validation(format!("sprite name `{name}` collides with a frame of the same name")));
            }
            theme.sprites.insert(name, sprite.into_sprite()?);
        }
        Ok(())
    }
}

impl FrameDto {
    fn into_frame(self) -> Result<Frame> {
        if let Some(texture) = self.texture {
            self.generated.reject_for_texture("texture frame")?;
            return Ok(Frame::Texture { texture, insets: self.insets.unwrap_or_default().into_insets() });
        }
        if self.insets.is_some() {
            return Err(Error::Validation(
                "generated frame does not accept `insets`; set `texture` for nine-slice frames".into(),
            ));
        }

        Ok(Frame::Generated(self.generated.into_style(GeneratedKind::Panel)?))
    }
}

impl SpriteDto {
    fn into_sprite(self) -> Result<SpriteDef> {
        if let Some(texture) = self.texture {
            self.generated.reject_for_texture("texture sprite")?;
            let size = match (self.width, self.height) {
                (Some(width), Some(height)) => Some(Size::new(width, height)),
                (None, None) => None,
                _ => {
                    return Err(Error::Validation(
                        "texture sprite must set both `width` and `height`, or neither".into(),
                    ));
                }
            };
            return Ok(SpriteDef::Texture { texture, size });
        }

        let width = self.width.ok_or_else(|| Error::Validation("generated sprite requires `width`".into()))?;
        let height = self.height.ok_or_else(|| Error::Validation("generated sprite requires `height`".into()))?;
        Ok(SpriteDef::Generated {
            size: Size::new(width, height),
            style: self.generated.into_style(GeneratedKind::Badge)?,
        })
    }
}

impl GeneratedStyleDto {
    fn reject_for_texture(&self, context: &str) -> Result<()> {
        if let Some(field) = self.first_present_field() {
            return Err(Error::Validation(format!(
                "{context} does not accept generated style field `{field}` when `texture` is set"
            )));
        }
        Ok(())
    }

    fn first_present_field(&self) -> Option<&str> {
        if self.kind.is_some() {
            return Some("kind");
        }
        if self.fill.is_some() {
            return Some("fill");
        }
        if self.border_color.is_some() {
            return Some("border_color");
        }
        if self.border_width.is_some() {
            return Some("border_width");
        }
        if self.radius.is_some() {
            return Some("radius");
        }
        if self.inset_depth.is_some() {
            return Some("inset_depth");
        }
        if self.highlight_color.is_some() {
            return Some("highlight_color");
        }
        if self.shadow_color.is_some() {
            return Some("shadow_color");
        }
        if self.accent_color.is_some() {
            return Some("accent_color");
        }
        if self.stripe_color.is_some() {
            return Some("stripe_color");
        }
        if self.stripe_shadow_color.is_some() {
            return Some("stripe_shadow_color");
        }
        if self.stripe_width.is_some() {
            return Some("stripe_width");
        }
        self.extra.keys().next().map(String::as_str)
    }

    fn into_style(self, default_kind: GeneratedKind) -> Result<GeneratedStyle> {
        reject_extra_fields(&self.extra, "generated theme asset")?;
        let kind = self.kind.as_deref().map(parse_generated_kind).transpose()?.unwrap_or(default_kind);
        let mut style = GeneratedStyle::defaults(kind);
        if let Some(fill) = self.fill {
            style.fill = parse_rgb(&fill, "fill")?;
        }
        if let Some(border_color) = self.border_color {
            style.border_color = parse_rgb(&border_color, "border_color")?;
        }
        if let Some(border_width) = self.border_width {
            style.border_width = border_width;
        }
        if let Some(radius) = self.radius {
            style.radius = radius;
        }
        if let Some(inset_depth) = self.inset_depth {
            style.inset_depth = inset_depth;
        }
        if let Some(highlight_color) = self.highlight_color {
            style.highlight_color = Some(parse_rgb(&highlight_color, "highlight_color")?);
        }
        if let Some(shadow_color) = self.shadow_color {
            style.shadow_color = Some(parse_rgb(&shadow_color, "shadow_color")?);
        }
        if let Some(accent_color) = self.accent_color {
            style.accent_color = Some(parse_rgb(&accent_color, "accent_color")?);
        }
        if let Some(stripe_color) = self.stripe_color {
            style.stripe_color = Some(parse_rgb(&stripe_color, "stripe_color")?);
        }
        if let Some(stripe_shadow_color) = self.stripe_shadow_color {
            style.stripe_shadow_color = Some(parse_rgb(&stripe_shadow_color, "stripe_shadow_color")?);
        }
        if let Some(stripe_width) = self.stripe_width {
            style.stripe_width = stripe_width;
        }
        Ok(style)
    }
}

fn parse_generated_kind(value: &str) -> Result<GeneratedKind> {
    match value {
        "panel" => Ok(GeneratedKind::Panel),
        "button" | "chip" => Ok(GeneratedKind::Button),
        "slot" | "slot_cell" => Ok(GeneratedKind::Slot),
        "hazard" | "hazard_bar" => Ok(GeneratedKind::HazardBar),
        "vent" => Ok(GeneratedKind::Vent),
        "badge" | "corner_cap" => Ok(GeneratedKind::Badge),
        other => Err(Error::Validation(format!("generated theme asset has unknown kind `{other}`"))),
    }
}

fn parse_rgb(value: &str, field: &str) -> Result<Rgb> {
    Rgb::parse_hex(value).ok_or_else(|| {
        Error::Validation(format!("generated theme asset has invalid {field} `{value}`; expected #rrggbb"))
    })
}

fn reject_extra_fields(extra: &BTreeMap<String, serde_json::Value>, context: &str) -> Result<()> {
    if let Some(field) = extra.keys().next() {
        return Err(Error::Validation(format!("{context} does not accept field `{field}`")));
    }
    Ok(())
}
