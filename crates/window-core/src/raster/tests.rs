use super::*;
use crate::ir::Rgb;

fn rgba_at(texture: &Texture, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * texture.width + x) * 4) as usize;
    [texture.rgba[i], texture.rgba[i + 1], texture.rgba[i + 2], texture.rgba[i + 3]]
}

#[test]
fn panel_renders_border_and_fill() {
    let style = GeneratedStyle::defaults(GeneratedKind::Panel);
    let texture = render(&style, Size::new(20, 12)).unwrap();
    assert_eq!(texture.width, 20);
    assert_eq!(texture.height, 12);
    let border = style.border_color;
    assert_eq!(rgba_at(&texture, 10, 0), [border.r, border.g, border.b, 255]);

    let fill = style.fill;
    assert_eq!(rgba_at(&texture, 10, 4), [fill.r, fill.g, fill.b, 255]);
    assert_eq!(rgba_at(&texture, 10, 7), [fill.r, fill.g, fill.b, 255]);
}

#[test]
fn inset_bevel_respects_rounded_corners() {
    let mut style = GeneratedStyle::defaults(GeneratedKind::Panel);
    style.fill = Rgb::new(10, 10, 10);
    style.border_color = Rgb::new(20, 20, 20);
    style.border_width = 1;
    style.radius = 6;
    style.inset_depth = 3;
    style.highlight_color = Some(Rgb::new(240, 240, 240));
    style.shadow_color = Some(Rgb::new(30, 30, 30));

    let mut pixmap = Pixmap::new(20, 20).unwrap();
    draw_panel(&mut pixmap, &style);
    let texture = texture_from_pixmap(&pixmap);

    assert_eq!(rgba_at(&texture, 2, 1), [20, 20, 20, 255]);
    assert_eq!(rgba_at(&texture, 17, 18), [20, 20, 20, 255]);
    assert_eq!(rgba_at(&texture, 8, 1), [240, 240, 240, 255]);
    assert_eq!(rgba_at(&texture, 1, 8), [240, 240, 240, 255]);
    assert_eq!(rgba_at(&texture, 18, 8), [30, 30, 30, 255]);
    assert_eq!(rgba_at(&texture, 8, 18), [30, 30, 30, 255]);
}

#[test]
fn hazard_bar_contains_stripes() {
    let style = GeneratedStyle::defaults(GeneratedKind::HazardBar);
    let texture = render(&style, Size::new(64, 12)).unwrap();
    let stripe_pixels = texture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[0] > 150 && (50..140).contains(&px[1]) && px[2] < 40 && px[3] == 255)
        .count();
    assert!(stripe_pixels > 0, "hazard stripes should be visible");
}

#[test]
fn indicator_bar_sits_inside_the_bottom_bevel() {
    let lamp = Rgb::new(0xff, 0x83, 0x00);
    let style = GeneratedStyle {
        border_width: 1,
        inset_depth: 1,
        radius: 0,
        indicator_color: Some(lamp),
        ..GeneratedStyle::defaults(GeneratedKind::Button)
    };
    let texture = render(&style, Size::new(52, 16)).unwrap();
    let lamp = [lamp.r, lamp.g, lamp.b, 255];
    let fill = [style.fill.r, style.fill.g, style.fill.b, 255];
    assert_eq!(rgba_at(&texture, 3, 12), lamp);
    assert_eq!(rgba_at(&texture, 48, 13), lamp);
    assert_eq!(rgba_at(&texture, 2, 12), fill);
    assert_eq!(rgba_at(&texture, 10, 11), fill);
    assert_ne!(rgba_at(&texture, 10, 14), lamp);
}

#[test]
fn indicator_bar_stays_clear_of_rounded_corners() {
    let style = GeneratedStyle {
        border_width: 1,
        inset_depth: 1,
        radius: 16,
        indicator_color: Some(Rgb::new(0xff, 0x83, 0x00)),
        ..GeneratedStyle::defaults(GeneratedKind::Button)
    };
    let texture = render(&style, Size::new(52, 32)).unwrap();
    assert_eq!(rgba_at(&texture, 3, 29)[3], 0);
    assert_eq!(rgba_at(&texture, 26, 29)[..3], [0xff, 0x83, 0x00]);

    // The frame clamps its radius to half the shorter side, and the bar follows the clamped corners.
    let texture = render(&style, Size::new(24, 12)).unwrap();
    assert_eq!(rgba_at(&texture, 12, 9)[..3], [0xff, 0x83, 0x00]);
}
