//! Previews: a window or HUD rendered to a PNG from supplied handle state.
//!
//! A preview draws the laid-out surface the way the client composes it: the static composite, the active case of
//! every switch, runtime sprites, and text fitted as the runtime fits it. Inventory items have no art in the
//! compiler, so item slots draw as outlined placeholders. Text in Minecraft's default font is drawn with the bundled
//! small-caps glyphs at the default font's advances.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::authoring::{ParsedProject, Replacement, sprite_key};
use crate::compose::{Composite, Texture, blit_sprite};
use crate::geometry::{Point, Rect, Size};
use crate::inventory::InventorySlotRef;
use crate::ir::{
    Align, AnvilInputIr, CollectionIr, Handle, HandleKind, HandleRole, HandleUse, ItemIr, LaidOutHud, LaidOutWindow,
    Layer, RegionIr, Rgb, SlotIr, SpriteSlotIr, SwitchIr,
};
use crate::model::SpriteDef;
use crate::surface::{ContainerKind, Surface};
use crate::{Error, Result, text_font};

use super::glyphs::{Layers, compose_layers};
use super::metrics::{bitmap_metrics, scale_boundary};
use super::sprites::texture_source_path;
use super::{CompileInput, OutputFile, anvil, decode_textures};

mod text;

#[cfg(test)]
mod tests;

use text::{Shadow, TextPainter};

const MAX_SCALE: u32 = 16;
/// The most pixels a finished preview image may hold, the area of 4096 x 4096.
const MAX_PIXELS: u64 = 4096 * 4096;
const ITEM_OUTLINE: [u8; 4] = [0xff, 0x40, 0xff, 0xc0];
const REGION_OUTLINE: [u8; 4] = [0x60, 0xff, 0x20, 0xc0];
/// Where the client draws the anvil's rename text inside the text field.
const ANVIL_TEXT_INSET: Point = Point::new(3, 4);

/// One preview to render.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewRequest {
    /// The output file stem.
    pub name: String,
    /// The window or HUD to draw.
    pub target: PreviewTarget,
    /// Handle values; a handle without one takes its default.
    #[serde(default)]
    pub state: Vec<StateEntry>,
    /// Integer nearest-neighbour upscale, 1 when omitted.
    #[serde(default)]
    pub scale: Option<u32>,
    /// Whether to outline the active regions, which otherwise draw nothing.
    #[serde(default)]
    pub outlines: bool,
}

/// The surface a preview draws.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum PreviewTarget {
    Window(String),
    Hud(String),
}

/// The value of one handle, or of one member of an indexed handle.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateEntry {
    /// The handle id.
    pub handle: String,
    /// The member of an indexed handle; empty otherwise.
    #[serde(default)]
    pub at: Vec<u32>,
    pub value: StateValue,
}

/// A handle value as authored.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StateValue {
    /// A flag or toggle, or whether an `items` handle holds an item.
    Bool(bool),
    /// Text, a value or selection case, a catalog sprite name, or an input's contents.
    Text(String),
    /// A collection's filled cells and selected cell.
    #[serde(rename_all = "snake_case")]
    Collection {
        /// How many leading cells hold an item; every cell when omitted.
        #[serde(default)]
        count: Option<u32>,
        /// The cell drawing the collection's selected sprite; none when omitted.
        #[serde(default)]
        selected: Option<u32>,
    },
    /// Any other JSON, which no handle takes.
    Other(serde_json::Value),
}

impl std::fmt::Display for StateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::Text(value) => write!(f, "{value:?}"),
            Self::Collection { .. } => f.write_str("a collection state"),
            Self::Other(value) => write!(f, "{value}"),
        }
    }
}

/// Parse the plugin's JSON list of preview requests.
pub fn requests_from_json(bytes: &[u8]) -> Result<Vec<PreviewRequest>> {
    serde_json::from_slice(bytes)
        .map_err(|error| Error::Parse { path: "window previews".into(), message: error.to_string() })
}

/// Render each request to `<name>.png`. The files are not pack output.
///
/// Fails naming the preview when its target is unknown, or when a state entry names a handle the target does not
/// use or gives it a value of the wrong kind.
pub fn render(project: &ParsedProject, input: &CompileInput, requests: &[PreviewRequest]) -> Result<Vec<OutputFile>> {
    if requests.is_empty() {
        return Ok(Vec::new());
    }
    let mut names = BTreeSet::new();
    for request in requests {
        let name = &request.name;
        let valid = !name.is_empty()
            && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-');
        if !valid {
            return Err(Error::Validation(format!("preview name `{name}` must match ^[a-z0-9_-]+$")));
        }
        if !names.insert(name) {
            return Err(Error::Validation(format!("two previews are named `{name}`; give one a `name`")));
        }
        if !(1..=MAX_SCALE).contains(&request.scale.unwrap_or(1)) {
            return Err(fail(name, format!("`scale` must be an integer from 1 to {MAX_SCALE}")));
        }
    }

    let textures = decode_textures(&input.files)?;
    let texture_size = |path: &str| -> Option<Size> { textures.get(path).map(|t| Size::new(t.width, t.height)) };
    let text_fonts = text_font::resolve(&project.fonts, &textures)?;
    let wants = |hud: bool| requests.iter().any(|r| matches!(r.target, PreviewTarget::Hud(_)) == hud);
    let windows = if wants(false) { crate::layout::solve(project, &texture_size, &text_fonts)? } else { Vec::new() };
    let huds = if wants(true) { crate::layout::solve_huds(project, &texture_size, &text_fonts)? } else { Vec::new() };

    let scene = Scene { project, textures: &textures, painter: TextPainter::new(&text_fonts) };
    let mut files = Vec::with_capacity(requests.len());
    for request in requests {
        let preview = &request.name;
        let canvas = match &request.target {
            PreviewTarget::Window(name) => {
                let window = windows
                    .iter()
                    .find(|w| &w.name == name)
                    .ok_or_else(|| fail(preview, format!("there is no window `{name}`")))?;
                let mut composites = compose_layers(&[window], &[], &textures)?;
                let mut layers = composites.windows.remove(0);
                anvil::open_field(window, &mut layers, &mut Vec::new());
                scene.draw(request, &Pane::window(window), &layers)?
            }
            PreviewTarget::Hud(name) => {
                let hud = huds
                    .iter()
                    .find(|h| &h.name == name)
                    .ok_or_else(|| fail(preview, format!("there is no HUD `{name}`")))?;
                let mut composites = compose_layers(&[], &[hud], &textures)?;
                scene.draw(request, &Pane::hud(hud), &composites.huds.remove(0))?
            }
        };
        let scale = request.scale.unwrap_or(1);
        let image = resize(&canvas.texture, canvas.texture.width * scale, canvas.texture.height * scale);
        files.push(OutputFile::binary(format!("{preview}.png"), image.encode_png()?));
    }
    Ok(files)
}

/// Fails when the image of `rect` at `scale` would hold more than [`MAX_PIXELS`], before any buffer exists.
fn check_size(preview: &str, rect: &Rect, scale: u32) -> Result<()> {
    let pixels = u64::from(rect.width)
        .checked_mul(u64::from(scale))
        .zip(u64::from(rect.height).checked_mul(u64::from(scale)))
        .and_then(|(width, height)| width.checked_mul(height));
    if pixels.is_none_or(|pixels| pixels > MAX_PIXELS) {
        let (width, height) = (rect.width, rect.height);
        return Err(fail(
            preview,
            format!("its {width}x{height} image at scale {scale} exceeds the limit of {MAX_PIXELS} pixels"),
        ));
    }
    Ok(())
}

fn fail(preview: &str, message: String) -> Error {
    Error::Validation(format!("preview `{preview}`: {message}"))
}

/// The parts of a laid-out window or HUD a preview draws.
struct Pane<'a> {
    /// `window` or `HUD`, then the name, for error messages.
    label: String,
    /// The surface rect in GUI space.
    rect: Rect,
    /// The shadow of slot text.
    shadow: Shadow,
    container: Option<ContainerKind>,
    slots: &'a [SlotIr],
    sprite_slots: &'a [SpriteSlotIr],
    switches: &'a [SwitchIr],
    layers: &'a [Layer],
    handles: &'a BTreeMap<String, Handle>,
    regions: &'a [RegionIr],
    items: &'a [ItemIr],
    collections: &'a [CollectionIr],
    input: Option<&'a AnvilInputIr>,
}

impl<'a> Pane<'a> {
    fn window(w: &'a LaidOutWindow) -> Self {
        let Surface::Container(kind) = w.surface;
        Self {
            label: format!("window `{}`", w.name),
            rect: Rect::from_parts(Point::new(0, 0), kind.gui_size()),
            shadow: Shadow::Slot,
            container: Some(kind),
            slots: &w.slots,
            sprite_slots: &w.sprite_slots,
            switches: &w.switches,
            layers: &w.layers,
            handles: &w.handles,
            regions: &w.regions,
            items: &w.items,
            collections: &w.collections,
            input: w.inputs.first(),
        }
    }

    fn hud(h: &'a LaidOutHud) -> Self {
        Self {
            label: format!("HUD `{}`", h.name),
            rect: Rect::new(0, 0, h.width, h.height),
            shadow: if h.shader.is_some() { Shadow::ShaderHud } else { Shadow::Slot },
            container: None,
            slots: &h.slots,
            sprite_slots: &[],
            switches: &h.switches,
            layers: &h.layers,
            handles: &h.handles,
            regions: &[],
            items: &[],
            collections: &[],
            input: None,
        }
    }

    /// The GUI rect of a backing inventory slot.
    fn slot_rect(&self, slot: InventorySlotRef) -> Option<Rect> {
        self.container.and_then(|kind| kind.slot_ref_rect(slot))
    }
}

/// The handle one entry binds, with the use binding it.
#[derive(Clone, Copy)]
struct Bound<'a> {
    id: &'a str,
    handle: &'a Handle,
    use_: &'a HandleUse,
}

/// Validated state of one preview: the authored values by handle and member, over the pane's declared handles.
struct State<'a> {
    preview: &'a str,
    values: BTreeMap<(&'a str, &'a [u32]), Cow<'a, StateValue>>,
    /// The handle and index behind each entry a role binds.
    uses: BTreeMap<(HandleRole, &'a str), Bound<'a>>,
}

impl<'a> State<'a> {
    fn new(request: &'a PreviewRequest, pane: &Pane<'a>, project: &ParsedProject) -> Result<Self> {
        let preview = request.name.as_str();
        let mut values = BTreeMap::new();
        for entry in &request.state {
            let id = entry.handle.as_str();
            let handle = pane.handles.get(id).filter(|handle| handle.kind != HandleKind::Builtin).ok_or_else(|| {
                let known: Vec<&str> = pane
                    .handles
                    .iter()
                    .filter(|(_, handle)| !matches!(handle.kind, HandleKind::Builtin | HandleKind::Action))
                    .map(|(id, _)| id.as_str())
                    .collect();
                fail(preview, format!("{} has no handle `{id}`; its handles: {}", pane.label, known.join(", ")))
            })?;
            let member = crate::ir::indexed_entry(id, &entry.at);
            if entry.at.len() != handle.shape.len() || entry.at.iter().zip(&handle.shape).any(|(i, n)| i >= n) {
                let shape = if handle.shape.is_empty() {
                    "is not indexed".to_string()
                } else {
                    format!("has shape {:?}", handle.shape)
                };
                return Err(fail(
                    preview,
                    format!("`{member}` does not name a member of handle `{id}`, which {shape}"),
                ));
            }
            let value = match (handle.kind, &entry.value) {
                (HandleKind::Sprite, StateValue::Text(sprite)) => sprite_key(sprite, "sprite")
                    .map_or(Cow::Borrowed(&entry.value), |key| Cow::Owned(StateValue::Text(key))),
                (_, value) => Cow::Borrowed(value),
            };
            let expected = match (handle.kind, &*value) {
                (HandleKind::Flag | HandleKind::Toggle | HandleKind::Items, StateValue::Bool(_))
                | (HandleKind::Text | HandleKind::Input, StateValue::Text(_))
                | (HandleKind::Collection, StateValue::Collection { .. }) => None,
                (HandleKind::Value | HandleKind::Selection, StateValue::Text(value))
                    if handle.values.contains(value) =>
                {
                    None
                }
                (HandleKind::Sprite, StateValue::Text(sprite))
                    if project.art.runtime.contains(sprite)
                        && (handle.only.is_empty() || handle.only.contains(sprite)) =>
                {
                    None
                }
                (HandleKind::Flag | HandleKind::Toggle | HandleKind::Items, _) => Some("true or false".to_string()),
                (HandleKind::Text | HandleKind::Input, _) => Some("a string".to_string()),
                (HandleKind::Value | HandleKind::Selection, _) => Some(format!("one of {}", handle.values.join(", "))),
                (HandleKind::Sprite, _) if handle.only.is_empty() => Some("a catalog sprite name".to_string()),
                (HandleKind::Sprite, _) => Some(format!("one of {}", handle.only.join(", "))),
                (HandleKind::Collection, _) => Some("`{ count, selected }`".to_string()),
                (HandleKind::Action | HandleKind::Builtin, _) => Some("no state".to_string()),
            };
            if let Some(expected) = expected {
                let kind = handle.kind.id();
                let got = &entry.value;
                return Err(fail(preview, format!("handle `{member}` is a {kind} and takes {expected}; got {got}")));
            }
            if values.insert((id, entry.at.as_slice()), value).is_some() {
                return Err(fail(preview, format!("handle `{member}` is set twice")));
            }
        }
        let mut uses = BTreeMap::new();
        for (id, handle) in pane.handles {
            for use_ in &handle.uses {
                uses.insert((use_.role, use_.entry.as_str()), Bound { id, handle, use_ });
            }
        }
        Ok(Self { preview, values, uses })
    }

    fn get(&self, role: HandleRole, entry: &str) -> Option<&StateValue> {
        let bound = self.uses.get(&(role, entry))?;
        self.values.get(&(bound.id, bound.use_.at.as_slice())).map(|value| &**value)
    }

    /// The string set for a role's entry, if any.
    fn text(&self, role: HandleRole, entry: &str) -> Option<&str> {
        match self.get(role, entry) {
            Some(StateValue::Text(text)) => Some(text),
            _ => None,
        }
    }

    /// The case value the switch `name` shows: its handle's value, or whether that value is the one it compares with.
    fn case(&self, name: &str) -> Option<Cow<'_, str>> {
        let Bound { id, handle, use_ } = *self.uses.get(&(HandleRole::Switch, name))?;
        let value: Cow<'_, str> = match self.values.get(&(id, use_.at.as_slice())).map(|value| &**value) {
            Some(StateValue::Bool(value)) => value.to_string().into(),
            Some(StateValue::Text(value)) => value.as_str().into(),
            Some(StateValue::Collection { .. } | StateValue::Other(_)) => return None,
            None => match (&handle.initial, handle.kind) {
                (Some(initial), _) => initial.as_str().into(),
                (None, HandleKind::Value | HandleKind::Selection) => handle.values.first()?.as_str().into(),
                (None, _) => "false".into(),
            },
        };
        Some(match use_.value.as_deref() {
            Some(compared) => (value == compared).to_string().into(),
            None => value,
        })
    }

    /// The filled cell count and selected cell of `collection`.
    fn collection(&self, collection: &CollectionIr) -> Result<(usize, Option<usize>)> {
        let cells = collection.slots.len();
        let Some(StateValue::Collection { count, selected }) = self.get(HandleRole::Collection, &collection.name)
        else {
            return Ok((cells, None));
        };
        let name = &collection.name;
        if let Some(over) = count.filter(|n| *n as usize > cells).or(selected.filter(|n| *n as usize >= cells)) {
            return Err(fail(self.preview, format!("collection `{name}` has {cells} cells; got {over}")));
        }
        if selected.is_some() && collection.selected_sprite.is_none() {
            return Err(fail(self.preview, format!("collection `{name}` draws no selected cell")));
        }
        Ok((count.map_or(cells, |n| n as usize), selected.map(|n| n as usize)))
    }
}

/// What a name inside a switch case names. One handle can name both a switch and the region toggling it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Member {
    Slot,
    SpriteSlot,
    Region,
    Item,
    Collection,
    Switch,
}

/// Everything inside a switch case that is not showing.
#[derive(Default)]
struct Hidden<'a> {
    members: BTreeSet<(Member, &'a str)>,
}

impl<'a> Hidden<'a> {
    /// `active[i]` is the case index switch `i` shows, if any.
    fn new(switches: &'a [SwitchIr], active: &[Option<usize>]) -> Self {
        let mut hidden = Self::default();
        for (switch, shown) in switches.iter().zip(active) {
            for (i, case) in switch.cases.iter().enumerate() {
                if *shown != Some(i) {
                    hidden.hide_case(switches, case);
                }
            }
        }
        hidden
    }

    fn hide_case(&mut self, switches: &'a [SwitchIr], case: &'a crate::ir::SwitchCaseIr) {
        let members = [
            (Member::Slot, &case.slots),
            (Member::SpriteSlot, &case.sprite_slots),
            (Member::Region, &case.regions),
            (Member::Item, &case.items),
            (Member::Collection, &case.collections),
            (Member::Switch, &case.switches),
        ];
        for (member, names) in members {
            self.members.extend(names.iter().map(|name| (member, name.as_str())));
        }
        for nested in switches.iter().filter(|switch| case.switches.contains(&switch.name)) {
            for case in &nested.cases {
                self.hide_case(switches, case);
            }
        }
    }

    fn shows(&self, member: Member, name: &str) -> bool {
        !self.members.contains(&(member, name))
    }
}

/// A drawing surface covering a GUI-space rect.
struct Canvas {
    texture: Texture,
    origin: Point,
}

impl Canvas {
    fn new(rect: Rect) -> Self {
        Self { texture: Texture::transparent(rect.width, rect.height), origin: rect.origin() }
    }

    /// Draws `texture` with its top-left at GUI point `at`.
    fn blit(&mut self, texture: &Texture, at: Point) {
        let dest = Rect::new(at.x - self.origin.x, at.y - self.origin.y, texture.width, texture.height);
        blit_sprite(&mut self.texture, texture, &dest);
    }

    fn blend(&mut self, x: i32, y: i32, rgba: [u8; 4]) {
        let (x, y) = (x - self.origin.x, y - self.origin.y);
        if x >= 0 && y >= 0 {
            self.texture.blend(x as u32, y as u32, rgba);
        }
    }

    /// Draws the 1px border of `rect`.
    fn outline(&mut self, rect: &Rect, rgba: [u8; 4]) {
        for x in rect.x..rect.right() {
            self.blend(x, rect.y, rgba);
            self.blend(x, rect.bottom() - 1, rgba);
        }
        for y in rect.y + 1..rect.bottom() - 1 {
            self.blend(rect.x, y, rgba);
            self.blend(rect.right() - 1, y, rgba);
        }
    }
}

/// What every preview of a build draws with.
struct Scene<'a> {
    project: &'a ParsedProject,
    textures: &'a BTreeMap<String, Texture>,
    painter: TextPainter<'a>,
}

impl Scene<'_> {
    fn draw(&self, request: &PreviewRequest, pane: &Pane<'_>, layers: &Layers) -> Result<Canvas> {
        let state = State::new(request, pane, self.project)?;
        let active: Vec<Option<usize>> = pane
            .switches
            .iter()
            .map(|switch| {
                let value = state.case(&switch.name)?;
                switch.cases.iter().position(|case| case.value == value)
            })
            .collect();
        let hidden = Hidden::new(pane.switches, &active);

        let composites = std::iter::once(&layers.base).chain(layers.cases.iter().flatten());
        let rect = composites.filter(|comp| comp.has_content).fold(pane.rect, |rect, comp| rect.union(&comp.bounds));
        check_size(&request.name, &rect, request.scale.unwrap_or(1))?;
        let mut canvas = Canvas::new(rect);
        let field = ContainerKind::ANVIL_TEXT_FIELD;
        if pane.container == Some(ContainerKind::Anvil)
            && let Replacement::Sprite(sprite) = &self.project.options.vanilla.anvil.field
        {
            canvas.blit(&self.sprite(sprite)?.0, field.origin());
        }
        blit_composite(&mut canvas, &layers.base);

        for layer in pane.layers {
            match layer {
                Layer::Slot(name) if hidden.shows(Member::Slot, name) => {
                    let slot = find(pane.slots, name, |slot| &slot.name)?;
                    let content = slot.text.as_deref().or_else(|| state.text(HandleRole::Slot, name));
                    self.painter.draw_slot(&mut canvas, slot, content.unwrap_or_default(), pane.shadow)?;
                }
                Layer::SpriteSlot(name) if hidden.shows(Member::SpriteSlot, name) => {
                    if let Some(sprite) = state.text(HandleRole::SpriteSlot, name) {
                        let slot = find(pane.sprite_slots, name, |slot| &slot.name)?;
                        self.draw_sprite(&mut canvas, sprite, &slot.rect, slot.align)?;
                    }
                }
                Layer::Switch(name) if hidden.shows(Member::Switch, name) => {
                    let index = pane.switches.iter().position(|switch| &switch.name == name);
                    if let Some(index) = index
                        && let Some(case) = active[index]
                    {
                        blit_composite(&mut canvas, &layers.cases[index][case]);
                    }
                }
                Layer::Collection(name) if hidden.shows(Member::Collection, name) => {
                    let collection = find(pane.collections, name, |collection| &collection.name)?;
                    if let (Some(sprite), (_, Some(selected))) =
                        (&collection.selected_sprite, state.collection(collection)?)
                        && let Some(cell) = pane.slot_rect(collection.slots[selected])
                    {
                        self.draw_sprite(&mut canvas, sprite, &cell.slot_box(), Align::Left)?;
                    }
                }
                _ => {}
            }
        }

        for item in pane.items.iter().filter(|item| hidden.shows(Member::Item, &item.name)) {
            if state.get(HandleRole::Item, &item.name) != Some(&StateValue::Bool(false)) {
                self.placeholders(&mut canvas, pane, &item.slots);
            }
        }
        for collection in
            pane.collections.iter().filter(|collection| hidden.shows(Member::Collection, &collection.name))
        {
            let (count, _) = state.collection(collection)?;
            self.placeholders(&mut canvas, pane, &collection.slots[..count]);
        }
        if let Some(input) = pane.input {
            self.placeholders(&mut canvas, pane, &[InventorySlotRef::container(0)]);
            let content = state.text(HandleRole::Input, &input.name).unwrap_or(&input.initial);
            let at = Point::new(field.x + ANVIL_TEXT_INSET.x, field.y + ANVIL_TEXT_INSET.y);
            self.painter.draw_line(&mut canvas, None, content, at, Rgb::new(0xff, 0xff, 0xff), Shadow::Vanilla, false);
        }
        if request.outlines {
            for region in pane.regions.iter().filter(|region| hidden.shows(Member::Region, &region.name)) {
                canvas.outline(&region.rect, REGION_OUTLINE);
            }
        }
        Ok(canvas)
    }

    fn placeholders(&self, canvas: &mut Canvas, pane: &Pane<'_>, slots: &[InventorySlotRef]) {
        for rect in slots.iter().filter_map(|slot| pane.slot_rect(*slot)) {
            canvas.outline(&rect, ITEM_OUTLINE);
        }
    }

    /// Draws catalog sprite `name` in `rect` as the runtime places it: its ink aligned within the rect's width.
    fn draw_sprite(&self, canvas: &mut Canvas, name: &str, rect: &Rect, align: Align) -> Result<()> {
        let (texture, size) = self.sprite(name)?;
        let metrics = bitmap_metrics(&texture, size);
        let width = scale_boundary(texture.width, texture.height, size.height);
        let texture = resize(&texture, width, size.height);
        let x = text::origin(align, rect.x, rect.width, metrics.glyph_width) - metrics.x_offset as i32;
        canvas.blit(&texture, Point::new(x, rect.y));
        Ok(())
    }

    /// The pixels and rendered size of sprite `name`.
    fn sprite(&self, name: &str) -> Result<(Cow<'_, Texture>, Size)> {
        let unknown =
            || Error::Validation(format!("previews cannot draw sprite `{name}`: its texture is not provided"));
        match self.project.art.sprites.get(name).ok_or_else(unknown)? {
            SpriteDef::Texture { texture, size } => {
                let texture = self.textures.get(&texture_source_path(texture)).ok_or_else(unknown)?;
                Ok((Cow::Borrowed(texture), size.unwrap_or(Size::new(texture.width, texture.height))))
            }
            SpriteDef::Generated { size, style } => Ok((Cow::Owned(crate::raster::render(style, *size)?), *size)),
        }
    }
}

fn find<'a, T>(items: &'a [T], name: &str, key: impl Fn(&T) -> &String) -> Result<&'a T> {
    items
        .iter()
        .find(|item| key(item) == name)
        .ok_or_else(|| Error::Validation(format!("layer `{name}` has no laid-out entry")))
}

fn blit_composite(canvas: &mut Canvas, comp: &Composite) {
    if comp.has_content {
        canvas.blit(&comp.texture, comp.bounds.origin());
    }
}

/// `texture` resampled to `width` x `height` with nearest-neighbour sampling.
fn resize(texture: &Texture, width: u32, height: u32) -> Cow<'_, Texture> {
    if (texture.width, texture.height) == (width, height) || texture.width == 0 || texture.height == 0 {
        return Cow::Borrowed(texture);
    }
    let mut out = Texture::transparent(width, height);
    for y in 0..height {
        let source_y = (u64::from(y) * u64::from(texture.height) / u64::from(height)) as u32;
        for x in 0..width {
            let source_x = (u64::from(x) * u64::from(texture.width) / u64::from(width)) as u32;
            let i = ((y * width + x) * 4) as usize;
            out.rgba[i..i + 4].copy_from_slice(&texture.get(source_x, source_y));
        }
    }
    Cow::Owned(out)
}
