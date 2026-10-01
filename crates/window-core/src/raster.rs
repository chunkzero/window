//! Procedural theme rasterization backed by tiny-skia.
//!
//! This module is the only place that knows about tiny-skia's premultiplied
//! pixel storage. It returns Window's straight-alpha [`Texture`] type so the
//! existing compositor can blend generated and PNG-backed assets identically.

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Rect as SkRect, Transform};

use crate::compose::Texture;
use crate::geometry::Size;
use crate::ir::Rgb;
use crate::model::{GeneratedKind, GeneratedStyle};
use crate::{Error, Result};

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

    if let Some(highlight) = style.highlight_color {
        fill_rect(pixmap, border, border, inner_w, 1.0, highlight);
    }
    if let Some(shadow) = style.shadow_color {
        fill_rect(pixmap, border, border + inner_h - 1.0, inner_w, 1.0, shadow);
    }
    stroke_rect(pixmap, 0.0, 0.0, w, h, border, style.border_color);
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

#[derive(Clone, Copy)]
struct RoundedRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    r: f32,
}

impl RoundedRect {
    fn new(x: u32, y: u32, w: u32, h: u32, r: f32) -> Self {
        let r = r.max(0.0).min(w as f32 / 2.0).min(h as f32 / 2.0);
        Self { x, y, w, h, r }
    }
}

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Left,
    Bottom,
    Right,
}

fn draw_rounded_edge(pixmap: &mut Pixmap, rect: RoundedRect, edge: Edge, color: Rgb) {
    if rect.w == 0 || rect.h == 0 {
        return;
    }

    match edge {
        Edge::Top => {
            let y = rect.y;
            for x in rect.x..rect.x + rect.w {
                set_if_in_round_rect(pixmap, rect, x, y, color);
            }
        }
        Edge::Left => {
            let x = rect.x;
            for y in rect.y..rect.y + rect.h {
                set_if_in_round_rect(pixmap, rect, x, y, color);
            }
        }
        Edge::Bottom => {
            let y = rect.y + rect.h - 1;
            for x in rect.x..rect.x + rect.w {
                set_if_in_round_rect(pixmap, rect, x, y, color);
            }
        }
        Edge::Right => {
            let x = rect.x + rect.w - 1;
            for y in rect.y..rect.y + rect.h {
                set_if_in_round_rect(pixmap, rect, x, y, color);
            }
        }
    }
}

fn set_if_in_round_rect(pixmap: &mut Pixmap, rect: RoundedRect, x: u32, y: u32, color: Rgb) {
    if contains_round_rect_pixel(rect, x, y) {
        set_pixel(pixmap, x, y, color);
    }
}

fn contains_round_rect_pixel(rect: RoundedRect, px: u32, py: u32) -> bool {
    if px < rect.x || py < rect.y || px >= rect.x + rect.w || py >= rect.y + rect.h {
        return false;
    }
    if rect.r < 0.5 {
        return true;
    }

    let x = px as f32 + 0.5;
    let y = py as f32 + 0.5;
    let left = rect.x as f32 + rect.r;
    let top = rect.y as f32 + rect.r;
    let right = (rect.x + rect.w) as f32 - rect.r;
    let bottom = (rect.y + rect.h) as f32 - rect.r;
    let nearest_x = x.clamp(left, right);
    let nearest_y = y.clamp(top, bottom);
    let dx = x - nearest_x;
    let dy = y - nearest_y;
    dx * dx + dy * dy <= rect.r * rect.r
}

fn fill_round_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Rgb) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let radius = radius.max(0.0).min(w / 2.0).min(h / 2.0);
    let path = if radius < 0.5 {
        let Some(rect) = SkRect::from_xywh(x, y, w, h) else {
            return;
        };
        PathBuilder::from_rect(rect)
    } else {
        round_rect_path(x, y, w, h, radius)
    };
    pixmap.fill_path(&path, &paint(color), FillRule::Winding, Transform::identity(), None);
}

fn round_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> tiny_skia::Path {
    let right = x + w;
    let bottom = y + h;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(right - r, y);
    pb.quad_to(right, y, right, y + r);
    pb.line_to(right, bottom - r);
    pb.quad_to(right, bottom, right - r, bottom);
    pb.line_to(x + r, bottom);
    pb.quad_to(x, bottom, x, bottom - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish().expect("rounded rect path has content")
}

fn fill_slanted_quad(pixmap: &mut Pixmap, x: f32, top: f32, stripe_width: f32, bottom: f32, slant: f32, color: Rgb) {
    let mut pb = PathBuilder::new();
    pb.move_to(x + slant, top);
    pb.line_to(x + slant + stripe_width, top);
    pb.line_to(x + stripe_width, bottom);
    pb.line_to(x, bottom);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, &paint(color), FillRule::Winding, Transform::identity(), None);
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
    let Some(rect) = SkRect::from_xywh(x, y, w, h) else {
        return;
    };
    pixmap.fill_rect(rect, &paint(color), Transform::identity(), None);
}

fn set_pixel(pixmap: &mut Pixmap, x: u32, y: u32, color: Rgb) {
    if x >= pixmap.width() || y >= pixmap.height() {
        return;
    }
    let i = (y * pixmap.width() + x) as usize;
    pixmap.pixels_mut()[i] = PremultipliedColorU8::from_rgba(color.r, color.g, color.b, 255)
        .expect("opaque RGB is a valid premultiplied color");
}

fn stroke_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, width: f32, color: Rgb) {
    if width <= 0.0 || w <= 0.0 || h <= 0.0 {
        return;
    }
    fill_rect(pixmap, x, y, w, width.min(h), color);
    fill_rect(pixmap, x, y + h - width.min(h), w, width.min(h), color);
    fill_rect(pixmap, x, y, width.min(w), h, color);
    fill_rect(pixmap, x + w - width.min(w), y, width.min(w), h, color);
}

fn paint(color: Rgb) -> Paint<'static> {
    let mut paint = Paint { anti_alias: false, ..Paint::default() };
    paint.set_color_rgba8(color.r, color.g, color.b, 255);
    paint
}

fn fade(color: Rgb, step: u32, total: u32) -> Rgb {
    if total == 0 {
        return color;
    }
    let mix = |channel: u8| -> u8 {
        let value = channel as u32 * step / total;
        value.min(255) as u8
    };
    Rgb::new(mix(color.r), mix(color.g), mix(color.b))
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
mod tests {
    use super::*;

    fn rgba_at(texture: &Texture, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * texture.width + x) * 4) as usize;
        [texture.rgba[i], texture.rgba[i + 1], texture.rgba[i + 2], texture.rgba[i + 3]]
    }

    #[test]
    fn panel_renders_border_and_fill() {
        let style = GeneratedStyle::defaults(GeneratedKind::Panel);
        let texture = render(&style, Size::new(20, 12)).unwrap();
        assert_eq!(texture.width, 20);
        assert_eq!(texture.height, 12);
        let border = 10 * 4;
        assert_eq!(texture.rgba[border + 3], 255);
        assert!(texture.rgba[border] >= style.border_color.r);

        let top = ((3 * texture.width + 10) * 4) as usize;
        let bottom = ((10 * texture.width + 10) * 4) as usize;
        assert!(texture.rgba[top] > texture.rgba[bottom]);
    }

    #[test]
    fn inset_bevel_respects_rounded_corners() {
        let mut style = GeneratedStyle::defaults(GeneratedKind::Panel);
        style.fill = Rgb::new(10, 10, 10);
        style.border_color = Rgb::new(20, 20, 20);
        style.border_width = 1;
        style.radius = 6;
        style.inset_depth = 3;
        style.highlight_color = Some(Rgb::new(240, 240, 240));
        style.shadow_color = Some(Rgb::new(30, 30, 30));

        let mut pixmap = Pixmap::new(20, 20).unwrap();
        draw_panel(&mut pixmap, &style);
        let texture = texture_from_pixmap(&pixmap);

        assert_eq!(rgba_at(&texture, 2, 1), [20, 20, 20, 255]);
        assert_eq!(rgba_at(&texture, 17, 18), [20, 20, 20, 255]);
        assert_eq!(rgba_at(&texture, 8, 1), [240, 240, 240, 255]);
        assert_eq!(rgba_at(&texture, 1, 8), [240, 240, 240, 255]);
        assert_eq!(rgba_at(&texture, 18, 8), [30, 30, 30, 255]);
        assert_eq!(rgba_at(&texture, 8, 18), [30, 30, 30, 255]);
    }

    #[test]
    fn hazard_bar_contains_stripes() {
        let style = GeneratedStyle::defaults(GeneratedKind::HazardBar);
        let texture = render(&style, Size::new(64, 12)).unwrap();
        let stripe_pixels = texture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[0] > 150 && (50..140).contains(&px[1]) && px[2] < 40 && px[3] == 255)
            .count();
        assert!(stripe_pixels > 0, "hazard stripes should be visible");
    }
}
