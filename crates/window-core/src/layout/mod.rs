//! Layout solver: the authored model -> absolute GUI-space rects
//! ([`crate::ir::LaidOutWindow`]).
//!
//! Semantics are specified in `docs/AUTHORING.md`:
//!
//! * The window's and HUD's children are placed in the full GUI or HUD rect at
//!   origin `(0, 0)`: an auto-positioned box fills it, and anything else sits at
//!   its explicit position or the origin.
//! * Boxes, switches, and sections are solved with taffy; see [`flex`].
//! * `sprite` takes its art's intrinsic size. `label`/`slot` are 8px tall, or a multi-line
//!   slot `(lines - 1) * line_height + 8`.

mod checks;
mod controls;
mod flex;
mod measure;
mod slots;
mod target;
mod visuals;

use std::collections::HashSet;

use crate::Result;
use crate::authoring::ParsedProject;
use crate::geometry::{Insets, Point, Rect, Size};
use crate::ir::{
    AnvilInputIr, CollectionIr, Draw, ItemIr, LaidOutHud, LaidOutWindow, Layer, RegionIr, SlotIr, SpriteSlotIr,
    SwitchIr,
};
use crate::model::{Element, Hud, TextStyle, Window};
use crate::surface::Surface;
use crate::text_font::{TextFont, TextFonts};
use crate::vanilla;

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
        out.push(
            solve_window(project, window, texture_size, fonts)
                .map_err(|error| error.with_debug_name(window.debug_name.as_deref()))?,
        );
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
        out.push(
            solve_hud(project, hud, texture_size, fonts)
                .map_err(|error| error.with_debug_name(hud.debug_name.as_deref()))?,
        );
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
    solver.place_children(&window.children, gui_rect)?;
    let Solved { draws, slots, sprite_slots, regions, items, collections, inputs, switches, layers, warnings } =
        solver.finish();

    Ok(LaidOutWindow {
        name: window.name.clone(),
        surface,
        draws,
        slots,
        sprite_slots,
        regions,
        items,
        collections,
        inputs,
        switches,
        layers,
        handles: window.handles.clone(),
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
    solver.place_children(&hud.children, rect)?;
    let Solved { draws, slots, switches, layers, warnings, .. } = solver.finish();

    Ok(LaidOutHud {
        name: hud.name.clone(),
        channel: hud.channel,
        width: size.width,
        height: size.height,
        shader: hud.shader,
        draws,
        slots,
        switches,
        layers,
        handles: hud.handles.clone(),
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
        Element::Sprite { pos, .. }
        | Element::SpriteSlot { pos, .. }
        | Element::Label { pos, .. }
        | Element::Slot { pos, .. } => *pos,
        Element::Flex(node) => node.pos,
        Element::Switch(switch) => switch.pos,
        Element::Item { .. }
        | Element::Collection { .. }
        | Element::AnvilInput { .. }
        | Element::Region(_)
        | Element::Section(_) => None,
    }
}

/// The authored debug name of `el`, if it has one.
fn debug_name_of(el: &Element) -> Option<&str> {
    match el {
        Element::Sprite { debug_name, .. }
        | Element::SpriteSlot { debug_name, .. }
        | Element::Item { debug_name, .. }
        | Element::Collection { debug_name, .. }
        | Element::AnvilInput { debug_name, .. }
        | Element::Label { debug_name, .. }
        | Element::Slot { debug_name, .. } => debug_name.as_deref(),
        Element::Region(region) => region.debug_name.as_deref(),
        Element::Flex(node) => node.debug_name.as_deref(),
        Element::Section(section) => section.debug_name.as_deref(),
        Element::Switch(switch) => switch.debug_name.as_deref(),
    }
}

struct Solved {
    draws: Vec<Draw>,
    slots: Vec<SlotIr>,
    sprite_slots: Vec<SpriteSlotIr>,
    regions: Vec<RegionIr>,
    items: Vec<ItemIr>,
    collections: Vec<CollectionIr>,
    inputs: Vec<AnvilInputIr>,
    switches: Vec<SwitchIr>,
    layers: Vec<Layer>,
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
    regions: Vec<RegionIr>,
    items: Vec<ItemIr>,
    collections: Vec<CollectionIr>,
    inputs: Vec<AnvilInputIr>,
    switches: Vec<SwitchIr>,
    layers: Vec<Layer>,
    warnings: Vec<String>,
    next_label: usize,
    next_region: usize,
    names: HashSet<String>,
    /// How many switch cases enclose the element being emitted.
    case_depth: usize,
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
            regions: Vec::new(),
            items: Vec::new(),
            collections: Vec::new(),
            inputs: Vec::new(),
            switches: Vec::new(),
            layers: Vec::new(),
            warnings: Vec::new(),
            next_label: 0,
            next_region: 0,
            names: HashSet::new(),
            case_depth: 0,
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
            regions: self.regions,
            items: self.items,
            collections: self.collections,
            inputs: self.inputs,
            switches: self.switches,
            layers: self.layers,
            warnings: self.warnings,
        }
    }

    /// Places `children` in the content box `content`.
    fn place_children(&mut self, children: &[Element], content: Rect) -> Result<()> {
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
        self.named(debug_name_of(el), |s| s.place_element(el, origin))
    }

    /// Runs `emit` for an element with `debug_name`: its errors name the element, and so do the text and sprite
    /// slots it emits that no nested element named.
    fn named<R>(&mut self, debug_name: Option<&str>, emit: impl FnOnce(&mut Self) -> Result<R>) -> Result<R> {
        let starts = (self.slots.len(), self.sprite_slots.len());
        let out = emit(self).map_err(|error| error.with_debug_name(debug_name))?;
        if let Some(name) = debug_name {
            let sources = self.slots[starts.0..].iter_mut().map(|slot| &mut slot.source);
            let sprite_sources = self.sprite_slots[starts.1..].iter_mut().map(|slot| &mut slot.source);
            for source in sources.chain(sprite_sources) {
                source.get_or_insert_with(|| name.to_string());
            }
        }
        Ok(out)
    }

    fn place_element(&mut self, el: &Element, origin: Point) -> Result<Size> {
        match el {
            Element::Item { name, slots, pattern, .. } => {
                self.require_interaction(name)?;
                let slots = self.resolve_required_slots(name, slots.as_ref(), pattern.as_ref())?;
                self.place_item(name, slots)
            }
            Element::Collection { name, slots, pattern, frame, selected_sprite, action, .. } => {
                self.require_interaction(name)?;
                let slots = self.resolve_required_slots(name, slots.as_ref(), pattern.as_ref())?;
                self.place_collection(name, slots, frame.as_deref(), selected_sprite.clone(), *action)
            }
            Element::Region(region) => {
                let size = region.size.ok_or_else(|| {
                    self.target.layout_err("a region without `width` and `height` must fill a box or section area")
                })?;
                self.place_region(region, Rect::from_parts(origin, size), None)?;
                Ok(size)
            }
            Element::AnvilInput { name, initial, item_model, .. } => {
                self.place_anvil_input(name, initial, item_model.as_deref())
            }
            Element::Sprite { name, .. } => Ok(self.emit_sprite(name, origin)?.size()),
            Element::SpriteSlot { name, size, align, .. } => {
                self.place_sprite_slot(name, Rect::from_parts(origin, *size), *align)
            }
            Element::Label { text, width, style, .. } => self.place_label(text, *width, style, origin),
            Element::Slot { name, width, style, fit, .. } => self.place_slot(name, *width, style, *fit, origin),
            Element::Flex(node) => self.place_flex(node, origin, None),
            Element::Section(section) => self.place_section(section),
            Element::Switch(switch) => self.place_switch(switch, origin, None),
        }
    }
}

#[cfg(test)]
mod tests;
