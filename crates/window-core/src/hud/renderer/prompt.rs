//! The keyboard text prompt's anvil as Minecraft 26.2 draws it, and the shader snippets that hide it while the prompt
//! is open, which the server signals by holding the world age in the prompt band (see
//! [`crate::fullscreen::PROMPT_WORLD_AGE`]).
//!
//! Only what the anvil draws is hidden, so chat, titles, the action bar, and HUD panes overlapping its rect stay:
//!
//! - GUI sprites (`core/position_tex_color`) anywhere inside the anvil's rect, since none of those overlays use
//!   sprites;
//! - GUI fills (`core/gui`): the screen's dimming gradient, quick-craft highlights over the anvil's slots, and
//!   anything in the rename field's band, where the caret and selection draw;
//! - GUI text (`core/text`): the rename field's band and the label bands.
//!
//! Positions are GUI pixels from the anvil's top-left. Clipping happens per fragment, so anything crossing a band's
//! edge is cut rather than distorted.

use crate::fullscreen::PROMPT_TIME_SPLIT;

/// The anvil screen's size; vanilla centres it with integer division.
const ANVIL_SIZE: [u32; 2] = [176, 166];

/// The rename field's band `[x, y, width, height]`: its text at (62, 24) with a shadow, the selection starting one
/// pixel left of it, the caret and selection one pixel above and below it, and an end-of-text caret that may run past
/// the field's 103-pixel width.
const FIELD_BAND: [u32; 4] = [61, 22, 115, 14];

/// The title's band at y 6 and the inventory label's at y 72, nine pixels of text and a pixel of slack each.
const LABEL_BANDS: [[u32; 4]; 2] = [[0, 6, 176, 10], [0, 72, 176, 10]];

/// The anvil's slot grids `[x, y, columns, rows]`: its two inputs and result, the player's main inventory, and the
/// hotbar. Slots are 16 pixels on an 18-pixel pitch.
const SLOT_GRIDS: [[u32; 4]; 5] = [[27, 47, 1, 1], [76, 47, 1, 1], [134, 47, 1, 1], [8, 84, 9, 3], [8, 142, 9, 1]];
const SLOT_SIZE: u32 = 16;
const SLOT_PITCH: u32 = 18;

/// The ARGB fill vanilla draws over each slot of a quick-craft drag.
const SLOT_FILL: u32 = 0x80FF_FFFF;

/// The ARGB top and bottom of the gradient that dims the whole screen behind the anvil.
const DIM_GRADIENT: [u32; 2] = [0xC010_1010, 0xD010_1010];

/// The prompt's vertex outputs, written by `window_prompt_clip`.
const VERTEX_OUTPUTS: &str = "
flat out int windowPromptOpen;
out vec2 windowPromptPos;";

/// The fragment inputs matching [`VERTEX_OUTPUTS`].
const FRAGMENT_INPUTS: &str = "
flat in int windowPromptOpen;
in vec2 windowPromptPos;";

const VERTEX_HELPERS: &str = r#"
const float WINDOW_PROMPT_TIME_SPLIT = @SPLIT@;
const vec2 WINDOW_PROMPT_ANVIL = @ANVIL@;

// Whether the prompt's anvil is open and this vertex is drawn by an orthographic GUI pass. The HUD band lies below the
// split, so the HUD stays hidden in both bands.
bool window_prompt_open() {
    return GameTime < 0.0 && GameTime > WINDOW_PROMPT_TIME_SPLIT && ProjMat[3][3] == 1.0;
}

// The GUI projection's size, which is fractional when the GUI scale does not divide the window.
vec2 window_prompt_projected_size() {
    return vec2(2.0 / ProjMat[0][0], -2.0 / ProjMat[1][1]);
}

// The GUI size screens lay out in: the projection's size rounded up.
vec2 window_prompt_gui_size() {
    return ceil(window_prompt_projected_size() - 0.01);
}

// A GUI vertex in GUI pixels from the screen's top-left.
vec2 window_prompt_gui(vec4 clip) {
    return vec2(clip.x + 1.0, 1.0 - clip.y) * 0.5 * window_prompt_projected_size();
}

// Sets the outputs the fragment shader clips with: whether the anvil is open, and the vertex from its top-left.
void window_prompt_clip(vec4 clip) {
    windowPromptOpen = window_prompt_open() ? 1 : 0;
    windowPromptPos = window_prompt_gui(clip) - floor((window_prompt_gui_size() - WINDOW_PROMPT_ANVIL) / 2.0);
}
"#;

const FRAGMENT_HELPERS: &str = r#"
const vec2 WINDOW_PROMPT_ANVIL = @ANVIL@;
const vec4 WINDOW_PROMPT_FIELD = @FIELD@;
const vec4 WINDOW_PROMPT_LABELS[@LABEL_COUNT@] = vec4[](@LABELS@);

// Whether the fragment lies inside rect (x, y, width, height) of the open anvil.
bool window_prompt_in(vec4 rect) {
    return windowPromptOpen == 1 && all(greaterThanEqual(windowPromptPos, rect.xy))
        && all(lessThan(windowPromptPos, rect.xy + rect.zw));
}
"#;

/// The slot grids and their helpers, for `core/gui`.
const SLOT_HELPERS: &str = r#"
const vec4 WINDOW_PROMPT_SLOT_GRIDS[@GRID_COUNT@] = vec4[](@GRIDS@);
const float WINDOW_PROMPT_SLOT_SIZE = @SLOT_SIZE@;
const float WINDOW_PROMPT_SLOT_PITCH = @SLOT_PITCH@;
"#;

const SLOT_CORNER_HELPERS: &str = r#"
// Whether v, from a grid's first slot along one axis, is a slot's near or far edge in a grid of count slots.
bool window_prompt_slot_edge(float v, float count) {
    float m = mod(v + 0.25, WINDOW_PROMPT_SLOT_PITCH);
    return v > -0.5 && v < (count - 1.0) * WINDOW_PROMPT_SLOT_PITCH + WINDOW_PROMPT_SLOT_SIZE + 0.5
        && (m < 0.5 || abs(m - WINDOW_PROMPT_SLOT_SIZE - 0.25) < 0.25);
}

// Whether p, from the anvil's top-left, is the corner of one of its slots.
bool window_prompt_slot_corner(vec2 p) {
    for (int i = 0; i < WINDOW_PROMPT_SLOT_GRIDS.length(); i++) {
        vec4 grid = WINDOW_PROMPT_SLOT_GRIDS[i];
        if (window_prompt_slot_edge(p.x - grid.x, grid.z) && window_prompt_slot_edge(p.y - grid.y, grid.w)) {
            return true;
        }
    }
    return false;
}
"#;

const SLOT_INSIDE_HELPERS: &str = r#"
// Whether the fragment lies inside one of the anvil's slots.
bool window_prompt_in_slot() {
    for (int i = 0; i < WINDOW_PROMPT_SLOT_GRIDS.length(); i++) {
        vec4 grid = WINDOW_PROMPT_SLOT_GRIDS[i];
        vec2 local = windowPromptPos - grid.xy;
        vec2 cell = floor(local / WINDOW_PROMPT_SLOT_PITCH);
        if (all(greaterThanEqual(cell, vec2(0.0))) && all(lessThan(cell, grid.zw))
            && all(lessThan(local - cell * WINDOW_PROMPT_SLOT_PITCH, vec2(WINDOW_PROMPT_SLOT_SIZE)))) {
            return true;
        }
    }
    return false;
}
"#;

fn vec2(value: [u32; 2]) -> String {
    format!("vec2({}.0, {}.0)", value[0], value[1])
}

fn vec4(value: [u32; 4]) -> String {
    format!("vec4({}.0, {}.0, {}.0, {}.0)", value[0], value[1], value[2], value[3])
}

fn vec4_list(values: &[[u32; 4]]) -> String {
    values.iter().map(|value| vec4(*value)).collect::<Vec<_>>().join(", ")
}

/// An ARGB color as the `ivec4` of a vertex color's RGBA bytes.
fn rgba(argb: u32) -> String {
    let [a, r, g, b] = argb.to_be_bytes();
    format!("ivec4({r}, {g}, {b}, {a})")
}

/// The prompt's vertex outputs and helpers. Needs `ProjMat` and `GameTime`.
pub(super) fn render_prompt_vertex_helpers() -> String {
    VERTEX_OUTPUTS.to_string()
        + &VERTEX_HELPERS.replace("@SPLIT@", &format!("{PROMPT_TIME_SPLIT:?}")).replace("@ANVIL@", &vec2(ANVIL_SIZE))
}

/// The prompt's fragment inputs and helpers.
pub(super) fn render_prompt_fragment_helpers() -> String {
    FRAGMENT_INPUTS.to_string()
        + &FRAGMENT_HELPERS
            .replace("@ANVIL@", &vec2(ANVIL_SIZE))
            .replace("@FIELD@", &vec4(FIELD_BAND))
            .replace("@LABEL_COUNT@", &LABEL_BANDS.len().to_string())
            .replace("@LABELS@", &vec4_list(&LABEL_BANDS))
}

fn slot_helpers() -> String {
    SLOT_HELPERS
        .replace("@GRID_COUNT@", &SLOT_GRIDS.len().to_string())
        .replace("@GRIDS@", &vec4_list(&SLOT_GRIDS))
        .replace("@SLOT_SIZE@", &format!("{SLOT_SIZE}.0"))
        .replace("@SLOT_PITCH@", &format!("{SLOT_PITCH}.0"))
}

/// The GUI variant's prompt outputs and helpers for `core/text.vsh`.
pub(super) fn render_prompt_text_vertex_helpers() -> String {
    format!("\n#ifdef IS_GUI{}#endif", render_prompt_vertex_helpers())
}

/// Runs last in the text vertex `main`, after any hover glyph has moved.
pub(super) const PROMPT_TEXT_VERTEX_APPLY: &str = "
#ifdef IS_GUI
    window_prompt_clip(gl_Position);
#endif";

/// The GUI variant's prompt inputs and helpers for `core/text.fsh`.
pub(super) fn render_prompt_text_fragment_helpers() -> String {
    format!("\n#ifdef IS_GUI{}#endif", render_prompt_fragment_helpers())
}

/// Runs first in the text fragment `main`: hides GUI text in the field and label bands.
pub(super) const PROMPT_TEXT_FRAGMENT_CLIP: &str = "
#ifdef IS_GUI
    if (window_prompt_in(WINDOW_PROMPT_FIELD) || window_prompt_in(WINDOW_PROMPT_LABELS[0])
        || window_prompt_in(WINDOW_PROMPT_LABELS[1])) {
        discard;
    }
#endif";

/// Runs first in the sprite fragment `main`: hides GUI sprites anywhere in the anvil's rect.
pub(super) const PROMPT_SPRITE_FRAGMENT_CLIP: &str = "
    if (window_prompt_in(vec4(vec2(0.0), WINDOW_PROMPT_ANVIL))) {
        discard;
    }";

/// Renders `core/gui.vsh`, vanilla's GUI fill shader plus the prompt's outputs, and a flag marking the vertices of the
/// anvil's own fills: 1 for the dimming gradient, at the screen's corners, and 2 for a quick-craft slot highlight, at
/// a slot's corners. Like vanilla, it declares its uniform blocks inline.
pub(in crate::hud) fn render_gui_vertex() -> String {
    format!(
        r#"#version 330

layout(std140) uniform DynamicTransforms {{
    mat4 ModelViewMat;
    vec4 ColorModulator;
    vec3 ModelOffset;
    mat4 TextureMat;
}};
layout(std140) uniform Projection {{
    mat4 ProjMat;
}};
layout(std140) uniform Globals {{
    ivec3 CameraBlockPos;
    vec3 CameraOffset;
    vec2 ScreenSize;
    float GlintAlpha;
    float GameTime;
    int MenuBlurRadius;
    int UseRgss;
}};

in vec3 Position;
in vec4 Color;

out vec4 vertexColor;
flat out int windowPromptFill;
{helpers}{slots}{corners}
// Whether v lies within a quarter pixel of a or b.
bool window_prompt_near(float v, float a, float b) {{
    return abs(v - a) < 0.25 || abs(v - b) < 0.25;
}}

int window_prompt_fill(vec4 clip, vec4 color) {{
    if (windowPromptOpen == 0) {{
        return 0;
    }}
    ivec4 rgba = ivec4(round(color * 255.0));
    vec2 gui = window_prompt_gui(clip);
    vec2 size = window_prompt_gui_size();
    if ((rgba == {dim_top} || rgba == {dim_bottom}) && window_prompt_near(gui.x, 0.0, size.x)
        && window_prompt_near(gui.y, 0.0, size.y)) {{
        return 1;
    }}
    return rgba == {slot_fill} && window_prompt_slot_corner(windowPromptPos) ? 2 : 0;
}}

void main() {{
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);

    vertexColor = Color;

    window_prompt_clip(gl_Position);
    windowPromptFill = window_prompt_fill(gl_Position, Color);
}}
"#,
        helpers = render_prompt_vertex_helpers(),
        slots = slot_helpers(),
        corners = SLOT_CORNER_HELPERS,
        dim_top = rgba(DIM_GRADIENT[0]),
        dim_bottom = rgba(DIM_GRADIENT[1]),
        slot_fill = rgba(SLOT_FILL),
    )
}

/// Renders `core/gui.fsh`, vanilla's GUI fill shader, which while the prompt's anvil is open discards the dimming
/// gradient, quick-craft highlights inside the anvil's slots, and anything in the rename field's band.
pub(in crate::hud) fn render_gui_fragment() -> String {
    format!(
        r#"#version 330

layout(std140) uniform DynamicTransforms {{
    mat4 ModelViewMat;
    vec4 ColorModulator;
    vec3 ModelOffset;
    mat4 TextureMat;
}};

in vec4 vertexColor;
flat in int windowPromptFill;
{helpers}{slots}{inside}
out vec4 fragColor;

void main() {{
    if (windowPromptFill == 1 || (windowPromptFill == 2 && window_prompt_in_slot())
        || window_prompt_in(WINDOW_PROMPT_FIELD)) {{
        discard;
    }}
    vec4 color = vertexColor;
    if (color.a == 0.0) {{
        discard;
    }}
    fragColor = color * ColorModulator;
}}
"#,
        helpers = render_prompt_fragment_helpers(),
        slots = slot_helpers(),
        inside = SLOT_INSIDE_HELPERS,
    )
}
