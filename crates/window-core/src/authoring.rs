//! TypeScript-authored project model.
//!
//! The rpp plugin loads `window/**/*.ts` definitions, merges their exported values,
//! JSON-encodes the result, and hands it to this module. The accepted JSON is
//! intentionally close to the public model: frames and sprites live under
//! `theme`, and windows carry a tree of typed elements.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::geometry::{Insets, Point, Size};
use crate::inventory::{
    InventorySlotArea, InventorySlotRef, InventorySlotSection, SlotGridPattern, SlotPattern, SlotRectClaim,
    SlotRectPattern,
};
use crate::ir::{Align, ButtonDefault, ButtonState, ButtonTooltip, HudChannel, HudShader, Rgb};
use crate::model::{
    CrossAlign, Element, Frame, GeneratedKind, GeneratedStyle, Hud, SpriteDef, TextStyle, Theme, Window,
};
use crate::surface::ContainerKind;
use crate::{Error, Result};

/// Everything parsed from a TypeScript-authored Window project.
#[derive(Clone, Debug, Default)]
pub struct ParsedProject {
    /// Merged theme definitions (global names; duplicates are errors).
    pub theme: Theme,
    /// All windows (duplicate window names are errors).
    pub windows: Vec<Window>,
    /// All HUDs (duplicate HUD names are errors).
    pub huds: Vec<Hud>,
    /// Build/runtime options supplied by the rpp host.
    pub options: BuildOptions,
    /// Target Minecraft/resource-pack version metadata supplied by the rpp host.
    pub target: PackTarget,
}

/// Build options supplied by the rpp plugin host.
#[derive(Clone, Debug, Default)]
pub struct BuildOptions {
    /// Whether to emit generated core shader overrides for HUD relocation.
    pub hud_shaders: bool,
}

/// Target pack metadata supplied by the rpp plugin host.
#[derive(Clone, Debug, Default)]
pub struct PackTarget {
    /// Resource pack format, when known.
    pub pack_format: Option<u32>,
}

/// Parse the plugin's JSON project payload into the authored model.
pub fn project_from_json(bytes: &[u8]) -> Result<ParsedProject> {
    let dto: ProjectDto = serde_json::from_slice(bytes)
        .map_err(|error| Error::Parse { path: "window lua project".into(), message: error.to_string() })?;
    dto.into_project()
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectDto {
    #[serde(default)]
    theme: ThemeDto,
    #[serde(default)]
    themes: Vec<ThemeDto>,
    #[serde(default)]
    windows: Vec<WindowDto>,
    #[serde(default)]
    huds: Vec<HudDto>,
    #[serde(default)]
    options: OptionsDto,
    #[serde(default)]
    target: TargetDto,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeDto {
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum InsetsDto {
    Uniform(u32),
    Edges(InsetsEdgesDto),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InsetsEdgesDto {
    #[serde(default)]
    top: u32,
    #[serde(default)]
    right: u32,
    #[serde(default)]
    bottom: u32,
    #[serde(default)]
    left: u32,
}

impl Default for InsetsDto {
    fn default() -> Self {
        Self::Uniform(0)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowDto {
    name: String,
    container: String,
    #[serde(default)]
    bleed: InsetsDto,
    #[serde(default)]
    children: Vec<ElementDto>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HudDto {
    name: String,
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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptionsDto {
    #[serde(default)]
    hud_shaders: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetDto {
    pack_format: Option<u32>,
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

#[derive(Debug)]
struct ElementDto {
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

#[derive(Clone, Debug)]
struct SlotPatternDto {
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
struct SlotRectPatternDto {
    #[serde(default)]
    section: String,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Clone, Debug)]
enum SlotRefDto {
    ContainerIndex(u32),
    Object(SlotRefObjectDto),
    Range(SlotRangeDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotRefObjectDto {
    #[serde(default)]
    area: String,
    index: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotRangeDto {
    #[serde(default)]
    area: String,
    first: u32,
    last: u32,
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

fn json_object_fields<E>(value: &serde_json::Value, context: &str) -> std::result::Result<BTreeSet<String>, E>
where
    E: serde::de::Error,
{
    match value {
        serde_json::Value::Object(object) => Ok(object.keys().cloned().collect()),
        _ => Err(E::custom(format!("{context} must be an object"))),
    }
}

impl ProjectDto {
    fn into_project(self) -> Result<ParsedProject> {
        let mut theme = Theme::default();

        let mut theme_docs = Vec::with_capacity(self.themes.len() + 1);
        theme_docs.push(self.theme);
        theme_docs.extend(self.themes);
        for theme_doc in theme_docs {
            for (name, frame) in theme_doc.frames {
                validate_name(&name, "frame")?;
                if theme.frames.contains_key(&name) {
                    return Err(Error::Validation(format!("duplicate frame name `{name}`")));
                }
                if theme.sprites.contains_key(&name) {
                    return Err(Error::Validation(format!(
                        "frame name `{name}` collides with a sprite of the same name"
                    )));
                }
                theme.frames.insert(name, frame.into_frame()?);
            }

            for (name, sprite) in theme_doc.sprites {
                validate_name(&name, "sprite")?;
                if theme.sprites.contains_key(&name) {
                    return Err(Error::Validation(format!("duplicate sprite name `{name}`")));
                }
                if theme.frames.contains_key(&name) {
                    return Err(Error::Validation(format!(
                        "sprite name `{name}` collides with a frame of the same name"
                    )));
                }
                theme.sprites.insert(name, sprite.into_sprite()?);
            }
        }

        let mut seen_windows = BTreeMap::new();
        let mut windows = Vec::with_capacity(self.windows.len());
        for window in self.windows {
            validate_name(&window.name, "window")?;
            if seen_windows.insert(window.name.clone(), ()).is_some() {
                return Err(Error::Validation(format!("duplicate window name `{}`", window.name)));
            }
            let container = ContainerKind::parse(&window.container).ok_or_else(|| {
                let valid: Vec<&str> = ContainerKind::ALL.iter().map(|k| k.id()).collect();
                Error::Validation(format!(
                    "window `{}` uses unknown container `{}`; valid ids: {}",
                    window.name,
                    window.container,
                    valid.join(", ")
                ))
            })?;
            let mut children = Vec::with_capacity(window.children.len());
            for child in window.children {
                children.push(child.into_element()?);
            }
            windows.push(Window { name: window.name, container, bleed: window.bleed.into_insets(), children });
        }

        let mut seen_huds = BTreeMap::new();
        let mut huds = Vec::with_capacity(self.huds.len());
        for hud in self.huds {
            validate_name(&hud.name, "hud")?;
            if seen_huds.insert(hud.name.clone(), ()).is_some() {
                return Err(Error::Validation(format!("duplicate hud name `{}`", hud.name)));
            }
            let mut children = Vec::with_capacity(hud.children.len());
            for child in hud.children {
                children.push(child.into_element()?);
            }
            huds.push(Hud {
                name: hud.name,
                channel: parse_hud_channel(&hud.channel)?,
                size: Size::new(hud.width, hud.height),
                bleed: hud.bleed.into_insets(),
                shader: hud.shader.map(HudShaderDto::into_shader).transpose()?,
                children,
            });
        }

        Ok(ParsedProject {
            theme,
            windows,
            huds,
            options: BuildOptions { hud_shaders: self.options.hud_shaders },
            target: PackTarget { pack_format: self.target.pack_format },
        })
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

impl InsetsDto {
    fn into_insets(self) -> Insets {
        match self {
            InsetsDto::Uniform(v) => Insets::uniform(v),
            InsetsDto::Edges(InsetsEdgesDto { top, right, bottom, left }) => Insets { top, right, bottom, left },
        }
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

impl ElementDto {
    fn into_element(self) -> Result<Element> {
        self.validate_fields()?;
        match self.kind.as_str() {
            "panel" => Ok(Element::Panel {
                frame: self.required("frame")?,
                pos: self.pos()?,
                size: self.size()?,
                padding: self.padding,
                children: convert_children(self.children)?,
            }),
            "row" => Ok(Element::Row {
                pos: self.pos()?,
                gap: self.gap,
                padding: self.padding,
                align: self.cross_align()?,
                children: convert_children(self.children)?,
            }),
            "column" => Ok(Element::Column {
                pos: self.pos()?,
                gap: self.gap,
                padding: self.padding,
                align: self.cross_align()?,
                children: convert_children(self.children)?,
            }),
            "sprite" => Ok(Element::Sprite { name: self.required("name")?, pos: self.pos()? }),
            "sprite_slot" => Ok(Element::SpriteSlot {
                name: self.required("name")?,
                size: self.size()?,
                pos: self.pos()?,
                align: self.text_align()?,
                sprite: self.sprite.clone(),
            }),
            "button" => Ok(Element::Button {
                name: self.required("name")?,
                frame: self.frame.clone(),
                pos: self.pos()?,
                size: self.optional_size()?,
                slots: self.slots()?,
                pattern: self.pattern()?,
                padding: self.padding,
                default: self.button_default()?,
                tooltip: self.tooltip()?,
                states: self.states()?,
                children: convert_children(self.children)?,
            }),
            "hotspot" => Ok(Element::Hotspot {
                name: self.required("name")?,
                pos: self.pos()?,
                size: self.optional_size()?,
                slots: self.slots()?,
                pattern: self.pattern()?,
                tooltip: self.tooltip()?,
                states: self.states()?,
            }),
            "item" => Ok(Element::Item {
                name: self.required("name")?,
                slots: self.slots()?,
                pattern: self.pattern()?,
                cell_slot: self.cell_slot()?,
            }),
            "collection" => Ok(Element::Collection {
                name: self.required("name")?,
                slots: self.slots()?,
                pattern: self.pattern()?,
                frame: self.frame.clone(),
                action: self.action.unwrap_or(true),
            }),
            "anvil_input" => Ok(Element::AnvilInput {
                name: self.required("name")?,
                initial: self.initial.unwrap_or_default(),
                item_model: self.item_model,
            }),
            "slot_rects" => Ok(Element::SlotRects {
                name: self.required("name")?,
                frame: self.frame.clone(),
                pattern: self.required_pattern()?,
                claim: self.claim()?,
            }),
            "repeater" => Ok(Element::Repeater {
                name: self.required("name")?,
                pattern: self.required_pattern()?,
                frame: self.frame.clone(),
                padding: self.padding,
                children: convert_children(self.children)?,
            }),
            "label" => Ok(Element::Label {
                text: self.required("text")?,
                width: self.width,
                pos: self.pos()?,
                style: self.text_style()?,
            }),
            "slot" => Ok(Element::Slot {
                name: self.required("name")?,
                width: self.width,
                pos: self.pos()?,
                style: self.text_style()?,
            }),
            other => Err(Error::Validation(format!("unknown element type `{other}`"))),
        }
    }

    fn validate_fields(&self) -> Result<()> {
        let allowed = match self.kind.as_str() {
            "panel" => &["type", "frame", "width", "height", "x", "y", "padding", "children"][..],
            "row" | "column" => &["type", "x", "y", "gap", "padding", "align", "children"],
            "sprite" => &["type", "name", "x", "y"],
            "sprite_slot" => &["type", "name", "x", "y", "width", "height", "align", "sprite"],
            "button" => &[
                "type",
                "name",
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
            ],
            "hotspot" => {
                &["type", "name", "width", "height", "x", "y", "slots", "pattern", "transform", "tooltip", "states"]
            }
            "item" => &["type", "name", "slots", "pattern", "transform", "cell_slot"],
            "collection" => &["type", "name", "frame", "slots", "pattern", "transform", "action"],
            "anvil_input" => &["type", "name", "initial", "item_model"],
            "slot_rects" => &["type", "name", "frame", "pattern", "transform", "claim"],
            "repeater" => &["type", "name", "frame", "pattern", "transform", "padding", "children"],
            "label" => &[
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
            ],
            "slot" => &[
                "type",
                "name",
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
            ],
            _ => return Ok(()),
        };
        reject_unexpected_fields(&self.fields, allowed, &format!("{} element", self.kind))
    }

    fn required(&self, field: &str) -> Result<String> {
        match field {
            "name" => self.name.clone(),
            "frame" => self.frame.clone(),
            "text" => self.text.clone(),
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

    fn pos(&self) -> Result<Option<Point>> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Ok(Some(Point::new(x, y))),
            (None, None) => Ok(None),
            _ => Err(Error::Validation(format!("{} element must set both `x` and `y`, or neither", self.kind))),
        }
    }

    fn size(&self) -> Result<Size> {
        Ok(Size::new(self.required_u32("width")?, self.required_u32("height")?))
    }

    fn optional_size(&self) -> Result<Option<Size>> {
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
    fn cell_slot(&self) -> Result<Option<u32>> {
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

    fn cross_align(&self) -> Result<CrossAlign> {
        match self.align.as_str() {
            "" | "start" => Ok(CrossAlign::Start),
            "center" => Ok(CrossAlign::Center),
            "end" => Ok(CrossAlign::End),
            other => Err(Error::Validation(format!("{} element has unknown align `{other}`", self.kind))),
        }
    }

    fn text_style(&self) -> Result<TextStyle> {
        let align = if self.align.is_empty() { None } else { Some(self.text_align()?) };
        let color = match &self.color {
            Some(color) => Rgb::parse_hex(color).ok_or_else(|| {
                Error::Validation(format!("{} element has invalid color `{color}`; expected #rrggbb", self.kind))
            })?,
            None => Rgb::DEFAULT_TEXT,
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
        })
    }

    fn text_align(&self) -> Result<Align> {
        match self.align.as_str() {
            "" | "left" => Ok(Align::Left),
            "center" => Ok(Align::Center),
            "right" => Ok(Align::Right),
            other => Err(Error::Validation(format!("{} element has unknown text align `{other}`", self.kind))),
        }
    }

    fn button_default(&self) -> Result<Option<ButtonDefault>> {
        match self.default.as_deref() {
            None => Ok(None),
            Some("close") => Ok(Some(ButtonDefault::Close)),
            Some(other) => Err(Error::Validation(format!("button element has unknown default `{other}`"))),
        }
    }

    fn slots(&self) -> Result<Option<Vec<InventorySlotRef>>> {
        self.slots.clone().map(parse_slot_refs).transpose()
    }

    fn pattern(&self) -> Result<Option<SlotPattern>> {
        match (&self.pattern, &self.transform) {
            (Some(_), Some(_)) => {
                Err(Error::Validation(format!("{} element must set only one of `pattern` or `transform`", self.kind)))
            }
            (Some(pattern), None) => pattern.clone().into_pattern().map(Some),
            (None, Some(transform)) => transform.clone().into_rect_pattern().map(Some),
            (None, None) => Ok(None),
        }
    }

    fn required_pattern(&self) -> Result<SlotPattern> {
        self.pattern()?.ok_or_else(|| Error::Validation(format!("{} element requires `pattern`", self.kind)))
    }

    fn claim(&self) -> Result<SlotRectClaim> {
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

    fn tooltip(&self) -> Result<Option<ButtonTooltip>> {
        self.tooltip.clone().map(TooltipDto::into_tooltip).transpose()
    }

    fn states(&self) -> Result<BTreeMap<String, ButtonState>> {
        let mut states = BTreeMap::new();
        for (name, state) in &self.states {
            validate_name(name, "button state")?;
            states.insert(name.clone(), state.clone().into_state()?);
        }
        Ok(states)
    }
}

impl SlotPatternDto {
    fn into_pattern(self) -> Result<SlotPattern> {
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
            "grid" => {
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
            other => Err(Error::Validation(format!(
                "slot pattern has unknown kind `{other}`; valid kinds: rect, slots, grid"
            ))),
        }
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
    fn into_rect_pattern(self) -> Result<SlotPattern> {
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

fn parse_slot_refs(slots: Vec<SlotRefDto>) -> Result<Vec<InventorySlotRef>> {
    let mut out = Vec::new();
    for slot in slots {
        slot.append_to(&mut out)?;
    }
    Ok(out)
}

const MAX_SLOT_RANGE_LEN: u64 = 1024;

fn parse_slot_area(value: &str) -> Result<InventorySlotArea> {
    match value {
        "" | "container" => Ok(InventorySlotArea::Container),
        "player" => Ok(InventorySlotArea::Player),
        other => Err(Error::Validation(format!("slot ref has unknown area `{other}`; valid areas: container, player"))),
    }
}

fn required_pattern_u32(value: Option<u32>, field: &str, kind: &str) -> Result<u32> {
    value.ok_or_else(|| Error::Validation(format!("{kind} slot pattern requires `{field}`")))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum TooltipDto {
    Title(String),
    Object(TooltipObjectDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TooltipObjectDto {
    title: String,
    #[serde(default)]
    lines: Vec<String>,
}

impl TooltipDto {
    fn into_tooltip(self) -> Result<ButtonTooltip> {
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
struct ButtonStateDto {
    item_model: Option<String>,
    sprite: Option<String>,
    tooltip: Option<TooltipDto>,
}

impl ButtonStateDto {
    fn into_state(self) -> Result<ButtonState> {
        Ok(ButtonState {
            item_model: self.item_model,
            sprite: self.sprite,
            tooltip: self.tooltip.map(TooltipDto::into_tooltip).transpose()?,
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

fn default_actionbar_source_bottom() -> i32 {
    59
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

fn reject_unexpected_fields(fields: &BTreeSet<String>, allowed: &[&str], context: &str) -> Result<()> {
    for field in fields {
        if !allowed.contains(&field.as_str()) {
            return Err(Error::Validation(format!("{context} does not accept field `{field}`")));
        }
    }
    Ok(())
}

fn validate_name(name: &str, kind: &str) -> Result<()> {
    let mut bytes = name.bytes();
    let valid = bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if valid {
        Ok(())
    } else {
        Err(Error::Validation(format!("{kind} name `{name}` is invalid; must match ^[a-z][a-z0-9_]*$")))
    }
}

fn convert_children(children: Vec<ElementDto>) -> Result<Vec<Element>> {
    children.into_iter().map(ElementDto::into_element).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_project_json() {
        let json = br##"{
          "theme": {
            "frames": {
              "panel": { "texture": "window/sprites/panel.png", "insets": 8 }
            },
            "sprites": {
              "coin": { "texture": "window/sprites/coin.png" }
            }
          },
          "windows": [{
            "name": "shop",
            "container": "generic_9x6",
            "children": [
              { "type": "panel", "frame": "panel", "x": 0, "y": 0, "width": 176, "height": 222,
                "children": [
                  { "type": "slot", "name": "title", "width": 160, "align": "center", "color": "#ffd700" },
                  { "type": "button", "name": "buy", "width": 18, "height": 18,
                    "tooltip": { "title": "Buy", "lines": ["Spend coins"] },
                    "states": {
                      "on": { "item_model": "demo:gui/buy_on", "sprite": "coin", "tooltip": "Ready" },
                      "off": { "item_model": "demo:gui/buy_off" }
                    }
                  },
                  { "type": "hotspot", "name": "info", "width": 18, "height": 18,
                    "tooltip": "Info"
                  },
                  { "type": "sprite", "name": "coin" }
                ]
              }
            ]
          }]
        }"##;

        let project = project_from_json(json).unwrap();
        assert_eq!(
            project.theme.frames["panel"],
            Frame::Texture { texture: "window/sprites/panel.png".into(), insets: Insets::uniform(8) }
        );
        assert_eq!(
            project.theme.sprites["coin"],
            SpriteDef::Texture { texture: "window/sprites/coin.png".into(), size: None }
        );
        assert_eq!(project.windows[0].name, "shop");
        assert_eq!(project.windows[0].children.len(), 1);
        let Element::Panel { children, .. } = &project.windows[0].children[0] else {
            panic!("expected panel");
        };
        let Element::Button { tooltip, states, .. } = &children[1] else {
            panic!("expected button");
        };
        assert_eq!(tooltip.as_ref().unwrap().title, "Buy");
        assert_eq!(states["on"].item_model.as_deref(), Some("demo:gui/buy_on"));
        assert_eq!(states["on"].sprite.as_deref(), Some("coin"));
        assert_eq!(states["on"].tooltip.as_ref().unwrap().title, "Ready");
        let Element::Hotspot { tooltip, .. } = &children[2] else {
            panic!("expected hotspot");
        };
        assert_eq!(tooltip.as_ref().unwrap().title, "Info");
    }

    #[test]
    fn rejects_bad_names() {
        let err = project_from_json(br#"{"windows":[{"name":"Shop","container":"generic_9x3"}]}"#).unwrap_err();
        assert!(err.to_string().contains("window name `Shop` is invalid"));
    }

    #[test]
    fn parses_fixed_sprite_slot() {
        let project = project_from_json(
            br#"{
              "theme": { "sprites": { "coin": { "kind": "badge", "width": 8, "height": 8 } } },
              "windows": [{
                "name": "shop",
                "container": "generic_9x3",
                "children": [{
                  "type": "sprite_slot", "name": "coin_icon",
                  "x": 8, "y": 6, "width": 8, "height": 8, "sprite": "coin"
                }]
              }]
            }"#,
        )
        .unwrap();
        let Element::SpriteSlot { sprite, .. } = &project.windows[0].children[0] else {
            panic!("expected sprite slot");
        };
        assert_eq!(sprite.as_deref(), Some("coin"));
    }

    #[test]
    fn rejects_duplicate_theme_names_across_documents() {
        let err = project_from_json(
            br#"{
              "themes": [
                { "frames": { "panel": { "kind": "panel" } } },
                { "frames": { "panel": { "kind": "button" } } }
              ]
            }"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("duplicate frame name `panel`"));
    }

    #[test]
    fn expands_slot_ranges_in_rust() {
        let json = br#"{
          "windows": [{
            "name": "shop",
            "container": "generic_9x3",
            "children": [{
              "type": "button",
              "name": "buy",
              "width": 18,
              "height": 18,
              "slots": [
                { "area": "container", "first": 3, "last": 1 },
                { "area": "player", "first": 0, "last": 2 }
              ]
            }]
          }]
        }"#;

        let project = project_from_json(json).unwrap();
        let Element::Button { slots: Some(slots), .. } = &project.windows[0].children[0] else {
            panic!("expected button with slots");
        };
        assert_eq!(
            slots.as_slice(),
            &[
                InventorySlotRef::container(3),
                InventorySlotRef::container(2),
                InventorySlotRef::container(1),
                InventorySlotRef::player(0),
                InventorySlotRef::player(1),
                InventorySlotRef::player(2),
            ]
        );
    }

    #[test]
    fn rejects_fractional_slot_range_endpoints() {
        let err = project_from_json(
            br#"{
              "windows": [{
                "name": "shop",
                "container": "generic_9x3",
                "children": [{
                  "type": "button",
                  "name": "buy",
                  "width": 18,
                  "height": 18,
                  "slots": [{ "area": "container", "first": 0, "last": 2.5 }]
                }]
              }]
            }"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid type"), "{err}");
    }

    #[test]
    fn rejects_fields_that_do_not_belong_to_element_kind() {
        let err = project_from_json(
            br#"{
              "windows": [{
                "name": "shop",
                "container": "generic_9x3",
                "children": [{
                  "type": "button",
                  "name": "buy",
                  "width": 18,
                  "height": 18,
                  "claim": "unowned"
                }]
              }]
            }"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("button element does not accept field `claim`"), "{err}");
    }

    #[test]
    fn rejects_unknown_element_fields() {
        let err = project_from_json(
            br#"{
              "windows": [{
                "name": "shop",
                "container": "generic_9x3",
                "children": [{
                  "type": "button",
                  "name": "buy",
                  "width": 18,
                  "height": 18,
                  "tooltp": "Buy"
                }]
              }]
            }"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("button element does not accept field `tooltp`"), "{err}");
    }

    #[test]
    fn rejects_fields_that_do_not_belong_to_pattern_kind() {
        let err = project_from_json(
            br#"{
              "windows": [{
                "name": "shop",
                "container": "generic_9x3",
                "children": [{
                  "type": "item",
                  "name": "entry",
                  "pattern": { "kind": "slots", "slots": [1], "x": 0 }
                }]
              }]
            }"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("slot pattern does not accept field `x`"), "{err}");
    }

    #[test]
    fn rejects_unknown_theme_asset_fields() {
        let err = project_from_json(
            br##"{
              "theme": {
                "frames": {
                  "panel": { "kind": "panel", "widht": 18 }
                }
              }
            }"##,
        )
        .unwrap_err();
        assert!(err.to_string().contains("generated theme asset does not accept field `widht`"), "{err}");
    }

    #[test]
    fn parses_generated_theme_assets() {
        let json = br##"{
          "theme": {
            "frames": {
              "panel": {
                "kind": "panel",
                "fill": "#123456",
                "border_color": "#abcdef",
                "border_width": 3,
                "radius": 5,
                "inset_depth": 2
              }
            },
            "sprites": {
              "badge": {
                "kind": "badge",
                "width": 18,
                "height": 14,
                "accent_color": "#00ffff"
              }
            }
          }
        }"##;

        let project = project_from_json(json).unwrap();
        let Frame::Generated(panel) = &project.theme.frames["panel"] else {
            panic!("expected generated panel");
        };
        assert_eq!(panel.fill, Rgb::new(0x12, 0x34, 0x56));
        assert_eq!(panel.border_color, Rgb::new(0xab, 0xcd, 0xef));
        assert_eq!(panel.border_width, 3);
        assert_eq!(panel.radius, 5);
        let SpriteDef::Generated { size, style } = &project.theme.sprites["badge"] else {
            panic!("expected generated sprite");
        };
        assert_eq!(*size, Size::new(18, 14));
        assert_eq!(style.accent_color, Some(Rgb::new(0x00, 0xff, 0xff)));
    }

    #[test]
    fn parses_hud_project_json() {
        let json = br##"{
          "options": { "hud_shaders": true },
          "target": { "pack_format": 84 },
          "theme": {
            "sprites": {
              "meter": { "texture": "window/sprites/meter.png" }
            }
          },
          "huds": [{
            "name": "status",
            "channel": "actionbar",
            "width": 120,
            "height": 16,
            "shader": {
              "source_bottom": 59,
              "origin": { "x": 0.5, "y": 0.08 },
              "anchor": { "x": 0.5, "y": 0.0 },
              "x": 0,
              "y": 0
            },
            "children": [
              { "type": "sprite", "name": "meter", "x": 0, "y": 0 },
              { "type": "slot", "name": "coins", "x": 12, "y": 4, "width": 80 }
            ]
          }]
        }"##;

        let project = project_from_json(json).unwrap();
        assert!(project.options.hud_shaders);
        assert_eq!(project.target.pack_format, Some(84));
        assert_eq!(project.huds[0].name, "status");
        assert_eq!(project.huds[0].channel, HudChannel::ActionBar);
        assert_eq!(project.huds[0].size, Size::new(120, 16));
        let shader = project.huds[0].shader.unwrap();
        assert_eq!(shader.origin_x, 0.5);
        assert_eq!(shader.origin_y, 0.08);
        assert_eq!(shader.anchor_x, 0.5);
        assert_eq!(shader.anchor_y, 0.0);
        assert_eq!(shader.offset_y, 0);
    }
}
