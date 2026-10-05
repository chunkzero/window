//! Flexbox/grid style and item layout fields for `flex` and `section` elements.

use serde::Deserialize;
use taffy::style_helpers::{
    TaffyAuto, TaffyGridLine, TaffyGridSpan, TaffyZero, auto, fr, length, max_content, min_content, percent,
};
use taffy::{
    AlignContent, AlignItems, Dimension, FlexDirection, FlexWrap, GridAutoFlow, GridPlacement, GridTemplateComponent,
    LengthPercentage, LengthPercentageAuto, Line, MinTrackSizingFunction, Rect, Size, TrackSizingFunction,
};

use crate::model::ItemLayout;
use crate::{Error, Result};

/// A length: pixels, `"N%"`, or a keyword.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum LengthDto {
    Px(f32),
    Keyword(String),
}

/// Uniform pixels or per-edge lengths.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum EdgesDto {
    Uniform(LengthDto),
    Edges(EdgesObjectDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EdgesObjectDto {
    top: Option<LengthDto>,
    right: Option<LengthDto>,
    bottom: Option<LengthDto>,
    left: Option<LengthDto>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum GapDto {
    Uniform(LengthDto),
    Axes([LengthDto; 2]),
}

/// A grid template: a count of equal `1fr` tracks, or explicit track sizes.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum TracksDto {
    Count(u16),
    List(Vec<LengthDto>),
}

/// A grid line: a one-based start line, or `{ start?, end?, span? }`. Negative lines count from the end.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum GridLineDto {
    Start(i16),
    Object(GridLineObjectDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GridLineObjectDto {
    start: Option<i16>,
    end: Option<i16>,
    span: Option<u16>,
}

/// Container style of a `flex` element.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FlexStyleDto {
    display: Option<String>,
    direction: Option<String>,
    wrap: Option<bool>,
    justify: Option<String>,
    align: Option<String>,
    align_content: Option<String>,
    gap: Option<GapDto>,
    padding: Option<EdgesDto>,
    width: Option<LengthDto>,
    height: Option<LengthDto>,
    min_width: Option<LengthDto>,
    min_height: Option<LengthDto>,
    max_width: Option<LengthDto>,
    max_height: Option<LengthDto>,
    aspect_ratio: Option<f32>,
    columns: Option<TracksDto>,
    rows: Option<TracksDto>,
    auto_flow: Option<String>,
}

/// Item-level layout of a child of a `flex` or `section` element.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ItemLayoutDto {
    grow: Option<f32>,
    shrink: Option<f32>,
    basis: Option<LengthDto>,
    align_self: Option<String>,
    justify_self: Option<String>,
    margin: Option<EdgesDto>,
    position: Option<String>,
    top: Option<LengthDto>,
    right: Option<LengthDto>,
    bottom: Option<LengthDto>,
    left: Option<LengthDto>,
    min_width: Option<LengthDto>,
    min_height: Option<LengthDto>,
    max_width: Option<LengthDto>,
    max_height: Option<LengthDto>,
    column: Option<GridLineDto>,
    row: Option<GridLineDto>,
    translate: Option<[i32; 2]>,
}

impl LengthDto {
    fn percent(text: &str) -> Option<f32> {
        text.strip_suffix('%')?.trim().parse::<f32>().ok().filter(|v| v.is_finite()).map(|v| v / 100.0)
    }

    fn px(&self, field: &str) -> Result<Option<f32>> {
        match self {
            LengthDto::Px(px) if px.is_finite() => Ok(Some(*px)),
            LengthDto::Px(_) => Err(invalid(field, "must be a finite number")),
            LengthDto::Keyword(_) => Ok(None),
        }
    }

    fn dimension(&self, field: &str) -> Result<Dimension> {
        if let Some(px) = self.px(field)? {
            return Ok(Dimension::length(px));
        }
        let LengthDto::Keyword(text) = self else { unreachable!() };
        Ok(match text.as_str() {
            "auto" => Dimension::auto(),
            "min-content" => Dimension::min_content(),
            "max-content" => Dimension::max_content(),
            "fit-content" => Dimension::fit_content(),
            "stretch" => Dimension::stretch(),
            other => match Self::percent(other) {
                Some(p) => Dimension::percent(p),
                None => return Err(invalid(field, &format!("has unknown length `{other}`"))),
            },
        })
    }

    fn auto(&self, field: &str) -> Result<LengthPercentageAuto> {
        if let Some(px) = self.px(field)? {
            return Ok(LengthPercentageAuto::length(px));
        }
        let LengthDto::Keyword(text) = self else { unreachable!() };
        match text.as_str() {
            "auto" => Ok(LengthPercentageAuto::auto()),
            other => Self::percent(other)
                .map(LengthPercentageAuto::percent)
                .ok_or_else(|| invalid(field, &format!("has unknown length `{other}`"))),
        }
    }

    fn fixed(&self, field: &str) -> Result<LengthPercentage> {
        if let Some(px) = self.px(field)? {
            return Ok(LengthPercentage::length(px));
        }
        let LengthDto::Keyword(text) = self else { unreachable!() };
        Self::percent(text)
            .map(LengthPercentage::percent)
            .ok_or_else(|| invalid(field, &format!("has unknown length `{text}`")))
    }

    fn track(&self, field: &str) -> Result<TrackSizingFunction> {
        if let Some(px) = self.px(field)? {
            return Ok(length(px));
        }
        let LengthDto::Keyword(text) = self else { unreachable!() };
        Ok(match text.as_str() {
            "auto" => auto(),
            "min-content" => min_content(),
            "max-content" => max_content(),
            other => {
                if let Some(value) = other.strip_suffix("fr").and_then(|v| v.trim().parse::<f32>().ok()) {
                    nonnegative(value, field)?;
                    TrackSizingFunction { min: MinTrackSizingFunction::AUTO, max: fr(value) }
                } else if let Some(p) = Self::percent(other) {
                    percent(p)
                } else {
                    return Err(invalid(field, &format!("has unknown track size `{other}`")));
                }
            }
        })
    }
}

impl EdgesDto {
    fn map<T: Copy>(&self, field: &str, default: T, f: impl Fn(&LengthDto, &str) -> Result<T>) -> Result<Rect<T>> {
        let edge =
            |value: &Option<LengthDto>| value.as_ref().map(|v| f(v, field)).transpose().map(|v| v.unwrap_or(default));
        Ok(match self {
            EdgesDto::Uniform(value) => {
                let value = f(value, field)?;
                Rect { left: value, right: value, top: value, bottom: value }
            }
            EdgesDto::Edges(EdgesObjectDto { top, right, bottom, left }) => {
                Rect { left: edge(left)?, right: edge(right)?, top: edge(top)?, bottom: edge(bottom)? }
            }
        })
    }
}

impl GridLineDto {
    fn line(&self, field: &str) -> Result<Line<GridPlacement>> {
        let (start, end, span) = match self {
            GridLineDto::Start(start) => (Some(*start), None, None),
            GridLineDto::Object(GridLineObjectDto { start, end, span }) => (*start, *end, *span),
        };
        if start == Some(0) || end == Some(0) {
            return Err(invalid(field, "grid lines are one-based; 0 is not a line"));
        }
        if span == Some(0) {
            return Err(invalid(field, "span must be at least 1"));
        }
        let line = |n: Option<i16>| n.map_or(GridPlacement::AUTO, GridPlacement::from_line_index);
        Ok(match (start, end, span) {
            (_, Some(_), Some(_)) => return Err(invalid(field, "sets both `end` and `span`")),
            (start, None, Some(span)) if start.is_some() => {
                Line { start: line(start), end: GridPlacement::from_span(span) }
            }
            (None, None, Some(span)) => Line { start: GridPlacement::from_span(span), end: GridPlacement::AUTO },
            (start, end, _) => Line { start: line(start), end: line(end) },
        })
    }
}

impl FlexStyleDto {
    pub(super) fn into_style(self) -> Result<taffy::Style> {
        let display = match self.display.as_deref() {
            None | Some("flex") => taffy::Display::Flex,
            Some("grid") => taffy::Display::Grid,
            Some(other) => return Err(invalid("style.display", &format!("has unknown display `{other}`"))),
        };
        let mut style = taffy::Style { display, ..Default::default() };
        if let Some(direction) = &self.direction {
            style.flex_direction = match direction.as_str() {
                "row" => FlexDirection::Row,
                "column" => FlexDirection::Column,
                "row-reverse" => FlexDirection::RowReverse,
                "column-reverse" => FlexDirection::ColumnReverse,
                other => return Err(invalid("style.direction", &format!("has unknown direction `{other}`"))),
            };
        }
        if self.wrap == Some(true) {
            style.flex_wrap = FlexWrap::Wrap;
        }
        style.justify_content = self.justify.as_deref().map(|v| content_align(v, "style.justify")).transpose()?;
        style.align_items = self.align.as_deref().map(|v| item_align(v, "style.align")).transpose()?;
        style.align_content =
            self.align_content.as_deref().map(|v| content_align(v, "style.align_content")).transpose()?;
        if let Some(gap) = &self.gap {
            style.gap = match gap {
                GapDto::Uniform(v) => {
                    let v = v.fixed("style.gap")?;
                    Size { width: v, height: v }
                }
                GapDto::Axes([x, y]) => Size { width: x.fixed("style.gap")?, height: y.fixed("style.gap")? },
            };
        }
        if let Some(padding) = &self.padding {
            style.padding = padding.map("style.padding", LengthPercentage::ZERO, LengthDto::fixed)?;
        }
        let dim = |v: &Option<LengthDto>, field: &str| v.as_ref().map(|v| v.dimension(field)).transpose();
        let auto = |v: &Option<LengthDto>, field: &str| v.as_ref().map(|v| v.auto(field)).transpose();
        style.size = Size {
            width: dim(&self.width, "style.width")?.unwrap_or(Dimension::AUTO),
            height: dim(&self.height, "style.height")?.unwrap_or(Dimension::AUTO),
        };
        style.min_size = Size {
            width: auto(&self.min_width, "style.min_width")?.unwrap_or(LengthPercentageAuto::AUTO),
            height: auto(&self.min_height, "style.min_height")?.unwrap_or(LengthPercentageAuto::AUTO),
        };
        style.max_size = Size {
            width: auto(&self.max_width, "style.max_width")?.unwrap_or(LengthPercentageAuto::AUTO),
            height: auto(&self.max_height, "style.max_height")?.unwrap_or(LengthPercentageAuto::AUTO),
        };
        if let Some(ratio) = self.aspect_ratio {
            nonnegative(ratio, "style.aspect_ratio")?;
            if ratio == 0.0 {
                return Err(invalid("style.aspect_ratio", "must be greater than zero"));
            }
        }
        style.aspect_ratio = self.aspect_ratio;
        if let Some(columns) = &self.columns {
            style.grid_template_columns = tracks(columns, "style.columns")?;
        }
        if let Some(rows) = &self.rows {
            style.grid_template_rows = tracks(rows, "style.rows")?;
        }
        if let Some(flow) = &self.auto_flow {
            style.grid_auto_flow = parse_auto_flow(flow)?;
        }
        Ok(style)
    }
}

impl ItemLayoutDto {
    pub(super) fn into_layout(self) -> Result<ItemLayout> {
        if let Some(grow) = self.grow {
            nonnegative(grow, "layout.grow")?;
        }
        if let Some(shrink) = self.shrink {
            nonnegative(shrink, "layout.shrink")?;
        }
        let auto = |v: &Option<LengthDto>, field: &str| v.as_ref().map(|v| v.auto(field)).transpose();
        let has_inset = self.top.is_some() || self.right.is_some() || self.bottom.is_some() || self.left.is_some();
        let absolute = match self.position.as_deref() {
            None | Some("relative") => false,
            Some("absolute") => true,
            Some(other) => return Err(invalid("layout.position", &format!("has unknown position `{other}`"))),
        };
        let size = |w: &Option<LengthDto>, h: &Option<LengthDto>, field: &str| -> Result<_> {
            if w.is_none() && h.is_none() {
                return Ok(None);
            }
            Ok(Some(Size {
                width: auto(w, field)?.unwrap_or(LengthPercentageAuto::AUTO),
                height: auto(h, field)?.unwrap_or(LengthPercentageAuto::AUTO),
            }))
        };
        Ok(ItemLayout {
            grow: self.grow,
            shrink: self.shrink,
            basis: self.basis.as_ref().map(|v| v.dimension("layout.basis")).transpose()?,
            align_self: self.align_self.as_deref().map(|v| item_align(v, "layout.align_self")).transpose()?,
            justify_self: self.justify_self.as_deref().map(|v| item_align(v, "layout.justify_self")).transpose()?,
            margin: self
                .margin
                .as_ref()
                .map(|m| m.map("layout.margin", LengthPercentageAuto::ZERO, LengthDto::auto))
                .transpose()?,
            absolute,
            inset: has_inset
                .then(|| -> Result<_> {
                    Ok(Rect {
                        top: auto(&self.top, "layout.top")?.unwrap_or(LengthPercentageAuto::AUTO),
                        right: auto(&self.right, "layout.right")?.unwrap_or(LengthPercentageAuto::AUTO),
                        bottom: auto(&self.bottom, "layout.bottom")?.unwrap_or(LengthPercentageAuto::AUTO),
                        left: auto(&self.left, "layout.left")?.unwrap_or(LengthPercentageAuto::AUTO),
                    })
                })
                .transpose()?,
            min_size: size(&self.min_width, &self.min_height, "layout.min")?,
            max_size: size(&self.max_width, &self.max_height, "layout.max")?,
            column: self.column.as_ref().map(|v| v.line("layout.column")).transpose()?,
            row: self.row.as_ref().map(|v| v.line("layout.row")).transpose()?,
            translate: self.translate.map_or_else(Default::default, |[x, y]| crate::geometry::Point::new(x, y)),
        })
    }
}

pub(super) fn parse_auto_flow(flow: &str) -> Result<GridAutoFlow> {
    Ok(match flow {
        "row" => GridAutoFlow::Row,
        "column" => GridAutoFlow::Column,
        "row-dense" => GridAutoFlow::RowDense,
        "column-dense" => GridAutoFlow::ColumnDense,
        other => return Err(invalid("flow", &format!("has unknown auto flow `{other}`"))),
    })
}

fn tracks(tracks: &TracksDto, field: &str) -> Result<Vec<GridTemplateComponent<String>>> {
    match tracks {
        TracksDto::Count(0) => Err(invalid(field, "needs at least one track")),
        TracksDto::Count(count) => Ok(taffy::style_helpers::evenly_sized_tracks(*count)),
        TracksDto::List(list) => list.iter().map(|t| t.track(field).map(GridTemplateComponent::Single)).collect(),
    }
}

fn item_align(value: &str, field: &str) -> Result<AlignItems> {
    Ok(match value {
        "start" => AlignItems::START,
        "end" => AlignItems::END,
        "center" => AlignItems::CENTER,
        "stretch" => AlignItems::STRETCH,
        "baseline" => AlignItems::BASELINE,
        other => return Err(invalid(field, &format!("has unknown alignment `{other}`"))),
    })
}

fn content_align(value: &str, field: &str) -> Result<AlignContent> {
    Ok(match value {
        "start" => AlignContent::START,
        "end" => AlignContent::END,
        "center" => AlignContent::CENTER,
        "stretch" => AlignContent::STRETCH,
        "between" => AlignContent::SPACE_BETWEEN,
        "around" => AlignContent::SPACE_AROUND,
        "evenly" => AlignContent::SPACE_EVENLY,
        other => return Err(invalid(field, &format!("has unknown alignment `{other}`"))),
    })
}

fn invalid(field: &str, message: &str) -> Error {
    Error::Validation(format!("`{field}` {message}"))
}

fn nonnegative(value: f32, field: &str) -> Result<()> {
    if value.is_finite() && value >= 0.0 { Ok(()) } else { Err(invalid(field, "must be finite and non-negative")) }
}
