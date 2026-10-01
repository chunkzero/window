//! Layout solver: the authored model -> absolute GUI-space rects
//! ([`crate::ir::LaidOutWindow`]).
//!
//! Semantics are specified in `docs/AUTHORING.md` (sizing, flow, explicit
//! positioning, validation rules) and the frontend task brief:
//!
//! * The window's children are laid out like a panel's content box covering
//!   the full GUI rect at origin `(0, 0)` with padding 0.
//! * `panel` children without an explicit position overlay at the content
//!   origin. Direct button children without a position are centered in the
//!   padded content rect; explicit children use `content_origin + pos`.
//! * `row`/`column` flow their in-flow children along the main axis separated
//!   by `gap`, aligning on the cross axis per [`CrossAlign`]. A child with an
//!   explicit position is taken out of flow and positioned relative to the
//!   container content origin without consuming flow space.
//! * `sprite` takes its texture's intrinsic size. `label`/`slot` are 8px tall.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::Result;
use crate::authoring::ParsedProject;
use crate::error::Error;
use crate::geometry::{Insets, Point, Rect, Size};
use crate::inventory::{InventorySlotArea, InventorySlotRef, InventorySlotSection, SlotGridPattern, SlotPattern};
use crate::ir::{
    Align, AnvilInputIr, ButtonIr, CollectionIr, Draw, ItemIr, LaidOutHud, LaidOutWindow, RepeatBindingIr, SlotIr,
    SlotRectIr, SpriteSlotIr, TextureKey,
};
use crate::model::{CrossAlign, Element, Frame, Hud, SpriteDef, TextStyle, Window};
use crate::surface::{ContainerKind, Surface};
use crate::vanilla;

/// Solve layout for every window in the project.
///
/// `texture_size` resolves a pack-source-relative texture path to its pixel
/// size (used for intrinsic sprite sizing); returning `None` means the
/// texture is missing, which is a validation error when referenced.
pub fn solve(project: &ParsedProject, texture_size: &dyn Fn(&str) -> Option<Size>) -> Result<Vec<LaidOutWindow>> {
    let mut out = Vec::with_capacity(project.windows.len());
    for window in &project.windows {
        out.push(solve_window(project, window, texture_size)?);
    }
    Ok(out)
}

/// Solve layout for every HUD in the project.
pub fn solve_huds(project: &ParsedProject, texture_size: &dyn Fn(&str) -> Option<Size>) -> Result<Vec<LaidOutHud>> {
    let mut out = Vec::with_capacity(project.huds.len());
    for hud in &project.huds {
        out.push(solve_hud(project, hud, texture_size)?);
    }
    Ok(out)
}

fn solve_window(
    project: &ParsedProject,
    window: &Window,
    texture_size: &dyn Fn(&str) -> Option<Size>,
) -> Result<LaidOutWindow> {
    let surface = Surface::Container(window.container);
    let gui = surface.gui_size();
    let gui_rect = Rect::from_parts(Point::new(0, 0), gui);
    let target = WindowTarget {
        name: &window.name,
        container: window.container,
        bounds: gui_rect,
        visual_bounds: expanded(gui_rect, window.bleed),
        title_origin: surface.title_origin(),
    };
    let mut solver = Solver::new(project, texture_size, target);

    // The window's children are laid out like a panel's content box: the full
    // GUI rect at origin (0, 0), padding 0.
    solver.layout_container_children(&window.children, Point::new(0, 0), Insets::default())?;
    let Solved { draws, slots, sprite_slots, buttons, items, collections, inputs, slot_rects, warnings } =
        solver.finish();

    Ok(LaidOutWindow {
        name: window.name.clone(),
        surface,
        draws,
        slots,
        sprite_slots,
        buttons,
        items,
        collections,
        inputs,
        slot_rects,
        warnings,
    })
}

fn solve_hud(project: &ParsedProject, hud: &Hud, texture_size: &dyn Fn(&str) -> Option<Size>) -> Result<LaidOutHud> {
    let target = HudTarget {
        name: &hud.name,
        bounds: Rect::new(0, 0, hud.size.width, hud.size.height),
        visual_bounds: expanded(Rect::new(0, 0, hud.size.width, hud.size.height), hud.bleed),
    };
    let mut solver = Solver::new(project, texture_size, target);

    solver.layout_container_children(&hud.children, Point::new(0, 0), Insets::default())?;
    let Solved { draws, slots, sprite_slots: _, slot_rects: _, warnings, .. } = solver.finish();

    Ok(LaidOutHud {
        name: hud.name.clone(),
        channel: hud.channel,
        width: hud.size.width,
        height: hud.size.height,
        shader: hud.shader,
        draws,
        slots,
        warnings,
    })
}

fn styled_text_width(text: &str, style: &TextStyle) -> u32 {
    if style.bold { vanilla::bold_text_visible_width(text) } else { vanilla::text_visible_width(text) }
}

fn expanded(rect: Rect, bleed: Insets) -> Rect {
    let x = rect.x - bleed.left as i32;
    let y = rect.y - bleed.top as i32;
    Rect::new(x, y, rect.width + bleed.left + bleed.right, rect.height + bleed.top + bleed.bottom)
}

struct Solved {
    draws: Vec<Draw>,
    slots: Vec<SlotIr>,
    sprite_slots: Vec<SpriteSlotIr>,
    buttons: Vec<ButtonIr>,
    items: Vec<ItemIr>,
    collections: Vec<CollectionIr>,
    inputs: Vec<AnvilInputIr>,
    slot_rects: Vec<SlotRectIr>,
    warnings: Vec<String>,
}

trait LayoutTarget {
    fn name(&self) -> &str;
    fn kind(&self) -> &'static str;
    fn bounds(&self) -> Rect;
    fn visual_bounds(&self) -> Rect;
    fn min_text_y(&self) -> i32;
    fn allows_interaction(&self) -> bool;
    fn allows_sprite_slots(&self) -> bool;
    fn container_kind(&self) -> Option<ContainerKind>;
    fn duplicate_namespace(&self) -> &'static str;

    fn layout_err(&self, message: impl Into<String>) -> Error {
        Error::Layout { window: self.name().to_string(), message: message.into() }
    }

    fn missing_texture_err(&self, texture: &str, referenced_by: &str) -> Error {
        Error::Validation(format!(
            "missing texture `{texture}` referenced by {referenced_by} in {} `{}`",
            self.kind(),
            self.name()
        ))
    }

    fn frame_too_small_err(&self, frame: &str, texture: &str, size: Size, min_w: u32, min_h: u32) -> Error {
        Error::Validation(format!(
            "frame `{frame}` texture `{texture}` is {}x{}, too small for its insets \
             (need at least {min_w}x{min_h}) in {} `{}`",
            size.width,
            size.height,
            self.kind(),
            self.name()
        ))
    }

    fn overflow_warning(&self, rect: Rect, what: &str) -> String {
        let bounds = self.visual_bounds();
        format!(
            "{} `{}`: {what} draw {rect:?} extends outside the {}x{} visual {} rect",
            self.kind(),
            self.name(),
            bounds.width,
            bounds.height,
            self.bounds_label()
        )
    }

    fn outside_err(&self, rect: Rect, what: &str) -> Error {
        let bounds = self.bounds();
        self.layout_err(format!(
            "{what} rect {rect:?} is outside the {}x{} {} rect",
            bounds.width,
            bounds.height,
            self.bounds_label()
        ))
    }

    fn text_limit_err(&self, kind: &str, name: &str, y: i32) -> Error {
        self.layout_err(format!(
            "{kind} `{name}` y={y} is above the minimum {} text y of {} \
             (shifted-font ascent limit: y must be >= {})",
            self.kind(),
            self.min_text_y(),
            self.min_text_y()
        ))
    }

    fn interaction_err(&self, name: &str) -> Error {
        self.layout_err(format!("{} `{}` cannot contain interactive element `{name}`", self.kind(), self.name()))
    }

    fn sprite_slot_err(&self, name: &str) -> Error {
        self.layout_err(format!("{} `{}` cannot contain runtime sprite slot `{name}`", self.kind(), self.name()))
    }

    fn slot_pattern_err(&self, name: &str) -> Error {
        self.layout_err(format!("{} `{}` cannot contain slot pattern element `{name}`", self.kind(), self.name()))
    }

    fn bounds_label(&self) -> &'static str {
        match self.kind() {
            "window" => "GUI",
            "hud" => "HUD",
            _ => "layout",
        }
    }
}

#[derive(Clone, Copy)]
struct WindowTarget<'a> {
    name: &'a str,
    container: ContainerKind,
    bounds: Rect,
    visual_bounds: Rect,
    title_origin: Point,
}

impl LayoutTarget for WindowTarget<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> &'static str {
        "window"
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn visual_bounds(&self) -> Rect {
        self.visual_bounds
    }

    fn min_text_y(&self) -> i32 {
        self.title_origin.y - 1
    }

    fn allows_interaction(&self) -> bool {
        true
    }

    fn allows_sprite_slots(&self) -> bool {
        true
    }

    fn container_kind(&self) -> Option<ContainerKind> {
        Some(self.container)
    }

    fn duplicate_namespace(&self) -> &'static str {
        "slot/sprite/button/label names share one namespace"
    }
}

#[derive(Clone, Copy)]
struct HudTarget<'a> {
    name: &'a str,
    bounds: Rect,
    visual_bounds: Rect,
}

impl LayoutTarget for HudTarget<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> &'static str {
        "hud"
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn visual_bounds(&self) -> Rect {
        self.visual_bounds
    }

    fn min_text_y(&self) -> i32 {
        -1
    }

    fn allows_interaction(&self) -> bool {
        false
    }

    fn allows_sprite_slots(&self) -> bool {
        false
    }

    fn container_kind(&self) -> Option<ContainerKind> {
        None
    }

    fn duplicate_namespace(&self) -> &'static str {
        "slot/label names share one namespace"
    }
}

struct Solver<'a, T> {
    project: &'a ParsedProject,
    texture_size: &'a dyn Fn(&str) -> Option<Size>,
    target: T,
    draws: Vec<Draw>,
    slots: Vec<SlotIr>,
    sprite_slots: Vec<SpriteSlotIr>,
    buttons: Vec<ButtonIr>,
    items: Vec<ItemIr>,
    collections: Vec<CollectionIr>,
    inputs: Vec<AnvilInputIr>,
    slot_rects: Vec<SlotRectIr>,
    warnings: Vec<String>,
    next_label: usize,
    names: HashSet<String>,
    active_repeat: Option<ActiveRepeat>,
}

#[derive(Clone, Debug)]
struct ActiveRepeat {
    group: String,
    index: u32,
    /// The cell's own backing slots, in authoring order. `cell_slot` on a child
    /// item indexes into this list.
    slots: Vec<InventorySlotRef>,
    /// Cell slots already yielded to child item controls, in claim order.
    yielded: Vec<InventorySlotRef>,
}

impl<'a, T: LayoutTarget> Solver<'a, T> {
    fn new(project: &'a ParsedProject, texture_size: &'a dyn Fn(&str) -> Option<Size>, target: T) -> Self {
        Self {
            project,
            texture_size,
            target,
            draws: Vec::new(),
            slots: Vec::new(),
            sprite_slots: Vec::new(),
            buttons: Vec::new(),
            items: Vec::new(),
            collections: Vec::new(),
            inputs: Vec::new(),
            slot_rects: Vec::new(),
            warnings: Vec::new(),
            next_label: 0,
            names: HashSet::new(),
            active_repeat: None,
        }
    }

    fn finish(self) -> Solved {
        Solved {
            draws: self.draws,
            slots: self.slots,
            sprite_slots: self.sprite_slots,
            buttons: self.buttons,
            items: self.items,
            collections: self.collections,
            inputs: self.inputs,
            slot_rects: self.slot_rects,
            warnings: self.warnings,
        }
    }

    fn pos_of(el: &Element) -> Option<Point> {
        match el {
            Element::Panel { pos, .. }
            | Element::Row { pos, .. }
            | Element::Column { pos, .. }
            | Element::Sprite { pos, .. }
            | Element::SpriteSlot { pos, .. }
            | Element::Button { pos, .. }
            | Element::Hotspot { pos, .. }
            | Element::Label { pos, .. }
            | Element::Slot { pos, .. } => *pos,
            Element::Item { .. }
            | Element::Collection { .. }
            | Element::AnvilInput { .. }
            | Element::SlotRects { .. }
            | Element::Repeater { .. } => None,
        }
    }

    /// The size an element occupies, independent of where it is placed.
    fn measure(&self, el: &Element) -> Result<Size> {
        match el {
            Element::Panel { size, .. } => Ok(*size),
            Element::Button { name, size, pattern, .. } | Element::Hotspot { name, size, pattern, .. } => {
                if self.target.allows_interaction() {
                    self.control_size(name, *size, pattern.as_ref())
                } else {
                    Err(self.target.interaction_err(name))
                }
            }
            Element::Sprite { name, .. } => {
                let def = self
                    .project
                    .theme
                    .sprites
                    .get(name)
                    .ok_or_else(|| self.target.layout_err(format!("unknown sprite `{name}`")))?;
                match def {
                    SpriteDef::Texture { texture, size } => size
                        .map(Ok)
                        .unwrap_or_else(|| self.intrinsic_texture_size(texture, &format!("sprite `{name}`"))),
                    SpriteDef::Generated { size, .. } => Ok(*size),
                }
            }
            Element::SpriteSlot { name, size, .. } => {
                if self.target.allows_sprite_slots() {
                    Ok(*size)
                } else {
                    Err(self.target.sprite_slot_err(name))
                }
            }
            Element::Label { text, width, style, .. } => {
                let w = width.unwrap_or_else(|| styled_text_width(text, style));
                Ok(Size::new(w, 8))
            }
            Element::Slot { name, width, .. } => Ok(Size::new(self.required_slot_width(name, *width)?, 8)),
            Element::Item { .. }
            | Element::Collection { .. }
            | Element::AnvilInput { .. }
            | Element::SlotRects { .. } => Ok(Size::new(0, 0)),
            Element::Repeater { name, pattern, .. } => {
                if self.target.allows_interaction() {
                    let cells = self.resolve_pattern_cells(name, pattern)?;
                    Ok(cells
                        .iter()
                        .map(|cell| cell.rect)
                        .reduce(|a, b| a.union(&b))
                        .map_or(Size::new(0, 0), |rect| rect.size()))
                } else {
                    Err(self.target.slot_pattern_err(name))
                }
            }
            Element::Row { gap, padding, children, .. } => self.measure_flow(*gap, *padding, children, Axis::Row),
            Element::Column { gap, padding, children, .. } => self.measure_flow(*gap, *padding, children, Axis::Column),
        }
    }

    fn measure_flow(&self, gap: u32, padding: u32, children: &[Element], axis: Axis) -> Result<Size> {
        let pad = Insets::uniform(padding);
        let mut main = 0u32;
        let mut cross = 0u32;
        let mut count = 0u32;
        for child in children {
            if Self::pos_of(child).is_some() {
                continue;
            }
            let s = self.measure(child)?;
            let (m, c) = axis.split(s);
            main += m;
            cross = cross.max(c);
            count += 1;
        }
        if count > 1 {
            main += gap * (count - 1);
        }
        let content = axis.join(main, cross);
        Ok(Size::new(content.width + pad.left + pad.right, content.height + pad.top + pad.bottom))
    }

    fn intrinsic_texture_size(&self, texture: &str, referenced_by: &str) -> Result<Size> {
        (self.texture_size)(texture).ok_or_else(|| self.target.missing_texture_err(texture, referenced_by))
    }

    /// Lay out children within a content box whose top-left is `content_origin`
    /// and padding `pad`.
    fn layout_container_children(&mut self, children: &[Element], box_origin: Point, pad: Insets) -> Result<()> {
        let content_origin = Point::new(box_origin.x + pad.left as i32, box_origin.y + pad.top as i32);
        for child in children {
            let origin = match Self::pos_of(child) {
                Some(p) => Point::new(content_origin.x + p.x, content_origin.y + p.y),
                None => content_origin,
            };
            self.place(child, origin)?;
        }
        Ok(())
    }

    /// Place `el` with its box top-left at `origin`, emitting IR for it and its subtree.
    fn place(&mut self, el: &Element, origin: Point) -> Result<Size> {
        match el {
            Element::Panel { frame, size, padding, children, .. } => {
                let rect = Rect::from_parts(origin, *size);
                self.emit_frame(frame, rect, &format!("panel frame `{frame}`"))?;
                self.layout_container_children(children, rect.origin(), Insets::uniform(*padding))?;
                Ok(*size)
            }
            Element::Button {
                name, frame, size, slots, pattern, padding, default, tooltip, states, children, ..
            } => {
                if !self.target.allows_interaction() {
                    return Err(self.target.interaction_err(name));
                }
                let resolved =
                    self.resolve_control_rect_and_slots(name, origin, *size, slots.as_ref(), pattern.as_ref())?;
                let rect = resolved.rect;
                if let Some(frame) = frame {
                    self.emit_frame(frame, rect, &format!("button `{name}` frame `{frame}`"))?;
                }
                self.layout_button_children(name, children, rect, *padding)?;
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                self.check_inside_bounds(rect, &format!("button `{name}`"))?;
                self.warn_if_reserved_gutter(rect, &format!("button `{name}`"));
                self.warn_if_control_slot_bounds_mismatch(name, rect, resolved.slots.as_deref());
                self.warn_if_sparse_anvil_control(name, rect, resolved.slots.as_deref());
                self.buttons.push(ButtonIr {
                    name: actual_name,
                    rect,
                    slots: resolved.slots,
                    yielded_slots: Vec::new(),
                    default: *default,
                    action: true,
                    tooltip: tooltip.clone(),
                    states: states.clone(),
                    repeat: self.repeat_binding(name),
                });
                Ok(rect.size())
            }
            Element::Hotspot { name, size, slots, pattern, tooltip, states, .. } => {
                if !self.target.allows_interaction() {
                    return Err(self.target.interaction_err(name));
                }
                let resolved =
                    self.resolve_control_rect_and_slots(name, origin, *size, slots.as_ref(), pattern.as_ref())?;
                let rect = resolved.rect;
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                self.check_inside_bounds(rect, &format!("hotspot `{name}`"))?;
                self.warn_if_reserved_gutter(rect, &format!("hotspot `{name}`"));
                self.warn_if_control_slot_bounds_mismatch(name, rect, resolved.slots.as_deref());
                self.warn_if_sparse_anvil_control(name, rect, resolved.slots.as_deref());
                if tooltip.is_none() && states.is_empty() {
                    return Err(self.target.layout_err(format!("hotspot `{name}` must define `tooltip` or `states`")));
                }
                self.buttons.push(ButtonIr {
                    name: actual_name,
                    rect,
                    slots: resolved.slots,
                    yielded_slots: Vec::new(),
                    default: None,
                    action: false,
                    tooltip: tooltip.clone(),
                    states: states.clone(),
                    repeat: self.repeat_binding(name),
                });
                Ok(rect.size())
            }
            Element::Item { name, slots, pattern, cell_slot } => {
                if !self.target.allows_interaction() {
                    return Err(self.target.interaction_err(name));
                }
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                let slots = match cell_slot {
                    Some(cell_slot) => vec![self.claim_cell_slot(name, *cell_slot)?],
                    None => self.resolve_required_slots(name, slots.as_ref(), pattern.as_ref())?,
                };
                if slots.is_empty() {
                    return Err(self.target.layout_err(format!("item `{name}` must define at least one slot")));
                }
                self.items.push(ItemIr { name: actual_name, slots, repeat: self.repeat_binding(name) });
                Ok(Size::new(0, 0))
            }
            Element::Collection { name, slots, pattern, frame, action } => {
                if !self.target.allows_interaction() {
                    return Err(self.target.interaction_err(name));
                }
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                let slots = self.resolve_required_slots(name, slots.as_ref(), pattern.as_ref())?;
                if slots.is_empty() {
                    return Err(self.target.layout_err(format!("collection `{name}` must define at least one slot")));
                }
                if let Some(frame) = frame {
                    for slot in &slots {
                        let rect = self.slot_rect(name, *slot)?;
                        self.emit_frame(frame, rect, &format!("collection `{name}` frame `{frame}`"))?;
                    }
                }
                self.collections.push(CollectionIr {
                    name: actual_name,
                    slots,
                    action: *action,
                    repeat: self.repeat_binding(name),
                });
                Ok(Size::new(0, 0))
            }
            Element::AnvilInput { name, initial, item_model } => {
                if self.target.container_kind() != Some(ContainerKind::Anvil) {
                    return Err(self.target.layout_err(format!("anvil_input `{name}` requires container `anvil`")));
                }
                if !self.inputs.is_empty() {
                    return Err(self.target.layout_err("anvil surfaces support exactly one anvil_input"));
                }
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                self.inputs.push(AnvilInputIr {
                    name: actual_name,
                    initial: initial.clone(),
                    item_model: item_model.clone(),
                });
                Ok(Size::new(0, 0))
            }
            Element::SlotRects { name, frame, pattern, claim } => {
                if !self.target.allows_interaction() {
                    return Err(self.target.slot_pattern_err(name));
                }
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                let slots = self.resolve_pattern_slots(name, pattern)?;
                if slots.is_empty() {
                    return Err(self.target.layout_err(format!("slot_rects `{name}` resolved to no slots")));
                }
                if let Some(frame) = frame {
                    for slot in &slots {
                        let rect = self.slot_rect(name, *slot)?;
                        self.emit_frame(frame, rect, &format!("slot_rects `{name}` frame `{frame}`"))?;
                    }
                }
                self.slot_rects.push(SlotRectIr { name: actual_name, slots, claim: *claim });
                Ok(Size::new(0, 0))
            }
            Element::Repeater { name, pattern, frame, padding, children } => {
                if !self.target.allows_interaction() {
                    return Err(self.target.slot_pattern_err(name));
                }
                self.register_name(name)?;
                let cells = self.resolve_pattern_cells(name, pattern)?;
                if cells.is_empty() {
                    return Err(self.target.layout_err(format!("repeater `{name}` resolved to no cells")));
                }
                for (index, cell) in cells.iter().enumerate() {
                    let index = index as u32;
                    let cell_name = format!("{name}_{index}");
                    if let Some(frame) = frame {
                        self.emit_frame(frame, cell.rect, &format!("repeater `{name}` frame `{frame}`"))?;
                    }
                    self.register_name(&cell_name)?;
                    self.check_inside_bounds(cell.rect, &format!("repeater `{name}` cell {index}"))?;
                    // The cell button keeps every cell slot as a click route. It
                    // is pushed before its children so document order is stable;
                    // any slot a child item claims is subtracted afterwards.
                    let button_index = self.buttons.len();
                    self.buttons.push(ButtonIr {
                        name: cell_name,
                        rect: cell.rect,
                        slots: Some(cell.slots.clone()),
                        yielded_slots: Vec::new(),
                        default: None,
                        action: true,
                        tooltip: None,
                        states: BTreeMap::new(),
                        repeat: Some(RepeatBindingIr { group: name.clone(), field: None, index }),
                    });
                    let previous = self.active_repeat.replace(ActiveRepeat {
                        group: name.clone(),
                        index,
                        slots: cell.slots.clone(),
                        yielded: Vec::new(),
                    });
                    self.layout_container_children(children, cell.rect.origin(), Insets::uniform(*padding))?;
                    let finished = std::mem::replace(&mut self.active_repeat, previous);
                    if let Some(finished) = finished {
                        self.buttons[button_index].yielded_slots = finished.yielded;
                    }
                }
                Ok(cells
                    .iter()
                    .map(|cell| cell.rect)
                    .reduce(|a, b| a.union(&b))
                    .map_or(Size::new(0, 0), |rect| rect.size()))
            }
            Element::Sprite { name, .. } => {
                let rect = self.emit_sprite(name, origin)?;
                Ok(rect.size())
            }
            Element::SpriteSlot { name, size, align, sprite, .. } => {
                if !self.target.allows_sprite_slots() {
                    return Err(self.target.sprite_slot_err(name));
                }
                let rect = Rect::from_parts(origin, *size);
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                self.check_inside_bounds(rect, &format!("sprite slot `{name}`"))?;
                if let Some(sprite) = sprite {
                    let sprite_size = self.measure(&Element::Sprite { name: sprite.clone(), pos: None })?;
                    if sprite_size.width > size.width || sprite_size.height > size.height {
                        return Err(self.target.layout_err(format!(
                            "fixed sprite `{sprite}` does not fit sprite slot `{name}` ({}x{} in {}x{})",
                            sprite_size.width, sprite_size.height, size.width, size.height
                        )));
                    }
                }
                self.sprite_slots.push(SpriteSlotIr {
                    name: actual_name,
                    rect,
                    align: *align,
                    sprite: sprite.clone(),
                    repeat: self.repeat_binding(name),
                });
                Ok(*size)
            }
            Element::Label { text, width, style, .. } => {
                let w = width.unwrap_or_else(|| styled_text_width(text, style));
                let size = Size::new(w, 8);
                let rect = Rect::from_parts(origin, size);
                let name = self.next_label_name();
                self.register_name(&name)?;
                self.check_text_constraints(rect, &name, false)?;
                self.warn_if_unsupported_static_text(text, &name);
                if width.is_some() {
                    self.warn_if_static_text_overflow(text, style, w, &name);
                }
                self.slots.push(SlotIr {
                    name,
                    text: Some(text.clone()),
                    rect,
                    align: style.align.unwrap_or(Align::Left),
                    color: style.color,
                    shadow: style.shadow,
                    bold: style.bold,
                    italic: style.italic,
                    underlined: style.underlined,
                    strikethrough: style.strikethrough,
                    obfuscated: style.obfuscated,
                    repeat: None,
                });
                Ok(size)
            }
            Element::Slot { name, width, style, .. } => {
                let width = self.required_slot_width(name, *width)?;
                let size = Size::new(width, 8);
                let rect = Rect::from_parts(origin, size);
                let actual_name = self.scoped_name(name);
                self.register_name(&actual_name)?;
                self.check_text_constraints(rect, name, true)?;
                self.slots.push(SlotIr {
                    name: actual_name,
                    text: None,
                    rect,
                    align: style.align.unwrap_or(Align::Left),
                    color: style.color,
                    shadow: style.shadow,
                    bold: style.bold,
                    italic: style.italic,
                    underlined: style.underlined,
                    strikethrough: style.strikethrough,
                    obfuscated: style.obfuscated,
                    repeat: self.repeat_binding(name),
                });
                Ok(size)
            }
            Element::Row { gap, padding, align, children, .. } => {
                self.place_flow(origin, *gap, *padding, *align, children, Axis::Row)
            }
            Element::Column { gap, padding, align, children, .. } => {
                self.place_flow(origin, *gap, *padding, *align, children, Axis::Column)
            }
        }
    }

    fn place_flow(
        &mut self,
        origin: Point,
        gap: u32,
        padding: u32,
        align: CrossAlign,
        children: &[Element],
        axis: Axis,
    ) -> Result<Size> {
        let pad = Insets::uniform(padding);
        let total = self.measure_flow(gap, pad.left, children, axis)?;
        let content_cross = self.content_cross(children, axis)?;
        let content_origin = Point::new(origin.x + pad.left as i32, origin.y + pad.top as i32);

        let mut cursor_main = 0i32;
        let mut first = true;
        for child in children {
            if Self::pos_of(child).is_some() {
                continue;
            }
            let s = self.measure(child)?;
            let (child_main, child_cross) = axis.split(s);
            if !first {
                cursor_main += gap as i32;
            }
            first = false;
            let cross_off = match align {
                CrossAlign::Start => 0,
                CrossAlign::Center => (content_cross as i32 - child_cross as i32) / 2,
                CrossAlign::End => content_cross as i32 - child_cross as i32,
            };
            self.place(child, axis.place(content_origin, cursor_main, cross_off))?;
            cursor_main += child_main as i32;
        }

        for child in children {
            if let Some(p) = Self::pos_of(child) {
                self.place(child, Point::new(content_origin.x + p.x, content_origin.y + p.y))?;
            }
        }

        Ok(total)
    }

    fn layout_button_children(
        &mut self,
        button_name: &str,
        children: &[Element],
        button_rect: Rect,
        padding: u32,
    ) -> Result<()> {
        let inset = padding
            .checked_mul(2)
            .ok_or_else(|| self.target.layout_err(format!("button `{button_name}` padding is too large")))?;
        if inset > button_rect.width || inset > button_rect.height {
            return Err(self.target.layout_err(format!(
                "button `{button_name}` padding {padding} leaves no valid content rect inside {button_rect:?}"
            )));
        }
        let content = Rect::new(
            button_rect.x + padding as i32,
            button_rect.y + padding as i32,
            button_rect.width - inset,
            button_rect.height - inset,
        );

        for authored_child in children {
            let auto_center = Self::pos_of(authored_child).is_none();
            let child = Self::resolve_button_child(authored_child, content.width, auto_center);
            let child_size = self.measure(&child)?;
            let origin = match Self::pos_of(&child) {
                Some(pos) => Point::new(content.x + pos.x, content.y + pos.y),
                None => Point::new(
                    content.x + (content.width as i32 - child_size.width as i32).div_euclid(2),
                    content.y + (content.height as i32 - child_size.height as i32).div_euclid(2),
                ),
            };
            if auto_center && (child_size.width > content.width || child_size.height > content.height) {
                self.warnings.push(format!(
                    "{} `{}`: centered child {}x{} exceeds button `{button_name}` content rect {}x{}",
                    self.target.kind(),
                    self.target.name(),
                    child_size.width,
                    child_size.height,
                    content.width,
                    content.height,
                ));
            }
            self.place(&child, origin)?;
        }
        Ok(())
    }

    fn resolve_button_child(authored: &Element, content_width: u32, auto_center: bool) -> Element {
        let mut child = authored.clone();
        if !auto_center {
            return child;
        }
        match &mut child {
            Element::Label { style, .. } => {
                style.align.get_or_insert(Align::Center);
            }
            Element::Slot { width, style, .. } => {
                width.get_or_insert(content_width);
                style.align.get_or_insert(Align::Center);
            }
            _ => {}
        }
        child
    }

    fn required_slot_width(&self, name: &str, width: Option<u32>) -> Result<u32> {
        width.ok_or_else(|| {
            self.target
                .layout_err(format!("slot `{name}` requires `width` unless it is a direct, unpositioned button child"))
        })
    }

    fn content_cross(&self, children: &[Element], axis: Axis) -> Result<u32> {
        let mut cross = 0u32;
        for child in children {
            if Self::pos_of(child).is_some() {
                continue;
            }
            let s = self.measure(child)?;
            cross = cross.max(axis.split(s).1);
        }
        Ok(cross)
    }

    fn control_size(&self, name: &str, size: Option<Size>, pattern: Option<&SlotPattern>) -> Result<Size> {
        if let Some(size) = size {
            return Ok(size);
        }
        let Some(pattern) = pattern else {
            return Err(self.target.layout_err(format!(
                "control `{name}` requires `width` and `height` unless it uses `pattern` or `transform`"
            )));
        };
        let rect = self.pattern_bounds(name, pattern)?;
        Ok(rect.size())
    }

    fn resolve_control_rect_and_slots(
        &self,
        name: &str,
        origin: Point,
        size: Option<Size>,
        slots: Option<&Vec<InventorySlotRef>>,
        pattern: Option<&SlotPattern>,
    ) -> Result<ResolvedControl> {
        let pattern_slots = pattern.map(|pattern| self.resolve_pattern_slots(name, pattern)).transpose()?;
        let rect = match (size, pattern) {
            (Some(size), _) => Rect::from_parts(origin, size),
            (None, Some(pattern)) => self.pattern_bounds(name, pattern)?,
            (None, None) => {
                return Err(self.target.layout_err(format!(
                    "control `{name}` requires `width` and `height` unless it uses `pattern` or `transform`"
                )));
            }
        };
        let slots = match (slots, pattern_slots) {
            (Some(explicit), Some(_)) => Some(explicit.clone()),
            (Some(explicit), None) => Some(explicit.clone()),
            (None, Some(pattern_slots)) => Some(pattern_slots),
            (None, None) => None,
        };
        Ok(ResolvedControl { rect, slots })
    }

    fn resolve_required_slots(
        &self,
        name: &str,
        slots: Option<&Vec<InventorySlotRef>>,
        pattern: Option<&SlotPattern>,
    ) -> Result<Vec<InventorySlotRef>> {
        match (slots, pattern) {
            (Some(slots), _) => Ok(slots.clone()),
            (None, Some(pattern)) => self.resolve_pattern_slots(name, pattern),
            (None, None) => {
                Err(self.target.layout_err(format!("control `{name}` requires `slots`, `pattern`, or `transform`")))
            }
        }
    }

    fn pattern_bounds(&self, name: &str, pattern: &SlotPattern) -> Result<Rect> {
        let cells = self.resolve_pattern_cells(name, pattern)?;
        cells
            .iter()
            .map(|cell| cell.rect)
            .reduce(|a, b| a.union(&b))
            .ok_or_else(|| self.target.layout_err(format!("pattern for `{name}` resolved to no slots")))
    }

    fn resolve_pattern_slots(&self, name: &str, pattern: &SlotPattern) -> Result<Vec<InventorySlotRef>> {
        let mut slots = Vec::new();
        for cell in self.resolve_pattern_cells(name, pattern)? {
            slots.extend(cell.slots);
        }
        Ok(slots)
    }

    fn resolve_pattern_cells(&self, name: &str, pattern: &SlotPattern) -> Result<Vec<PatternCell>> {
        let kind = self.target.container_kind().ok_or_else(|| self.target.slot_pattern_err(name))?;
        match pattern {
            SlotPattern::Rect(rect) => {
                let slots = kind
                    .section_rect_slots(rect.section, rect.x, rect.y, rect.width, rect.height)
                    .ok_or_else(|| self.pattern_outside_err(name, section_label(rect.section)))?;
                let rect = kind
                    .slot_ref_bounds(&slots)
                    .ok_or_else(|| self.target.layout_err(format!("pattern for `{name}` resolved to no slots")))?;
                Ok(vec![PatternCell { rect, slots }])
            }
            SlotPattern::Slots { section, slots } => {
                let mut out = Vec::with_capacity(slots.len());
                for slot in slots {
                    let slot_ref = kind
                        .section_slot_ref(*section, *slot)
                        .ok_or_else(|| self.pattern_outside_err(name, section_label(*section)))?;
                    let rect = kind
                        .slot_ref_rect(slot_ref)
                        .ok_or_else(|| self.pattern_outside_err(name, section_label(*section)))?;
                    out.push(PatternCell { rect, slots: vec![slot_ref] });
                }
                Ok(out)
            }
            SlotPattern::Grid(grid) => self.resolve_grid_cells(name, kind, grid),
        }
    }

    fn resolve_grid_cells(&self, name: &str, kind: ContainerKind, grid: &SlotGridPattern) -> Result<Vec<PatternCell>> {
        let mut out = Vec::with_capacity((grid.columns * grid.rows) as usize);
        for row in 0..grid.rows {
            for col in 0..grid.columns {
                let x = grid.x + col * grid.cell_width;
                let y = grid.y + row * grid.cell_height;
                let slots = kind
                    .section_rect_slots(grid.section, x, y, grid.cell_width, grid.cell_height)
                    .ok_or_else(|| self.pattern_outside_err(name, section_label(grid.section)))?;
                let rect = kind
                    .slot_ref_bounds(&slots)
                    .ok_or_else(|| self.target.layout_err(format!("grid pattern for `{name}` resolved to no slots")))?;
                out.push(PatternCell { rect, slots });
            }
        }
        Ok(out)
    }

    fn pattern_outside_err(&self, name: &str, section: &'static str) -> Error {
        self.target.layout_err(format!("slot pattern for `{name}` extends outside the visible `{section}` section"))
    }

    fn slot_rect(&self, name: &str, slot: InventorySlotRef) -> Result<Rect> {
        let kind = self.target.container_kind().ok_or_else(|| self.target.slot_pattern_err(name))?;
        kind.slot_ref_rect(slot)
            .ok_or_else(|| self.target.layout_err(format!("slot pattern for `{name}` references a hidden slot")))
    }

    /// Resolves a one-based `cell_slot` against the enclosing repeater cell and
    /// records it as yielded, so the cell button stops filling that slot.
    fn claim_cell_slot(&mut self, name: &str, cell_slot: u32) -> Result<InventorySlotRef> {
        let Some(repeat) = self.active_repeat.as_mut() else {
            return Err(self.target.layout_err(format!(
                "item `{name}` uses `cell_slot` outside a repeater; use `slots`, `pattern`, or \
                 `transform` for an absolute slot"
            )));
        };
        let cell_size = repeat.slots.len();
        let Some(slot) = cell_slot.checked_sub(1).and_then(|zero_based| repeat.slots.get(zero_based as usize)).copied()
        else {
            let group = repeat.group.clone();
            return Err(self.target.layout_err(format!(
                "item `{name}` sets `cell_slot` {cell_slot}, but repeater `{group}` has \
                 {cell_size} slot(s) per cell (valid range 1..={cell_size})"
            )));
        };
        if repeat.yielded.contains(&slot) {
            let group = repeat.group.clone();
            return Err(self.target.layout_err(format!(
                "item `{name}` sets `cell_slot` {cell_slot}, which another item already claims in \
                 repeater `{group}`"
            )));
        }
        repeat.yielded.push(slot);
        Ok(slot)
    }

    fn scoped_name(&self, name: &str) -> String {
        match &self.active_repeat {
            Some(repeat) => format!("{}_{}_{}", repeat.group, name, repeat.index),
            None => name.to_string(),
        }
    }

    fn repeat_binding(&self, field: &str) -> Option<RepeatBindingIr> {
        self.active_repeat.as_ref().map(|repeat| RepeatBindingIr {
            group: repeat.group.clone(),
            field: Some(field.to_string()),
            index: repeat.index,
        })
    }

    fn emit_sprite(&mut self, name: &str, origin: Point) -> Result<Rect> {
        let def = self
            .project
            .theme
            .sprites
            .get(name)
            .ok_or_else(|| self.target.layout_err(format!("unknown sprite `{name}`")))?
            .clone();
        let size = match &def {
            SpriteDef::Texture { texture, size } => {
                size.map(Ok).unwrap_or_else(|| self.intrinsic_texture_size(texture, &format!("sprite `{name}`")))?
            }
            SpriteDef::Generated { size, .. } => *size,
        };
        let rect = Rect::from_parts(origin, size);
        self.warn_if_overflow(rect, &format!("sprite `{name}`"));
        self.draws.push(match def {
            SpriteDef::Texture { texture, .. } => Draw::Sprite { texture: TextureKey(texture), dest: rect },
            SpriteDef::Generated { style, .. } => Draw::Generated { style, dest: rect },
        });
        Ok(rect)
    }

    fn emit_frame(&mut self, frame: &str, dest: Rect, referenced_by: &str) -> Result<()> {
        let def = self
            .project
            .theme
            .frames
            .get(frame)
            .ok_or_else(|| self.target.layout_err(format!("unknown frame `{frame}`")))?
            .clone();
        self.warn_if_overflow(dest, referenced_by);
        match def {
            Frame::Texture { texture, insets } => {
                let tex_size = self.intrinsic_texture_size(&texture, referenced_by)?;
                let min_w = insets.left + insets.right + 1;
                let min_h = insets.top + insets.bottom + 1;
                if tex_size.width < min_w || tex_size.height < min_h {
                    return Err(self.target.frame_too_small_err(frame, &texture, tex_size, min_w, min_h));
                }
                self.draws.push(Draw::NineSlice { texture: TextureKey(texture), insets, dest });
            }
            Frame::Generated(style) => self.draws.push(Draw::Generated { style, dest }),
        }
        Ok(())
    }

    fn warn_if_overflow(&mut self, rect: Rect, what: &str) {
        if !self.target.visual_bounds().contains(&rect) {
            self.warnings.push(self.target.overflow_warning(rect, what));
        }
    }

    fn warn_if_static_text_overflow(&mut self, text: &str, style: &TextStyle, reserved_width: u32, name: &str) {
        let visible_width = styled_text_width(text, style);
        if visible_width > reserved_width {
            self.warnings.push(format!(
                "{} `{}`: label `{name}` text `{text}` is {visible_width}px wide but reserves {reserved_width}px",
                self.target.kind(),
                self.target.name(),
            ));
        }
    }

    fn warn_if_unsupported_static_text(&mut self, text: &str, name: &str) {
        let unsupported: Vec<String> = text
            .chars()
            .filter(|character| vanilla::advance(*character).is_none())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|character| format!("U+{:04X}", character as u32))
            .collect();
        if !unsupported.is_empty() {
            self.warnings.push(format!(
                "{} `{}`: label `{name}` uses unsupported shifted-font glyph(s) {}; use printable ASCII, window.text.small_caps, Window UI markers, or a sprite",
                self.target.kind(),
                self.target.name(),
                unsupported.join(", "),
            ));
        }
    }

    fn warn_if_reserved_gutter(&mut self, rect: Rect, what: &str) {
        let Some(kind) = self.target.container_kind() else {
            return;
        };
        for gutter in kind.reserved_gutters() {
            if rect.intersects(&gutter.rect) {
                self.warnings.push(format!(
                    "window `{}`: {what} rect {rect:?} overlaps reserved {} gutter {:?}; use a section-backed pattern to preserve native spacing",
                    self.target.name(),
                    gutter.name,
                    gutter.rect,
                ));
            }
        }
    }

    fn warn_if_control_slot_bounds_mismatch(&mut self, name: &str, rect: Rect, slots: Option<&[InventorySlotRef]>) {
        let (Some(kind), Some(slots)) = (self.target.container_kind(), slots) else {
            return;
        };
        let Some(slot_bounds) = kind.slot_ref_bounds(slots) else {
            return;
        };
        if rect != slot_bounds {
            self.warnings.push(format!(
                "window `{}`: control `{name}` draws at {rect:?} but its backing slots are bounded by {slot_bounds:?}; clicks follow the backing slots, so omit x/y/width/height to use native bounds or align the artwork exactly",
                self.target.name(),
            ));
        }
    }

    fn warn_if_sparse_anvil_control(&mut self, name: &str, rect: Rect, slots: Option<&[InventorySlotRef]>) {
        if self.target.container_kind() != Some(ContainerKind::Anvil) {
            return;
        }
        let container_slots =
            slots.unwrap_or_default().iter().filter(|slot| slot.area == InventorySlotArea::Container).count();
        if container_slots > 1 && rect.width > 16 {
            self.warnings.push(format!(
                "window `{}`: control `{name}` spans {container_slots} nonuniform anvil container slots in rect {rect:?}; prefer separate single-slot controls",
                self.target.name(),
            ));
        }
    }

    fn check_inside_bounds(&self, rect: Rect, what: &str) -> Result<()> {
        if !self.target.bounds().contains(&rect) {
            return Err(self.target.outside_err(rect, what));
        }
        Ok(())
    }

    fn check_text_constraints(&self, rect: Rect, name: &str, is_slot: bool) -> Result<()> {
        let kind = if is_slot { "slot" } else { "label" };
        self.check_inside_bounds(rect, &format!("{kind} `{name}`"))?;
        if rect.y < self.target.min_text_y() {
            return Err(self.target.text_limit_err(kind, name, rect.y));
        }
        Ok(())
    }

    fn next_label_name(&mut self) -> String {
        let name = format!("label_{}", self.next_label);
        self.next_label += 1;
        name
    }

    fn register_name(&mut self, name: &str) -> Result<()> {
        if !self.names.insert(name.to_string()) {
            return Err(Error::Validation(format!(
                "duplicate name `{name}` in {} `{}` ({})",
                self.target.kind(),
                self.target.name(),
                self.target.duplicate_namespace()
            )));
        }
        Ok(())
    }
}

struct ResolvedControl {
    rect: Rect,
    slots: Option<Vec<InventorySlotRef>>,
}

struct PatternCell {
    rect: Rect,
    slots: Vec<InventorySlotRef>,
}

fn section_label(section: InventorySlotSection) -> &'static str {
    match section {
        InventorySlotSection::Container => "container",
        InventorySlotSection::Player => "player",
        InventorySlotSection::Hotbar => "hotbar",
    }
}

/// Main/cross axis split for row/column flow.
#[derive(Clone, Copy)]
enum Axis {
    Row,
    Column,
}

impl Axis {
    /// Split a size into `(main, cross)`.
    fn split(self, s: Size) -> (u32, u32) {
        match self {
            Axis::Row => (s.width, s.height),
            Axis::Column => (s.height, s.width),
        }
    }

    /// Join `(main, cross)` into a size.
    fn join(self, main: u32, cross: u32) -> Size {
        match self {
            Axis::Row => Size::new(main, cross),
            Axis::Column => Size::new(cross, main),
        }
    }

    /// A child origin given content origin, main offset and cross offset.
    fn place(self, content_origin: Point, main: i32, cross: i32) -> Point {
        match self {
            Axis::Row => Point::new(content_origin.x + main, content_origin.y + cross),
            Axis::Column => Point::new(content_origin.x + cross, content_origin.y + main),
        }
    }
}

#[cfg(test)]
mod tests;
