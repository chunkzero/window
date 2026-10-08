mod build;
mod fields;
mod handles;

use std::collections::BTreeSet;

use serde::Deserialize;

use super::art::{ArtRefDto, ArtUse, intern};
use super::flex::{FlexStyleDto, ItemLayoutDto};
use super::insets::InsetsDto;
use super::parse::json_object_fields;
use super::patterns::{SlotPatternDto, SlotRectPatternDto, SlotRefDto};
use super::tooltip::TooltipDto;
use crate::model::{Art, Element, LayoutChild};
use crate::{Error, Result};

use handles::HandleRefDto;
pub(super) use handles::{check_shared, collect_handles};

#[derive(Debug)]
pub(super) struct ElementDto {
    fields: BTreeSet<String>,
    kind: String,
    /// The entry name, assigned while collecting handles.
    name: Option<String>,
    frame: Option<ArtRefDto>,
    art: Option<ArtRefDto>,
    text: Option<String>,
    value: Option<String>,
    initial: Option<String>,
    item_model: Option<String>,
    selected_sprite: Option<ArtRefDto>,
    slots: Option<Vec<SlotRefDto>>,
    pattern: Option<SlotPatternDto>,
    transform: Option<SlotRectPatternDto>,
    claim: Option<String>,
    action: Option<bool>,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
    align: String,
    color: Option<String>,
    tooltip: Option<TooltipDto>,
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
    debug_name: Option<String>,
    /// The runtime action a region bound to a builtin runs, assigned while collecting handles.
    default_action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ElementShapeDto {
    #[serde(rename = "type")]
    kind: String,
    frame: Option<ArtRefDto>,
    art: Option<ArtRefDto>,
    text: Option<String>,
    value: Option<String>,
    initial: Option<String>,
    item_model: Option<String>,
    selected_sprite: Option<ArtRefDto>,
    #[serde(default)]
    slots: Option<Vec<SlotRefDto>>,
    pattern: Option<SlotPatternDto>,
    transform: Option<SlotRectPatternDto>,
    claim: Option<String>,
    action: Option<bool>,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(default)]
    align: String,
    color: Option<String>,
    tooltip: Option<TooltipDto>,
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
    debug_name: Option<String>,
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
            name: None,
            frame: shape.frame,
            art: shape.art,
            text: shape.text,
            value: shape.value,
            initial: shape.initial,
            item_model: shape.item_model,
            selected_sprite: shape.selected_sprite,
            slots: shape.slots,
            pattern: shape.pattern,
            transform: shape.transform,
            claim: shape.claim,
            action: shape.action,
            x: shape.x,
            y: shape.y,
            width: shape.width,
            height: shape.height,
            align: shape.align,
            color: shape.color,
            tooltip: shape.tooltip,
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
            debug_name: shape.debug_name,
            default_action: None,
        }
    }
}

/// Interns the inline art of `children` and their subtrees into `table`.
pub(super) fn intern_art(children: &mut [ElementDto], table: &mut Art) -> Result<()> {
    for child in children {
        intern(table, &mut child.frame, ArtUse::Frame)?;
        intern(table, &mut child.art, ArtUse::Image)?;
        intern(table, &mut child.selected_sprite, ArtUse::Sprite)?;
        intern_art(&mut child.children, table)?;
    }
    Ok(())
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
