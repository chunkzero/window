//! Preview text: the runtime's measuring, wrapping and ellipsizing, and glyph drawing.
//!
//! Characters a text font's sheet maps draw from that sheet. Every other character belongs to Minecraft's default
//! font, whose bitmaps Window does not ship: it draws as the bundled small-caps glyph centered in, and clipped to,
//! the default glyph's width, or as a box when small caps has no glyph for it.
//!
//! Legacy `§` formatting codes are consumed as the runtime's `forEachRenderedCodePoint` does. Bold and the sixteen
//! color codes take effect; italic, underline, strikethrough and obfuscation are consumed without being drawn.

use crate::Result;
use crate::compose::Texture;
use crate::geometry::{Point, Rect};
use crate::ir::{Align, Rgb, SlotIr};
use crate::text_font::{CELL_HEIGHT, TextFont, TextFonts};
use crate::{Error, vanilla};

use super::Canvas;

const ELLIPSIS: char = '…';
const ASCII_ELLIPSIS: &str = "...";
/// The opacity of the shadow the runtime gives slot text.
const SHADOW_ALPHA: u8 = 180;
const SECTION_SIGN: char = '§';
/// The colors of the codes `0` through `f`.
const LEGACY_COLORS: [Rgb; 16] = [
    Rgb::new(0x00, 0x00, 0x00),
    Rgb::new(0x00, 0x00, 0xaa),
    Rgb::new(0x00, 0xaa, 0x00),
    Rgb::new(0x00, 0xaa, 0xaa),
    Rgb::new(0xaa, 0x00, 0x00),
    Rgb::new(0xaa, 0x00, 0xaa),
    Rgb::new(0xff, 0xaa, 0x00),
    Rgb::new(0xaa, 0xaa, 0xaa),
    Rgb::new(0x55, 0x55, 0x55),
    Rgb::new(0x55, 0x55, 0xff),
    Rgb::new(0x55, 0xff, 0x55),
    Rgb::new(0x55, 0xff, 0xff),
    Rgb::new(0xff, 0x55, 0x55),
    Rgb::new(0xff, 0x55, 0xff),
    Rgb::new(0xff, 0xff, 0x55),
    Rgb::new(0xff, 0xff, 0xff),
];

/// A text shadow, drawn one pixel down and right of its text.
#[derive(Clone, Copy)]
pub(super) enum Shadow {
    None,
    /// The client's default: the text color at a quarter brightness.
    Vanilla,
    /// Window text: translucent black.
    Slot,
    /// Text of a shader-placed HUD: the translucent quarter-brightness text color.
    ShaderHud,
}

impl Shadow {
    fn rgba(self, color: Rgb) -> Option<[u8; 4]> {
        let dark = |alpha| [color.r / 4, color.g / 4, color.b / 4, alpha];
        match self {
            Self::None => None,
            Self::Vanilla => Some(dark(255)),
            Self::Slot => Some([0, 0, 0, SHADOW_ALPHA]),
            Self::ShaderHud => Some(dark(SHADOW_ALPHA)),
        }
    }
}

/// Cursor advance and visible ink width of measured text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Width {
    advance: i32,
    visual: i32,
}

impl Width {
    /// The width of this text followed by `next`, whose ink starts at this advance.
    fn append(self, next: Width) -> Width {
        let visual = if next.visual == 0 { self.visual } else { self.visual.max(self.advance + next.visual) };
        Width { advance: self.advance + next.advance, visual }
    }
}

/// The formatting a legacy code sequence leaves in effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Style {
    bold: bool,
    /// The code's color; the slot's color when absent.
    color: Option<Rgb>,
}

impl Style {
    /// The style after the legacy code `code` over `self`, where `base` is the style `§r` restores.
    fn after(self, code: char, base: Style) -> Style {
        let code = code.to_ascii_lowercase();
        match code.to_digit(16) {
            Some(index) => Style { bold: false, color: Some(LEGACY_COLORS[index as usize]) },
            None if code == 'l' => Style { bold: true, ..self },
            None if code == 'r' => base,
            None => self,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Glyph {
    c: char,
    width: Width,
    style: Style,
}

fn measure(glyphs: &[Glyph]) -> Width {
    glyphs.iter().fold(Width::default(), |width, glyph| width.append(glyph.width))
}

/// Where text `visual` pixels wide starts in a box at `x` that is `width` wide.
pub(super) fn origin(align: Align, x: i32, width: u32, visual: u32) -> i64 {
    let (x, width, visual) = (i64::from(x), i64::from(width), i64::from(visual));
    match align {
        Align::Left => x,
        Align::Center => x + (width - visual) / 2,
        Align::Right => x + width - visual,
    }
}

pub(super) struct TextPainter<'a> {
    fonts: &'a TextFonts,
    /// The bundled small-caps font standing in for Minecraft's default glyphs.
    stand_in: TextFont,
}

impl<'a> TextPainter<'a> {
    pub(super) fn new(fonts: &'a TextFonts) -> Self {
        Self { fonts, stand_in: crate::text_font::small_caps() }
    }

    /// Draws `content` in `slot`, in `color`, on the lines and at the positions the runtime gives it. `shadow` is the
    /// shadow of a slot that has one.
    pub(super) fn draw_slot(
        &self,
        canvas: &mut Canvas,
        slot: &SlotIr,
        content: &str,
        color: Rgb,
        shadow: Shadow,
    ) -> Result<()> {
        let font = match &slot.font {
            Some(name) => Some(
                self.fonts
                    .get(name)
                    .ok_or_else(|| Error::Validation(format!("slot `{}` uses unknown font `{name}`", slot.name)))?,
            ),
            None => None,
        };
        let glyphs = self.glyphs(font, content, slot.bold);
        let fit = slot.fit;
        let lines = if fit.lines == 1 && !fit.ellipsis {
            vec![glyphs]
        } else {
            let has_ellipsis = font.is_some_and(|font| cell(font, ELLIPSIS).is_some());
            let ellipsis = if has_ellipsis { ELLIPSIS.to_string() } else { ASCII_ELLIPSIS.to_string() };
            let base = Style { bold: slot.bold, color: None };
            // The ellipsis continues the style of the text it follows.
            let ellipsis = |last: Option<&Glyph>| self.styled(font, &ellipsis, last.map_or(base, |g| g.style), base);
            wrap(&glyphs, slot.rect.width as i32, fit.lines as usize, &ellipsis)
        };
        let shadow = if slot.shadow { shadow } else { Shadow::None };
        let unused = fit.lines as i32 - lines.len() as i32;
        for (j, line) in lines.iter().enumerate() {
            let step = if fit.lines > 1 { unused + 2 * j as i32 } else { 0 };
            let visual = measure(line).visual.max(0) as u32;
            let at = Point::new(
                origin(slot.align, slot.rect.x, slot.rect.width, visual) as i32,
                slot.rect.y + step * fit.line_height as i32 / 2,
            );
            self.draw_glyphs(canvas, font, line, at, color, shadow);
        }
        Ok(())
    }

    /// Draws `content` on one unfitted line starting at `at`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_line(
        &self,
        canvas: &mut Canvas,
        font: Option<&TextFont>,
        content: &str,
        at: Point,
        color: Rgb,
        shadow: Shadow,
        bold: bool,
    ) {
        self.draw_glyphs(canvas, font, &self.glyphs(font, content, bold), at, color, shadow);
    }

    fn glyphs(&self, font: Option<&TextFont>, content: &str, bold: bool) -> Vec<Glyph> {
        let base = Style { bold, color: None };
        self.styled(font, content, base, base)
    }

    /// The glyphs of `content` starting in style `start`, with legacy codes consumed. A trailing `§` is dropped.
    fn styled(&self, font: Option<&TextFont>, content: &str, start: Style, base: Style) -> Vec<Glyph> {
        let mut style = start;
        let mut glyphs = Vec::new();
        let mut chars = content.chars();
        while let Some(c) = chars.next() {
            if c == SECTION_SIGN {
                let Some(code) = chars.next() else { break };
                style = style.after(code, base);
                continue;
            }
            let bold = i32::from(style.bold) * vanilla::BOLD_ADVANCE as i32;
            let (advance, ink) = match font {
                Some(font) => (font.advance(c), font.glyph_width(c)),
                None => (vanilla::advance(c), vanilla::glyph_width(c)),
            };
            let advance = advance.unwrap_or(vanilla::FALLBACK_ADVANCE) as i32 + bold;
            let ink = ink.unwrap_or(vanilla::FALLBACK_GLYPH_WIDTH) as i32;
            let width = Width { advance, visual: if ink > 0 { ink + bold } else { 0 } };
            glyphs.push(Glyph { c, width, style });
        }
        glyphs
    }

    fn draw_glyphs(
        &self,
        canvas: &mut Canvas,
        font: Option<&TextFont>,
        glyphs: &[Glyph],
        at: Point,
        color: Rgb,
        shadow: Shadow,
    ) {
        for offset in [1, 0] {
            let mut x = at.x + offset;
            for glyph in glyphs {
                let color = glyph.style.color.unwrap_or(color);
                let rgba = if offset == 1 { shadow.rgba(color) } else { Some([color.r, color.g, color.b, 255]) };
                if let Some(rgba) = rgba {
                    for copy in 0..=i32::from(glyph.style.bold) {
                        self.stamp(canvas, font, glyph, Point::new(x + copy, at.y + offset), rgba);
                    }
                }
                x += glyph.width.advance;
            }
        }
    }

    fn stamp(&self, canvas: &mut Canvas, font: Option<&TextFont>, glyph: &Glyph, at: Point, color: [u8; 4]) {
        if let Some((font, source)) = font.and_then(|font| Some((font, cell(font, glyph.c)?))) {
            stamp_cell(canvas, &font.texture, &source, at, at.x..at.x + source.width as i32, color);
            return;
        }
        let ink = vanilla::glyph_width(glyph.c).unwrap_or(vanilla::FALLBACK_GLYPH_WIDTH) as i32;
        if ink == 0 {
            return;
        }
        let clip = at.x..at.x + ink;
        match cell(&self.stand_in, glyph.c) {
            Some(source) => {
                let own = self.stand_in.glyph_width(glyph.c).unwrap_or_default() as i32;
                let at = Point::new(at.x + (ink - own).div_euclid(2), at.y);
                stamp_cell(canvas, &self.stand_in.texture, &source, at, clip, color);
            }
            None => canvas.outline(&Rect::new(at.x, at.y, ink as u32, CELL_HEIGHT - 1), color),
        }
    }
}

/// The sheet cell of `c` in `font`, if its sheet maps `c`.
fn cell(font: &TextFont, c: char) -> Option<Rect> {
    let columns = font.chars.first()?.chars().count() as u32;
    let width = font.texture.width / columns;
    (0..).zip(&font.chars).find_map(|(row, line)| {
        let column = line.chars().position(|mapped| mapped == c && c != '\0')? as u32;
        Some(Rect::new((column * width) as i32, (row * CELL_HEIGHT) as i32, width, CELL_HEIGHT))
    })
}

/// Draws the `source` cell of `sheet` at `at`, multiplied by `tint`, keeping only the columns in `clip`.
fn stamp_cell(
    canvas: &mut Canvas,
    sheet: &Texture,
    source: &Rect,
    at: Point,
    clip: std::ops::Range<i32>,
    tint: [u8; 4],
) {
    for dy in 0..source.height {
        for dx in (0..source.width).filter(|dx| clip.contains(&(at.x + *dx as i32))) {
            let mut pixel = sheet.get(source.x as u32 + dx, source.y as u32 + dy);
            for (channel, tint) in pixel.iter_mut().zip(tint) {
                *channel = (u16::from(*channel) * u16::from(tint) / 255) as u8;
            }
            canvas.blend(at.x + dx as i32, at.y + dy as i32, pixel);
        }
    }
}

fn trim_end(glyphs: &[Glyph], from: usize, to: usize) -> usize {
    let mut end = to;
    while end > from && glyphs[end - 1].c == ' ' {
        end -= 1;
    }
    end
}

/// Wraps `glyphs` at spaces onto at most `max_lines` lines whose ink fits `width`, as the runtime's `wrapText` does:
/// a word wider than `width` ends in `ellipsis`, as does the last line when text is left over. Text that fits on one
/// line is one line; blank text has no lines.
fn wrap(
    glyphs: &[Glyph],
    width: i32,
    max_lines: usize,
    ellipsis: &dyn Fn(Option<&Glyph>) -> Vec<Glyph>,
) -> Vec<Vec<Glyph>> {
    if glyphs.iter().all(|glyph| glyph.c == ' ') {
        return Vec::new();
    }
    if measure(glyphs).visual <= width {
        return vec![glyphs.to_vec()];
    }
    let mut lines = Vec::new();
    let mut i = 0;
    while lines.len() < max_lines {
        while i < glyphs.len() && glyphs[i].c == ' ' {
            i += 1;
        }
        if i == glyphs.len() {
            break;
        }
        let rest = trim_end(glyphs, i, glyphs.len());
        if measure(&glyphs[i..rest]).visual <= width {
            lines.push(glyphs[i..rest].to_vec());
            break;
        }
        if lines.len() == max_lines - 1 {
            lines.push(ellipsized(glyphs, i, glyphs.len(), width, ellipsis));
            break;
        }
        match last_break(glyphs, i, width) {
            Some(break_at) => {
                lines.push(glyphs[i..trim_end(glyphs, i, break_at)].to_vec());
                i = break_at;
            }
            None => {
                let word_end = (i..glyphs.len()).find(|&j| glyphs[j].c == ' ').unwrap_or(glyphs.len());
                lines.push(ellipsized(glyphs, i, word_end, width, ellipsis));
                i = word_end;
            }
        }
    }
    lines
}

/// The last space after `from` whose preceding text fits `width`, or `None` when the first word is too wide.
fn last_break(glyphs: &[Glyph], from: usize, width: i32) -> Option<usize> {
    let mut found = None;
    let mut line = Width::default();
    for i in from..glyphs.len() {
        if i > from && glyphs[i].c == ' ' && glyphs[i - 1].c != ' ' {
            if line.visual > width {
                break;
            }
            found = Some(i);
        }
        line = line.append(glyphs[i].width);
    }
    found
}

/// The longest prefix of `[from, to)` that fits `width` when followed by `ellipsis`; empty when not even the
/// ellipsis fits.
fn ellipsized(
    glyphs: &[Glyph],
    from: usize,
    to: usize,
    width: i32,
    ellipsis: &dyn Fn(Option<&Glyph>) -> Vec<Glyph>,
) -> Vec<Glyph> {
    let mut prefixes = vec![Width::default()];
    for glyph in &glyphs[from..to] {
        prefixes.push(prefixes[prefixes.len() - 1].append(glyph.width));
    }
    let mut end = to;
    loop {
        let kept = trim_end(glyphs, from, end);
        let tail = ellipsis(glyphs[from..kept].last());
        if prefixes[kept - from].append(measure(&tail)).visual <= width {
            return glyphs[from..kept].iter().chain(&tail).copied().collect();
        }
        if kept <= from {
            return Vec::new();
        }
        end = kept - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_at_spaces_and_ellipsizes_the_last_line() {
        let fonts = TextFonts::new();
        let painter = TextPainter::new(&fonts);
        let glyphs = |text: &str| painter.glyphs(None, text, false);
        let lines = |text: &str, width, max_lines| -> Vec<String> {
            let lines = wrap(&glyphs(text), width, max_lines, &|_| glyphs(ASCII_ELLIPSIS));
            lines.iter().map(|line| line.iter().map(|glyph| glyph.c).collect()).collect()
        };

        assert_eq!(lines("alpha beta gamma", 30, 2), ["alpha", "beta..."]);
        assert_eq!(lines("alpha beta gamma", 30, 3), ["alpha", "beta", "gamma"]);
        assert_eq!(lines("alphabetical beta", 30, 2), ["alph...", "beta"]);
        assert_eq!(lines("alpha", 30, 2), ["alpha"]);
        assert!(lines("   ", 30, 2).is_empty());
    }
}
