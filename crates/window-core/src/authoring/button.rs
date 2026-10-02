use serde::Deserialize;

use crate::ir::{ButtonState, ButtonTooltip};
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
    pub(super) fn into_tooltip(self) -> Result<ButtonTooltip> {
        let tooltip = match self {
            TooltipDto::Title(title) => ButtonTooltip { title, lines: Vec::new() },
            TooltipDto::Object(TooltipObjectDto { title, lines }) => ButtonTooltip { title, lines },
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
    sprite: Option<String>,
    tooltip: Option<TooltipDto>,
}

impl ButtonStateDto {
    pub(super) fn into_state(self) -> Result<ButtonState> {
        Ok(ButtonState {
            item_model: self.item_model,
            sprite: self.sprite,
            tooltip: self.tooltip.map(TooltipDto::into_tooltip).transpose()?,
        })
    }
}
