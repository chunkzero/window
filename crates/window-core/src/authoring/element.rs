mod build;
mod fields;

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::button::{ButtonStateDto, TooltipDto};
use super::parse::json_object_fields;
use super::patterns::{SlotPatternDto, SlotRectPatternDto, SlotRefDto};
use crate::Result;
use crate::model::Element;

#[derive(Debug)]
pub(super) struct ElementDto {
    fields: BTreeSet<String>,
    kind: String,
    name: Option<String>,
    frame: Option<String>,
    text: Option<String>,
    initial: Option<String>,
    item_model: Option<String>,
    sprite: Option<String>,
    default: Option<String>,
    slots: Option<Vec<SlotRefDto>>,
    pattern: Option<SlotPatternDto>,
    transform: Option<SlotRectPatternDto>,
    claim: Option<String>,
    action: Option<bool>,
    cell_slot: Option<u32>,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
    padding: u32,
    gap: u32,
    align: String,
    color: Option<String>,
    tooltip: Option<TooltipDto>,
    states: BTreeMap<String, ButtonStateDto>,
    shadow: bool,
    bold: bool,
    italic: bool,
    underlined: bool,
    strikethrough: bool,
    obfuscated: bool,
    children: Vec<ElementDto>,
}

#[derive(Debug, Deserialize)]
struct ElementShapeDto {
    #[serde(rename = "type")]
    kind: String,
    name: Option<String>,
    frame: Option<String>,
    text: Option<String>,
    initial: Option<String>,
    item_model: Option<String>,
    sprite: Option<String>,
    default: Option<String>,
    #[serde(default)]
    slots: Option<Vec<SlotRefDto>>,
    pattern: Option<SlotPatternDto>,
    transform: Option<SlotRectPatternDto>,
    claim: Option<String>,
    action: Option<bool>,
    cell_slot: Option<u32>,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(default)]
    padding: u32,
    #[serde(default)]
    gap: u32,
    #[serde(default)]
    align: String,
    color: Option<String>,
    tooltip: Option<TooltipDto>,
    #[serde(default)]
    states: BTreeMap<String, ButtonStateDto>,
    #[serde(default)]
    shadow: bool,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underlined: bool,
    #[serde(default)]
    strikethrough: bool,
    #[serde(default)]
    obfuscated: bool,
    #[serde(default)]
    children: Vec<ElementDto>,
}

impl<'de> Deserialize<'de> for ElementDto {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let fields = json_object_fields::<D::Error>(&value, "element")?;
        let shape = serde_json::from_value::<ElementShapeDto>(value).map_err(serde::de::Error::custom)?;
        Ok(Self::from_shape(fields, shape))
    }
}

impl ElementDto {
    fn from_shape(fields: BTreeSet<String>, shape: ElementShapeDto) -> Self {
        Self {
            fields,
            kind: shape.kind,
            name: shape.name,
            frame: shape.frame,
            text: shape.text,
            initial: shape.initial,
            item_model: shape.item_model,
            sprite: shape.sprite,
            default: shape.default,
            slots: shape.slots,
            pattern: shape.pattern,
            transform: shape.transform,
            claim: shape.claim,
            action: shape.action,
            cell_slot: shape.cell_slot,
            x: shape.x,
            y: shape.y,
            width: shape.width,
            height: shape.height,
            padding: shape.padding,
            gap: shape.gap,
            align: shape.align,
            color: shape.color,
            tooltip: shape.tooltip,
            states: shape.states,
            shadow: shape.shadow,
            bold: shape.bold,
            italic: shape.italic,
            underlined: shape.underlined,
            strikethrough: shape.strikethrough,
            obfuscated: shape.obfuscated,
            children: shape.children,
        }
    }
}

fn convert_children(children: Vec<ElementDto>) -> Result<Vec<Element>> {
    children.into_iter().map(ElementDto::into_element).collect()
}
