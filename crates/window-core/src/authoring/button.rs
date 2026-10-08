use serde::Deserialize;

use super::art::{ArtRefDto, ArtUse, intern};

use crate::ir::Tooltip;
use crate::model::ControlState;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum TooltipDto {
    Title(String),
    Object(TooltipObjectDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TooltipObjectDto {
    title: String,
    #[serde(default)]
    lines: Vec<String>,
}

impl TooltipDto {
    pub(super) fn into_tooltip(self) -> Result<Tooltip> {
        let tooltip = match self {
            TooltipDto::Title(title) => Tooltip { title, lines: Vec::new() },
            TooltipDto::Object(TooltipObjectDto { title, lines }) => Tooltip { title, lines },
        };
        if tooltip.title.is_empty() {
            return Err(Error::Validation("tooltip title must not be empty".into()));
        }
        Ok(tooltip)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ButtonStateDto {
    item_model: Option<String>,
    frame: Option<ArtRefDto>,
    sprite: Option<ArtRefDto>,
    tooltip: Option<TooltipDto>,
}

impl ButtonStateDto {
    /// Interns the state's inline art into `theme`.
    pub(super) fn intern_art(&mut self, theme: &mut crate::model::Theme) -> Result<()> {
        intern(theme, &mut self.frame, ArtUse::Frame)?;
        intern(theme, &mut self.sprite, ArtUse::Sprite)
    }

    pub(super) fn into_state(self) -> Result<ControlState> {
        Ok(ControlState {
            item_model: self.item_model,
            frame: self.frame.map(|frame| frame.name()).transpose()?,
            sprite: self.sprite.map(|sprite| sprite.name()).transpose()?,
            tooltip: self.tooltip.map(TooltipDto::into_tooltip).transpose()?,
        })
    }
}
