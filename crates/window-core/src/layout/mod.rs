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
//!
//! [`CrossAlign`]: crate::model::CrossAlign

mod checks;
mod controls;
mod flow;
mod measure;
mod slots;
mod target;
mod visuals;

use std::collections::HashSet;

use crate::Result;
use crate::authoring::ParsedProject;
use crate::geometry::{Insets, Point, Rect, Size};
use crate::inventory::InventorySlotRef;
use crate::ir::{
    AnvilInputIr, ButtonIr, CollectionIr, Draw, ItemIr, LaidOutHud, LaidOutWindow, SlotIr, SlotRectIr, SpriteSlotIr,
};
use crate::model::{Element, Hud, TextStyle, Window};
use crate::surface::Surface;
use crate::vanilla;

use self::controls::ControlSpec;
use self::flow::Axis;
use self::target::{HudTarget, LayoutTarget, WindowTarget};

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

/// The explicit position of `el`, or `None` when it is placed by its parent.
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

    /// Lay out children within a content box whose top-left is `content_origin`
    /// and padding `pad`.
    fn layout_container_children(&mut self, children: &[Element], box_origin: Point, pad: Insets) -> Result<()> {
        let content_origin = Point::new(box_origin.x + pad.left as i32, box_origin.y + pad.top as i32);
        for child in children {
            let origin = match pos_of(child) {
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
                self.place_panel(frame, Rect::from_parts(origin, *size), *padding, children)
            }
            Element::Button {
                name, frame, size, slots, pattern, padding, default, tooltip, states, children, ..
            } => {
                let control = ControlSpec::new(name, *size, slots.as_ref(), pattern.as_ref(), tooltip, states);
                self.place_button(control, frame.as_deref(), *padding, *default, children, origin)
            }
            Element::Hotspot { name, size, slots, pattern, tooltip, states, .. } => {
                let control = ControlSpec::new(name, *size, slots.as_ref(), pattern.as_ref(), tooltip, states);
                self.place_hotspot(control, origin)
            }
            Element::Item { name, slots, pattern, cell_slot } => {
                self.place_item(name, slots.as_ref(), pattern.as_ref(), *cell_slot)
            }
            Element::Collection { name, slots, pattern, frame, action } => {
                self.place_collection(name, slots.as_ref(), pattern.as_ref(), frame.as_deref(), *action)
            }
            Element::AnvilInput { name, initial, item_model } => {
                self.place_anvil_input(name, initial, item_model.as_deref())
            }
            Element::SlotRects { name, frame, pattern, claim } => {
                self.place_slot_rects(name, frame.as_deref(), pattern, *claim)
            }
            Element::Repeater { name, pattern, frame, padding, children } => {
                self.place_repeater(name, pattern, frame.as_deref(), *padding, children)
            }
            Element::Sprite { name, .. } => Ok(self.emit_sprite(name, origin)?.size()),
            Element::SpriteSlot { name, size, align, sprite, .. } => {
                self.place_sprite_slot(name, Rect::from_parts(origin, *size), *align, sprite.as_deref())
            }
            Element::Label { text, width, style, .. } => self.place_label(text, *width, style, origin),
            Element::Slot { name, width, style, .. } => self.place_slot(name, *width, style, origin),
            Element::Row { gap, padding, align, children, .. } => {
                self.place_flow(origin, *gap, *padding, *align, children, Axis::Row)
            }
            Element::Column { gap, padding, align, children, .. } => {
                self.place_flow(origin, *gap, *padding, *align, children, Axis::Column)
            }
        }
    }
}

#[cfg(test)]
mod tests;
