use std::collections::BTreeMap;

use super::ElementDto;
use crate::authoring::button::TooltipDto;
use crate::authoring::parse::{reject_unexpected_fields, validate_name};
use crate::authoring::patterns::parse_slot_refs;
use crate::geometry::{Point, Size};
use crate::inventory::{InventorySlotRef, SlotPattern, SlotRectClaim};
use crate::ir::{Align, CLOSE_ACTION, Rgb, Tooltip};
use crate::model::{ControlState, CrossAlign, TextFit, TextStyle};
use crate::{Error, Result, text_font};

const PANEL_FIELDS: &[&str] = &["type", "frame", "width", "height", "x", "y", "padding", "children"];
const STACK_FIELDS: &[&str] = &["type", "x", "y", "gap", "padding", "align", "children"];
const SPRITE_FIELDS: &[&str] = &["type", "name", "art", "x", "y", "debug_name"];
const SPRITE_SLOT_FIELDS: &[&str] =
    &["type", "name", "handle", "index", "x", "y", "width", "height", "align", "sprite", "debug_name"];
const BUTTON_FIELDS: &[&str] = &[
    "type",
    "name",
    "on_click",
    "enabled",
    "state",
    "frame",
    "width",
    "height",
    "x",
    "y",
    "slots",
    "pattern",
    "transform",
    "default",
    "tooltip",
    "states",
    "padding",
    "children",
    "source",
];
const HOTSPOT_FIELDS: &[&str] =
    &["type", "name", "width", "height", "x", "y", "slots", "pattern", "transform", "tooltip", "states"];
const ITEM_FIELDS: &[&str] = &["type", "name", "handle", "slots", "pattern", "transform", "cell_slot", "debug_name"];
const COLLECTION_FIELDS: &[&str] =
    &["type", "name", "handle", "frame", "selected_sprite", "slots", "pattern", "transform", "action", "debug_name"];
const ANVIL_INPUT_FIELDS: &[&str] = &["type", "name", "handle", "initial", "item_model", "debug_name"];
const SLOT_RECTS_FIELDS: &[&str] = &["type", "name", "frame", "pattern", "transform", "claim"];
const REPEATER_FIELDS: &[&str] =
    &["type", "name", "on_click", "frame", "pattern", "transform", "padding", "children", "cells"];
const LABEL_FIELDS: &[&str] = &[
    "type",
    "text",
    "x",
    "y",
    "width",
    "align",
    "color",
    "shadow",
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
    "font",
    "small_caps",
    "debug_name",
];
const FLEX_FIELDS: &[&str] = &["type", "x", "y", "frame", "style", "children", "debug_name"];
const SWITCH_FIELDS: &[&str] = &["type", "name", "handle", "index", "x", "y", "children", "debug_name"];
const CASE_FIELDS: &[&str] = &["type", "value", "frame", "style", "children", "debug_name"];
const SECTION_FIELDS: &[&str] = &["type", "section", "frame", "outset", "claim", "flow", "children", "debug_name"];
const REGION_FIELDS: &[&str] = &["type", "on_click", "tooltip", "item_model", "width", "height", "debug_name"];
const SLOT_FIELDS: &[&str] = &[
    "type",
    "name",
    "handle",
    "index",
    "width",
    "x",
    "y",
    "align",
    "color",
    "shadow",
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
    "font",
    "small_caps",
    "overflow",
    "lines",
    "line_height",
    "debug_name",
];
/// Tallest box a wrapped text slot may span, in pixels.
const MAX_TEXT_HEIGHT: u32 = 1024;
const FIT_FIELDS: &[&str] = &["overflow", "lines", "line_height"];

fn allowed_fields(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "panel" => PANEL_FIELDS,
        "row" | "column" => STACK_FIELDS,
        "sprite" => SPRITE_FIELDS,
        "sprite_slot" => SPRITE_SLOT_FIELDS,
        "button" => BUTTON_FIELDS,
        "hotspot" => HOTSPOT_FIELDS,
        "item" => ITEM_FIELDS,
        "collection" => COLLECTION_FIELDS,
        "anvil_input" => ANVIL_INPUT_FIELDS,
        "slot_rects" => SLOT_RECTS_FIELDS,
        "repeater" => REPEATER_FIELDS,
        "label" => LABEL_FIELDS,
        "slot" => SLOT_FIELDS,
        "flex" => FLEX_FIELDS,
        "section" => SECTION_FIELDS,
        "switch" => SWITCH_FIELDS,
        "case" => CASE_FIELDS,
        "region" => REGION_FIELDS,
        _ => return None,
    })
}

impl ElementDto {
    pub(super) fn validate_fields(&self) -> Result<()> {
        let Some(allowed) = allowed_fields(&self.kind) else {
            return Ok(());
        };
        let mut fields = self.fields.clone();
        fields.remove("layout");
        if self.kind == "label"
            && let Some(field) = FIT_FIELDS.iter().find(|field| fields.contains(**field))
        {
            return Err(Error::Validation(format!(
                "label element does not accept `{field}`: static labels are fixed at compile time, so only bound \
                 (dynamic) text slots fit their content at runtime"
            )));
        }
        reject_unexpected_fields(&fields, allowed, &format!("{} element", self.kind))
    }

    pub(super) fn required(&self, field: &str) -> Result<String> {
        match field {
            "name" => self.name.clone(),
            "frame" => self.frame.as_ref().map(|frame| frame.name()).transpose()?,
            "text" => self.text.clone(),
            "value" => self.value.clone(),
            _ => None,
        }
        .ok_or_else(|| Error::Validation(format!("{} element requires `{field}`", self.kind)))
    }

    fn required_u32(&self, field: &str) -> Result<u32> {
        match field {
            "width" => self.width,
            "height" => self.height,
            _ => None,
        }
        .ok_or_else(|| Error::Validation(format!("{} element requires `{field}`", self.kind)))
    }

    pub(super) fn pos(&self) -> Result<Option<Point>> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Ok(Some(Point::new(x, y))),
            (None, None) => Ok(None),
            _ => Err(Error::Validation(format!("{} element must set both `x` and `y`, or neither", self.kind))),
        }
    }

    pub(super) fn size(&self) -> Result<Size> {
        Ok(Size::new(self.required_u32("width")?, self.required_u32("height")?))
    }

    pub(super) fn optional_size(&self) -> Result<Option<Size>> {
        match (self.width, self.height) {
            (Some(width), Some(height)) => Ok(Some(Size::new(width, height))),
            (None, None) => Ok(None),
            _ => {
                Err(Error::Validation(format!("{} element must set both `width` and `height`, or neither", self.kind)))
            }
        }
    }

    /// The authored one-based `cell_slot` index, validated against the other
    /// slot sources. Layout resolves it against the enclosing repeater cell.
    pub(super) fn cell_slot(&self) -> Result<Option<u32>> {
        let Some(cell_slot) = self.cell_slot else {
            return Ok(None);
        };
        if self.slots.is_some() || self.pattern.is_some() || self.transform.is_some() {
            return Err(Error::Validation(format!(
                "{} element `cell_slot` cannot be combined with `slots`, `pattern`, or `transform`",
                self.kind
            )));
        }
        if cell_slot == 0 {
            return Err(Error::Validation(format!(
                "{} element `cell_slot` is one-based; use 1 for the first slot of the cell",
                self.kind
            )));
        }
        Ok(Some(cell_slot))
    }

    pub(super) fn cross_align(&self) -> Result<CrossAlign> {
        match self.align.as_str() {
            "" | "start" => Ok(CrossAlign::Start),
            "center" => Ok(CrossAlign::Center),
            "end" => Ok(CrossAlign::End),
            other => Err(Error::Validation(format!("{} element has unknown align `{other}`", self.kind))),
        }
    }

    pub(super) fn text_style(&self) -> Result<TextStyle> {
        let align = if self.align.is_empty() { None } else { Some(self.text_align()?) };
        let color = match &self.color {
            Some(color) => Rgb::parse_hex(color).ok_or_else(|| {
                Error::Validation(format!("{} element has invalid color `{color}`; expected #rrggbb", self.kind))
            })?,
            None => Rgb::DEFAULT_TEXT,
        };
        let font = match (&self.font, self.small_caps) {
            (Some(_), true) => {
                return Err(Error::Validation(format!("{} element sets both `font` and `small_caps`", self.kind)));
            }
            (Some(font), false) => Some(font.clone()),
            (None, small_caps) => small_caps.then(|| text_font::SMALL_CAPS.to_string()),
        };
        Ok(TextStyle {
            align,
            color,
            shadow: self.shadow,
            bold: self.bold,
            italic: self.italic,
            underlined: self.underlined,
            strikethrough: self.strikethrough,
            obfuscated: self.obfuscated,
            font,
        })
    }

    pub(super) fn text_fit(&self) -> Result<TextFit> {
        let ellipsis = match self.overflow.as_deref() {
            None => false,
            Some("ellipsis") => true,
            Some(other) => {
                return Err(Error::Validation(format!(
                    "{} element has unknown overflow `{other}`; valid overflow: ellipsis",
                    self.kind
                )));
            }
        };
        let lines = self.lines.unwrap_or(1);
        if lines == 0 {
            return Err(Error::Validation(format!("{} element `lines` must be at least 1", self.kind)));
        }
        let line_height = match self.line_height {
            Some(_) if self.lines.is_none() => {
                return Err(Error::Validation(format!("{} element sets `line_height` without `lines`", self.kind)));
            }
            Some(0) => {
                return Err(Error::Validation(format!("{} element `line_height` must be at least 1", self.kind)));
            }
            Some(line_height) => line_height,
            None => TextFit::DEFAULT_LINE_HEIGHT,
        };
        let height = (lines - 1).checked_mul(line_height).and_then(|height| height.checked_add(8));
        if height.is_none_or(|height| height > MAX_TEXT_HEIGHT) {
            return Err(Error::Validation(format!(
                "{} element `lines` span more than {MAX_TEXT_HEIGHT} pixels",
                self.kind
            )));
        }
        Ok(TextFit { ellipsis: ellipsis || lines > 1, lines, line_height })
    }

    pub(super) fn text_align(&self) -> Result<Align> {
        match self.align.as_str() {
            "" | "left" => Ok(Align::Left),
            "center" => Ok(Align::Center),
            "right" => Ok(Align::Right),
            other => Err(Error::Validation(format!("{} element has unknown text align `{other}`", self.kind))),
        }
    }

    /// The runtime action a button's `default` names.
    pub(super) fn default_action(&self) -> Result<Option<String>> {
        match self.default.as_deref() {
            None => Ok(None),
            Some("close") => Ok(Some(CLOSE_ACTION.to_string())),
            Some(other) => Err(Error::Validation(format!("button element has unknown default `{other}`"))),
        }
    }

    pub(super) fn slots(&self) -> Result<Option<Vec<InventorySlotRef>>> {
        self.slots.clone().map(parse_slot_refs).transpose()
    }

    pub(super) fn pattern(&self) -> Result<Option<SlotPattern>> {
        match (&self.pattern, &self.transform) {
            (Some(_), Some(_)) => {
                Err(Error::Validation(format!("{} element must set only one of `pattern` or `transform`", self.kind)))
            }
            (Some(pattern), None) => pattern.clone().into_pattern().map(Some),
            (None, Some(transform)) => transform.clone().into_rect_pattern().map(Some),
            (None, None) => Ok(None),
        }
    }

    pub(super) fn required_pattern(&self) -> Result<SlotPattern> {
        self.pattern()?.ok_or_else(|| Error::Validation(format!("{} element requires `pattern`", self.kind)))
    }

    pub(super) fn claim(&self) -> Result<SlotRectClaim> {
        match self.claim.as_deref() {
            None | Some("") | Some("none") => Ok(SlotRectClaim::None),
            Some("all") => Ok(SlotRectClaim::All),
            Some("unowned") => Ok(SlotRectClaim::Unowned),
            Some(other) => Err(Error::Validation(format!(
                "{} element has unknown claim `{other}`; valid claims: none, all, unowned",
                self.kind
            ))),
        }
    }

    pub(super) fn tooltip(&self) -> Result<Option<Tooltip>> {
        self.tooltip.clone().map(TooltipDto::into_tooltip).transpose()
    }

    pub(super) fn states(&self) -> Result<BTreeMap<String, ControlState>> {
        let mut states = BTreeMap::new();
        for (name, state) in &self.states {
            validate_name(name, "button state")?;
            states.insert(name.clone(), state.clone().into_state()?);
        }
        Ok(states)
    }
}
