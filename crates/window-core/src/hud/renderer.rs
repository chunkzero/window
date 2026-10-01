use crate::ir::Rgb;

use super::{HudShaderRule, HudShaderTexture};

// Minecraft 26.1.2's Gui.extractOverlayMessage translates to guiHeight - 68
// and draws text at y=-4, placing Window's authored y=0 at guiHeight - 72.
// `source_bottom` defaults to 59, so its nominal point is 13px below that
// fixed source origin. This is independent of the custom bitmap's height.
const ACTIONBAR_SOURCE_INSET: i32 = 13;

#[derive(Clone, Copy, Debug)]
enum ScreenSizeSource {
    ProjectionMatrix,
    Globals,
}

fn join_rule_values<F>(rules: &[HudShaderRule], mut value: F) -> String
where
    F: FnMut(&HudShaderRule) -> String,
{
    rules.iter().map(&mut value).collect::<Vec<_>>().join(", ")
}

fn render_gui_size_helper(screen_size: ScreenSizeSource) -> &'static str {
    match screen_size {
        ScreenSizeSource::ProjectionMatrix => {
            r#"
vec2 window_hud_gui_size() {
    return vec2(abs(2.0 / ProjMat[0][0]), abs(2.0 / ProjMat[1][1]));
}
"#
        }
        ScreenSizeSource::Globals => {
            r#"
vec2 window_hud_gui_size() {
    vec2 pixel = abs(vec2(ProjMat[0][0], ProjMat[1][1])) / 2.0;
    float guiScale = max(1.0, round(pixel.x * ScreenSize.x));
    return ScreenSize / guiScale;
}
"#
        }
    }
}

fn render_text_helpers(rules: &[HudShaderRule], screen_size: ScreenSizeSource, texture_mode_output: bool) -> String {
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
    let texture_modes = segments.iter().map(|s| s.texture.mode().to_string()).collect::<Vec<_>>().join(", ");
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
{gui_size}

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

fn render_background_helpers(rules: &[HudShaderRule], screen_size: ScreenSizeSource) -> String {
    let count = rules.len();
    let bottoms = join_rule_values(rules, |r| r.source_bottom.to_string());
    let widths = join_rule_values(rules, |r| r.width.to_string());
    let gui_size = render_gui_size_helper(screen_size);

    format!(
        r#"
const int WINDOW_HUD_RULES = {count};
const int WINDOW_HUD_SOURCE_BOTTOMS[WINDOW_HUD_RULES] = int[]({bottoms});
const int WINDOW_HUD_WIDTHS[WINDOW_HUD_RULES] = int[]({widths});
{gui_size}

bool window_hud_in_background_band(int offsetFromBottom, int sourceBottom) {{
    return offsetFromBottom >= sourceBottom - 24 && offsetFromBottom <= sourceBottom + 24;
}}

bool window_hud_background_matches(float x, float y) {{
    vec2 guiSize = window_hud_gui_size();
    float guiWidth = guiSize.x;
    float guiHeight = guiSize.y;
    int offsetFromBottom = int(round(guiHeight - y));
    for (int i = 0; i < WINDOW_HUD_RULES; i++) {{
        float width = float(WINDOW_HUD_WIDTHS[i]);
        float sourceLeft = guiWidth * 0.5;
        float sourceRight = sourceLeft + width;
        if (x >= sourceLeft - 16.0 && x <= sourceRight + 16.0 &&
            window_hud_in_background_band(offsetFromBottom, WINDOW_HUD_SOURCE_BOTTOMS[i])) {{
            return true;
        }}
    }}
    return false;
}}
"#
    )
}

fn format_glsl_float(value: f32) -> String {
    let mut s = format!("{value:.6}");
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    s
}

fn rgb_vec3(color: Rgb) -> String {
    format!(
        "vec3({}, {}, {})",
        format_glsl_float(color.r as f32 / 255.0),
        format_glsl_float(color.g as f32 / 255.0),
        format_glsl_float(color.b as f32 / 255.0)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShaderSegment {
    rule: usize,
    marker: Rgb,
    display: Rgb,
    texture: HudShaderTexture,
}

fn segment_table(rules: &[HudShaderRule]) -> Vec<ShaderSegment> {
    rules
        .iter()
        .enumerate()
        .flat_map(|(rule, hud_rule)| {
            hud_rule.segments.iter().copied().map(move |segment| ShaderSegment {
                rule,
                marker: segment.marker,
                display: segment.display,
                texture: segment.texture,
            })
        })
        .collect()
}

impl HudShaderTexture {
    fn mode(self) -> i32 {
        match self {
            Self::Intensity => 0,
            Self::Rgba => 1,
        }
    }
}

pub(super) fn marker_id(marker: Rgb) -> u16 {
    debug_assert_eq!(marker.g, 0);
    ((marker.r as u16) << 8) | marker.b as u16
}

pub(super) fn render_v150_text_shader(
    rules: &[HudShaderRule],
    namespaced_import: bool,
    legacy_fog: bool,
    texture_mode_output: bool,
    see_through: bool,
) -> String {
    let import = if see_through {
        ""
    } else if namespaced_import {
        "#moj_import <minecraft:fog.glsl>"
    } else {
        "#moj_import <fog.glsl>"
    };
    let ivew = if legacy_fog && !see_through { "\nuniform mat3 IViewRotMat;" } else { "" };
    let fog_shape = if see_through { "" } else { "\nuniform int FogShape;" };
    let fog_distance = if legacy_fog {
        "fog_distance(ModelViewMat, IViewRotMat * Position, FogShape)"
    } else {
        "fog_distance(Position, FogShape)"
    };
    let vertex_distance_decl = if see_through { "" } else { "out float vertexDistance;\n" };
    let vertex_distance_assign =
        if see_through { String::new() } else { format!("\n    vertexDistance = {fog_distance};") };
    let helpers = render_text_helpers(rules, ScreenSizeSource::ProjectionMatrix, texture_mode_output);
    let texture_mode_decl = if texture_mode_output { "\nflat out int windowHudTextureMode;" } else { "" };
    let texture_mode_assign =
        if texture_mode_output { "\n    windowHudTextureMode = window_hud_texture_mode(Color);" } else { "" };

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

pub(super) fn render_v150_text_background_shader(
    rules: &[HudShaderRule],
    namespaced_import: bool,
    legacy_fog: bool,
    see_through: bool,
) -> String {
    let import = if see_through {
        ""
    } else if namespaced_import {
        "#moj_import <minecraft:fog.glsl>"
    } else {
        "#moj_import <fog.glsl>"
    };
    let ivew = if legacy_fog && !see_through { "\nuniform mat3 IViewRotMat;" } else { "" };
    let fog_shape = if see_through { "" } else { "\nuniform int FogShape;" };
    let fog_distance = if legacy_fog {
        "fog_distance(ModelViewMat, IViewRotMat * Position, FogShape)"
    } else {
        "fog_distance(Position, FogShape)"
    };
    let vertex_distance_decl = if see_through { "" } else { "out float vertexDistance;\n" };
    let vertex_distance_assign =
        if see_through { String::new() } else { format!("\n    vertexDistance = {fog_distance};") };
    let helpers = render_background_helpers(rules, ScreenSizeSource::ProjectionMatrix);

    format!(
        r#"#version 150

{import}

in vec3 Position;
in vec4 Color;
in ivec2 UV2;

uniform sampler2D Sampler2;

uniform mat4 ModelViewMat;
uniform mat4 ProjMat;{ivew}
{fog_shape}

{vertex_distance_decl}
out vec4 vertexColor;
{helpers}
void main() {{
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);
{vertex_distance_assign}
    vertexColor = Color * texelFetch(Sampler2, UV2 / 16, 0);
    if (window_hud_background_matches(Position.x, Position.y)) {{
        vertexColor.a = 0.0;
    }}
}}
"#
    )
}

pub(super) fn render_v150_dynamic_text_shader(
    rules: &[HudShaderRule],
    texture_mode_output: bool,
    see_through: bool,
) -> String {
    let fog_import = if see_through { "" } else { "#moj_import <minecraft:fog.glsl>\n" };
    let fog_outputs =
        if see_through { "" } else { "out float sphericalVertexDistance;\nout float cylindricalVertexDistance;\n" };
    let fog_assign = if see_through {
        ""
    } else {
        "\n    sphericalVertexDistance = fog_spherical_distance(Position);\n    cylindricalVertexDistance = fog_cylindrical_distance(Position);"
    };
    let helpers = render_text_helpers(rules, ScreenSizeSource::Globals, texture_mode_output);
    let texture_mode_decl = if texture_mode_output { "\nflat out int windowHudTextureMode;" } else { "" };
    let texture_mode_assign =
        if texture_mode_output { "\n    windowHudTextureMode = window_hud_texture_mode(Color);" } else { "" };
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

pub(super) fn render_v150_dynamic_text_background_shader(rules: &[HudShaderRule], see_through: bool) -> String {
    let fog_import = if see_through { "" } else { "#moj_import <minecraft:fog.glsl>\n" };
    let fog_outputs =
        if see_through { "" } else { "out float sphericalVertexDistance;\nout float cylindricalVertexDistance;\n" };
    let fog_assign = if see_through {
        ""
    } else {
        "\n    sphericalVertexDistance = fog_spherical_distance(Position);\n    cylindricalVertexDistance = fog_cylindrical_distance(Position);"
    };
    let helpers = render_background_helpers(rules, ScreenSizeSource::Globals);
    format!(
        r#"#version 150

{fog_import}
#moj_import <minecraft:globals.glsl>
#moj_import <minecraft:dynamictransforms.glsl>
#moj_import <minecraft:projection.glsl>

in vec3 Position;
in vec4 Color;
in ivec2 UV2;

uniform sampler2D Sampler2;

{fog_outputs}
out vec4 vertexColor;
{helpers}
void main() {{
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);
{fog_assign}
    vertexColor = Color * texelFetch(Sampler2, UV2 / 16, 0);
    if (window_hud_background_matches(Position.x, Position.y)) {{
        vertexColor.a = 0.0;
    }}
}}
"#
    )
}

pub(super) fn render_v330_text_shader(rules: &[HudShaderRule], texture_mode_output: bool, see_through: bool) -> String {
    let fog_import = if see_through { "" } else { "#moj_import <minecraft:fog.glsl>\n" };
    let fog_outputs =
        if see_through { "" } else { "out float sphericalVertexDistance;\nout float cylindricalVertexDistance;\n" };
    let fog_assign = if see_through {
        ""
    } else {
        "\n    sphericalVertexDistance = fog_spherical_distance(Position);\n    cylindricalVertexDistance = fog_cylindrical_distance(Position);"
    };
    let helpers = render_text_helpers(rules, ScreenSizeSource::Globals, texture_mode_output);
    let texture_mode_decl = if texture_mode_output { "\nflat out int windowHudTextureMode;" } else { "" };
    let texture_mode_assign =
        if texture_mode_output { "\n    windowHudTextureMode = window_hud_texture_mode(Color);" } else { "" };
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

fn intensity_sample_function() -> &'static str {
    r#"
vec4 window_hud_intensity_sample(sampler2D sampler, vec2 coord) {
    vec4 texel = texture(sampler, coord);
    if (windowHudTextureMode == 1) {
        return texel;
    }
    return texel.rrrr;
}
"#
}

pub(super) fn render_v150_text_intensity_fragment(
    namespaced_import: bool,
    dynamic_fog: bool,
    see_through: bool,
) -> String {
    let import = if namespaced_import { "#moj_import <minecraft:fog.glsl>" } else { "#moj_import <fog.glsl>" };
    if see_through {
        return r#"#version 150

uniform sampler2D Sampler0;

uniform vec4 ColorModulator;

in vec4 vertexColor;
in vec2 texCoord0;
flat in int windowHudTextureMode;

out vec4 fragColor;
"#
        .to_string()
            + intensity_sample_function()
            + r#"
void main() {
    vec4 color = window_hud_intensity_sample(Sampler0, texCoord0) * vertexColor;
    if (color.a < 0.1) {
        discard;
    }
    fragColor = color * ColorModulator;
}
"#;
    }
    if dynamic_fog {
        return render_dynamic_text_intensity_fragment(150, false);
    }

    format!(
        r#"#version 150

{import}

uniform sampler2D Sampler0;

uniform vec4 ColorModulator;
uniform float FogStart;
uniform float FogEnd;
uniform vec4 FogColor;

in float vertexDistance;
in vec4 vertexColor;
in vec2 texCoord0;
flat in int windowHudTextureMode;

out vec4 fragColor;
{sample}
void main() {{
    vec4 color = window_hud_intensity_sample(Sampler0, texCoord0) * vertexColor * ColorModulator;
    if (color.a < 0.1) {{
        discard;
    }}
    fragColor = linear_fog(color, vertexDistance, FogStart, FogEnd, FogColor);
}}
"#,
        sample = intensity_sample_function()
    )
}

pub(super) fn render_dynamic_text_intensity_fragment(version: u32, see_through: bool) -> String {
    if see_through {
        return format!(
            r#"#version {version}

#moj_import <minecraft:dynamictransforms.glsl>

uniform sampler2D Sampler0;

in vec4 vertexColor;
in vec2 texCoord0;
flat in int windowHudTextureMode;

out vec4 fragColor;
{sample}
void main() {{
    vec4 color = window_hud_intensity_sample(Sampler0, texCoord0) * vertexColor;
    if (color.a < 0.1) {{
        discard;
    }}
    fragColor = color * ColorModulator;
}}
"#,
            sample = intensity_sample_function()
        );
    }

    format!(
        r#"#version {version}

#moj_import <minecraft:fog.glsl>
#moj_import <minecraft:dynamictransforms.glsl>

uniform sampler2D Sampler0;

in float sphericalVertexDistance;
in float cylindricalVertexDistance;
in vec4 vertexColor;
in vec2 texCoord0;
flat in int windowHudTextureMode;

out vec4 fragColor;
{sample}
void main() {{
    vec4 color = window_hud_intensity_sample(Sampler0, texCoord0) * vertexColor * ColorModulator;
    if (color.a < 0.1) {{
        discard;
    }}
    fragColor = apply_fog(color, sphericalVertexDistance, cylindricalVertexDistance, FogEnvironmentalStart, FogEnvironmentalEnd, FogRenderDistanceStart, FogRenderDistanceEnd, FogColor);
}}
"#,
        sample = intensity_sample_function()
    )
}

pub(super) fn render_v330_text_background_shader(rules: &[HudShaderRule], see_through: bool) -> String {
    let fog_import = if see_through { "" } else { "#moj_import <minecraft:fog.glsl>\n" };
    let fog_outputs =
        if see_through { "" } else { "out float sphericalVertexDistance;\nout float cylindricalVertexDistance;\n" };
    let fog_assign = if see_through {
        ""
    } else {
        "\n    sphericalVertexDistance = fog_spherical_distance(Position);\n    cylindricalVertexDistance = fog_cylindrical_distance(Position);"
    };
    let helpers = render_background_helpers(rules, ScreenSizeSource::Globals);
    format!(
        r#"#version 330

{fog_import}
#moj_import <minecraft:globals.glsl>
#moj_import <minecraft:dynamictransforms.glsl>
#moj_import <minecraft:projection.glsl>
#moj_import <minecraft:sample_lightmap.glsl>

in vec3 Position;
in vec4 Color;
in ivec2 UV2;

uniform sampler2D Sampler2;

{fog_outputs}
out vec4 vertexColor;
{helpers}
void main() {{
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);
{fog_assign}
    vertexColor = Color * sample_lightmap(Sampler2, UV2);
    if (window_hud_background_matches(Position.x, Position.y)) {{
        vertexColor.a = 0.0;
    }}
}}
"#
    )
}
