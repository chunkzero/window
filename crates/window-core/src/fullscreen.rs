//! The full-screen canvas contract.
//!
//! A full-screen window is drawn with no screen open. The runtime sends its glyph runs as the single line of a
//! client-only text display at the player's eye, and the core text shader moves every marked glyph onto a fixed
//! virtual canvas, letterboxed or pillarboxed to the screen. The player's rotation drives the cursor: the shader
//! reads it from the camera, and the server maps the same rotation to hit-test clicks and pick the hovered control.
//!
//! Everything both sides must agree on is defined here once and emitted twice: as the GLSL include at
//! [`INCLUDE_PATH`], and as [`crate::manifest::FullscreenCanvasEntry`] in the generated Kotlin.
//!
//! # Runs
//!
//! A full-screen window is composed like a shader-placed HUD: net-zero segments in authored layer order, positioned
//! with spacers and shifted fonts from the line origin. A glyph's canvas position is its position in that line: `x`
//! text pixels right of where the line starts, and `y` text pixels below the top of the line (where a shifted font's
//! row 0 draws). The cursor and backdrop runs start at the line origin too.
//!
//! # Markers
//!
//! The text display's text opacity is [`MARKER_ALPHA`], and every run's text color is a [`Marker`]. Its 24 RGB bits
//! are, from the most significant:
//!
//! | Bits  | Field                                                         |
//! |-------|---------------------------------------------------------------|
//! | 23-20 | [`MARKER_SIGNATURE`]                                          |
//! | 19-18 | [`MarkerKind`]                                                |
//! | 17-10 | layer: depth order, higher draws over lower                   |
//! | 9-0   | color index into the include's `WINDOW_FS_COLORS`             |
//!
//! The shader multiplies the glyph's texels by the indexed color, so text glyphs take the color of their slot or
//! tint, and art takes its tint or white. Text shadows recolor a glyph's vertices, so marked runs are sent without
//! shadows. Every marker color is computed by the compiler; the runtime sends the ones the manifest lists.

use std::fmt::Write as _;

use crate::compose::Texture;
use crate::geometry::{Point, Rect, Size};
use crate::ir::Rgb;

pub mod horse;

/// The virtual canvas size in canvas pixels, which full-screen windows are laid out on.
pub const CANVAS: Size = Size::new(480, 270);

/// Degrees of yaw or pitch per canvas pixel; the canvas spans 90 degrees of yaw.
pub const DEGREES_PER_PIXEL: f64 = 0.1875;

/// The yaw from 0 at which the cursor reaches the left or right canvas edge: half the canvas width in degrees.
pub const EDGE_YAW: f64 = CANVAS.width as f64 / 2.0 * DEGREES_PER_PIXEL;

/// The pitch from 0 at which the cursor reaches the top or bottom canvas edge: half the canvas height in degrees.
pub const EDGE_PITCH: f64 = CANVAS.height as f64 / 2.0 * DEGREES_PER_PIXEL;

/// Added before flooring a cursor coordinate, so a rotation landing on a pixel boundary snaps the same way in the
/// shader's single precision and the server's double precision.
pub const SNAP_EPSILON: f64 = 0.01;

/// Canvas pixels a hit-test grows each control by, so a click one pixel outside a hovered control still lands.
pub const HIT_MARGIN: u32 = 1;

/// The uniform scale of the text display carrying the runs. A text pixel is `0.025 * DISPLAY_SCALE` blocks, which
/// makes a few blocks of drift between the display and the eye round away in the shader.
pub const DISPLAY_SCALE: f64 = 256.0;

/// The text opacity of the text display carrying the runs, which marks its glyphs for the shader.
pub const MARKER_ALPHA: u8 = 253;

/// The top four bits of every marker color.
pub const MARKER_SIGNATURE: u32 = 0xA;

/// The layer of a backdrop run, below everything else.
pub const BACKDROP_LAYER: u8 = 0;

/// The layer of a window's static art.
pub const STATIC_LAYER: u8 = 1;

/// The highest layer of a window's runtime layers, which take the layers from `STATIC_LAYER + 1` in authored order.
pub const MAX_CONTENT_LAYER: u8 = 253;

/// The layer of the runtime's own overlays, such as server-driven hover outlines, above every window layer.
pub const OVERLAY_LAYER: u8 = 254;

/// The layer of the cursor, above everything else.
pub const CURSOR_LAYER: u8 = 255;

/// How many colors the include's color table holds at most: what the marker's color index addresses.
pub const MAX_COLORS: usize = 1 << 10;

/// The world age the server sends while a full-screen window hides the HUD. Minecraft's `GameTime` is the world age
/// modulo 24000 over 24000 with Java's remainder, so it is negative, and a negative `GameTime` hides the HUD strip.
/// The client ticks the age up between time packets, and this value stays 16000 ticks clear of the next multiple of
/// 24000.
pub const HUD_HIDDEN_WORLD_AGE: i64 = -1_000_000_001;

/// The height of the HUD strip in GUI pixels, from the bottom of the screen: the hotbar, XP bar, hearts, armour, and
/// hunger.
pub const HUD_STRIP_HEIGHT: u32 = 56;

/// Half the width of the HUD strip in GUI pixels, either side of the screen's center.
pub const HUD_STRIP_HALF_WIDTH: u32 = 96;

/// The most max health the client is told while the HUD is hidden, so its hearts fit in one row inside the strip.
pub const HUD_MAX_HEALTH: u32 = 20;

/// The alpha of the opaque texels in the pack's override of the vanilla crosshair sprite. While the HUD is hidden,
/// `core/position_tex_color` discards GUI texels with this alpha, which removes the crosshair without cutting
/// anything else. With the crosshair's inverting blend, 254 draws the same as 255.
pub const CROSSHAIR_MARKER_ALPHA: u8 = 254;

/// The sprite the crosshair pipeline draws, relative to the pack root.
pub const CROSSHAIR_SPRITE_PATH: &str = "assets/minecraft/textures/gui/sprites/hud/crosshair.png";

/// The distance in normalized device depth between adjacent layers: 2^-20, sixteen ulps of the 32-bit float depth
/// buffer just below 1.0, so every layer's depth is exact in f32 and distinct once stored.
pub const LAYER_DEPTH_STEP: f64 = 1.0 / 1_048_576.0;

/// The normalized device depth of a layer, written to `gl_Position.z` with `w` of 1. The level projection is
/// reversed, with the near plane at 1.0 and the far plane at or below 0.0, so depth is about near / distance. All
/// layers sit within 256 steps below the near plane, covering only geometry within about 1e-5 blocks of it, and at
/// least one step below it, so the camera's tilt, bobbing, nausea, and FOV, which only transform what the projection
/// maps, cannot clip them. Higher layers are nearer.
pub fn layer_depth(layer: u8) -> f64 {
    1.0 - (f64::from(CURSOR_LAYER - layer) + 1.0) * LAYER_DEPTH_STEP
}

/// Where the include is emitted; core shaders import it as `#moj_import <minecraft:window_fullscreen.glsl>`.
pub const INCLUDE_PATH: &str = "assets/minecraft/shaders/include/window_fullscreen.glsl";

/// How the shader places a marked glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerKind {
    /// At its canvas position: its position in the line.
    Content = 0,
    /// At the cursor: its position in the line plus the cursor's canvas point, so the cursor glyph's top-left
    /// texel is the hot spot.
    Cursor = 1,
    /// Over the whole screen, letterbox included: each corner of the glyph moves to the matching screen corner. The
    /// built-in backdrop glyph's left and top edges lie on the line origin.
    Backdrop = 2,
}

/// A run's text color: what the shader decodes from each of its vertices. See the module docs for the layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marker {
    pub kind: MarkerKind,
    pub layer: u8,
    /// An index into the color table, below [`MAX_COLORS`].
    pub color: u16,
}

impl Marker {
    /// The marker's text color.
    pub fn rgb(self) -> Rgb {
        debug_assert!((self.color as usize) < MAX_COLORS);
        let bits =
            MARKER_SIGNATURE << 20 | (self.kind as u32) << 18 | u32::from(self.layer) << 10 | u32::from(self.color);
        Rgb::new((bits >> 16) as u8, (bits >> 8) as u8, bits as u8)
    }
}

/// The cursor's canvas point for a rotation from yaw and pitch 0, which the session faces the player on open.
///
/// Yaw wraps into `[-180, 180)`, so the seam behind the player lies past the left edge. Each coordinate is
/// `floor(canvas / 2 + angle / DEGREES_PER_PIXEL + SNAP_EPSILON)`, clamped to the canvas. The include's
/// `window_fs_cursor_at` computes the same.
pub fn cursor(yaw: f64, pitch: f64) -> Point {
    let yaw = yaw - 360.0 * ((yaw + 180.0) / 360.0).floor();
    let axis = |size: u32, angle: f64| {
        let pixel = (f64::from(size) / 2.0 + angle / DEGREES_PER_PIXEL + SNAP_EPSILON).floor();
        pixel.clamp(0.0, f64::from(size) - 1.0) as i32
    };
    Point::new(axis(CANVAS.width, yaw), axis(CANVAS.height, pitch))
}

/// The built-in cursor: an arrow whose top-left texel is the hot spot. `B` is black, `W` white.
const CURSOR_ROWS: [&str; 12] = [
    "B.......", "BB......", "BWB.....", "BWWB....", "BWWWB...", "BWWWWB..", "BWWWWWB.", "BWWWWWWB", "BWWWBBBB",
    "BWBWB...", "BB.BWB..", "B...BB..",
];

/// The built-in cursor glyph's texture.
pub(crate) fn cursor_texture() -> Texture {
    let mut texture = Texture::transparent(CURSOR_ROWS[0].len() as u32, CURSOR_ROWS.len() as u32);
    for (y, row) in CURSOR_ROWS.iter().enumerate() {
        for (x, pixel) in row.bytes().enumerate() {
            let rgb = match pixel {
                b'B' => 0x00,
                b'W' => 0xff,
                _ => continue,
            };
            let at = (y * texture.width as usize + x) * 4;
            texture.rgba[at..at + 4].copy_from_slice(&[rgb, rgb, rgb, 0xff]);
        }
    }
    texture
}

const CROSSHAIR_CENTER: u32 = 7;
const CROSSHAIR_ARM: u32 = 4;

/// The vanilla crosshair sprite, a 15x15 white plus, with its opaque texels at [`CROSSHAIR_MARKER_ALPHA`].
pub(crate) fn crosshair_marker_texture() -> Texture {
    let mut texture = Texture::transparent(15, 15);
    for y in 0..texture.height {
        for x in 0..texture.width {
            let (dx, dy) = (x.abs_diff(CROSSHAIR_CENTER), y.abs_diff(CROSSHAIR_CENTER));
            if (dx == 0 && dy <= CROSSHAIR_ARM) || (dy == 0 && dx <= CROSSHAIR_ARM) {
                let at = (y * texture.width + x) as usize * 4;
                texture.rgba[at..at + 4].copy_from_slice(&[0xff, 0xff, 0xff, CROSSHAIR_MARKER_ALPHA]);
            }
        }
    }
    texture
}

/// The built-in backdrop glyph's texture: solid white, which the shader stretches over the screen. Its left and top
/// corners lie on the line origin.
pub(crate) fn backdrop_texture() -> Texture {
    let mut texture = Texture::transparent(8, 8);
    texture.rgba.fill(0xff);
    texture
}

/// The include: the contract's constants, the color table `colors`, the control rects `controls` in canvas pixels,
/// and the functions decoding markers and mapping rotation to the cursor.
pub(crate) fn include(colors: &[Rgb], controls: &[Rect], cursor_size: Size) -> String {
    let rgb = |color: &Rgb| format!("vec3({}.0, {}.0, {}.0)", color.r, color.g, color.b);
    let colors_glsl = colors.iter().map(rgb).collect::<Vec<_>>().join(", ");
    let rects: Vec<String> = controls
        .iter()
        .map(|rect| format!("vec4({}.0, {}.0, {}.0, {}.0)", rect.x, rect.y, rect.width, rect.height))
        .collect();
    let rects = if rects.is_empty() { "vec4(0.0)".to_string() } else { rects.join(", ") };
    let mut out = String::new();
    let _ = write!(
        out,
        "\
// Generated by Window: the full-screen canvas contract. Mirrors the full-screen canvas entry of the generated Kotlin.
// A marker's RGB bits: 23-20 signature, 19-18 kind, 17-10 layer, 9-0 color index.

const vec2 WINDOW_FS_CANVAS = vec2({width}.0, {height}.0);
const float WINDOW_FS_DEGREES_PER_PIXEL = {dpp:?};
// The yaw and pitch from 0 at which the cursor reaches the canvas edges.
const vec2 WINDOW_FS_EDGE_ANGLES = vec2({edge_yaw:?}, {edge_pitch:?});
const float WINDOW_FS_SNAP_EPSILON = {epsilon:?};
// Blocks per text pixel on the text display carrying the runs.
const float WINDOW_FS_BLOCKS_PER_PIXEL = {blocks_per_pixel:?};

const int WINDOW_FS_MARKER_ALPHA = {alpha};
const int WINDOW_FS_MARKER_SIGNATURE = {signature};
const int WINDOW_FS_KIND_CONTENT = {content};
const int WINDOW_FS_KIND_CURSOR = {cursor};
const int WINDOW_FS_KIND_BACKDROP = {backdrop};
const int WINDOW_FS_BACKDROP_LAYER = {backdrop_layer};
const int WINDOW_FS_OVERLAY_LAYER = {overlay_layer};
const int WINDOW_FS_CURSOR_LAYER = {cursor_layer};
const vec2 WINDOW_FS_CURSOR_SIZE = vec2({cursor_width}.0, {cursor_height}.0);

// While GameTime is negative, GUI sprites inside this bottom-center strip are hidden: (half width, height) in GUI
// pixels.
const vec2 WINDOW_FS_HUD_STRIP = vec2({strip_half_width}.0, {strip_height}.0);

// Marker colors, 0-255 per channel; index 0 is white.
const int WINDOW_FS_COLOR_COUNT = {color_count};
const vec3 WINDOW_FS_COLORS[{color_count}] = vec3[]({colors_glsl});

// Every full-screen control rect (x, y, width, height) in canvas pixels, by the control's manifest index.
const int WINDOW_FS_CONTROL_COUNT = {control_count};
const vec4 WINDOW_FS_CONTROLS[{control_slots}] = vec4[]({rects});

int window_fs_payload(vec4 color) {{
    ivec3 rgb = ivec3(round(color.rgb * 255.0));
    return (rgb.r << 16) | (rgb.g << 8) | rgb.b;
}}

// Whether a text vertex color is a full-screen marker.
bool window_fs_is_marker(vec4 color) {{
    return int(round(color.a * 255.0)) == WINDOW_FS_MARKER_ALPHA
        && (window_fs_payload(color) >> 20) == WINDOW_FS_MARKER_SIGNATURE;
}}

int window_fs_kind(int payload) {{
    return (payload >> 18) & 3;
}}

int window_fs_layer(int payload) {{
    return (payload >> 10) & 255;
}}

// The color a marked glyph's texels are multiplied by.
vec3 window_fs_color(int payload) {{
    return WINDOW_FS_COLORS[min(payload & 1023, WINDOW_FS_COLOR_COUNT - 1)] / 255.0;
}}

// The cursor's canvas point for a rotation from yaw and pitch 0. Yaw wraps into [-180, 180), and each coordinate
// floors, then clamps to the canvas, as the server's mapping does.
vec2 window_fs_cursor_at(float yaw, float pitch) {{
    yaw -= 360.0 * floor((yaw + 180.0) / 360.0);
    vec2 cursor = floor(WINDOW_FS_CANVAS * 0.5 + vec2(yaw, pitch) / WINDOW_FS_DEGREES_PER_PIXEL + WINDOW_FS_SNAP_EPSILON);
    return clamp(cursor, vec2(0.0), WINDOW_FS_CANVAS - 1.0);
}}
",
        width = CANVAS.width,
        height = CANVAS.height,
        dpp = DEGREES_PER_PIXEL,
        edge_yaw = EDGE_YAW,
        edge_pitch = EDGE_PITCH,
        epsilon = SNAP_EPSILON,
        blocks_per_pixel = 0.025 * DISPLAY_SCALE,
        alpha = MARKER_ALPHA,
        signature = MARKER_SIGNATURE,
        content = MarkerKind::Content as u8,
        cursor = MarkerKind::Cursor as u8,
        backdrop = MarkerKind::Backdrop as u8,
        backdrop_layer = BACKDROP_LAYER,
        overlay_layer = OVERLAY_LAYER,
        cursor_layer = CURSOR_LAYER,
        cursor_width = cursor_size.width,
        cursor_height = cursor_size.height,
        strip_half_width = HUD_STRIP_HALF_WIDTH,
        strip_height = HUD_STRIP_HEIGHT,
        color_count = colors.len(),
        control_count = controls.len(),
        control_slots = controls.len().max(1),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_pack_signature_kind_layer_and_color() {
        let marker = Marker { kind: MarkerKind::Cursor, layer: CURSOR_LAYER, color: 0x3ff };
        assert_eq!(marker.rgb(), Rgb::new(0xa7, 0xff, 0xff));
        let marker = Marker { kind: MarkerKind::Content, layer: STATIC_LAYER, color: 2 };
        assert_eq!(marker.rgb(), Rgb::new(0xa0, 0x04, 0x02));
    }

    #[test]
    fn cursor_reaches_the_edges_at_the_edge_angles_and_wraps_yaw() {
        assert_eq!(cursor(0.0, 0.0), Point::new(240, 135));
        assert_eq!(cursor(EDGE_YAW, EDGE_PITCH), Point::new(479, 269));
        assert_eq!(cursor(-EDGE_YAW, -EDGE_PITCH), Point::new(0, 0));
        // Just short of a pixel boundary still snaps onto it.
        assert_eq!(cursor(DEGREES_PER_PIXEL - 0.001, 0.0), Point::new(241, 135));
        assert_eq!(cursor(360.0 + DEGREES_PER_PIXEL, 0.0), Point::new(241, 135));
        assert_eq!(cursor(179.0, 0.0), Point::new(479, 135));
    }
}
