mod build;
mod fields;
mod handles;
mod indexed;

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::button::{ButtonStateDto, TooltipDto};
use super::flex::{FlexStyleDto, ItemLayoutDto};
use super::insets::InsetsDto;
use super::parse::json_object_fields;
use super::patterns::{SlotPatternDto, SlotRectPatternDto, SlotRefDto};
use crate::model::{Element, LayoutChild};
use crate::{Error, Result};

use handles::HandleRefDto;
pub(super) use handles::{check_shared, collect_handles};
pub(super) use indexed::flatten_indexed;

#[derive(Debug)]
pub(super) struct ElementDto {
    fields: BTreeSet<String>,
    kind: String,
    name: Option<String>,
    index: Option<Vec<u32>>,
    frame: Option<String>,
    text: Option<String>,
    value: Option<String>,
    initial: Option<String>,
    item_model: Option<String>,
    sprite: Option<String>,
    selected_sprite: Option<String>,
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
    font: Option<String>,
    small_caps: bool,
    overflow: Option<String>,
    lines: Option<u32>,
    line_height: Option<u32>,
    style: Option<FlexStyleDto>,
    layout: Option<ItemLayoutDto>,
    section: Option<String>,
    outset: Option<InsetsDto>,
    flow: Option<String>,
    children: Vec<ElementDto>,
    handle: Option<HandleRefDto>,
    on_click: Option<HandleRefDto>,
    enabled: Option<HandleRefDto>,
    state: Option<HandleRefDto>,
    cells: Option<Vec<Vec<ElementDto>>>,
    /// Cell button names of a repeater with `cells`, assigned while collecting handles.
    cell_buttons: Vec<String>,
    /// Whether the cells of a repeater with `cells` route clicks to a handler.
    cell_action: bool,
}

/// An indexed binding's `index`: one number, or one number per dimension.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IndexDto {
    One(u32),
    Many(Vec<u32>),
}

#[derive(Debug, Deserialize)]
struct ElementShapeDto {
    #[serde(rename = "type")]
    kind: String,
    name: Option<String>,
    index: Option<IndexDto>,
    frame: Option<String>,
    text: Option<String>,
    value: Option<String>,
    initial: Option<String>,
    item_model: Option<String>,
    sprite: Option<String>,
    selected_sprite: Option<String>,
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
    font: Option<String>,
    #[serde(default)]
    small_caps: bool,
    overflow: Option<String>,
    lines: Option<u32>,
    line_height: Option<u32>,
    style: Option<FlexStyleDto>,
    layout: Option<ItemLayoutDto>,
    section: Option<String>,
    outset: Option<InsetsDto>,
    flow: Option<String>,
    #[serde(default)]
    children: Vec<ElementDto>,
    handle: Option<HandleRefDto>,
    on_click: Option<HandleRefDto>,
    enabled: Option<HandleRefDto>,
    state: Option<HandleRefDto>,
    cells: Option<Vec<Vec<ElementDto>>>,
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
            index: shape.index.map(|index| match index {
                IndexDto::One(index) => vec![index],
                IndexDto::Many(index) => index,
            }),
            frame: shape.frame,
            text: shape.text,
            value: shape.value,
            initial: shape.initial,
            item_model: shape.item_model,
            sprite: shape.sprite,
            selected_sprite: shape.selected_sprite,
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
            font: shape.font,
            small_caps: shape.small_caps,
            overflow: shape.overflow,
            lines: shape.lines,
            line_height: shape.line_height,
            style: shape.style,
            layout: shape.layout,
            section: shape.section,
            outset: shape.outset,
            flow: shape.flow,
            children: shape.children,
            handle: shape.handle,
            on_click: shape.on_click,
            enabled: shape.enabled,
            state: shape.state,
            cells: shape.cells,
            cell_buttons: Vec::new(),
            cell_action: false,
        }
    }
}

pub(super) fn convert_children(children: Vec<ElementDto>) -> Result<Vec<Element>> {
    children
        .into_iter()
        .map(|child| {
            if child.layout.is_some() {
                return Err(Error::Validation(format!(
                    "{} element sets `layout`, which only applies inside a flex or section element",
                    child.kind
                )));
            }
            child.into_element()
        })
        .collect()
}

fn convert_layout_children(children: Vec<ElementDto>) -> Result<Vec<LayoutChild>> {
    children
        .into_iter()
        .map(|mut child| {
            let layout = child.layout.take().map(ItemLayoutDto::into_layout).transpose()?.unwrap_or_default();
            Ok(LayoutChild { layout, element: child.into_element()? })
        })
        .collect()
}
