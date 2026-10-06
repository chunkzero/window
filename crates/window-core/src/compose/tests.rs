use super::*;

/// Build a solid-color texture.
fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Texture {
    Texture { width, height, rgba: rgba.iter().copied().cycle().take((width * height * 4) as usize).collect() }
}

/// Build a texture from an explicit list of pixels (row-major).
fn from_pixels(width: u32, height: u32, pixels: &[[u8; 4]]) -> Texture {
    assert_eq!(pixels.len() as u32, width * height);
    Texture { width, height, rgba: pixels.iter().flatten().copied().collect() }
}

fn pixel(t: &Texture, x: u32, y: u32) -> [u8; 4] {
    t.get(x, y)
}

#[test]
fn sprite_blit_is_one_to_one() {
    let mut textures = BTreeMap::new();
    let src = from_pixels(2, 2, &[[10, 20, 30, 255], [40, 50, 60, 255], [70, 80, 90, 255], [100, 110, 120, 255]]);
    textures.insert("s.png".to_string(), src);
    let draws = vec![Draw::Sprite { texture: TextureKey("s.png".into()), dest: Rect::new(1, 1, 2, 2) }];
    let c = compose_draws(&draws, "w", &textures).unwrap();
    assert!(c.has_content);
    assert_eq!(c.bounds, Rect::new(1, 1, 2, 2));
    assert_eq!(pixel(&c.texture, 0, 0), [10, 20, 30, 255]);
    assert_eq!(pixel(&c.texture, 1, 0), [40, 50, 60, 255]);
    assert_eq!(pixel(&c.texture, 0, 1), [70, 80, 90, 255]);
    assert_eq!(pixel(&c.texture, 1, 1), [100, 110, 120, 255]);
}

#[test]
fn alpha_over_blends_straight_alpha() {
    // Opaque red base, then 50% green on top.
    let opaque = over([0, 255, 0, 128], [255, 0, 0, 255]);
    // out_a = 128 + 255*(255-128)/255 = 128 + 127 = 255.
    assert_eq!(opaque[3], 255);
    // R: src_r=0, dst_r=255. num = 0 + 255*255*127; den = 255*255 -> 127.
    assert_eq!(opaque[0], 127);
    // G: src_g=255. num = 255*128*255; den = 255*255 -> 128.
    assert_eq!(opaque[1], 128);
    assert_eq!(opaque[2], 0);
}

#[test]
fn over_onto_transparent_keeps_src() {
    let out = over([10, 20, 30, 200], [0, 0, 0, 0]);
    assert_eq!(out, [10, 20, 30, 200]);
}

#[test]
fn fully_transparent_src_is_noop() {
    let mut textures = BTreeMap::new();
    textures.insert("base.png".to_string(), solid(2, 2, [1, 2, 3, 255]));
    textures.insert("clear.png".to_string(), solid(2, 2, [9, 9, 9, 0]));
    let draws = vec![
        Draw::Sprite { texture: TextureKey("base.png".into()), dest: Rect::new(0, 0, 2, 2) },
        Draw::Sprite { texture: TextureKey("clear.png".into()), dest: Rect::new(0, 0, 2, 2) },
    ];
    let c = compose_draws(&draws, "w", &textures).unwrap();
    assert_eq!(pixel(&c.texture, 0, 0), [1, 2, 3, 255]);
}

#[test]
fn union_bounds_with_negative_origin() {
    let mut textures = BTreeMap::new();
    textures.insert("a.png".to_string(), solid(2, 2, [255, 0, 0, 255]));
    textures.insert("b.png".to_string(), solid(2, 2, [0, 255, 0, 255]));
    let draws = vec![
        Draw::Sprite { texture: TextureKey("a.png".into()), dest: Rect::new(-3, -1, 2, 2) },
        Draw::Sprite { texture: TextureKey("b.png".into()), dest: Rect::new(2, 3, 2, 2) },
    ];
    let c = compose_draws(&draws, "w", &textures).unwrap();
    // x from -3..4 -> width 7; y from -1..5 -> height 6.
    assert_eq!(c.bounds, Rect::new(-3, -1, 7, 6));
    // a.png lands at local (0,0)..(2,2).
    assert_eq!(pixel(&c.texture, 0, 0), [255, 0, 0, 255]);
    // b.png at dest (2,3) -> local (5,4).
    assert_eq!(pixel(&c.texture, 5, 4), [0, 255, 0, 255]);
    // empty corner is transparent.
    assert_eq!(pixel(&c.texture, 6, 0), [0, 0, 0, 0]);
}

#[test]
fn empty_draws_have_no_content() {
    let textures = BTreeMap::new();
    let c = compose_draws(&[], "w", &textures).unwrap();
    assert!(!c.has_content);
    assert_eq!(c.bounds, Rect::new(0, 0, 0, 0));
    assert_eq!(c.texture.width, 0);
    assert_eq!(c.texture.height, 0);
}

#[test]
fn missing_texture_errors() {
    let textures = BTreeMap::new();
    let draws = vec![Draw::Sprite { texture: TextureKey("nope.png".into()), dest: Rect::new(0, 0, 2, 2) }];
    assert!(compose_draws(&draws, "w", &textures).is_err());
}

#[test]
fn nine_slice_3x3_inset_stretched_larger() {
    // 3×3 source: distinct corner colors, edges, and a center.
    // Layout (1px insets all around):
    //   C T C
    //   L M R
    //   C B C
    let c = [255, 0, 0, 255]; // corners
    let t = [0, 255, 0, 255]; // top edge
    let l = [0, 0, 255, 255]; // left edge
    let m = [255, 255, 0, 255]; // center
    let r = [255, 0, 255, 255]; // right edge
    let b = [0, 255, 255, 255]; // bottom edge
    let src = from_pixels(3, 3, &[c, t, c, l, m, r, c, b, c]);
    let mut textures = BTreeMap::new();
    textures.insert("frame.png".to_string(), src);
    let draws = vec![Draw::NineSlice {
        texture: TextureKey("frame.png".into()),
        insets: Insets::uniform(1),
        dest: Rect::new(0, 0, 5, 5),
    }];
    let comp = compose_draws(&draws, "w", &textures).unwrap();
    let img = &comp.texture;
    // Corners stay 1:1 at the four extremes.
    assert_eq!(pixel(img, 0, 0), c);
    assert_eq!(pixel(img, 4, 0), c);
    assert_eq!(pixel(img, 0, 4), c);
    assert_eq!(pixel(img, 4, 4), c);
    // Top edge spans the middle 3 columns of row 0.
    assert_eq!(pixel(img, 1, 0), t);
    assert_eq!(pixel(img, 2, 0), t);
    assert_eq!(pixel(img, 3, 0), t);
    // Left edge spans middle rows of column 0.
    assert_eq!(pixel(img, 0, 1), l);
    assert_eq!(pixel(img, 0, 3), l);
    // Right edge.
    assert_eq!(pixel(img, 4, 2), r);
    // Bottom edge.
    assert_eq!(pixel(img, 2, 4), b);
    // Center fills the 3×3 interior.
    assert_eq!(pixel(img, 2, 2), m);
    assert_eq!(pixel(img, 1, 3), m);
}

#[test]
fn nine_slice_degenerate_clamps_corners() {
    // Source corners are 2px; dest is only 3px wide → left corner 2, right 1.
    let src = solid(6, 6, [100, 100, 100, 255]);
    let mut textures = BTreeMap::new();
    textures.insert("f.png".to_string(), src);
    let draws = vec![Draw::NineSlice {
        texture: TextureKey("f.png".into()),
        insets: Insets::uniform(2),
        dest: Rect::new(0, 0, 3, 3),
    }];
    let comp = compose_draws(&draws, "w", &textures).unwrap();
    // No panic and image is the requested size; all pixels written.
    assert_eq!(comp.texture.width, 3);
    assert_eq!(comp.texture.height, 3);
    for y in 0..3 {
        for x in 0..3 {
            assert_eq!(pixel(&comp.texture, x, y), [100, 100, 100, 255]);
        }
    }
}

#[test]
fn png_round_trip_is_deterministic() {
    let t = from_pixels(2, 1, &[[1, 2, 3, 255], [4, 5, 6, 128]]);
    let a = t.encode_png().unwrap();
    let b = t.encode_png().unwrap();
    assert_eq!(a, b);
    let decoded = Texture::decode_png(&a).unwrap();
    assert_eq!(decoded, t);
}

#[test]
fn encode_png_rejects_buffers_smaller_than_dimensions() {
    for texture in [
        Texture { width: u32::MAX, height: u32::MAX, rgba: Vec::new() },
        Texture { width: 2, height: 2, rgba: vec![0; 15] },
    ] {
        assert!(matches!(texture.encode_png(), Err(Error::Texture { .. })));
    }
}

#[test]
fn over_fast_paths_match_general_formula() {
    fn general(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
        let (sa, da) = (src[3] as u32, dst[3] as u32);
        let out_a = sa + (da * (255 - sa) + 127) / 255;
        let mut out = [0, 0, 0, out_a as u8];
        for c in 0..3 {
            let num = (src[c] as u32) * sa * 255 + (dst[c] as u32) * da * (255 - sa);
            out[c] = ((num + out_a * 255 / 2) / (out_a * 255)) as u8;
        }
        out
    }
    for a in 0..=255u8 {
        for color in [0, 1, 127, 128, 254, 255] {
            let opaque = [color, 255 - color, color / 2, 255];
            let dst = [255 - color, color, 77, a];
            assert_eq!(over(opaque, dst), general(opaque, dst));
            if a != 0 {
                let src = [color, 255 - color, color / 2, a];
                assert_eq!(over(src, [0, 0, 0, 0]), general(src, [0, 0, 0, 0]));
            }
        }
    }
}

#[test]
fn repeated_generated_draws_match_individual_renders() {
    use crate::model::{GeneratedKind, GeneratedStyle};
    let draws: Vec<Draw> = [
        (GeneratedKind::Panel, Rect::new(-4, -3, 30, 20)),
        (GeneratedKind::Button, Rect::new(2, 5, 18, 18)),
        (GeneratedKind::Button, Rect::new(12, 9, 18, 18)),
        (GeneratedKind::Badge, Rect::new(20, 0, 9, 9)),
        (GeneratedKind::Button, Rect::new(0, 20, 18, 18)),
    ]
    .into_iter()
    .map(|(kind, dest)| Draw::Generated { style: GeneratedStyle::defaults(kind), dest })
    .collect();

    let composite = compose_draws(&draws, "w", &BTreeMap::new()).unwrap();

    let bounds = composite.bounds;
    let mut expected = Texture::transparent(bounds.width, bounds.height);
    for draw in &draws {
        let Draw::Generated { style, dest } = draw else { unreachable!() };
        let texture = crate::raster::render(style, dest.size()).unwrap();
        for y in 0..texture.height {
            for x in 0..texture.width {
                let (dx, dy) = (dest.x - bounds.x + x as i32, dest.y - bounds.y + y as i32);
                expected.blend(dx as u32, dy as u32, texture.get(x, y));
            }
        }
    }
    assert_eq!(composite.texture, expected);
}
