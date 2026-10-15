//! The pack_format 85-88 `core/entity` vertex shader of horse-mode full-screen windows: vanilla's, plus a branch that
//! draws the cursor from the horse's armour. See [`crate::fullscreen::horse`].

use crate::fullscreen::LAYER_DEPTH_STEP;
use crate::fullscreen::horse::{ARMOR_CURSOR_X, ARMOR_SIGNATURE, OFFSET};

use super::fullscreen::CANVAS_PLACEMENT;

const ENTITY_VERTEX: &str = r#"#version 330

#if defined(PER_FACE_LIGHTING) || !defined(NO_CARDINAL_LIGHTING)
#moj_import <minecraft:light.glsl>
#endif
#moj_import <minecraft:fog.glsl>
#moj_import <minecraft:dynamictransforms.glsl>
#moj_import <minecraft:projection.glsl>
#moj_import <minecraft:sample_lightmap.glsl>
#if defined(NO_OVERLAY) && defined(PER_FACE_LIGHTING)
#moj_import <minecraft:globals.glsl>
#moj_import <minecraft:window_fullscreen.glsl>
#endif

in vec3 Position;
in vec4 Color;
in vec2 UV0;
in ivec2 UV1;
in ivec2 UV2;
in vec3 Normal;

#ifndef NO_OVERLAY
uniform sampler2D Sampler1;
#endif

#ifndef EMISSIVE
uniform sampler2D Sampler2;
#endif

out float sphericalVertexDistance;
out float cylindricalVertexDistance;

#ifdef PER_FACE_LIGHTING
out vec4 vertexPerFaceColorBack;
out vec4 vertexPerFaceColorFront;
#else
out vec4 vertexColor;
#endif

#ifndef EMISSIVE
out vec4 lightMapColor;
#endif

#ifndef NO_OVERLAY
out vec4 overlayColor;
#endif

out vec2 texCoord0;

#if defined(NO_OVERLAY) && defined(PER_FACE_LIGHTING)
// Equipment layers, which draw the horse's cursor armour.
uniform sampler2D Sampler0;

// Texel (0, 0) of the cursor armour's texture.
const ivec4 WINDOW_FS_ARMOR_SIGNATURE = ivec4(@SIGNATURE@);
// Where the cursor starts in the armour's texture, in texels.
const vec2 WINDOW_FS_ARMOR_CURSOR = vec2(@CURSOR_X@.0, 0.0);
// Where the horse stands relative to the camera, in blocks.
const vec3 WINDOW_FS_HORSE_OFFSET = vec3(@OFFSET@);
// The depth distance between adjacent layers.
const float WINDOW_FS_LAYER_DEPTH_STEP = @DEPTH_STEP@;
// Below every fog start, so the cursor is never fogged.
const float WINDOW_FS_NO_FOG_DISTANCE = -1.0e30;
// Outside the clip volume: a triangle with every corner here has no area.
const vec4 WINDOW_FS_HIDDEN = vec4(2.0, 2.0, 2.0, 1.0);

// The vanilla horse armour model: the equine body mesh inflated by 0.1, drawn at the adult scale of 1.1. In the
// horse's unturned space (facing -Z, Y up, in blocks), the head parts pivot at WINDOW_FS_HEAD_PIVOT and pitch about X
// by 30 degrees plus the horse's pitch. WINDOW_FS_HEAD_TOP is the head cube's top face in the head parts' pixels,
// before the model's Y flip.
const vec3 WINDOW_FS_HEAD_PIVOT = vec3(0.0, 1.3761, -0.825);
const float WINDOW_FS_HORSE_SCALE = 1.1 / 16.0;
const float WINDOW_FS_HEAD_REST_PITCH = 30.0;
const float WINDOW_FS_HEAD_TOP = -11.1;

@PLACEMENT@

bool window_fs_is_cursor_armor() {
    return ivec4(round(texelFetch(Sampler0, ivec2(0), 0) * 255.0)) == WINDOW_FS_ARMOR_SIGNATURE;
}

// Draws the head cube's top face as the cursor and hides every other vertex of the cursor armour. Each corner recovers
// the horse's yaw and pitch from its own position relative to the camera: the horse stands at WINDOW_FS_HORSE_OFFSET
// and only turns about its origin, and the client sets its yaw to the rider's and its pitch to half the rider's.
void window_fs_horse_cursor() {
    gl_Position = WINDOW_FS_HIDDEN;
    sphericalVertexDistance = WINDOW_FS_NO_FOG_DISTANCE;
    cylindricalVertexDistance = WINDOW_FS_NO_FOG_DISTANCE;
    vertexPerFaceColorBack = vec4(1.0);
    vertexPerFaceColorFront = vec4(1.0);
#ifndef EMISSIVE
    lightMapColor = vec4(1.0);
#endif
    texCoord0 = UV0;

    // Texels of the model's 64x64 texture space. The top face's corners share UVs with corners of the side faces and,
    // at u = 13, the bottom face; the normal tells them apart below.
    vec2 uv = round(UV0 * 64.0);
    if (!((uv.x == 7.0 || uv.x == 13.0) && (uv.y == 13.0 || uv.y == 20.0)) || Normal.y <= 0.1) {
        return;
    }
    vec3 head = vec3(uv.x == 13.0 ? 3.1 : -3.1, WINDOW_FS_HEAD_TOP, uv.y == 13.0 ? 5.1 : -2.1);
    vec3 local = Position - WINDOW_FS_HORSE_OFFSET;

    // Turning keeps the corner's height and horizontal distance, which fix its unturned x and z.
    float x = -WINDOW_FS_HORSE_SCALE * head.x;
    float z = -sqrt(max(dot(local.xz, local.xz) - x * x, 0.0));
    vec2 pitched = vec2(WINDOW_FS_HEAD_PIVOT.y - local.y, z - WINDOW_FS_HEAD_PIVOT.z) / WINDOW_FS_HORSE_SCALE;
    float headPitch = degrees(atan(pitched.y, pitched.x) - atan(head.z, head.y));
    headPitch -= 360.0 * floor((headPitch + 180.0) / 360.0);
    // The renderer turns the model by 180 - yaw about Y.
    float yaw = 180.0 - degrees(atan(local.x, local.z)) + degrees(atan(x, z));

    float turn = radians(180.0 - yaw);
    float tilt = radians(headPitch);
    vec3 topNormal = vec3(-sin(tilt) * sin(turn), cos(tilt), -sin(tilt) * cos(turn));
    if (dot(Normal, topNormal) <= 0.9) {
        return;
    }

    vec2 cursor = window_fs_cursor_at(yaw, 2.0 * (headPitch - WINDOW_FS_HEAD_REST_PITCH));
    vec2 corner = vec2(uv.x == 13.0 ? 1.0 : 0.0, uv.y == 20.0 ? 1.0 : 0.0) * WINDOW_FS_CURSOR_SIZE;
    gl_Position = vec4(window_fs_ndc(cursor + corner), window_fs_depth(WINDOW_FS_CURSOR_LAYER), 1.0);
    texCoord0 = (WINDOW_FS_ARMOR_CURSOR + corner) / vec2(textureSize(Sampler0, 0));
}
#endif

void main() {
#if defined(NO_OVERLAY) && defined(PER_FACE_LIGHTING)
    if (window_fs_is_cursor_armor()) {
        window_fs_horse_cursor();
        return;
    }
#endif
    gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0);

    sphericalVertexDistance = fog_spherical_distance(Position);
    cylindricalVertexDistance = fog_cylindrical_distance(Position);

#ifdef PER_FACE_LIGHTING
    vec2 light = minecraft_compute_light(Light0_Direction, Light1_Direction, Normal);
    vertexPerFaceColorBack = minecraft_mix_light_separate(-light, Color);
    vertexPerFaceColorFront = minecraft_mix_light_separate(light, Color);
#elif defined(NO_CARDINAL_LIGHTING)
    vertexColor = Color;
#else
    vertexColor = minecraft_mix_light(Light0_Direction, Light1_Direction, Normal, Color);
#endif

#ifndef EMISSIVE
    lightMapColor = sample_lightmap(Sampler2, UV2);
#endif

#ifndef NO_OVERLAY
    overlayColor = texelFetch(Sampler1, UV1, 0);
#endif

    texCoord0 = UV0;

#ifdef APPLY_TEXTURE_MATRIX
    texCoord0 = (TextureMat * vec4(UV0, 0.0, 1.0)).xy;
#endif
}
"#;

/// Renders `core/entity.vsh`. Equipment layers (`NO_OVERLAY` with `PER_FACE_LIGHTING`) whose texture starts with
/// the armour signature draw the cursor; everything else takes vanilla's path.
pub(in crate::hud) fn render_horse_entity_vertex() -> String {
    let join = |values: &mut dyn Iterator<Item = String>| values.collect::<Vec<_>>().join(", ");
    ENTITY_VERTEX
        .replace("@PLACEMENT@", CANVAS_PLACEMENT)
        .replace("@SIGNATURE@", &join(&mut ARMOR_SIGNATURE.iter().map(u8::to_string)))
        .replace("@CURSOR_X@", &ARMOR_CURSOR_X.to_string())
        .replace("@OFFSET@", &join(&mut OFFSET.iter().map(|value| format!("{value:?}"))))
        .replace("@DEPTH_STEP@", &format!("{LAYER_DEPTH_STEP:?}"))
}
