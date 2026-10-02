use crate::hud::rule::HudShaderRule;

use super::glsl::{
    DynamicFog, ScreenSizeSource, V150Fog, dynamic_fog, join_rule_values, render_gui_size_helper, v150_fog,
};

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

pub(in crate::hud) fn render_v150_text_background_shader(
    rules: &[HudShaderRule],
    namespaced_import: bool,
    legacy_fog: bool,
    see_through: bool,
) -> String {
    let V150Fog { import, ivew, fog_shape, vertex_distance_decl, vertex_distance_assign } =
        v150_fog(namespaced_import, legacy_fog, see_through);
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

pub(in crate::hud) fn render_v150_dynamic_text_background_shader(rules: &[HudShaderRule], see_through: bool) -> String {
    let DynamicFog { fog_import, fog_outputs, fog_assign } = dynamic_fog(see_through);
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

pub(in crate::hud) fn render_v330_text_background_shader(rules: &[HudShaderRule], see_through: bool) -> String {
    let DynamicFog { fog_import, fog_outputs, fog_assign } = dynamic_fog(see_through);
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
