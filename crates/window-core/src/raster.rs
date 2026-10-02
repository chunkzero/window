//! Procedural theme rasterization backed by tiny-skia.
//!
//! This module is the only place that knows about tiny-skia's premultiplied
//! pixel storage. It returns Window's straight-alpha [`Texture`] type so the
//! existing compositor can blend generated and PNG-backed assets identically.

use tiny_skia::{FillRule, PathBuilder, Pixmap, Transform};

use crate::compose::Texture;
use crate::geometry::Size;
use crate::model::{GeneratedKind, GeneratedStyle};
use crate::{Error, Result};

mod shapes;

use shapes::{
    Edge, RoundedRect, draw_rounded_edge, fade, fill_rect, fill_round_rect, fill_slanted_quad, paint, stroke_rect,
};

/// Render a generated theme style at `size`.
pub fn render(style: &GeneratedStyle, size: Size) -> Result<Texture> {
    if size.width == 0 || size.height == 0 {
        return Ok(Texture::transparent(size.width, size.height));
    }

    let mut pixmap = Pixmap::new(size.width, size.height).ok_or_else(|| Error::Texture {
        path: generated_path(style),
        message: format!("invalid generated texture size {}x{}", size.width, size.height),
    })?;

    match style.kind {
        GeneratedKind::HazardBar => draw_hazard_bar(&mut pixmap, style),
        GeneratedKind::Vent => {
            draw_panel(&mut pixmap, style);
            draw_vents(&mut pixmap, style);
        }
        GeneratedKind::Badge => {
            draw_panel(&mut pixmap, style);
            draw_badge_mark(&mut pixmap, style);
        }
        GeneratedKind::Panel | GeneratedKind::Button | GeneratedKind::Slot => {
            draw_panel(&mut pixmap, style);
        }
    }

    let mut texture = texture_from_pixmap(&pixmap);
    apply_lighting(&mut texture, style);
    Ok(texture)
}

fn draw_panel(pixmap: &mut Pixmap, style: &GeneratedStyle) {
    let w = pixmap.width() as f32;
    let h = pixmap.height() as f32;
    let border = style.border_width.min(pixmap.width().min(pixmap.height()) / 2) as f32;

    fill_round_rect(pixmap, 0.0, 0.0, w, h, style.radius as f32, style.border_color);

    if border < w && border < h {
        fill_round_rect(
            pixmap,
            border,
            border,
            w - border * 2.0,
            h - border * 2.0,
            style.radius.saturating_sub(style.border_width) as f32,
            style.fill,
        );
    }

    draw_inset(pixmap, style, border as u32);
}

fn draw_hazard_bar(pixmap: &mut Pixmap, style: &GeneratedStyle) {
    let w = pixmap.width() as f32;
    let h = pixmap.height() as f32;
    let border = style.border_width.min(pixmap.width().min(pixmap.height()) / 2) as f32;

    fill_round_rect(pixmap, 0.0, 0.0, w, h, style.radius as f32, style.border_color);

    let inner_w = w - border * 2.0;
    let inner_h = h - border * 2.0;
    if inner_w <= 0.0 || inner_h <= 0.0 {
        return;
    }
    fill_round_rect(
        pixmap,
        border,
        border,
        inner_w,
        inner_h,
        style.radius.saturating_sub(style.border_width) as f32,
        style.fill,
    );

    let Some(stripe) = style.stripe_color else {
        return;
    };
    let top = border + 1.0;
    let bottom = border + inner_h - 1.0;
    if bottom <= top {
        return;
    }

    let stripe_width = style.stripe_width.max(4) as f32;
    let period = stripe_width * 2.0;
    let slant = ((bottom - top) * 0.65).max(2.0);
    let mut x = border - slant - period;
    while x < border + inner_w + period {
        if let Some(shadow) = style.stripe_shadow_color {
            fill_slanted_quad(pixmap, x + 1.0, top + 1.0, stripe_width, bottom, slant, shadow);
        }
        fill_slanted_quad(pixmap, x, top, stripe_width, bottom, slant, stripe);
        x += period;
    }

    draw_hazard_bevel(pixmap, style, border, inner_w, inner_h);
}

fn draw_hazard_bevel(pixmap: &mut Pixmap, style: &GeneratedStyle, border: f32, inner_w: f32, inner_h: f32) {
    if let Some(highlight) = style.highlight_color {
        fill_rect(pixmap, border, border, inner_w, 1.0, highlight);
    }
    if let Some(shadow) = style.shadow_color {
        fill_rect(pixmap, border, border + inner_h - 1.0, inner_w, 1.0, shadow);
    }
    stroke_rect(pixmap, 0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32, border, style.border_color);
}

fn draw_vents(pixmap: &mut Pixmap, style: &GeneratedStyle) {
    let color = style.accent_color.unwrap_or(style.shadow_color.unwrap_or(style.border_color));
    let start_x = style.border_width.saturating_add(3);
    let end_x = pixmap.width().saturating_sub(style.border_width.saturating_add(3));
    if end_x <= start_x {
        return;
    }
    let mut y = style.border_width.saturating_add(4);
    while y + 2 < pixmap.height().saturating_sub(style.border_width.saturating_add(2)) {
        fill_rect(pixmap, start_x as f32, y as f32, (end_x - start_x) as f32, 2.0, color);
        y += 5;
    }
}

fn draw_badge_mark(pixmap: &mut Pixmap, style: &GeneratedStyle) {
    let Some(accent) = style.accent_color else {
        return;
    };
    let cx = pixmap.width() as f32 / 2.0;
    let cy = pixmap.height() as f32 / 2.0;
    let r = pixmap.width().min(pixmap.height()) as f32 * 0.25;

    let mut pb = PathBuilder::new();
    pb.move_to(cx, cy - r);
    pb.line_to(cx + r, cy);
    pb.line_to(cx, cy + r);
    pb.line_to(cx - r, cy);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &paint(accent), FillRule::Winding, Transform::identity(), None);
    }
}

fn draw_inset(pixmap: &mut Pixmap, style: &GeneratedStyle, inset: u32) {
    let depth = style.inset_depth;
    if depth == 0 {
        return;
    }
    let x = inset;
    let y = inset;
    let w = pixmap.width().saturating_sub(inset * 2);
    let h = pixmap.height().saturating_sub(inset * 2);
    if w == 0 || h == 0 {
        return;
    }
    let radius = style.radius.saturating_sub(inset) as f32;

    for i in 0..depth.min(w).min(h) {
        let rect = RoundedRect::new(x + i, y + i, w.saturating_sub(i * 2), h.saturating_sub(i * 2), radius - i as f32);
        let alpha_step = depth.saturating_sub(i);
        if let Some(highlight) = style.highlight_color {
            let color = fade(highlight, alpha_step, depth);
            draw_rounded_edge(pixmap, rect, Edge::Top, color);
            draw_rounded_edge(pixmap, rect, Edge::Left, color);
        }
        if let Some(shadow) = style.shadow_color {
            let color = fade(shadow, alpha_step, depth);
            draw_rounded_edge(pixmap, rect, Edge::Bottom, color);
            draw_rounded_edge(pixmap, rect, Edge::Right, color);
        }
    }
}

fn apply_lighting(texture: &mut Texture, style: &GeneratedStyle) {
    let w = texture.width;
    let h = texture.height;
    if w == 0 || h == 0 {
        return;
    }

    let (top_light, bottom_shadow, side_light, side_shadow) = match style.kind {
        GeneratedKind::HazardBar => (3, 8, 1, 2),
        GeneratedKind::Button | GeneratedKind::Slot => (4, 10, 1, 3),
        GeneratedKind::Vent | GeneratedKind::Badge => (4, 9, 1, 3),
        GeneratedKind::Panel => (5, 11, 1, 3),
    };

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if texture.rgba[i + 3] == 0 {
                continue;
            }

            let vertical = if h > 1 { y as i32 * (top_light + bottom_shadow) / (h - 1) as i32 } else { 0 };
            let mut delta = top_light - vertical;

            let edge = w.min(h).clamp(2, 5);
            if x < edge {
                delta += side_light * (edge - x) as i32 / edge as i32;
            }
            if x + edge >= w {
                delta -= side_shadow * (x + edge + 1 - w) as i32 / edge as i32;
            }
            if y + edge >= h {
                delta -= bottom_shadow / 3;
            }

            shade_pixel(&mut texture.rgba[i..i + 4], delta);
        }
    }
}

fn shade_pixel(px: &mut [u8], delta: i32) {
    for channel in &mut px[0..3] {
        *channel = (*channel as i32 + delta).clamp(0, 255) as u8;
    }
}

fn texture_from_pixmap(pixmap: &Pixmap) -> Texture {
    let mut rgba = Vec::with_capacity((pixmap.width() * pixmap.height() * 4) as usize);
    for pixel in pixmap.pixels() {
        let color = pixel.demultiply();
        rgba.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
    }
    Texture { width: pixmap.width(), height: pixmap.height(), rgba }
}

fn generated_path(style: &GeneratedStyle) -> String {
    format!("generated:{:?}", style.kind).to_ascii_lowercase()
}

#[cfg(test)]
mod tests;
