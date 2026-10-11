use super::Solver;
use super::target::LayoutTarget;
use crate::Result;
use crate::geometry::{Point, Rect, Size};
use crate::ir::{Align, ArtIr, Draw, Layer, SlotIr, SpriteSlotIr, TextureKey};
use crate::model::{Bar, Frame, SpriteDef, TextFit, TextStyle, Tint};
use crate::pipeline::texture_source_path;

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn place_sprite_slot(
        &mut self,
        name: &str,
        rect: Rect,
        align: Align,
        tint: Option<String>,
    ) -> Result<Size> {
        self.require_sprite_slots(name)?;
        self.register_name(name)?;
        self.check_inside_bounds(rect, &format!("sprite slot `{name}`"))?;
        self.layers.push(Layer::SpriteSlot(name.to_string()));
        self.sprite_slots.push(SpriteSlotIr { name: name.to_string(), rect, align, tint, source: None });
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
        self.layers.push(Layer::Slot(name.clone()));
        self.slots.push(text_slot_ir(name, Some(text.to_string()), rect, style, TextFit::default()));
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
        self.register_name(name)?;
        self.check_text_constraints(rect, name, true)?;
        self.text_font(style)?;
        self.layers.push(Layer::Slot(name.to_string()));
        self.slots.push(text_slot_ir(name.to_string(), None, rect, style, fit));
        Ok(rect.size())
    }

    pub(super) fn emit_sprite(&mut self, name: &str, origin: Point, tint: Option<&Tint>) -> Result<Rect> {
        let def = self.sprite_def(name)?;
        let rect = Rect::from_parts(origin, self.sprite_size(name, def)?);
        self.warn_if_overflow(rect, &format!("image `{name}`"));
        let draw = match def {
            SpriteDef::Texture { texture, .. } => {
                Draw::Sprite { texture: TextureKey(texture_source_path(texture)), dest: rect }
            }
            SpriteDef::Generated { style, .. } => Draw::Generated { style: style.clone(), dest: rect },
        };
        self.emit_draw(draw, tint);
        Ok(rect)
    }

    /// Adds `draw` to the pane or switch case being emitted.
    ///
    /// Art tinted from a palette becomes an art layer, drawn at its authored place among the runtime layers. Untinted
    /// art authored after it that covers an art layer cannot join the composite underneath, so it collects in an art
    /// layer of its own, placed where its first draw was authored.
    fn emit_draw(&mut self, draw: Draw, tint: Option<&Tint>) {
        let (draw, palette) = match tint {
            Some(Tint::Color(color)) => (Draw::Tinted { draw: Box::new(draw), color: *color }, None),
            Some(Tint::Palette(id)) => (draw, Some(id.clone())),
            None => (draw, None),
        };
        let dest = *draw.dest();
        if palette.is_none() && !self.art_run.covered.iter().any(|rect| rect.intersects(&dest)) {
            self.draws.push(draw);
            return;
        }
        let open = if palette.is_some() { self.art_run.tinted } else { self.art_run.untinted }.filter(|&index| {
            let art = &self.art[index];
            art.tint == palette && self.layers.last() == Some(&Layer::Art(art.name.clone()))
        });
        let index = open.unwrap_or_else(|| {
            let name = format!("art/{}", self.art.len());
            self.layers.push(Layer::Art(name.clone()));
            self.art.push(ArtIr { name, draws: Vec::new(), tint: palette.clone(), bar: None });
            self.art.len() - 1
        });
        self.art[index].draws.push(draw);
        self.art_run.covered.push(dest);
        match palette {
            Some(_) => {
                self.art_run.tinted = Some(index);
                self.art_run.untinted = None;
            }
            None => self.art_run.untinted = Some(index),
        }
    }

    /// Places bar `name` over `rect`: its track in the art underneath, and its fill as an art layer of its own
    /// that the runtime draws up to the bar's fraction.
    pub(super) fn place_bar(&mut self, bar: &Bar, rect: Rect) -> Result<Size> {
        let name = &bar.name;
        self.register_name(name)?;
        if let Some(track) = &bar.track {
            self.emit_frame(track, rect, None, &format!("bar `{name}` track"))?;
        }
        let draw = self.frame_draw(&bar.fill, rect, &format!("bar `{name}` fill"))?;
        let (draw, tint) = match &bar.tint {
            Some(Tint::Color(color)) => (Draw::Tinted { draw: Box::new(draw), color: *color }, None),
            Some(Tint::Palette(id)) => (draw, Some(id.clone())),
            None => (draw, None),
        };
        let art = format!("art/{}", self.art.len());
        self.layers.push(Layer::Art(art.clone()));
        self.art.push(ArtIr { name: art, draws: vec![draw], tint, bar: Some(name.clone()) });
        self.art_run.covered.push(rect);
        self.art_run.untinted = None;
        Ok(rect.size())
    }

    /// Draws `frame` stretched over `dest`, after checking the frame and warning about overflow.
    pub(super) fn emit_frame(
        &mut self,
        frame: &str,
        dest: Rect,
        tint: Option<&Tint>,
        referenced_by: &str,
    ) -> Result<()> {
        let draw = self.frame_draw(frame, dest, referenced_by)?;
        self.emit_draw(draw, tint);
        Ok(())
    }

    /// `frame` stretched over `dest`, after checking the frame and warning about overflow.
    fn frame_draw(&mut self, frame: &str, dest: Rect, referenced_by: &str) -> Result<Draw> {
        let project = self.project;
        let def =
            project.art.frames.get(frame).ok_or_else(|| self.target.layout_err(format!("unknown frame `{frame}`")))?;
        self.warn_if_overflow(dest, referenced_by);
        Ok(match def {
            Frame::Texture { texture, insets } => {
                let tex_size = self.intrinsic_texture_size(texture, referenced_by)?;
                let min_w = insets.left + insets.right + 1;
                let min_h = insets.top + insets.bottom + 1;
                if tex_size.width < min_w || tex_size.height < min_h {
                    return Err(self.target.frame_too_small_err(frame, texture, tex_size, min_w, min_h));
                }
                Draw::NineSlice { texture: TextureKey(texture_source_path(texture)), insets: *insets, dest }
            }
            Frame::Generated(style) => Draw::Generated { style: style.clone(), dest },
        })
    }
}

fn text_slot_ir(name: String, text: Option<String>, rect: Rect, style: &TextStyle, fit: TextFit) -> SlotIr {
    SlotIr {
        name,
        text,
        rect,
        align: style.align,
        color: style.color,
        shadow: style.shadow,
        bold: style.bold,
        italic: style.italic,
        underlined: style.underlined,
        strikethrough: style.strikethrough,
        obfuscated: style.obfuscated,
        font: style.font.clone(),
        fit,
        tint: style.tint.clone(),
        source: None,
    }
}
