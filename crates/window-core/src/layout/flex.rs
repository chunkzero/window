//! Taffy-solved `flex` boxes and slot `section` grids.
//!
//! A box's subtree is solved as one taffy tree and then emitted through the regular placement code, so
//! every element keeps its usual validation. Leaves are fixed-size taffy nodes measured from fonts and
//! sprites; text is vertically centered in its box and fills its width, and fixed-size art is centered.

use std::collections::BTreeSet;

use taffy::style_helpers::{TaffyAuto, TaffyGridLine, TaffyGridSpan, TaffyMaxContent, length};
use taffy::{AvailableSpace, Dimension, FlexDirection, GridPlacement, LengthPercentageAuto, Line, NodeId, TaffyTree};

use super::target::LayoutTarget;
use super::{Solver, debug_name_of, pos_of};
use crate::geometry::{Point, Rect, Size};
use crate::inventory::{SlotGridPattern, SlotPattern, SlotRectClaim, SlotRectPattern};
use crate::ir::{Layer, SwitchCaseIr, SwitchIr};
use crate::model::{Element, FlexBox, ItemLayout, SlotSection, Switch};
use crate::{Error, Result};

type Tree = TaffyTree<()>;

impl<T: LayoutTarget> Solver<'_, T> {
    /// The max-content size of a box placed outside any content box.
    pub(super) fn measure_flex(&self, node: &FlexBox) -> Result<Size> {
        let (tree, root) = self.solve(None, |s, tree| s.add_box(tree, node, None))?;
        let layout = tree.layout(root).map_err(|e| self.taffy_err(e))?;
        Ok(Size::new(px(layout.size.width), px(layout.size.height)))
    }

    /// Solves and emits a box at `origin`. `fill` is the parent content box size an auto-sized box fills.
    pub(super) fn place_flex(&mut self, node: &FlexBox, origin: Point, fill: Option<Size>) -> Result<Size> {
        let (tree, root) = self.solve(fill, |s, tree| s.add_box(tree, node, None))?;
        let rect = self.emit_box(&tree, root, node, origin)?;
        Ok(rect.size())
    }

    /// The size of a switch placed outside any content box: its largest case.
    pub(super) fn measure_switch(&self, switch: &Switch) -> Result<Size> {
        let (tree, root) = self.solve(None, |s, tree| s.add_switch(tree, switch, None))?;
        let layout = tree.layout(root).map_err(|e| self.taffy_err(e))?;
        Ok(Size::new(px(layout.size.width), px(layout.size.height)))
    }

    /// Solves and emits a switch at `origin`, filling `fill` like an auto-sized box.
    pub(super) fn place_switch(&mut self, switch: &Switch, origin: Point, fill: Option<Size>) -> Result<Size> {
        let (tree, root) = self.solve(fill, |s, tree| s.add_switch(tree, switch, None))?;
        let rect = self.emit_switch(&tree, root, switch, origin)?;
        Ok(rect.size())
    }

    /// Places `child` in a parent content box: an auto-positioned box fills it, anything else is placed
    /// at its explicit position relative to the content origin, or at the origin.
    pub(super) fn place_in_box(&mut self, child: &Element, content: Rect) -> Result<()> {
        let origin = match pos_of(child) {
            Some(p) => Point::new(content.x + p.x, content.y + p.y),
            None => content.origin(),
        };
        match child {
            Element::Flex(node) if node.pos.is_none() => {
                self.place_flex(node, origin, Some(content.size()))?;
            }
            Element::Switch(switch) if switch.pos.is_none() => {
                self.place_switch(switch, origin, Some(content.size()))?;
            }
            _ if fills_box(child) => self.place_leaf(child, content)?,
            _ => {
                self.place(child, origin)?;
            }
        }
        Ok(())
    }

    fn solve(
        &self,
        fill: Option<Size>,
        build: impl FnOnce(&Self, &mut Tree) -> Result<NodeId>,
    ) -> Result<(Tree, NodeId)> {
        let mut tree = Tree::new();
        let root = build(self, &mut tree)?;
        let mut available = taffy::Size { width: AvailableSpace::MaxContent, height: AvailableSpace::MaxContent };
        if let Some(fill) = fill {
            let mut style = tree.style(root).map_err(|e| self.taffy_err(e))?.clone();
            if style.size.width.is_auto() {
                style.size.width = length(fill.width as f32);
            }
            if style.size.height.is_auto() {
                style.size.height = length(fill.height as f32);
            }
            tree.set_style(root, style).map_err(|e| self.taffy_err(e))?;
            available = taffy::Size {
                width: AvailableSpace::Definite(fill.width as f32),
                height: AvailableSpace::Definite(fill.height as f32),
            };
        }
        tree.compute_layout(root, available).map_err(|e| self.taffy_err(e))?;
        Ok((tree, root))
    }

    fn add_box(&self, tree: &mut Tree, node: &FlexBox, item: Option<&ItemLayout>) -> Result<NodeId> {
        self.add_box_node(tree, node, item).map_err(|error| error.with_debug_name(node.debug_name.as_deref()))
    }

    fn add_box_node(&self, tree: &mut Tree, node: &FlexBox, item: Option<&ItemLayout>) -> Result<NodeId> {
        let mut style = node.style.clone();
        if let Some(item) = item {
            if let Some(pos) = node.pos {
                absolute_at(&mut style, pos);
            }
            item.apply(&mut style);
        }
        let horizontal = style.display == taffy::Display::Flex
            && matches!(style.flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(match &child.element {
                Element::Flex(inner) => self.add_box(tree, inner, Some(&child.layout))?,
                Element::Switch(switch) => self.add_switch(tree, switch, Some(&child.layout))?,
                leaf => self.add_leaf(tree, leaf, &child.layout, horizontal)?,
            });
        }
        tree.new_with_children(style, &children).map_err(|e| self.taffy_err(e))
    }

    /// A switch is a one-cell grid whose cases all occupy the cell, so it takes the largest case's size and
    /// every case box fills it.
    fn add_switch(&self, tree: &mut Tree, switch: &Switch, item: Option<&ItemLayout>) -> Result<NodeId> {
        self.add_switch_node(tree, switch, item).map_err(|error| error.with_debug_name(switch.debug_name.as_deref()))
    }

    fn add_switch_node(&self, tree: &mut Tree, switch: &Switch, item: Option<&ItemLayout>) -> Result<NodeId> {
        self.check_switch(switch)?;
        let mut style = taffy::Style { display: taffy::Display::Grid, ..Default::default() };
        if let Some(item) = item {
            if let Some(pos) = switch.pos {
                absolute_at(&mut style, pos);
            }
            item.apply(&mut style);
        }
        let first = Line { start: GridPlacement::from_line_index(1), end: GridPlacement::AUTO };
        let cell = ItemLayout { column: Some(first.clone()), row: Some(first), ..Default::default() };
        let mut cases = Vec::with_capacity(switch.cases.len());
        for case in &switch.cases {
            cases.push(self.add_box(tree, &case.body, Some(&cell))?);
        }
        tree.new_with_children(style, &cases).map_err(|e| self.taffy_err(e))
    }

    /// Rejects widgets that cannot be inside `switch`'s cases.
    fn check_switch(&self, switch: &Switch) -> Result<()> {
        for case in &switch.cases {
            for child in &case.body.children {
                if let Some(error) = case_conflict(&child.element, &switch.name) {
                    return Err(self.target.layout_err(error));
                }
            }
        }
        Ok(())
    }

    /// Text leaves stretch across their line and are at least as tall as their lines; dynamic text without a
    /// width also grows along a horizontal parent. Other leaves keep their measured size.
    fn add_leaf(&self, tree: &mut Tree, el: &Element, item: &ItemLayout, horizontal: bool) -> Result<NodeId> {
        let mut style = taffy::Style { flex_shrink: 0.0, ..Default::default() };
        match el {
            _ if fills_box(el) => {
                style.position = taffy::Position::Absolute;
                style.inset = taffy::Rect {
                    left: LengthPercentageAuto::length(0.0),
                    right: LengthPercentageAuto::length(0.0),
                    top: LengthPercentageAuto::length(0.0),
                    bottom: LengthPercentageAuto::length(0.0),
                };
            }
            // Placed by their own slots, not by the box.
            Element::Item { .. } | Element::Collection { .. } | Element::AnvilInput { .. } | Element::Section(_) => {
                style.position = taffy::Position::Absolute;
            }
            Element::Label { .. } | Element::Slot { .. } => {
                let size = self.leaf_size(el)?;
                let unsized_slot = matches!(el, Element::Slot { width: None, .. });
                let width = if unsized_slot { Dimension::AUTO } else { length(size.width as f32) };
                style.size = taffy::Size { width, height: Dimension::AUTO };
                style.min_size.height = LengthPercentageAuto::length(size.height as f32);
                if unsized_slot && horizontal {
                    style.flex_grow = 1.0;
                }
            }
            _ => {
                let size = self.leaf_size(el)?;
                style.size = taffy::Size { width: length(size.width as f32), height: length(size.height as f32) };
            }
        }
        if let Some(pos) = pos_of(el) {
            absolute_at(&mut style, pos);
        }
        item.apply(&mut style);
        tree.new_leaf(style).map_err(|e| self.taffy_err(e))
    }

    fn leaf_size(&self, el: &Element) -> Result<Size> {
        match el {
            Element::Slot { width: None, fit, .. } => Ok(Size::new(0, fit.height())),
            Element::Label { .. }
            | Element::Slot { .. }
            | Element::Sprite { .. }
            | Element::SpriteSlot { .. }
            | Element::Region(_) => self.measure(el),
            other => Err(self.target.layout_err(format!(
                "{} cannot be laid out inside a flex box; give it a slot source or place it in a section",
                describe(other)
            ))),
        }
    }

    fn emit_box(&mut self, tree: &Tree, id: NodeId, node: &FlexBox, parent: Point) -> Result<Rect> {
        self.named(node.debug_name.as_deref(), |s| s.emit_box_contents(tree, id, node, parent))
    }

    fn emit_box_contents(&mut self, tree: &Tree, id: NodeId, node: &FlexBox, parent: Point) -> Result<Rect> {
        let rect = self.node_rect(tree, id, parent)?;
        if let Some(frame) = &node.frame {
            self.emit_frame(frame, rect, &format!("flex frame `{frame}`"))?;
        }
        let ids = tree.children(id).map_err(|e| self.taffy_err(e))?;
        for (child, child_id) in node.children.iter().zip(ids) {
            let t = child.layout.translate;
            let origin = Point::new(rect.x + t.x, rect.y + t.y);
            match &child.element {
                Element::Flex(inner) => {
                    self.emit_box(tree, child_id, inner, origin)?;
                }
                Element::Switch(switch) => {
                    self.emit_switch(tree, child_id, switch, origin)?;
                }
                leaf => {
                    let leaf_rect = self.node_rect(tree, child_id, origin)?;
                    self.place_leaf(leaf, leaf_rect)?;
                }
            }
        }
        Ok(rect)
    }

    /// Emits each case box over the switch rect, collecting its draws and the regions directly inside it into its
    /// own case.
    fn emit_switch(&mut self, tree: &Tree, id: NodeId, switch: &Switch, parent: Point) -> Result<Rect> {
        self.named(switch.debug_name.as_deref(), |s| s.emit_switch_cases(tree, id, switch, parent))
    }

    fn emit_switch_cases(&mut self, tree: &Tree, id: NodeId, switch: &Switch, parent: Point) -> Result<Rect> {
        self.register_name(&switch.name)?;
        self.layers.push(Layer::Switch(switch.name.clone()));
        let rect = self.node_rect(tree, id, parent)?;
        let ids = tree.children(id).map_err(|e| self.taffy_err(e))?;
        let mut cases = Vec::with_capacity(switch.cases.len());
        for (case, case_id) in switch.cases.iter().zip(ids) {
            self.case_depth += 1;
            let emitted =
                self.emit_case(&case.value, |s| s.emit_box(tree, case_id, &case.body, rect.origin()).map(|_| ()));
            self.case_depth -= 1;
            cases.push(emitted?);
        }
        let source = switch.debug_name.clone();
        self.switches.push(SwitchIr { name: switch.name.clone(), source, cases });
        Ok(rect)
    }

    /// Runs `emit` as switch case `value`: its draws, and the slots, sprite slots, regions, items, collections, and
    /// switches it emits outside any nested switch, become the case.
    fn emit_case(&mut self, value: &str, emit: impl FnOnce(&mut Self) -> Result<()>) -> Result<SwitchCaseIr> {
        let outer_draws = std::mem::take(&mut self.draws);
        let starts = (
            self.slots.len(),
            self.sprite_slots.len(),
            self.regions.len(),
            self.switches.len(),
            self.items.len(),
            self.collections.len(),
        );
        let emitted = emit(self);
        let draws = std::mem::replace(&mut self.draws, outer_draws);
        emitted?;
        let nested = &self.switches[starts.3..];
        let inner = |pick: fn(&SwitchCaseIr) -> &Vec<String>| -> BTreeSet<&String> {
            nested.iter().flat_map(|switch| switch.cases.iter().flat_map(pick)).collect()
        };
        let direct = |names: Vec<&String>, inner: BTreeSet<&String>| -> Vec<String> {
            names.into_iter().filter(|name| !inner.contains(name)).cloned().collect()
        };
        Ok(SwitchCaseIr {
            value: value.to_string(),
            draws,
            slots: direct(self.slots[starts.0..].iter().map(|slot| &slot.name).collect(), inner(|c| &c.slots)),
            sprite_slots: direct(
                self.sprite_slots[starts.1..].iter().map(|slot| &slot.name).collect(),
                inner(|c| &c.sprite_slots),
            ),
            regions: direct(
                self.regions[starts.2..].iter().map(|region| &region.name).collect(),
                inner(|c| &c.regions),
            ),
            switches: direct(nested.iter().map(|switch| &switch.name).collect(), inner(|c| &c.switches)),
            items: direct(self.items[starts.4..].iter().map(|item| &item.name).collect(), inner(|c| &c.items)),
            collections: direct(
                self.collections[starts.5..].iter().map(|collection| &collection.name).collect(),
                inner(|c| &c.collections),
            ),
        })
    }

    /// Emits a leaf in its solved box: text takes the box width and is vertically centered, and other
    /// elements keep their size, centered in the box.
    fn place_leaf(&mut self, el: &Element, rect: Rect) -> Result<()> {
        self.named(debug_name_of(el), |s| s.place_leaf_in(el, rect))
    }

    fn place_leaf_in(&mut self, el: &Element, rect: Rect) -> Result<()> {
        let centered_y = |height: u32| rect.y + (rect.height as i32 - height as i32).div_euclid(2);
        match el {
            Element::Label { text, style, .. } => {
                self.place_label(text, Some(rect.width), style, Point::new(rect.x, centered_y(8)))?;
            }
            Element::Slot { name, style, fit, .. } => {
                self.place_slot(name, Some(rect.width), style, *fit, Point::new(rect.x, centered_y(fit.height())))?;
            }
            Element::Region(region) => {
                let slots = self.slots_under("region", rect)?;
                self.place_region(region, rect, Some(slots))?;
            }
            Element::Item { name, .. } if fills_box(el) => {
                self.require_interaction(name)?;
                let slots = self.slots_under(name, rect)?;
                self.place_item(name, slots)?;
            }
            Element::Collection { name, frame, selected_sprite, action, .. } if fills_box(el) => {
                self.require_interaction(name)?;
                let slots = self.slots_under(name, rect)?;
                self.place_collection(name, slots, frame.as_deref(), selected_sprite.clone(), *action)?;
            }
            other => {
                let size = self.measure(other)?;
                let origin = Point::new(
                    rect.x + (rect.width as i32 - size.width as i32).div_euclid(2),
                    rect.y + (rect.height as i32 - size.height as i32).div_euclid(2),
                );
                self.place(other, origin)?;
            }
        }
        Ok(())
    }

    fn node_rect(&self, tree: &Tree, id: NodeId, parent: Point) -> Result<Rect> {
        let layout = tree.layout(id).map_err(|e| self.taffy_err(e))?;
        Ok(Rect::new(
            parent.x + layout.location.x.round() as i32,
            parent.y + layout.location.y.round() as i32,
            px(layout.size.width),
            px(layout.size.height),
        ))
    }

    /// Lays a section's children out on a grid with one track per slot, then places each child on its area.
    pub(super) fn place_section(&mut self, section: &SlotSection) -> Result<Size> {
        let label = section_name(section);
        let kind = self
            .target
            .container_kind()
            .ok_or_else(|| self.target.layout_err(format!("{label} sections are only available in windows")))?;
        let dims = kind.section_size(section.section);
        let bounds = kind.section_bounds(section.section);
        if let Some(frame) = &section.frame {
            let o = section.outset;
            let rect = Rect::new(
                bounds.x - 1 - o.left as i32,
                bounds.y - 1 - o.top as i32,
                bounds.width + 2 + o.left + o.right,
                bounds.height + 2 + o.top + o.bottom,
            );
            self.emit_frame(frame, rect, &format!("{label} section frame `{frame}`"))?;
        }

        let mut tree = Tree::new();
        let mut placed = Vec::with_capacity(section.children.len());
        for child in &section.children {
            let Some(span) = grid_span(&child.element)? else {
                placed.push(None);
                continue;
            };
            if span.width > dims.width || span.height > dims.height {
                return Err(self
                    .target
                    .layout_err(format!("{} does not fit the {label} section", describe(&child.element))));
            }
            let mut style = taffy::Style {
                grid_column: Line { start: GridPlacement::AUTO, end: GridPlacement::from_span(span.width as u16) },
                grid_row: Line { start: GridPlacement::AUTO, end: GridPlacement::from_span(span.height as u16) },
                ..Default::default()
            };
            let item = &child.layout;
            if item.align_self.is_some()
                || item.justify_self.is_some()
                || item.margin.is_some()
                || item.absolute
                || item.inset.is_some()
                || item.min_size.is_some()
                || item.max_size.is_some()
            {
                return Err(self.target.layout_err(format!(
                    "{} section children support grid placement and translate; put pixel layout inside a flex box",
                    describe(&child.element)
                )));
            }
            if let Some(column) = &item.column {
                style.grid_column = column.clone();
            }
            if let Some(row) = &item.row {
                style.grid_row = row.clone();
            }
            validate_section_axis(&style.grid_column, dims.width)?;
            validate_section_axis(&style.grid_row, dims.height)?;
            placed.push(Some(tree.new_leaf(style).map_err(|e| self.taffy_err(e))?));
        }
        let grid = taffy::Style {
            display: taffy::Display::Grid,
            size: taffy::Size { width: length(dims.width as f32), height: length(dims.height as f32) },
            grid_template_columns: vec![length(1.0); dims.width as usize],
            grid_template_rows: vec![length(1.0); dims.height as usize],
            grid_auto_columns: vec![length(1.0)],
            grid_auto_rows: vec![length(1.0)],
            grid_auto_flow: section.flow,
            ..Default::default()
        };
        let ids: Vec<NodeId> = placed.iter().flatten().copied().collect();
        let root = tree.new_with_children(grid, &ids).map_err(|e| self.taffy_err(e))?;
        tree.compute_layout(root, taffy::Size::MAX_CONTENT).map_err(|e| self.taffy_err(e))?;

        for (child, id) in section.children.iter().zip(placed) {
            let Some(id) = id else {
                self.place(&child.element, Point::new(0, 0))?;
                continue;
            };
            let layout = tree.layout(id).map_err(|e| self.taffy_err(e))?;
            let area = SlotRectPattern {
                section: section.section,
                x: layout.location.x.round() as u32,
                y: layout.location.y.round() as u32,
                width: px(layout.size.width),
                height: px(layout.size.height),
            };
            if area.x.checked_add(area.width).is_none_or(|right| right > dims.width)
                || area.y.checked_add(area.height).is_none_or(|bottom| bottom > dims.height)
                || area.width == 0
                || area.height == 0
            {
                return Err(self.target.layout_err(format!(
                    "{} does not fit the {label} section: it needs {}x{} slots at ({}, {}) in a {}x{} grid",
                    describe(&child.element),
                    area.width.max(1),
                    area.height.max(1),
                    area.x,
                    area.y,
                    dims.width,
                    dims.height
                )));
            }
            let span = grid_span(&child.element)?.expect("grid child has a span");
            if span.width > area.width || span.height > area.height {
                return Err(self.target.layout_err(format!(
                    "{} pattern does not fit its {}x{} slot grid area",
                    describe(&child.element),
                    area.width,
                    area.height
                )));
            }
            self.place_on_area(&child.element, &area, child.layout.translate)?;
        }

        if section.claim != SlotRectClaim::None {
            let all = SlotPattern::Rect(SlotRectPattern {
                section: section.section,
                x: 0,
                y: 0,
                width: dims.width,
                height: dims.height,
            });
            let base = format!("{label}_section");
            // Sections in switch cases may repeat, one per case.
            let name = match self.case_depth {
                0 => base,
                _ => (2..).map(|n| format!("{base}~{n}")).find(|name| !self.names.contains(name)).expect("free"),
            };
            self.place_claim(&name, &format!("{label} section"), &all, section.claim)?;
        }
        Ok(Size::new(0, 0))
    }

    fn place_on_area(&mut self, el: &Element, area: &SlotRectPattern, translate: Point) -> Result<()> {
        self.named(debug_name_of(el), |s| s.place_on_area_in(el, area, translate))
    }

    fn place_on_area_in(&mut self, el: &Element, area: &SlotRectPattern, translate: Point) -> Result<()> {
        let interior = self
            .target
            .container_kind()
            .and_then(|kind| kind.section_rect_bounds(area.section, area.x, area.y, area.width, area.height));
        let Some(interior) = interior else {
            return Err(self.target.layout_err(format!("{} lands outside its section", describe(el))));
        };
        let cell = Rect::new(
            interior.x - 1 + translate.x,
            interior.y - 1 + translate.y,
            interior.width + 2,
            interior.height + 2,
        );
        match el {
            Element::Item { .. } | Element::Collection { .. } | Element::Region(_) => {
                if translate != Point::default() {
                    return Err(self.target.layout_err(format!(
                        "{} is slot-bound and cannot be translated; translate its visual children instead",
                        describe(el)
                    )));
                }
                if let Element::Region(region) = el {
                    let pattern = SlotPattern::Rect(area.clone());
                    let slots = self.resolve_pattern_slots("region", &pattern)?;
                    let rect = self.pattern_bounds("region", &pattern)?;
                    self.place_region(region, rect, Some(slots))?;
                    return Ok(());
                }
                let anchored = anchor_pattern(el, area);
                self.place(&anchored, cell.origin())?;
            }
            Element::Flex(node) => {
                self.place_flex(node, cell.origin(), Some(cell.size()))?;
            }
            Element::Switch(switch) => {
                self.place_switch(switch, cell.origin(), Some(cell.size()))?;
            }
            other => self.place_leaf(other, cell)?,
        }
        Ok(())
    }

    fn taffy_err(&self, error: taffy::TaffyError) -> crate::Error {
        self.target.layout_err(format!("flex layout failed: {error}"))
    }
}

/// The default grid span of a section child, or `None` when it is not placed on the grid: items and
/// collections with explicit slots or a `slots` pattern, and anvil inputs.
fn grid_span(el: &Element) -> Result<Option<Size>> {
    let pattern = match el {
        Element::Item { slots: Some(_), .. }
        | Element::Collection { slots: Some(_), .. }
        | Element::AnvilInput { .. } => {
            return Ok(None);
        }
        Element::Item { pattern, .. } | Element::Collection { pattern, .. } => pattern.as_ref(),
        _ => None,
    };
    let size = match pattern {
        None => Some((1, 1)),
        Some(SlotPattern::Rect(rect)) => {
            Some((u64::from(rect.x) + u64::from(rect.width), u64::from(rect.y) + u64::from(rect.height)))
        }
        Some(SlotPattern::Grid(grid)) => Some((
            u64::from(grid.x) + u64::from(grid.columns) * u64::from(grid.cell_width),
            u64::from(grid.y) + u64::from(grid.rows) * u64::from(grid.cell_height),
        )),
        Some(SlotPattern::Slots { .. }) => None,
    };
    size.map(|(width, height)| {
        let invalid = || Error::Validation(format!("{} section pattern is too large", describe(el)));
        Ok(Size::new(u32::try_from(width).map_err(|_| invalid())?, u32::try_from(height).map_err(|_| invalid())?))
    })
    .transpose()
}

fn validate_section_axis(line: &Line<GridPlacement>, tracks: u32) -> Result<()> {
    for placement in [&line.start, &line.end] {
        let fits = match placement {
            GridPlacement::Line(line) => i32::from(line.as_i16()).unsigned_abs() <= tracks + 1,
            GridPlacement::Span(span) => u32::from(*span) <= tracks,
            _ => true,
        };
        if !fits {
            return Err(Error::Validation("section grid placement is outside its slot tracks".into()));
        }
    }
    if let GridPlacement::Line(end) = &line.end {
        let span = match &line.start {
            GridPlacement::Span(span) => *span,
            GridPlacement::Auto => 1,
            _ => return Ok(()),
        };
        let end = i32::from(end.as_i16());
        let end = if end < 0 { end + tracks as i32 + 1 } else { end - 1 };
        if end < i32::from(span) {
            return Err(Error::Validation("section grid placement extends before its first slot track".into()));
        }
    }
    Ok(())
}

/// Moves an item's or collection's area-relative pattern onto `area`; an unpatterned one covers the whole area.
fn anchor_pattern(el: &Element, area: &SlotRectPattern) -> Element {
    let anchor = |pattern: Option<&SlotPattern>| -> SlotPattern {
        match pattern {
            None => SlotPattern::Rect(area.clone()),
            Some(SlotPattern::Rect(rect)) => SlotPattern::Rect(SlotRectPattern {
                section: area.section,
                x: area.x + rect.x,
                y: area.y + rect.y,
                ..rect.clone()
            }),
            Some(SlotPattern::Grid(grid)) => SlotPattern::Grid(SlotGridPattern {
                section: area.section,
                x: area.x + grid.x,
                y: area.y + grid.y,
                ..grid.clone()
            }),
            Some(other) => other.clone(),
        }
    };
    let mut out = el.clone();
    if let Element::Item { pattern, .. } | Element::Collection { pattern, .. } = &mut out {
        *pattern = Some(anchor(pattern.as_ref()));
    }
    out
}

fn absolute_at(style: &mut taffy::Style, pos: Point) {
    style.position = taffy::Position::Absolute;
    style.inset = taffy::Rect {
        left: LengthPercentageAuto::length(pos.x as f32),
        top: LengthPercentageAuto::length(pos.y as f32),
        right: LengthPercentageAuto::AUTO,
        bottom: LengthPercentageAuto::AUTO,
    };
}

fn px(value: f32) -> u32 {
    value.round().max(0.0) as u32
}

fn section_name(section: &SlotSection) -> &'static str {
    match section.section {
        crate::inventory::InventorySlotSection::Container => "container",
        crate::inventory::InventorySlotSection::Player => "player",
        crate::inventory::InventorySlotSection::Hotbar => "hotbar",
    }
}

fn describe(el: &Element) -> String {
    match el {
        Element::Item { name, .. } => format!("item `{name}`"),
        Element::Collection { name, .. } => format!("collection `{name}`"),
        Element::AnvilInput { name, .. } => format!("anvil input `{name}`"),
        Element::Slot { name, .. } => format!("slot `{name}`"),
        Element::SpriteSlot { name, .. } => format!("sprite slot `{name}`"),
        Element::Sprite { name, .. } => format!("image `{name}`"),
        Element::Label { text, .. } => format!("label `{text}`"),
        Element::Section(section) => format!("{} section", section_name(section)),
        Element::Flex(_) => "flex box".into(),
        Element::Switch(switch) => format!("switch `{}`", switch.name),
        Element::Region(region) => match (&region.debug_name, &region.name) {
            (Some(name), _) | (None, Some(name)) => format!("region `{name}`"),
            (None, None) => "region".into(),
        },
    }
}

/// Whether `el` is slot-bound and takes its slots from its laid-out rect, which fills its parent box: an unsized
/// region, or an item or collection without a slot source.
fn fills_box(el: &Element) -> bool {
    match el {
        Element::Region(region) => region.size.is_none(),
        Element::Item { slots: None, pattern: None, .. } | Element::Collection { slots: None, pattern: None, .. } => {
            true
        }
        _ => false,
    }
}

/// Why `el`, inside a case of switch `switch`, is not allowed there.
fn case_conflict(el: &Element, switch: &str) -> Option<String> {
    match el {
        Element::AnvilInput { .. } => Some(format!(
            "{} cannot be inside switch `{switch}`: the anvil's rename field is always shown",
            describe(el)
        )),
        Element::Section(section) => section.children.iter().find_map(|c| case_conflict(&c.element, switch)),
        Element::Switch(inner) => {
            inner.cases.iter().flat_map(|case| &case.body.children).find_map(|c| case_conflict(&c.element, switch))
        }
        Element::Flex(node) => node.children.iter().find_map(|c| case_conflict(&c.element, switch)),
        _ => None,
    }
}
