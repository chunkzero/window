//! Hover outline snippets of the pack_format 85-88 `core/text` shaders.
//!
//! A hover glyph rides in a tooltip title, so the client draws it exactly while its region is hovered. It is drawn
//! twice: the text copy, which the shader hides, and the shadow copy, which the shader moves onto the region. The
//! shadow color tells it where:
//!
//! - red and green: the region's left and top edge relative to the GUI center, plus 128;
//! - blue and the top 3 bits of alpha: the width of the tooltip's text;
//! - the low 5 bits of alpha: the tooltip's line count plus 1, at most 30, so alpha is never 0 or 255.
//!
//! The fragment shader discards the outline inside the tooltip box those describe.

use crate::hud::{HOVER_GLYPH_ID, HOVER_SIZE_MARK};

/// Runs last in the vertex `main`, replacing the position and color of hover glyph vertices.
pub(super) const HOVER_VERTEX_APPLY: &str = "\n#ifdef IS_GUI\n    window_hover_apply();\n#endif";

pub(super) fn render_hover_vertex_helpers() -> String {
    format!(
        r#"
#ifdef IS_GUI
uniform sampler2D Sampler0;

flat out int windowHover;
flat out vec4 windowHoverCutout;
out vec2 windowHoverPos;

const ivec3 WINDOW_HOVER_ID = ivec3({id_r}, {id_g}, {id_b});
const int WINDOW_HOVER_SIZE_MARK = {size_mark};
// Vanilla's tooltip box extends this far beyond its text.
const float WINDOW_HOVER_TOOLTIP_EDGE = 4.0;

// The glyph pixel `index` steps along its row from the corner `pixel`, toward the opposite corner.
ivec3 window_hover_pixel(ivec2 pixel, int direction, int index) {{
    return ivec3(round(texelFetch(Sampler0, pixel - ivec2(index * direction, 0), 0).rgb * 255.0));
}}

void window_hover_apply() {{
    windowHover = 0;
    windowHoverCutout = vec4(0.0);
    windowHoverPos = vec2(0.0);

    vec2 texel = UV0 * vec2(textureSize(Sampler0, 0));
    // Glyph UVs are inset by a fraction of a texel, so the fraction tells which corner of its quad this vertex is.
    vec2 corner = step(0.5, fract(texel));
    ivec2 pixel = ivec2(floor(texel));
    int direction = int(corner.x) * 2 - 1;
    ivec3 sizePixel = window_hover_pixel(pixel, direction, 1);
    if (window_hover_pixel(pixel, direction, 0) != WINDOW_HOVER_ID || sizePixel.b != WINDOW_HOVER_SIZE_MARK) {{
        return;
    }}
    ivec4 color = ivec4(round(Color * 255.0));
    if (color.a == 255) {{
        vertexColor = vec4(0.0);
        return;
    }}

    vec2 size = vec2(sizePixel.rg) + 1.0;
    vec2 guiSize = ceil(window_hud_gui_size() - 0.001);
    vec2 pos = floor(guiSize * 0.5) + vec2(color.rg) - 128.0 + corner * size;
    gl_Position = ProjMat * ModelViewMat * vec4(pos, Position.z, 1.0);
    vertexColor = vec4(1.0);
    windowHover = 1;
    windowHoverPos = pos;

    int lines = (color.a & 31) - 1;
    if (lines > 0) {{
        // The glyph has ascent 0 and this is its shadow, so its top-left is the tooltip's text origin plus (1, 8).
        vec2 origin = Position.xy - corner * size - vec2(1.0, 8.0);
        vec2 extent = vec2(float(((color.a >> 5) << 8) | color.b), lines == 1 ? 8.0 : 10.0 * float(lines));
        windowHoverCutout = vec4(origin - WINDOW_HOVER_TOOLTIP_EDGE, origin + extent + WINDOW_HOVER_TOOLTIP_EDGE);
    }}
}}
#endif
"#,
        id_r = HOVER_GLYPH_ID.r,
        id_g = HOVER_GLYPH_ID.g,
        id_b = HOVER_GLYPH_ID.b,
        size_mark = HOVER_SIZE_MARK,
    )
}

pub(super) const HOVER_FRAGMENT_INPUTS: &str = "
#ifdef IS_GUI
flat in int windowHover;
flat in vec4 windowHoverCutout;
in vec2 windowHoverPos;
#endif";

/// Paints a hover glyph's pixels white, whatever data their colors hold, except inside its tooltip's box.
pub(super) const HOVER_FRAGMENT_DRAW: &str = "
#ifdef IS_GUI
    if (windowHover == 1) {
        if (all(greaterThan(windowHoverPos, windowHoverCutout.xy)) && all(lessThan(windowHoverPos, windowHoverCutout.zw))) {
            discard;
        }
        texColor.rgb = vec3(1.0);
    }
#endif";
