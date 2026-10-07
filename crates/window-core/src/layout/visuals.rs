use super::Solver;
use super::target::LayoutTarget;
use crate::Result;
use crate::geometry::{Insets, Point, Rect, Size};
use crate::ir::{Align, Draw, RepeatBindingIr, SlotIr, SpriteSlotIr, TextureKey};
use crate::model::{Element, Frame, SpriteDef, TextFit, TextStyle};

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn place_panel(&mut self, frame: &str, rect: Rect, padding: u32, children: &[Element]) -> Result<Size> {
        self.emit_frame(frame, rect, &format!("panel frame `{frame}`"))?;
        self.layout_container_children(children, rect, Insets::uniform(padding))?;
        Ok(rect.size())
    }

    pub(super) fn place_sprite_slot(
        &mut self,
        name: &str,
        rect: Rect,
        align: Align,
        sprite: Option<&str>,
    ) -> Result<Size> {
        self.require_sprite_slots(name)?;
        // Fixed sprites need no binding, so they never join one shared across switch cases.
        let binding = self.case_binding(name).filter(|_| sprite.is_none()).map(|_| name.to_string());
        let actual_name =
            if binding.is_none() && self.active_repeat.is_none() { name.to_string() } else { self.scoped_name(name) };
        self.register_name(&actual_name)?;
        self.check_inside_bounds(rect, &format!("sprite slot `{name}`"))?;
        if let Some(sprite) = sprite {
            let sprite_size = self.sprite_size(sprite, self.sprite_def(sprite)?)?;
            if sprite_size.width > rect.width || sprite_size.height > rect.height {
                return Err(self.target.layout_err(format!(
                    "fixed sprite `{sprite}` does not fit sprite slot `{name}` ({}x{} in {}x{})",
                    sprite_size.width, sprite_size.height, rect.width, rect.height
                )));
            }
        }
        self.sprite_slots.push(SpriteSlotIr {
            name: actual_name,
            rect,
            align,
            sprite: sprite.map(str::to_string),
            repeat: self.repeat_binding(name),
            binding,
        });
        Ok(rect.size())
    }

    pub(super) fn place_label(
        &mut self,
        text: &str,
        width: Option<u32>,
        style: &TextStyle,
        origin: Point,
    ) -> Result<Size> {
        let font = self.text_font(style)?;
        let measured = self.text_width(text, style)?;
        let w = width.unwrap_or(measured);
        let rect = Rect::from_parts(origin, Size::new(w, 8));
        let name = self.next_label_name();
        self.register_name(&name)?;
        self.check_text_constraints(rect, &name, false)?;
        self.warn_if_unsupported_static_text(text, font, &name);
        if width.is_some() {
            self.warn_if_static_text_overflow(text, measured, w, &name);
        }
        self.slots.push(text_slot_ir(name, Some(text.to_string()), rect, style, None, None, TextFit::default()));
        Ok(rect.size())
    }

    pub(super) fn place_slot(
        &mut self,
        name: &str,
        width: Option<u32>,
        style: &TextStyle,
        fit: TextFit,
        origin: Point,
    ) -> Result<Size> {
        let width = self.required_slot_width(name, width)?;
        let rect = Rect::from_parts(origin, Size::new(width, fit.height()));
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        self.check_text_constraints(rect, name, true)?;
        self.text_font(style)?;
        let repeat = self.repeat_binding(name);
        let binding = self.case_binding(name).map(|_| name.to_string());
        self.slots.push(text_slot_ir(actual_name, None, rect, style, repeat, binding, fit));
        Ok(rect.size())
    }

    pub(super) fn emit_sprite(&mut self, name: &str, origin: Point) -> Result<Rect> {
        let def = self.sprite_def(name)?;
        let rect = Rect::from_parts(origin, self.sprite_size(name, def)?);
        self.warn_if_overflow(rect, &format!("sprite `{name}`"));
        self.draws.push(match def {
            SpriteDef::Texture { texture, .. } => Draw::Sprite { texture: TextureKey(texture.clone()), dest: rect },
            SpriteDef::Generated { style, .. } => Draw::Generated { style: style.clone(), dest: rect },
        });
        Ok(rect)
    }

    pub(super) fn emit_frame(&mut self, frame: &str, dest: Rect, referenced_by: &str) -> Result<()> {
        let project = self.project;
        let def = project
            .theme
            .frames
            .get(frame)
            .ok_or_else(|| self.target.layout_err(format!("unknown frame `{frame}`")))?;
        self.warn_if_overflow(dest, referenced_by);
        match def {
            Frame::Texture { texture, insets } => {
                let tex_size = self.intrinsic_texture_size(texture, referenced_by)?;
                let min_w = insets.left + insets.right + 1;
                let min_h = insets.top + insets.bottom + 1;
                if tex_size.width < min_w || tex_size.height < min_h {
                    return Err(self.target.frame_too_small_err(frame, texture, tex_size, min_w, min_h));
                }
                self.draws.push(Draw::NineSlice { texture: TextureKey(texture.clone()), insets: *insets, dest });
            }
            Frame::Generated(style) => self.draws.push(Draw::Generated { style: style.clone(), dest }),
        }
        Ok(())
    }
}

fn text_slot_ir(
    name: String,
    text: Option<String>,
    rect: Rect,
    style: &TextStyle,
    repeat: Option<RepeatBindingIr>,
    binding: Option<String>,
    fit: TextFit,
) -> SlotIr {
    SlotIr {
        name,
        text,
        rect,
        align: style.align.unwrap_or(Align::Left),
        color: style.color,
        shadow: style.shadow,
        bold: style.bold,
        italic: style.italic,
        underlined: style.underlined,
        strikethrough: style.strikethrough,
        obfuscated: style.obfuscated,
        font: style.font.clone(),
        repeat,
        binding,
        fit,
    }
}
