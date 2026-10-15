//! The pack_format 85-88 `core/entity` vertex shader of horse-mode full-screen windows: vanilla's, plus a branch that
//! draws the cursor from the horse's armour. See [`crate::fullscreen::horse`].

use crate::fullscreen::LAYER_DEPTH_STEP;
use crate::fullscreen::horse::{ARMOR_CURSOR_X, ARMOR_SIGNATURE, OFFSET};

use super::fullscreen::CANVAS_PLACEMENT;

/// Where the armour model's head parts pivot, in the horse's unturned space (facing -Z, Y up), in blocks: the part's
/// pose offset of (0, 4, -12) texels, scaled by 1.1 about the feet.
const HEAD_PIVOT: [f64; 3] = [0.0, 1.3761, -0.825];

/// The head cube's top face in the head parts' texels, before the model's Y flip: its box top of -11, inflated by 0.1.
const HEAD_TOP: f64 = -11.1;

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
const vec3 WINDOW_FS_HEAD_PIVOT = vec3(@HEAD_PIVOT@);
const float WINDOW_FS_HORSE_SCALE = 1.1 / 16.0;
const float WINDOW_FS_HEAD_REST_PITCH = 30.0;
const float WINDOW_FS_HEAD_TOP = @HEAD_TOP@;

@PLACEMENT@

bool window_fs_is_cursor_armor() {
    return ivec4(round(texelFetch(Sampler0, ivec2(0), 0) * 255.0)) == WINDOW_FS_ARMOR_SIGNATURE;
}

// Draws the head cube's top face as the cursor and hides every other vertex of the cursor armour. Each corner recovers
// the horse's yaw and pitch from its own position relative to the camera: the horse stands at WINDOW_FS_HORSE_OFFSET
// and only turns about its origin, and the client sets its yaw to the rider's and its pitch to half the rider's. The
// session keeps the horse at that offset, since a corner's position alone cannot tell the horse moving from it turning.
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
        .replace("@HEAD_PIVOT@", &join(&mut HEAD_PIVOT.iter().map(|value| format!("{value:?}"))))
        .replace("@HEAD_TOP@", &format!("{HEAD_TOP:?}"))
        .replace("@OFFSET@", &join(&mut OFFSET.iter().map(|value| format!("{value:?}"))))
        .replace("@DEPTH_STEP@", &format!("{LAYER_DEPTH_STEP:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fullscreen::{DEGREES_PER_PIXEL, cursor};

    const SCALE: f32 = 1.1 / 16.0;
    const REST_PITCH: f32 = 30.0;

    /// The top face's corners in the head parts' texels: (x, z) of each, with the face at `HEAD_TOP`.
    const CORNERS: [[f32; 2]; 4] = [[3.1, 5.1], [-3.1, 5.1], [3.1, -2.1], [-3.1, -2.1]];

    /// A corner's position relative to the camera for a horse at `OFFSET` turned to `yaw` with its head pitched to
    /// `head_pitch`, as the client's model transforms place it. It is the inverse of `recover`.
    fn place(yaw: f64, head_pitch: f64, [x, z]: [f32; 2]) -> [f32; 3] {
        let (x, z) = (f64::from(x), f64::from(z));
        let scale = f64::from(SCALE);
        let tilt = head_pitch.to_radians();
        let down = HEAD_TOP * tilt.cos() - z * tilt.sin();
        let forward = HEAD_TOP * tilt.sin() + z * tilt.cos();
        let (local_x, local_y, local_z) = (-scale * x, HEAD_PIVOT[1] - scale * down, HEAD_PIVOT[2] + scale * forward);
        let turn = (180.0 - yaw).to_radians();
        let turned_x = local_x * turn.cos() + local_z * turn.sin();
        let turned_z = -local_x * turn.sin() + local_z * turn.cos();
        [(turned_x + OFFSET[0]) as f32, (local_y + OFFSET[1]) as f32, (turned_z + OFFSET[2]) as f32]
    }

    /// The shader's recovery in single precision: a corner's yaw and head pitch from its position alone.
    fn recover(position: [f32; 3], [x, z]: [f32; 2]) -> (f32, f32) {
        let local = [position[0] - OFFSET[0] as f32, position[1] - OFFSET[1] as f32, position[2] - OFFSET[2] as f32];
        let unturned_x = -SCALE * x;
        let unturned_z = -(local[0] * local[0] + local[2] * local[2] - unturned_x * unturned_x).max(0.0).sqrt();
        let down = (HEAD_PIVOT[1] as f32 - local[1]) / SCALE;
        let forward = (unturned_z - HEAD_PIVOT[2] as f32) / SCALE;
        let head_pitch = (forward.atan2(down) - z.atan2(HEAD_TOP as f32)).to_degrees();
        let head_pitch = head_pitch - 360.0 * ((head_pitch + 180.0) / 360.0).floor();
        let yaw = 180.0 - local[0].atan2(local[2]).to_degrees() + unturned_x.atan2(unturned_z).to_degrees();
        (yaw - 360.0 * ((yaw + 180.0) / 360.0).floor(), head_pitch)
    }

    /// The cursor the shader draws for a corner, from `recover` and the include's mapping.
    fn drawn(position: [f32; 3], corner: [f32; 2]) -> (f64, f64) {
        let (yaw, head_pitch) = recover(position, corner);
        (f64::from(yaw), 2.0 * f64::from(head_pitch - REST_PITCH))
    }

    #[test]
    fn recovers_the_rotation_from_the_positions_vanilla_gives_the_head_corners() {
        // Each pose's corners were taken from the client's own armour model and render transforms, as
        // (yaw, rider pitch) and the camera-relative positions of the (+x, +z), (-x, +z), (+x, -z), (-x, -z) corners.
        let poses = [
            (
                (-33.66, 17.9),
                [
                    [0.749422, 0.439996, 3.740893],
                    [0.394637, 0.439996, 3.977148],
                    [0.962791, 0.128818, 4.061310],
                    [0.608005, 0.128818, 4.297565],
                ],
            ),
            (
                (45.0, 45.0),
                [
                    [-0.709833, 0.368831, 4.011238],
                    [-1.011238, 0.368831, 3.709833],
                    [-0.922911, -0.023879, 4.224315],
                    [-1.224315, -0.023879, 3.922910],
                ],
            ),
            (
                (-120.0, 80.0),
                [
                    [1.125084, 0.216584, 2.104337],
                    [1.338209, 0.216584, 2.473481],
                    [1.271702, -0.248564, 2.019687],
                    [1.484827, -0.248564, 2.388831],
                ],
            ),
        ];
        for ((yaw, rider_pitch), corners) in poses {
            for (position, corner) in corners.into_iter().zip(CORNERS) {
                let (got_yaw, head_pitch) = recover(position.map(|value| value as f32), corner);
                assert!((got_yaw - yaw).abs() < 0.002, "yaw {yaw}: {got_yaw}");
                let want = REST_PITCH + rider_pitch / 2.0;
                assert!((head_pitch - want).abs() < 0.002, "pitch {rider_pitch}: {}", head_pitch * 2.0 - 60.0);
            }
        }
    }

    #[test]
    fn the_drawn_cursor_is_the_servers_across_the_whole_canvas() {
        // Each pixel's center, from the edges in: well clear of the boundaries the snap epsilon decides.
        let (width, height) = (480, 270);
        for y in (0..height).step_by(13).chain([height - 1]) {
            for x in (0..width).step_by(17).chain([width - 1]) {
                let yaw = (f64::from(x) + 0.5 - f64::from(width) / 2.0) * DEGREES_PER_PIXEL;
                let pitch = (f64::from(y) + 0.5 - f64::from(height) / 2.0) * DEGREES_PER_PIXEL;
                let want = cursor(yaw, pitch);
                for corner in CORNERS {
                    let (got_yaw, got_pitch) = drawn(place(yaw, REST_PITCH as f64 + pitch / 2.0, corner), corner);
                    assert_eq!(cursor(got_yaw, got_pitch), want, "{x}, {y} via {corner:?}");
                }
            }
        }
    }

    #[test]
    fn a_horse_off_its_offset_reads_as_turned() {
        // Why the session pins the horse: a rise of 0.04 blocks alone moves the cursor several pixels.
        let corner = CORNERS[1];
        let mut position = place(0.0, REST_PITCH as f64, corner);
        position[1] += 0.04;
        let (_, pitch) = drawn(position, corner);
        assert!(pitch.abs() > 2.0 * DEGREES_PER_PIXEL, "{pitch}");
    }
}
