use serde::Deserialize;

use super::element::ElementDto;
use super::insets::InsetsDto;
use crate::geometry::Size;
use crate::ir::{HudChannel, HudShader};
use crate::model::Hud;
use crate::{Error, Result};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HudDto {
    pub(super) name: String,
    #[serde(default)]
    channel: String,
    width: u32,
    height: u32,
    #[serde(default)]
    bleed: InsetsDto,
    shader: Option<HudShaderDto>,
    #[serde(default)]
    children: Vec<ElementDto>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HudShaderDto {
    #[serde(default = "default_actionbar_source_bottom")]
    source_bottom: i32,
    origin: Option<HudShaderPointDto>,
    anchor: Option<HudShaderPointDto>,
    origin_x: Option<f32>,
    origin_y: Option<f32>,
    anchor_x: Option<f32>,
    anchor_y: Option<f32>,
    x: Option<i32>,
    y: Option<i32>,
    offset_x: Option<i32>,
    offset_y: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HudShaderPointDto {
    x: Option<f32>,
    y: Option<f32>,
}

fn default_actionbar_source_bottom() -> i32 {
    59
}

impl HudDto {
    pub(super) fn into_hud(self) -> Result<Hud> {
        let mut children = Vec::with_capacity(self.children.len());
        for child in self.children {
            children.push(child.into_element()?);
        }
        Ok(Hud {
            name: self.name,
            channel: parse_hud_channel(&self.channel)?,
            size: Size::new(self.width, self.height),
            bleed: self.bleed.into_insets(),
            shader: self.shader.map(HudShaderDto::into_shader).transpose()?,
            children,
        })
    }
}

impl HudShaderDto {
    fn into_shader(self) -> Result<HudShader> {
        let origin_x = self.origin.as_ref().and_then(|point| point.x).or(self.origin_x).unwrap_or(0.5);
        let origin_y = self.origin.as_ref().and_then(|point| point.y).or(self.origin_y).unwrap_or(1.0);
        let anchor_x = self.anchor.as_ref().and_then(|point| point.x).or(self.anchor_x).unwrap_or(0.5);
        let anchor_y = self.anchor.as_ref().and_then(|point| point.y).or(self.anchor_y).unwrap_or(0.0);
        validate_unit(origin_x, "shader.origin_x")?;
        validate_unit(origin_y, "shader.origin_y")?;
        validate_unit(anchor_x, "shader.anchor_x")?;
        validate_unit(anchor_y, "shader.anchor_y")?;

        let offset_x = self.x.or(self.offset_x).unwrap_or(0);
        let offset_y = match (self.y, self.offset_y) {
            (Some(y), _) => y,
            (None, Some(relative_y)) => -self.source_bottom + relative_y,
            (None, None) => -self.source_bottom,
        };

        Ok(HudShader { source_bottom: self.source_bottom, origin_x, origin_y, anchor_x, anchor_y, offset_x, offset_y })
    }
}

fn validate_unit(value: f32, field: &str) -> Result<()> {
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(Error::Validation(format!("{field} must be between 0.0 and 1.0, got {value}")))
    }
}

fn parse_hud_channel(value: &str) -> Result<HudChannel> {
    match value {
        "" | "actionbar" => Ok(HudChannel::ActionBar),
        "bossbar" => Ok(HudChannel::BossBar),
        "sidebar" => Ok(HudChannel::Sidebar),
        other => Err(Error::Validation(format!(
            "hud has unknown channel `{other}`; valid channels: actionbar, bossbar, sidebar"
        ))),
    }
}
