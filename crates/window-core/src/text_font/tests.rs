use std::collections::BTreeMap;

use super::*;

fn sheet(width: u32, height: u32, opaque: &[(u32, u32)]) -> Texture {
    let mut texture = Texture::transparent(width, height);
    for &(x, y) in opaque {
        let index = ((y * width + x) * 4) as usize;
        texture.rgba[index..index + 4].copy_from_slice(&[255; 4]);
    }
    texture
}

#[test]
fn bundled_small_caps_measures_its_glyphs_and_falls_back_to_vanilla() {
    let font = small_caps();
    assert_eq!(font.advance('a'), Some(5));
    assert_eq!(font.advance('A'), Some(5));
    assert_eq!(font.advance('M'), Some(6));
    assert_eq!(font.advance('1'), Some(4));
    assert_eq!(font.glyph_width('1'), Some(3));
    assert_eq!(font.advance('&'), vanilla::advance('&'));
    assert_eq!(font.metrics().advances[&'a'], 5);
    assert_eq!(font.metrics().glyph_widths[&' '], 0);
}

#[test]
fn project_fonts_replace_the_bundled_small_caps() {
    let textures = BTreeMap::from([
        ("window/fonts/tiny.png".to_string(), sheet(4, 8, &[(0, 0), (2, 7), (3, 7)])),
        ("window/fonts/tall.png".to_string(), sheet(4, 16, &[(0, 0), (2, 0)])),
    ]);
    let def = |texture: &str| FontDef { texture: texture.into(), chars: vec!["ab".into()] };
    let fonts = resolve(&BTreeMap::from([(SMALL_CAPS.to_string(), def("window/fonts/tiny.png"))]), &textures).unwrap();
    assert_eq!(fonts.len(), 1);
    assert_eq!(fonts[SMALL_CAPS].advance('a'), Some(2));
    assert_eq!(fonts[SMALL_CAPS].advance('b'), Some(3));

    let err = resolve(&BTreeMap::from([("tall".to_string(), def("window/fonts/tall.png"))]), &textures).unwrap_err();
    assert!(err.to_string().contains("does not divide into 2x1 cells 8px tall"), "{err}");
}

#[test]
fn rejects_mapped_cells_without_ink() {
    let err = TextFont::new("tiny", sheet(4, 8, &[(0, 0)]), vec!["ab".into()]).unwrap_err();
    assert!(err.to_string().contains("glyph `b` has no opaque pixels"), "{err}");
}
