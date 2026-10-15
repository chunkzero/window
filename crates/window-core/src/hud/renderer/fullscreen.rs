//! Full-screen snippets of the pack_format 85-88 `core/text` vertex shader, and the `core/position_tex_color`
//! shaders that hide the HUD while a full-screen window is open.
//!
//! The text shader moves every vertex of a full-screen marker (see [`crate::fullscreen`]) in the world pass straight
//! into clip space on the letterboxed canvas, so the projection, FOV, and view bobbing never move it. Unmarked text
//! takes the vanilla path.

use crate::fullscreen::{HUD_STRIP_HALF_WIDTH, HUD_STRIP_HEIGHT};

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

pub(super) const FULLSCREEN_VERTEX_HELPERS: &str = r#"
#if !defined(IS_GUI) && !defined(IS_SEE_THROUGH)
// The top-left of the text display's single zero-width line in its text space, where a shifted font's row 0 draws.
const vec2 WINDOW_FS_LINE_ORIGIN = vec2(1.0, -9.0);
// The view distance of the cursor layer, just past the 0.05-block near plane, and the distance between layers.
const float WINDOW_FS_NEAR_DISTANCE = 0.051;
const float WINDOW_FS_LAYER_DISTANCE = 0.0001;
// Below every fog start, so marked glyphs are never fogged.
const float WINDOW_FS_NO_FOG_DISTANCE = -1.0e30;

// The camera's yaw and pitch in degrees. ModelViewMat is the camera rotation: its first row is the camera's right
// vector, which stays level looking straight up or down, and its third row is the negated look vector.
vec2 window_fs_camera_rotation() {
    vec3 right = vec3(ModelViewMat[0][0], ModelViewMat[1][0], ModelViewMat[2][0]);
    float lookY = -ModelViewMat[1][2];
    return vec2(degrees(atan(-right.z, -right.x)), degrees(asin(clamp(-lookY, -1.0, 1.0))));
}

// A canvas point in normalized device coordinates, with the canvas letterboxed or pillarboxed to the screen.
vec2 window_fs_ndc(vec2 canvas) {
    float screenAspect = ScreenSize.x / ScreenSize.y;
    float canvasAspect = WINDOW_FS_CANVAS.x / WINDOW_FS_CANVAS.y;
    vec2 fit = screenAspect > canvasAspect ? vec2(canvasAspect / screenAspect, 1.0) : vec2(1.0, screenAspect / canvasAspect);
    vec2 unit = canvas / WINDOW_FS_CANVAS * 2.0 - 1.0;
    return vec2(unit.x, -unit.y) * fit;
}

// The depth of a layer: a point on the view axis just past the near plane, nearer for higher layers, projected by
// ProjMat so the depth convention (reversed or not) is the device's.
float window_fs_depth(int layer) {
    float distance = WINDOW_FS_NEAR_DISTANCE + float(WINDOW_FS_CURSOR_LAYER - layer) * WINDOW_FS_LAYER_DISTANCE;
    vec4 clip = ProjMat * vec4(0.0, 0.0, -distance, 1.0);
    return clip.z / clip.w;
}

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
    vertexColor = vec4(window_fs_color(payload), 1.0);
    texCoord0 = UV0;
#ifdef IS_GRAYSCALE
    windowHudTextureMode = 0;
#endif
}
#endif
"#;

/// Renders `core/position_tex_color.vsh`, vanilla's GUI sprite shader plus the HUD strip inputs. Like vanilla, it
/// declares its uniform blocks inline, since the shader is also compiled before resource packs load.
pub(in crate::hud) fn render_position_tex_color_vertex() -> String {
    r#"#version 330

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

void main() {
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);

    texCoord0 = UV0;
    vertexColor = Color;

    // A full-screen window holds the world age negative, which makes GameTime negative. GUI passes project
    // orthographically, so windowHudPos is in GUI pixels: x from the center, y up from the bottom.
    windowHudHidden = GameTime < 0.0 && ProjMat[3][3] == 1.0 ? 1 : 0;
    windowHudPos = vec2(gl_Position.x / ProjMat[0][0], (gl_Position.y + 1.0) / -ProjMat[1][1]);
}
"#
    .to_string()
}

/// Renders `core/position_tex_color.fsh`, which discards GUI sprite fragments inside the HUD strip while the HUD is
/// hidden. Clipping per fragment cuts sprites crossing the strip's edge instead of distorting them.
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

uniform sampler2D Sampler0;

in vec2 texCoord0;
in vec4 vertexColor;
in vec2 windowHudPos;
flat in int windowHudHidden;

out vec4 fragColor;

void main() {{
    if (windowHudHidden == 1 && abs(windowHudPos.x) < WINDOW_FS_HUD_STRIP.x && windowHudPos.y < WINDOW_FS_HUD_STRIP.y) {{
        discard;
    }}
    vec4 color = texture(Sampler0, texCoord0) * vertexColor;
    if (color.a == 0.0) {{
        discard;
    }}
    fragColor = color * ColorModulator;
}}
"#,
        half_width = HUD_STRIP_HALF_WIDTH,
        height = HUD_STRIP_HEIGHT,
    )
}
