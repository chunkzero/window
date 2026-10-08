use super::target::LayoutTarget;
use super::{Solver, debug_name_of};
use crate::Result;
use crate::geometry::Size;
use crate::model::{Element, SpriteDef};
use crate::pipeline::texture_source_path;

impl<'a, T: LayoutTarget> Solver<'a, T> {
    /// The size an element occupies, independent of where it is placed.
    pub(super) fn measure(&self, el: &Element) -> Result<Size> {
        self.measure_element(el).map_err(|error| error.with_debug_name(debug_name_of(el)))
    }

    fn measure_element(&self, el: &Element) -> Result<Size> {
        match el {
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
            Element::Slot { name, width, fit, .. } => {
                Ok(Size::new(self.required_slot_width(name, *width)?, fit.height()))
            }
            Element::Region(region) => {
                self.require_interaction("region")?;
                Ok(region.size.unwrap_or_default())
            }
            Element::Item { .. } | Element::Collection { .. } | Element::AnvilInput { .. } | Element::Section(_) => {
                Ok(Size::new(0, 0))
            }
            Element::Flex(node) => self.measure_flex(node),
            Element::Switch(switch) => self.measure_switch(switch),
        }
    }

    pub(super) fn sprite_def(&self, name: &str) -> Result<&'a SpriteDef> {
        let project = self.project;
        project.art.sprites.get(name).ok_or_else(|| self.target.layout_err(format!("unknown sprite `{name}`")))
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
        (self.texture_size)(&texture_source_path(texture))
            .ok_or_else(|| self.target.missing_texture_err(texture, referenced_by))
    }

    pub(super) fn required_slot_width(&self, name: &str, width: Option<u32>) -> Result<u32> {
        width.ok_or_else(|| self.target.layout_err(format!("slot `{name}` requires `width` unless a box lays it out")))
    }
}
