use crate::hud::rule::HudShaderRule;

use super::glsl::{
    DynamicFog, ScreenSizeSource, V150Fog, dynamic_fog, format_glsl_float, join_rule_values, render_gui_size_helper,
    rgb_vec3, segment_table, texture_mode, texture_mode_snippets, v150_fog,
};
use super::marker_id;

// Minecraft 26.1.2's Gui.extractOverlayMessage translates to guiHeight - 68
// and draws text at y=-4, placing Window's authored y=0 at guiHeight - 72.
// `source_bottom` defaults to 59, so its nominal point is 13px below that
// fixed source origin. This is independent of the custom bitmap's height.
const ACTIONBAR_SOURCE_INSET: i32 = 13;

fn render_text_helpers(rules: &[HudShaderRule], screen_size: ScreenSizeSource, texture_mode_output: bool) -> String {
    let texture_mode_function = if texture_mode_output {
        r#"
int window_hud_texture_mode(vec4 color) {
    int i = window_hud_segment_index(color);
    if (i >= 0) {
        return WINDOW_HUD_TEXTURE_MODES[i];
    }
    return 0;
}
"#
    } else {
        ""
    };
    render_helper_tables(rules, screen_size) + &render_helper_functions(texture_mode_function)
}

fn render_helper_tables(rules: &[HudShaderRule], screen_size: ScreenSizeSource) -> String {
    let count = rules.len();
    let bottoms = join_rule_values(rules, |r| r.source_bottom.to_string());
    let heights = join_rule_values(rules, |r| r.height.to_string());
    let widths = join_rule_values(rules, |r| r.width.to_string());
    let origins_x = join_rule_values(rules, |r| format_glsl_float(r.origin_x));
    let origins_y = join_rule_values(rules, |r| format_glsl_float(r.origin_y));
    let anchors_x = join_rule_values(rules, |r| format_glsl_float(r.anchor_x));
    let anchors_y = join_rule_values(rules, |r| format_glsl_float(r.anchor_y));
    let offsets_x = join_rule_values(rules, |r| format!("{}.0", r.offset_x));
    let offsets_y = join_rule_values(rules, |r| format!("{}.0", r.offset_y));
    let actionbar_source_inset = ACTIONBAR_SOURCE_INSET;
    let segments = segment_table(rules);
    let segment_count = segments.len();
    let marker_ids = segments.iter().map(|s| marker_id(s.marker).to_string()).collect::<Vec<_>>().join(", ");
    let segment_rules = segments.iter().map(|s| s.rule.to_string()).collect::<Vec<_>>().join(", ");
    let display_colors = segments.iter().map(|s| rgb_vec3(s.display)).collect::<Vec<_>>().join(", ");
    let texture_modes = segments.iter().map(|s| texture_mode(s.texture).to_string()).collect::<Vec<_>>().join(", ");
    let gui_size = render_gui_size_helper(screen_size);

    format!(
        r#"
const int WINDOW_HUD_RULES = {count};
const int WINDOW_HUD_SOURCE_BOTTOMS[WINDOW_HUD_RULES] = int[]({bottoms});
const int WINDOW_HUD_WIDTHS[WINDOW_HUD_RULES] = int[]({widths});
const int WINDOW_HUD_HEIGHTS[WINDOW_HUD_RULES] = int[]({heights});
const float WINDOW_HUD_ORIGIN_X[WINDOW_HUD_RULES] = float[]({origins_x});
const float WINDOW_HUD_ORIGIN_Y[WINDOW_HUD_RULES] = float[]({origins_y});
const float WINDOW_HUD_ANCHOR_X[WINDOW_HUD_RULES] = float[]({anchors_x});
const float WINDOW_HUD_ANCHOR_Y[WINDOW_HUD_RULES] = float[]({anchors_y});
const float WINDOW_HUD_OFFSET_X[WINDOW_HUD_RULES] = float[]({offsets_x});
const float WINDOW_HUD_OFFSET_Y[WINDOW_HUD_RULES] = float[]({offsets_y});
const float WINDOW_HUD_ACTIONBAR_SOURCE_INSET = {actionbar_source_inset}.0;
const int WINDOW_HUD_SEGMENT_COUNT = {segment_count};
const int WINDOW_HUD_MARKER_IDS[WINDOW_HUD_SEGMENT_COUNT] = int[]({marker_ids});
const int WINDOW_HUD_SEGMENT_RULES[WINDOW_HUD_SEGMENT_COUNT] = int[]({segment_rules});
const vec3 WINDOW_HUD_DISPLAY_COLORS[WINDOW_HUD_SEGMENT_COUNT] = vec3[]({display_colors});
const int WINDOW_HUD_TEXTURE_MODES[WINDOW_HUD_SEGMENT_COUNT] = int[]({texture_modes});
{gui_size}"#
    )
}

fn render_helper_functions(texture_mode_function: &str) -> String {
    format!(
        r#"

bool window_hud_in_band(int offsetFromBottom, int sourceBottom, int height) {{
    int sourceTop = sourceBottom + int(WINDOW_HUD_ACTIONBAR_SOURCE_INSET);
    return offsetFromBottom >= sourceTop - height - 9 && offsetFromBottom <= sourceTop + 9;
}}

int window_hud_marker_id(vec4 color) {{
    ivec3 rgb = ivec3(floor(color.rgb * 255.0 + 0.5));
    if (rgb.g != 0) {{
        return -1;
    }}
    return rgb.r * 256 + rgb.b;
}}

int window_hud_segment_index(vec4 color) {{
    int marker = window_hud_marker_id(color);
    if (marker < 0) {{
        return -1;
    }}
    for (int i = 0; i < WINDOW_HUD_SEGMENT_COUNT; i++) {{
        if (marker == WINDOW_HUD_MARKER_IDS[i]) {{
            return i;
        }}
    }}
    return -1;
}}

int window_hud_rule_index(float y, vec4 color) {{
    int segment = window_hud_segment_index(color);
    if (segment < 0) {{
        return -1;
    }}
    int rule = WINDOW_HUD_SEGMENT_RULES[segment];
    if (WINDOW_HUD_TEXTURE_MODES[segment] == 1) {{
        return rule;
    }}
    float guiHeight = window_hud_gui_size().y;
    int offsetFromBottom = int(round(guiHeight - y));
    if (window_hud_in_band(offsetFromBottom, WINDOW_HUD_SOURCE_BOTTOMS[rule], WINDOW_HUD_HEIGHTS[rule])) {{
        return rule;
    }}
    return -1;
}}

vec4 window_hud_vertex_color(vec4 color, vec4 lightColor, float y) {{
    int i = window_hud_segment_index(color);
    if (i >= 0 && window_hud_rule_index(y, color) >= 0) {{
        vec3 displayColor = WINDOW_HUD_DISPLAY_COLORS[i];
        if (color.a < 0.999) {{
            displayColor *= 0.25;
        }}
        return vec4(displayColor, color.a) * lightColor;
    }}
    return color * lightColor;
}}
{texture_mode_function}

vec2 window_hud_delta(int i) {{
    vec2 guiSize = window_hud_gui_size();
    float guiWidth = guiSize.x;
    float guiHeight = guiSize.y;
    float width = float(WINDOW_HUD_WIDTHS[i]);
    float height = float(WINDOW_HUD_HEIGHTS[i]);
    vec2 sourceTopLeft = vec2(
        guiWidth * 0.5,
        guiHeight - float(WINDOW_HUD_SOURCE_BOTTOMS[i]) - WINDOW_HUD_ACTIONBAR_SOURCE_INSET
    );
    vec2 targetTopLeft = vec2(
        guiWidth * WINDOW_HUD_ORIGIN_X[i] + WINDOW_HUD_OFFSET_X[i] - width * WINDOW_HUD_ANCHOR_X[i],
        guiHeight * WINDOW_HUD_ORIGIN_Y[i] + WINDOW_HUD_OFFSET_Y[i] - height * WINDOW_HUD_ANCHOR_Y[i]
    );
    return targetTopLeft - sourceTopLeft;
}}

void window_hud_apply(inout vec3 pos, vec4 color) {{
    int i = window_hud_rule_index(pos.y, color);
    if (i >= 0) {{
        pos.xy += window_hud_delta(i);
    }}
}}
"#
    )
}

pub(in crate::hud) fn render_v150_text_shader(
    rules: &[HudShaderRule],
    namespaced_import: bool,
    legacy_fog: bool,
    texture_mode_output: bool,
    see_through: bool,
) -> String {
    let V150Fog { import, ivew, fog_shape, vertex_distance_decl, vertex_distance_assign } =
        v150_fog(namespaced_import, legacy_fog, see_through);
    let helpers = render_text_helpers(rules, ScreenSizeSource::ProjectionMatrix, texture_mode_output);
    let (texture_mode_decl, texture_mode_assign) = texture_mode_snippets(texture_mode_output);

    format!(
        r#"#version 150

{import}

in vec3 Position;
in vec4 Color;
in vec2 UV0;
in ivec2 UV2;

uniform sampler2D Sampler2;

uniform mat4 ModelViewMat;
uniform mat4 ProjMat;{ivew}
{fog_shape}

{vertex_distance_decl}
out vec4 vertexColor;
out vec2 texCoord0;{texture_mode_decl}
{helpers}
void main() {{
    vec3 pos = Position;
    window_hud_apply(pos, Color);
    gl_Position = ProjMat * ModelViewMat * vec4(pos, 1.0);
{vertex_distance_assign}
    vertexColor = window_hud_vertex_color(Color, texelFetch(Sampler2, UV2 / 16, 0), Position.y);
    texCoord0 = UV0;{texture_mode_assign}
}}
"#
    )
}

pub(in crate::hud) fn render_v150_dynamic_text_shader(
    rules: &[HudShaderRule],
    texture_mode_output: bool,
    see_through: bool,
) -> String {
    let DynamicFog { fog_import, fog_outputs, fog_assign } = dynamic_fog(see_through);
    let helpers = render_text_helpers(rules, ScreenSizeSource::Globals, texture_mode_output);
    let (texture_mode_decl, texture_mode_assign) = texture_mode_snippets(texture_mode_output);

    format!(
        r#"#version 150

{fog_import}
#moj_import <minecraft:globals.glsl>
#moj_import <minecraft:dynamictransforms.glsl>
#moj_import <minecraft:projection.glsl>

in vec3 Position;
in vec4 Color;
in vec2 UV0;
in ivec2 UV2;

uniform sampler2D Sampler2;

{fog_outputs}
out vec4 vertexColor;
out vec2 texCoord0;{texture_mode_decl}
{helpers}
void main() {{
    vec3 pos = Position;
    window_hud_apply(pos, Color);
    gl_Position = ProjMat * ModelViewMat * vec4(pos, 1.0);
{fog_assign}
    vertexColor = window_hud_vertex_color(Color, texelFetch(Sampler2, UV2 / 16, 0), Position.y);
    texCoord0 = UV0;{texture_mode_assign}
}}
"#
    )
}

pub(in crate::hud) fn render_v330_text_shader(
    rules: &[HudShaderRule],
    texture_mode_output: bool,
    see_through: bool,
) -> String {
    let DynamicFog { fog_import, fog_outputs, fog_assign } = dynamic_fog(see_through);
    let helpers = render_text_helpers(rules, ScreenSizeSource::Globals, texture_mode_output);
    let (texture_mode_decl, texture_mode_assign) = texture_mode_snippets(texture_mode_output);

    format!(
        r#"#version 330

{fog_import}
#moj_import <minecraft:globals.glsl>
#moj_import <minecraft:dynamictransforms.glsl>
#moj_import <minecraft:projection.glsl>
#moj_import <minecraft:sample_lightmap.glsl>

in vec3 Position;
in vec4 Color;
in vec2 UV0;
in ivec2 UV2;

uniform sampler2D Sampler2;

{fog_outputs}
out vec4 vertexColor;
out vec2 texCoord0;{texture_mode_decl}
{helpers}
void main() {{
    vec3 pos = Position;
    window_hud_apply(pos, Color);
    gl_Position = ProjMat * ModelViewMat * vec4(pos, 1.0);
{fog_assign}
    vertexColor = window_hud_vertex_color(Color, sample_lightmap(Sampler2, UV2), Position.y);
    texCoord0 = UV0;{texture_mode_assign}
}}
"#
    )
}
