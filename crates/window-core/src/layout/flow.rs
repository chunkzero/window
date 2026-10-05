use super::target::LayoutTarget;
use super::{Solver, pos_of};
use crate::Result;
use crate::geometry::{Insets, Point, Rect, Size};
use crate::ir::Align;
use crate::model::{CrossAlign, Element};

/// Main/cross axis split for row/column flow.
#[derive(Clone, Copy)]
pub(super) enum Axis {
    Row,
    Column,
}

impl Axis {
    /// Split a size into `(main, cross)`.
    pub(super) fn split(self, s: Size) -> (u32, u32) {
        match self {
            Axis::Row => (s.width, s.height),
            Axis::Column => (s.height, s.width),
        }
    }

    /// Join `(main, cross)` into a size.
    pub(super) fn join(self, main: u32, cross: u32) -> Size {
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

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn place_flow(
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
            if pos_of(child).is_some() {
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
            if let Some(p) = pos_of(child) {
                self.place(child, Point::new(content_origin.x + p.x, content_origin.y + p.y))?;
            }
        }

        Ok(total)
    }

    pub(super) fn layout_button_children(
        &mut self,
        button_name: &str,
        children: &[Element],
        button_rect: Rect,
        padding: u32,
    ) -> Result<()> {
        let content = self.button_content_rect(button_name, button_rect, padding)?;
        for authored_child in children {
            if let Element::Flex(node) = authored_child {
                match node.pos {
                    None => self.place_flex(node, content.origin(), Some(content.size()))?,
                    Some(pos) => self.place_flex(node, Point::new(content.x + pos.x, content.y + pos.y), None)?,
                };
                continue;
            }
            if let Element::Switch(switch) = authored_child {
                match switch.pos {
                    None => self.place_switch(switch, content.origin(), Some(content.size()))?,
                    Some(pos) => self.place_switch(switch, Point::new(content.x + pos.x, content.y + pos.y), None)?,
                };
                continue;
            }
            let auto_center = pos_of(authored_child).is_none();
            let child = resolve_button_child(authored_child, content.width, auto_center);
            let child_size = self.measure(&child)?;
            let origin = match pos_of(&child) {
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

    fn button_content_rect(&self, button_name: &str, button_rect: Rect, padding: u32) -> Result<Rect> {
        let inset = padding
            .checked_mul(2)
            .ok_or_else(|| self.target.layout_err(format!("button `{button_name}` padding is too large")))?;
        if inset > button_rect.width || inset > button_rect.height {
            return Err(self.target.layout_err(format!(
                "button `{button_name}` padding {padding} leaves no valid content rect inside {button_rect:?}"
            )));
        }
        Ok(Rect::new(
            button_rect.x + padding as i32,
            button_rect.y + padding as i32,
            button_rect.width - inset,
            button_rect.height - inset,
        ))
    }
}

/// Unpositioned button text children default to centered alignment; slots also fill the content width.
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
