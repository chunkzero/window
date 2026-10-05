use super::flow::Axis;
use super::slots::cells_bounds;
use super::target::LayoutTarget;
use super::{Solver, pos_of};
use crate::Result;
use crate::geometry::{Insets, Size};
use crate::inventory::SlotPattern;
use crate::model::{Element, SpriteDef};

impl<'a, T: LayoutTarget> Solver<'a, T> {
    /// The size an element occupies, independent of where it is placed.
    pub(super) fn measure(&self, el: &Element) -> Result<Size> {
        match el {
            Element::Panel { size, .. } => Ok(*size),
            Element::Button { name, size, pattern, .. } | Element::Hotspot { name, size, pattern, .. } => {
                self.require_interaction(name)?;
                self.control_size(name, *size, pattern.as_ref())
            }
            Element::Sprite { name, .. } => self.sprite_size(name, self.sprite_def(name)?),
            Element::SpriteSlot { name, size, .. } => {
                self.require_sprite_slots(name)?;
                Ok(*size)
            }
            Element::Label { text, width, style, .. } => {
                let w = match width {
                    Some(width) => *width,
                    None => self.text_width(text, style)?,
                };
                Ok(Size::new(w, 8))
            }
            Element::Slot { name, width, .. } => Ok(Size::new(self.required_slot_width(name, *width)?, 8)),
            Element::Item { .. }
            | Element::Collection { .. }
            | Element::AnvilInput { .. }
            | Element::SlotRects { .. } => Ok(Size::new(0, 0)),
            Element::Repeater { name, pattern, .. } => {
                self.require_slot_patterns(name)?;
                let cells = self.resolve_pattern_cells(name, pattern)?;
                Ok(cells_bounds(&cells).map_or(Size::new(0, 0), |rect| rect.size()))
            }
            Element::Row { gap, padding, children, .. } => self.measure_flow(*gap, *padding, children, Axis::Row),
            Element::Column { gap, padding, children, .. } => self.measure_flow(*gap, *padding, children, Axis::Column),
            Element::Flex(node) => self.measure_flex(node),
            Element::Section(_) => Ok(Size::new(0, 0)),
        }
    }

    pub(super) fn measure_flow(&self, gap: u32, padding: u32, children: &[Element], axis: Axis) -> Result<Size> {
        let pad = Insets::uniform(padding);
        let mut main = 0u32;
        let mut cross = 0u32;
        let mut count = 0u32;
        for child in children {
            if pos_of(child).is_some() {
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

    pub(super) fn content_cross(&self, children: &[Element], axis: Axis) -> Result<u32> {
        let mut cross = 0u32;
        for child in children {
            if pos_of(child).is_some() {
                continue;
            }
            let s = self.measure(child)?;
            cross = cross.max(axis.split(s).1);
        }
        Ok(cross)
    }

    pub(super) fn sprite_def(&self, name: &str) -> Result<&'a SpriteDef> {
        let project = self.project;
        project.theme.sprites.get(name).ok_or_else(|| self.target.layout_err(format!("unknown sprite `{name}`")))
    }

    pub(super) fn sprite_size(&self, name: &str, def: &SpriteDef) -> Result<Size> {
        match def {
            SpriteDef::Texture { texture, size } => {
                size.map(Ok).unwrap_or_else(|| self.intrinsic_texture_size(texture, &format!("sprite `{name}`")))
            }
            SpriteDef::Generated { size, .. } => Ok(*size),
        }
    }

    pub(super) fn intrinsic_texture_size(&self, texture: &str, referenced_by: &str) -> Result<Size> {
        (self.texture_size)(texture).ok_or_else(|| self.target.missing_texture_err(texture, referenced_by))
    }

    pub(super) fn required_slot_width(&self, name: &str, width: Option<u32>) -> Result<u32> {
        width.ok_or_else(|| {
            self.target
                .layout_err(format!("slot `{name}` requires `width` unless it is a direct, unpositioned button child"))
        })
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
}
