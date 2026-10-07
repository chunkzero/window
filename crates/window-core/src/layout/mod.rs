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
//! * `sprite` takes its texture's intrinsic size. `label`/`slot` are 8px tall, or a multi-line
//!   slot `(lines - 1) * line_height + 8`.
//!
//! [`CrossAlign`]: crate::model::CrossAlign

mod checks;
mod controls;
mod flex;
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
    SwitchIr,
};
use crate::model::{Element, Hud, TextStyle, Window};
use crate::surface::Surface;
use crate::text_font::{TextFont, TextFonts};
use crate::vanilla;

use self::controls::ControlSpec;
use self::flow::Axis;
use self::target::{HudTarget, LayoutTarget, WindowTarget};

/// Solve layout for every window in the project.
///
/// `texture_size` resolves a pack-source-relative texture path to its pixel
/// size (used for intrinsic sprite sizing); returning `None` means the
/// texture is missing, which is a validation error when referenced. `fonts`
/// measures text drawn with a text font.
pub fn solve(
    project: &ParsedProject,
    texture_size: &dyn Fn(&str) -> Option<Size>,
    fonts: &TextFonts,
) -> Result<Vec<LaidOutWindow>> {
    let mut out = Vec::with_capacity(project.windows.len());
    for window in &project.windows {
        out.push(solve_window(project, window, texture_size, fonts)?);
    }
    Ok(out)
}

/// Solve layout for every HUD in the project.
pub fn solve_huds(
    project: &ParsedProject,
    texture_size: &dyn Fn(&str) -> Option<Size>,
    fonts: &TextFonts,
) -> Result<Vec<LaidOutHud>> {
    let mut out = Vec::with_capacity(project.huds.len());
    for hud in &project.huds {
        out.push(solve_hud(project, hud, texture_size, fonts)?);
    }
    Ok(out)
}

fn solve_window(
    project: &ParsedProject,
    window: &Window,
    texture_size: &dyn Fn(&str) -> Option<Size>,
    fonts: &TextFonts,
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
    let mut solver = Solver::new(project, texture_size, fonts, target);

    if let Some(frame) = &window.frame {
        solver.emit_frame(frame, expanded(gui_rect, window.bleed), &format!("window frame `{frame}`"))?;
    }
    // The window's children are laid out like a panel's content box: the full
    // GUI rect at origin (0, 0), padding 0.
    solver.layout_container_children(&window.children, gui_rect, Insets::default())?;
    let Solved { draws, slots, sprite_slots, buttons, items, collections, inputs, slot_rects, switches, warnings } =
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
        switches,
        indexed: window.indexed.clone(),
        warnings,
    })
}

fn solve_hud(
    project: &ParsedProject,
    hud: &Hud,
    texture_size: &dyn Fn(&str) -> Option<Size>,
    fonts: &TextFonts,
) -> Result<LaidOutHud> {
    let target = |size: Size| HudTarget {
        name: &hud.name,
        bounds: Rect::from_parts(Point::new(0, 0), size),
        visual_bounds: expanded(Rect::from_parts(Point::new(0, 0), size), hud.bleed),
    };
    let mut solver = Solver::new(project, texture_size, fonts, target(hud.size.unwrap_or_default()));
    let size = match hud.size {
        Some(size) => size,
        None => solver.children_extent(&hud.children)?,
    };
    solver.target = target(size);
    let rect = Rect::from_parts(Point::new(0, 0), size);
    if let Some(frame) = &hud.frame {
        solver.emit_frame(frame, expanded(rect, hud.bleed), &format!("hud frame `{frame}`"))?;
    }
    solver.layout_container_children(&hud.children, rect, Insets::default())?;
    let Solved { draws, slots, switches, warnings, .. } = solver.finish();

    Ok(LaidOutHud {
        name: hud.name.clone(),
        channel: hud.channel,
        width: size.width,
        height: size.height,
        shader: hud.shader,
        draws,
        slots,
        switches,
        indexed: hud.indexed.clone(),
        warnings,
    })
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
        Element::Flex(node) => node.pos,
        Element::Switch(switch) => switch.pos,
        Element::Item { .. }
        | Element::Collection { .. }
        | Element::AnvilInput { .. }
        | Element::SlotRects { .. }
        | Element::Repeater { .. }
        | Element::Section(_) => None,
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
    switches: Vec<SwitchIr>,
    warnings: Vec<String>,
}

struct Solver<'a, T> {
    project: &'a ParsedProject,
    texture_size: &'a dyn Fn(&str) -> Option<Size>,
    fonts: &'a TextFonts,
    target: T,
    draws: Vec<Draw>,
    slots: Vec<SlotIr>,
    sprite_slots: Vec<SpriteSlotIr>,
    buttons: Vec<ButtonIr>,
    items: Vec<ItemIr>,
    collections: Vec<CollectionIr>,
    inputs: Vec<AnvilInputIr>,
    slot_rects: Vec<SlotRectIr>,
    switches: Vec<SwitchIr>,
    warnings: Vec<String>,
    next_label: usize,
    names: HashSet<String>,
    active_repeat: Option<ActiveRepeat>,
    active_case: Option<ActiveCase>,
}

/// The switch case being emitted and the bindings its switch shares across cases.
#[derive(Clone, Debug)]
struct ActiveCase {
    value: String,
    shared: HashSet<String>,
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
    fn new(
        project: &'a ParsedProject,
        texture_size: &'a dyn Fn(&str) -> Option<Size>,
        fonts: &'a TextFonts,
        target: T,
    ) -> Self {
        Self {
            project,
            texture_size,
            fonts,
            target,
            draws: Vec::new(),
            slots: Vec::new(),
            sprite_slots: Vec::new(),
            buttons: Vec::new(),
            items: Vec::new(),
            collections: Vec::new(),
            inputs: Vec::new(),
            slot_rects: Vec::new(),
            switches: Vec::new(),
            warnings: Vec::new(),
            next_label: 0,
            names: HashSet::new(),
            active_repeat: None,
            active_case: None,
        }
    }

    /// The text font `style` draws with, or `None` for vanilla glyphs.
    fn text_font(&self, style: &TextStyle) -> Result<Option<&'a TextFont>> {
        let Some(name) = style.font.as_deref() else {
            return Ok(None);
        };
        self.fonts.get(name).map(Some).ok_or_else(|| self.target.layout_err(format!("unknown font `{name}`")))
    }

    /// Visible ink width of `text` drawn with `style`.
    fn text_width(&self, text: &str, style: &TextStyle) -> Result<u32> {
        Ok(match self.text_font(style)? {
            Some(font) => vanilla::visible_width(text, style.bold, |c| font.advance(c), |c| font.glyph_width(c)),
            None => vanilla::visible_width(text, style.bold, vanilla::advance, vanilla::glyph_width),
        })
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
            switches: self.switches,
            warnings: self.warnings,
        }
    }

    /// Lay out children within `outer` shrunk by padding `pad`.
    fn layout_container_children(&mut self, children: &[Element], outer: Rect, pad: Insets) -> Result<()> {
        let content = Rect::new(
            outer.x + pad.left as i32,
            outer.y + pad.top as i32,
            outer.width.saturating_sub(pad.left + pad.right),
            outer.height.saturating_sub(pad.top + pad.bottom),
        );
        for child in children {
            self.place_in_box(child, content)?;
        }
        Ok(())
    }

    /// The bottom-right extent of `children` placed from the origin.
    fn children_extent(&self, children: &[Element]) -> Result<Size> {
        let mut extent = Size::new(0, 0);
        for child in children {
            let pos = pos_of(child).unwrap_or_default();
            let size = self.measure(child)?;
            extent.width = extent.width.max((pos.x + size.width as i32).max(0) as u32);
            extent.height = extent.height.max((pos.y + size.height as i32).max(0) as u32);
        }
        Ok(extent)
    }

    /// Place `el` with its box top-left at `origin`, emitting IR for it and its subtree.
    fn place(&mut self, el: &Element, origin: Point) -> Result<Size> {
        match el {
            Element::Panel { frame, size, padding, children, .. } => {
                self.place_panel(frame, Rect::from_parts(origin, *size), *padding, children)
            }
            Element::Button {
                name,
                frame,
                size,
                slots,
                pattern,
                padding,
                default,
                tooltip,
                states,
                hover_outline,
                children,
                ..
            } => {
                let control =
                    ControlSpec::new(name, *size, slots.as_ref(), pattern.as_ref(), tooltip, states, *hover_outline);
                self.place_button(control, frame.as_deref(), *padding, *default, children, origin)
            }
            Element::Hotspot { name, size, slots, pattern, tooltip, states, hover_outline, .. } => {
                let control =
                    ControlSpec::new(name, *size, slots.as_ref(), pattern.as_ref(), tooltip, states, *hover_outline);
                self.place_hotspot(control, origin)
            }
            Element::Item { name, slots, pattern, cell_slot } => {
                self.place_item(name, slots.as_ref(), pattern.as_ref(), *cell_slot)
            }
            Element::Collection { name, slots, pattern, frame, selected_sprite, action } => self.place_collection(
                name,
                slots.as_ref(),
                pattern.as_ref(),
                frame.as_deref(),
                selected_sprite.clone(),
                *action,
            ),
            Element::AnvilInput { name, initial, item_model } => {
                self.place_anvil_input(name, initial, item_model.as_deref())
            }
            Element::SlotRects { name, frame, pattern, claim } => {
                self.place_slot_rects(name, frame.as_deref(), pattern, *claim)
            }
            Element::Repeater { name, pattern, frame, padding, hover_outline, children } => {
                self.place_repeater(name, pattern, frame.as_deref(), *padding, *hover_outline, children)
            }
            Element::Sprite { name, .. } => Ok(self.emit_sprite(name, origin)?.size()),
            Element::SpriteSlot { name, size, align, sprite, .. } => {
                self.place_sprite_slot(name, Rect::from_parts(origin, *size), *align, sprite.as_deref())
            }
            Element::Label { text, width, style, .. } => self.place_label(text, *width, style, origin),
            Element::Slot { name, width, style, fit, .. } => self.place_slot(name, *width, style, *fit, origin),
            Element::Row { gap, padding, align, children, .. } => {
                self.place_flow(origin, *gap, *padding, *align, children, Axis::Row)
            }
            Element::Column { gap, padding, align, children, .. } => {
                self.place_flow(origin, *gap, *padding, *align, children, Axis::Column)
            }
            Element::Flex(node) => self.place_flex(node, origin, None),
            Element::Section(section) => self.place_section(section),
            Element::Switch(switch) => self.place_switch(switch, origin, None),
        }
    }
}

#[cfg(test)]
mod tests;
