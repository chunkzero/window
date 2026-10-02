use std::collections::BTreeSet;

use serde::Deserialize;

use super::parse::{json_object_fields, reject_unexpected_fields};
use crate::inventory::{
    InventorySlotArea, InventorySlotRef, InventorySlotSection, SlotGridPattern, SlotPattern, SlotRectPattern,
};
use crate::{Error, Result};

const MAX_SLOT_RANGE_LEN: u64 = 1024;

#[derive(Clone, Debug)]
pub(super) struct SlotPatternDto {
    fields: BTreeSet<String>,
    kind: String,
    section: String,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    slots: Vec<u32>,
    columns: Option<u32>,
    rows: Option<u32>,
    cell_width: Option<u32>,
    cell_height: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
struct SlotPatternShapeDto {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    section: String,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(default)]
    slots: Vec<u32>,
    columns: Option<u32>,
    rows: Option<u32>,
    cell_width: Option<u32>,
    cell_height: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SlotRectPatternDto {
    #[serde(default)]
    section: String,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Clone, Debug)]
pub(super) enum SlotRefDto {
    ContainerIndex(u32),
    Object(SlotRefObjectDto),
    Range(SlotRangeDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SlotRefObjectDto {
    #[serde(default)]
    area: String,
    index: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SlotRangeDto {
    #[serde(default)]
    area: String,
    first: u32,
    last: u32,
}

impl<'de> Deserialize<'de> for SlotPatternDto {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let fields = json_object_fields::<D::Error>(&value, "slot pattern")?;
        let shape = serde_json::from_value::<SlotPatternShapeDto>(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            fields,
            kind: shape.kind,
            section: shape.section,
            x: shape.x,
            y: shape.y,
            width: shape.width,
            height: shape.height,
            slots: shape.slots,
            columns: shape.columns,
            rows: shape.rows,
            cell_width: shape.cell_width,
            cell_height: shape.cell_height,
        })
    }
}

impl<'de> Deserialize<'de> for SlotRefDto {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match &value {
            serde_json::Value::Number(number) => {
                let Some(index) = number.as_u64().and_then(|index| u32::try_from(index).ok()) else {
                    return Err(serde::de::Error::custom("slot ref index must be a non-negative integer"));
                };
                Ok(Self::ContainerIndex(index))
            }
            serde_json::Value::Object(_) => {
                let fields = json_object_fields::<D::Error>(&value, "slot ref")?;
                if fields.contains("index") {
                    reject_unexpected_fields(&fields, &["area", "index"], "slot ref")
                        .map_err(serde::de::Error::custom)?;
                    let slot = serde_json::from_value::<SlotRefObjectDto>(value).map_err(serde::de::Error::custom)?;
                    Ok(Self::Object(slot))
                } else if fields.contains("first") || fields.contains("last") {
                    reject_unexpected_fields(&fields, &["area", "first", "last"], "slot range")
                        .map_err(serde::de::Error::custom)?;
                    let range = serde_json::from_value::<SlotRangeDto>(value).map_err(serde::de::Error::custom)?;
                    Ok(Self::Range(range))
                } else {
                    Err(serde::de::Error::custom("slot ref object requires `index` or `first`/`last`"))
                }
            }
            _ => Err(serde::de::Error::custom("slot ref must be a non-negative integer or object")),
        }
    }
}

impl SlotPatternDto {
    pub(super) fn into_pattern(self) -> Result<SlotPattern> {
        self.validate_fields()?;
        match self.kind.as_str() {
            "" | "rect" => self.rect().into_rect_pattern(),
            "slots" => {
                let section = parse_slot_section(&self.section, "pattern.section")?;
                if self.slots.is_empty() {
                    return Err(Error::Validation("slot pattern `slots` requires at least one slot".into()));
                }
                Ok(SlotPattern::Slots { section, slots: self.slots })
            }
            "grid" => self.into_grid_pattern(),
            other => Err(Error::Validation(format!(
                "slot pattern has unknown kind `{other}`; valid kinds: rect, slots, grid"
            ))),
        }
    }

    fn into_grid_pattern(self) -> Result<SlotPattern> {
        let section = parse_slot_section(&self.section, "pattern.section")?;
        let x = required_pattern_u32(self.x, "x", "grid")?;
        let y = required_pattern_u32(self.y, "y", "grid")?;
        let columns = required_pattern_u32(self.columns, "columns", "grid")?;
        let rows = required_pattern_u32(self.rows, "rows", "grid")?;
        let cell_width = required_pattern_u32(self.cell_width, "cell_width", "grid")?;
        let cell_height = required_pattern_u32(self.cell_height, "cell_height", "grid")?;
        if columns == 0 || rows == 0 || cell_width == 0 || cell_height == 0 {
            return Err(Error::Validation("grid pattern dimensions must all be greater than zero".into()));
        }
        Ok(SlotPattern::Grid(SlotGridPattern { section, x, y, columns, rows, cell_width, cell_height }))
    }

    fn rect(&self) -> SlotRectPatternDto {
        SlotRectPatternDto {
            section: self.section.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }

    fn validate_fields(&self) -> Result<()> {
        let allowed = match self.kind.as_str() {
            "" | "rect" => &["kind", "section", "x", "y", "width", "height"][..],
            "slots" => &["kind", "section", "slots"],
            "grid" => &["kind", "section", "x", "y", "columns", "rows", "cell_width", "cell_height"],
            _ => return Ok(()),
        };
        reject_unexpected_fields(&self.fields, allowed, "slot pattern")
    }
}

impl SlotRectPatternDto {
    pub(super) fn into_rect_pattern(self) -> Result<SlotPattern> {
        let section = parse_slot_section(&self.section, "pattern.section")?;
        let x = required_pattern_u32(self.x, "x", "rect")?;
        let y = required_pattern_u32(self.y, "y", "rect")?;
        let width = required_pattern_u32(self.width, "width", "rect")?;
        let height = required_pattern_u32(self.height, "height", "rect")?;
        if width == 0 || height == 0 {
            return Err(Error::Validation("rect pattern width and height must be greater than zero".into()));
        }
        Ok(SlotPattern::Rect(SlotRectPattern { section, x, y, width, height }))
    }
}

impl SlotRefDto {
    fn append_to(self, out: &mut Vec<InventorySlotRef>) -> Result<()> {
        match self {
            SlotRefDto::ContainerIndex(index) => out.push(InventorySlotRef::container(index)),
            SlotRefDto::Object(slot) => {
                out.push(InventorySlotRef { area: parse_slot_area(&slot.area)?, index: slot.index })
            }
            SlotRefDto::Range(range) => {
                let area = parse_slot_area(&range.area)?;
                let count = u64::from(range.first.abs_diff(range.last)) + 1;
                if count > MAX_SLOT_RANGE_LEN {
                    return Err(Error::Validation(format!(
                        "slot range {}..{} contains {count} slots; maximum is {MAX_SLOT_RANGE_LEN}",
                        range.first, range.last
                    )));
                }
                if range.first <= range.last {
                    out.extend((range.first..=range.last).map(|index| InventorySlotRef { area, index }));
                } else {
                    out.extend((range.last..=range.first).rev().map(|index| InventorySlotRef { area, index }));
                }
            }
        }
        Ok(())
    }
}

pub(super) fn parse_slot_refs(slots: Vec<SlotRefDto>) -> Result<Vec<InventorySlotRef>> {
    let mut out = Vec::new();
    for slot in slots {
        slot.append_to(&mut out)?;
    }
    Ok(out)
}

fn parse_slot_area(value: &str) -> Result<InventorySlotArea> {
    match value {
        "" | "container" => Ok(InventorySlotArea::Container),
        "player" => Ok(InventorySlotArea::Player),
        other => Err(Error::Validation(format!("slot ref has unknown area `{other}`; valid areas: container, player"))),
    }
}

fn parse_slot_section(value: &str, field: &str) -> Result<InventorySlotSection> {
    match value {
        "" | "container" => Ok(InventorySlotSection::Container),
        "player" => Ok(InventorySlotSection::Player),
        "hotbar" => Ok(InventorySlotSection::Hotbar),
        other => Err(Error::Validation(format!(
            "{field} has unknown section `{other}`; valid sections: container, player, hotbar"
        ))),
    }
}

fn required_pattern_u32(value: Option<u32>, field: &str, kind: &str) -> Result<u32> {
    value.ok_or_else(|| Error::Validation(format!("{kind} slot pattern requires `{field}`")))
}
