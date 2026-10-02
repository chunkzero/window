const INTENSITY_SAMPLE_FUNCTION: &str = r#"
vec4 window_hud_intensity_sample(sampler2D sampler, vec2 coord) {
    vec4 texel = texture(sampler, coord);
    if (windowHudTextureMode == 1) {
        return texel;
    }
    return texel.rrrr;
}
"#;

pub(in crate::hud) fn render_v150_text_intensity_fragment(
    namespaced_import: bool,
    dynamic_fog: bool,
    see_through: bool,
) -> String {
    if see_through {
        return render_v150_see_through_intensity_fragment();
    }
    if dynamic_fog {
        return render_dynamic_text_intensity_fragment(150, false);
    }
    render_v150_fogged_intensity_fragment(namespaced_import)
}

fn render_v150_see_through_intensity_fragment() -> String {
    r#"#version 150

uniform sampler2D Sampler0;

uniform vec4 ColorModulator;

in vec4 vertexColor;
in vec2 texCoord0;
flat in int windowHudTextureMode;

out vec4 fragColor;
"#
    .to_string()
        + INTENSITY_SAMPLE_FUNCTION
        + r#"
void main() {
    vec4 color = window_hud_intensity_sample(Sampler0, texCoord0) * vertexColor;
    if (color.a < 0.1) {
        discard;
    }
    fragColor = color * ColorModulator;
}
"#
}

fn render_v150_fogged_intensity_fragment(namespaced_import: bool) -> String {
    let import = if namespaced_import { "#moj_import <minecraft:fog.glsl>" } else { "#moj_import <fog.glsl>" };

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
        sample = INTENSITY_SAMPLE_FUNCTION
    )
}

pub(in crate::hud) fn render_dynamic_text_intensity_fragment(version: u32, see_through: bool) -> String {
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
            sample = INTENSITY_SAMPLE_FUNCTION
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
        sample = INTENSITY_SAMPLE_FUNCTION
    )
}
