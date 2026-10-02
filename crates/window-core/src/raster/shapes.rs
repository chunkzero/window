//! Pixel-level drawing primitives shared by the generated-style renderers.

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Rect as SkRect, Transform};

use crate::ir::Rgb;
#[derive(Clone, Copy)]
pub(super) struct RoundedRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    r: f32,
}

impl RoundedRect {
    pub(super) fn new(x: u32, y: u32, w: u32, h: u32, r: f32) -> Self {
        let r = r.max(0.0).min(w as f32 / 2.0).min(h as f32 / 2.0);
        Self { x, y, w, h, r }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Edge {
    Top,
    Left,
    Bottom,
    Right,
}

pub(super) fn draw_rounded_edge(pixmap: &mut Pixmap, rect: RoundedRect, edge: Edge, color: Rgb) {
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

pub(super) fn fill_round_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Rgb) {
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

pub(super) fn fill_slanted_quad(
    pixmap: &mut Pixmap,
    x: f32,
    top: f32,
    stripe_width: f32,
    bottom: f32,
    slant: f32,
    color: Rgb,
) {
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

pub(super) fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
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

pub(super) fn stroke_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, width: f32, color: Rgb) {
    if width <= 0.0 || w <= 0.0 || h <= 0.0 {
        return;
    }
    fill_rect(pixmap, x, y, w, width.min(h), color);
    fill_rect(pixmap, x, y + h - width.min(h), w, width.min(h), color);
    fill_rect(pixmap, x, y, width.min(w), h, color);
    fill_rect(pixmap, x + w - width.min(w), y, width.min(w), h, color);
}

pub(super) fn paint(color: Rgb) -> Paint<'static> {
    let mut paint = Paint { anti_alias: false, ..Paint::default() };
    paint.set_color_rgba8(color.r, color.g, color.b, 255);
    paint
}

pub(super) fn fade(color: Rgb, step: u32, total: u32) -> Rgb {
    if total == 0 {
        return color;
    }
    let mix = |channel: u8| -> u8 {
        let value = channel as u32 * step / total;
        value.min(255) as u8
    };
    Rgb::new(mix(color.r), mix(color.g), mix(color.b))
}
