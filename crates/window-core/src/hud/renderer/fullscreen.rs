//! Full-screen snippets of the pack_format 85-88 `core/text` vertex shader, and the `core/position_tex_color`
//! shaders that hide the HUD, the vanilla crosshair, and the attack indicators while a full-screen window is open, and the keyboard prompt's
//! anvil while it is open (see [`super::prompt`]).
//!
//! The text shader moves every vertex of a full-screen marker (see [`crate::fullscreen`]) in the world pass straight
//! into clip space on the letterboxed canvas, so the projection, FOV, and view bobbing never move it. Unmarked text
//! takes the vanilla path.

use crate::fullscreen::{CROSSHAIR_MARKER_ALPHA, HUD_STRIP_HALF_WIDTH, HUD_STRIP_HEIGHT, LAYER_DEPTH_STEP};

use super::prompt::{PROMPT_SPRITE_FRAGMENT_CLIP, render_prompt_fragment_helpers, render_prompt_vertex_helpers};

/// Imports the canvas contract into the world variant of the text vertex shader.
pub(super) const FULLSCREEN_VERTEX_IMPORT: &str = "
#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)
#moj_import <minecraft:window_fullscreen.glsl>
#endif";

/// Runs first in the vertex `main`, placing a marked glyph's vertex and skipping the rest.
pub(super) const FULLSCREEN_VERTEX_APPLY: &str = "
#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)
    if (window_fs_is_marker(Color)) {
        window_fs_apply();
        return;
    }
#endif";

/// Places a canvas point on the screen and a layer at its depth: `window_fs_ndc` and `window_fs_depth`. Needs
/// `ScreenSize`, the include, and `WINDOW_FS_LAYER_DEPTH_STEP`.
pub(super) const CANVAS_PLACEMENT: &str = r#"// A canvas point in normalized device coordinates, with the canvas letterboxed or pillarboxed to the screen.
vec2 window_fs_ndc(vec2 canvas) {
    float screenAspect = ScreenSize.x / ScreenSize.y;
    float canvasAspect = WINDOW_FS_CANVAS.x / WINDOW_FS_CANVAS.y;
    vec2 fit = screenAspect > canvasAspect ? vec2(canvasAspect / screenAspect, 1.0) : vec2(1.0, screenAspect / canvasAspect);
    vec2 unit = canvas / WINDOW_FS_CANVAS * 2.0 - 1.0;
    return vec2(unit.x, -unit.y) * fit;
}

// The depth of a layer in normalized device coordinates, nearer for higher layers. The level projection is reversed,
// with the near plane at 1.0, and the layers fill the 256 steps just below it, so world geometry covers one only
// within a negligible distance of the near plane. ProjMat is not consulted, since it carries
// the hurt tilt, view bobbing, and nausea, which can move a projected point past the near plane and clip it.
float window_fs_depth(int layer) {
    return 1.0 - float(WINDOW_FS_CURSOR_LAYER - layer + 1) * WINDOW_FS_LAYER_DEPTH_STEP;
}"#;

const VERTEX_HELPERS: &str = r#"
#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)
// The top-left of the text display's single zero-width line in its text space, where a shifted font's row 0 draws.
const vec2 WINDOW_FS_LINE_ORIGIN = vec2(1.0, -9.0);
// The depth distance between adjacent layers.
const float WINDOW_FS_LAYER_DEPTH_STEP = @DEPTH_STEP@;
// Below every fog start, so marked glyphs are never fogged.
const float WINDOW_FS_NO_FOG_DISTANCE = -1.0e30;

// The camera's yaw and pitch in degrees. ModelViewMat is the camera rotation: its first row is the camera's right
// vector, which stays level looking straight up or down, and its third row is the negated look vector.
vec2 window_fs_camera_rotation() {
    vec3 right = vec3(ModelViewMat[0][0], ModelViewMat[1][0], ModelViewMat[2][0]);
    float lookY = -ModelViewMat[1][2];
    return vec2(degrees(atan(-right.z, -right.x)), degrees(asin(clamp(-lookY, -1.0, 1.0))));
}

@PLACEMENT@

void window_fs_apply() {
    int payload = window_fs_payload(Color);
    int kind = window_fs_kind(payload);
    // The vertex's position in the line, in text pixels. They are so large that the display's drift from the eye
    // rounds away.
    vec2 offset = floor(vec2(Position.x, -Position.y) / WINDOW_FS_BLOCKS_PER_PIXEL - WINDOW_FS_LINE_ORIGIN + 0.5);
    vec2 ndc;
    if (kind == WINDOW_FS_KIND_BACKDROP) {
        ndc = vec2(offset.x > 0.0 ? 1.0 : -1.0, offset.y > 0.0 ? -1.0 : 1.0);
    } else {
        if (kind == WINDOW_FS_KIND_CURSOR) {
            vec2 rotation = window_fs_camera_rotation();
            offset += window_fs_cursor_at(rotation.x, rotation.y);
        }
        ndc = window_fs_ndc(offset);
    }
    gl_Position = vec4(ndc, window_fs_depth(window_fs_layer(payload)), 1.0);
    sphericalVertexDistance = WINDOW_FS_NO_FOG_DISTANCE;
    cylindricalVertexDistance = WINDOW_FS_NO_FOG_DISTANCE;
    vertexColor = window_fs_color(payload);
    texCoord0 = UV0;
#ifdef IS_GRAYSCALE
    windowHudTextureMode = 0;
#endif
}
#endif
"#;

pub(super) fn render_fullscreen_vertex_helpers() -> String {
    VERTEX_HELPERS.replace("@PLACEMENT@", CANVAS_PLACEMENT).replace("@DEPTH_STEP@", &format!("{LAYER_DEPTH_STEP:?}"))
}

/// Renders `core/position_tex_color.vsh`, vanilla's GUI sprite shader plus the HUD strip, attack indicator, and prompt
/// inputs. Like
/// vanilla, it declares its uniform blocks inline, since the shader is also compiled before resource packs load.
pub(in crate::hud) fn render_position_tex_color_vertex() -> String {
    let source = r#"#version 330

layout(std140) uniform DynamicTransforms {
    mat4 ModelViewMat;
    vec4 ColorModulator;
    vec3 ModelOffset;
    mat4 TextureMat;
};
layout(std140) uniform Projection {
    mat4 ProjMat;
};
layout(std140) uniform Globals {
    ivec3 CameraBlockPos;
    vec3 CameraOffset;
    vec2 ScreenSize;
    float GlintAlpha;
    float GameTime;
    int MenuBlurRadius;
    int UseRgss;
};

in vec3 Position;
in vec2 UV0;
in vec4 Color;

out vec2 texCoord0;
out vec4 vertexColor;
out vec2 windowHudPos;
flat out int windowHudHidden;
out vec4 windowHudIndicator;
out vec2 windowHudGui;
flat out vec4 windowHudRects[4];
@PROMPT@
// Whether a vertex lies on a corner of `rect` (left, top, right, bottom in GUI pixels). `freeRight` also accepts any x
// right of the left edge up to 17 pixels past it, and `freeTop` any y from the top edge down to the bottom edge, for
// the progress sprites whose far edge follows the charge. A quad drawn at the rect has all four vertices on corners.
bool window_fs_on_corner(vec2 pos, vec4 rect, bool freeRight, bool freeTop) {
    bool onX = abs(pos.x - rect.x) < 0.01 || abs(pos.x - rect.z) < 0.01
        || (freeRight && pos.x > rect.x + 0.01 && pos.x <= rect.x + 17.0);
    bool onY = abs(pos.y - rect.w) < 0.01 || abs(pos.y - rect.y) < 0.01
        || (freeTop && pos.y >= rect.y - 0.01 && pos.y < rect.w - 0.01);
    return onX && onY;
}

// Sets, per attack indicator rect, whether a vertex lies on one of its corners (windowHudIndicator) and the rect's
// maximal extent (windowHudRects), in GUI pixels from the top-left with the GUI size gui: the crosshair indicator's 16x4
// background and progress and its 16x16 full sprite, which start at (gui.x / 2 - 8, gui.y / 2 + 9), and the 18x18
// hotbar indicator background and progress in the row gui.y - 20, either 97 pixels right of the centre or 113 left of
// it, depending on the main arm.
void window_fs_indicator(vec2 pos, vec2 gui, bool hidden) {
    vec2 centre = floor(gui / 2.0);
    vec4 crosshair = vec4(centre.x - 8.0, centre.y + 9.0, centre.x + 8.0, centre.y + 13.0);
    vec4 full = vec4(crosshair.xy, crosshair.zy + vec2(0.0, 16.0));
    vec4 hotbarRight = vec4(centre.x + 97.0, gui.y - 20.0, centre.x + 115.0, gui.y - 2.0);
    vec4 hotbarLeft = hotbarRight - vec4(210.0, 0.0, 210.0, 0.0);
    windowHudRects = vec4[4](crosshair + vec4(0.0, 0.0, 1.0, 0.0), full, hotbarRight, hotbarLeft);
    windowHudIndicator = hidden ? vec4(
        window_fs_on_corner(pos, crosshair, true, false), window_fs_on_corner(pos, full, false, false),
        window_fs_on_corner(pos, hotbarRight, false, true), window_fs_on_corner(pos, hotbarLeft, false, true))
        : vec4(0.0);
}

void main() {
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);

    texCoord0 = UV0;
    vertexColor = Color;

    // A full-screen window holds the world age negative, which makes GameTime negative. GUI passes project
    // orthographically, so windowHudPos is in GUI pixels: x from the center, y up from the bottom.
    windowHudHidden = GameTime < 0.0 && ProjMat[3][3] == 1.0 ? 1 : 0;
    windowHudPos = vec2(gl_Position.x / ProjMat[0][0], (gl_Position.y + 1.0) / -ProjMat[1][1]);

    // Three matching corners of an axis-aligned quad fix its fourth, so a rect's flag reaches 1.0 across a quad only if
    // it is that rect, and the fragment's bounds check confines any sliver of a partial match to the rect itself. The
    // GUI size is the client's: the fractional framebuffer size over the scale, rounded up. Fractional sizes are
    // multiples of 1 / scale, so 0.01 absorbs float error without rounding a true fraction down.
    vec2 gui = ceil(vec2(2.0 / ProjMat[0][0], -2.0 / ProjMat[1][1]) - 0.01);
    windowHudGui = vec2((gl_Position.x + 1.0) / ProjMat[0][0], (gl_Position.y - 1.0) / ProjMat[1][1]);
    window_fs_indicator(windowHudGui, gui, windowHudHidden == 1);

    window_prompt_clip(gl_Position);
}
"#;
    source.replace("@PROMPT@", &render_prompt_vertex_helpers())
}

/// Renders `core/position_tex_color.fsh`, which while the HUD is hidden discards GUI sprite fragments inside the HUD
/// strip, texels carrying the crosshair marker alpha, which the pack's crosshair sprite override sets on its opaque
/// texels, and quads the vertex shader matched to a vanilla attack indicator. Clipping per fragment cuts sprites crossing the strip's edge instead of distorting them. While the
/// prompt's anvil is open, it also discards GUI sprite fragments inside the anvil's rect.
pub(in crate::hud) fn render_position_tex_color_fragment() -> String {
    format!(
        r#"#version 330

layout(std140) uniform DynamicTransforms {{
    mat4 ModelViewMat;
    vec4 ColorModulator;
    vec3 ModelOffset;
    mat4 TextureMat;
}};

// The bottom-center strip holding the hotbar, XP bar, hearts, armour, and hunger: (half width, height) in GUI pixels.
const vec2 WINDOW_FS_HUD_STRIP = vec2({half_width}.0, {height}.0);
const int WINDOW_FS_CROSSHAIR_ALPHA = {crosshair_alpha};

uniform sampler2D Sampler0;

in vec2 texCoord0;
in vec4 vertexColor;
in vec2 windowHudPos;
flat in int windowHudHidden;
in vec4 windowHudIndicator;
in vec2 windowHudGui;
flat in vec4 windowHudRects[4];
{prompt_helpers}
out vec4 fragColor;

void main() {{{prompt_clip}
    if (windowHudHidden == 1 && abs(windowHudPos.x) < WINDOW_FS_HUD_STRIP.x && windowHudPos.y < WINDOW_FS_HUD_STRIP.y) {{
        discard;
    }}
    for (int i = 0; i < 4; i++) {{
        if (windowHudIndicator[i] > 0.99999 && all(greaterThanEqual(windowHudGui, windowHudRects[i].xy))
            && all(lessThanEqual(windowHudGui, windowHudRects[i].zw))) {{
            discard;
        }}
    }}
    vec4 texel = texture(Sampler0, texCoord0);
    if (windowHudHidden == 1 && int(round(texel.a * 255.0)) == WINDOW_FS_CROSSHAIR_ALPHA) {{
        discard;
    }}
    vec4 color = texel * vertexColor;
    if (color.a == 0.0) {{
        discard;
    }}
    fragColor = color * ColorModulator;
}}
"#,
        half_width = HUD_STRIP_HALF_WIDTH,
        height = HUD_STRIP_HEIGHT,
        crosshair_alpha = CROSSHAIR_MARKER_ALPHA,
        prompt_helpers = render_prompt_fragment_helpers(),
        prompt_clip = PROMPT_SPRITE_FRAGMENT_CLIP,
    )
}
