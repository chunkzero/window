//! Full-screen tooltips: a vanilla-styled panel of plain text, laid out beside its region.

use super::target::LayoutTarget;
use crate::Result;
use crate::geometry::{Point, Rect, Size};
use crate::ir::{Align, RegionIr, Rgb, SlotIr, Tooltip, TooltipIr};
use crate::model::TextFit;
use crate::vanilla;

/// The title's color, as in a container tooltip.
const TITLE_COLOR: Rgb = Rgb::new(0xff, 0xff, 0xff);
/// The other lines' color, as in a container tooltip.
const LINE_COLOR: Rgb = Rgb::new(0xaa, 0xaa, 0xaa);
/// The distance from the panel's edge to its text: a 1px rim, a 1px border, and 2px of padding.
const INSET: u32 = 4;
/// Vanilla's tooltip line pitch, and the gap it leaves below the title.
const LINE_PITCH: u32 = 10;
const TITLE_GAP: u32 = 2;
/// The gap between the region and the panel.
const GAP: i32 = 2;

/// The tooltip of `regions[index]`, centered below its region, or above it where the canvas ends first, and kept on
/// the canvas.
pub(super) fn tooltip(
    target: &impl LayoutTarget,
    index: usize,
    region: &RegionIr,
    tooltip: &Tooltip,
) -> Result<TooltipIr> {
    let authored = std::iter::once(tooltip.title.as_str()).chain(tooltip.lines.iter().map(String::as_str));
    if authored.clone().any(|text| text.contains('<')) {
        return Err(target.layout_err(format!(
            "{} has a tooltip with `<`: full-screen tooltips draw plain text, without MiniMessage tags",
            region.source
        )));
    }
    if authored.clone().any(|text| text.contains('§')) {
        return Err(target.layout_err(format!(
            "{} has a tooltip with `§`: full-screen tooltips draw plain text, without formatting codes",
            region.source
        )));
    }
    // An embedded newline starts another tooltip line, as in vanilla; the display itself never sees one.
    let texts: Vec<&str> = authored.flat_map(|text| text.split('\n')).collect();
    let measure = |text: &str| vanilla::visible_width(text, false, vanilla::advance, vanilla::glyph_width);
    let text_width = texts.iter().map(|text| measure(text)).max().unwrap_or(0);
    let extra = texts.len() as u32 - 1;
    let text_height = 8 + if extra > 0 { TITLE_GAP + extra * LINE_PITCH } else { 0 };
    let size = Size::new(text_width + 2 * INSET, text_height + 2 * INSET);
    let canvas = target.bounds();
    if size.width > canvas.width || size.height > canvas.height {
        return Err(target.layout_err(format!(
            "{} has a {}x{} tooltip, which does not fit the {}x{} canvas",
            region.source, size.width, size.height, canvas.width, canvas.height
        )));
    }
    let rect = region.rect;
    let (width, height) = (size.width as i32, size.height as i32);
    let below = rect.y + rect.height as i32 + GAP;
    let above = rect.y - GAP - height;
    let y = if below + height <= canvas.height as i32 || above < 0 { below } else { above };
    let x = rect.x + (rect.width as i32 - width) / 2;
    let panel = Rect::from_parts(
        Point::new(x.clamp(0, canvas.width as i32 - width), y.clamp(0, canvas.height as i32 - height)),
        size,
    );
    let lines = texts
        .iter()
        .enumerate()
        .filter(|(_, text)| !text.is_empty())
        .map(|(line, text)| {
            let top = if line == 0 { 0 } else { TITLE_GAP + line as u32 * LINE_PITCH };
            let origin = Point::new(panel.x + INSET as i32, panel.y + (INSET + top) as i32);
            SlotIr {
                name: format!("{}/tooltip/{line}", region.name),
                text: Some(text.to_string()),
                rect: Rect::from_parts(origin, Size::new(measure(text), 8)),
                align: Align::Left,
                color: if line == 0 { TITLE_COLOR } else { LINE_COLOR },
                shadow: false,
                bold: false,
                italic: false,
                underlined: false,
                strikethrough: false,
                obfuscated: false,
                font: None,
                fit: TextFit::default(),
                tint: None,
                source: Some(region.source.clone()),
            }
        })
        .collect();
    Ok(TooltipIr { region: index, panel, lines })
}
