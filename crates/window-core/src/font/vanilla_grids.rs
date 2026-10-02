//! Vanilla bitmap character grids referenced by the shifted fonts.

/// The canonical 16-row × 16-column character grid for vanilla `ascii.png`,
/// reproduced from the vanilla `minecraft:font/default.json` `ascii` provider.
///
/// Rows map to the glyph rows of the texture; positions without a glyph use the
/// null character (`U+0000`), as vanilla does for its unused cells.
pub(super) const VANILLA_ASCII_CHARS: [&str; 16] = [
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}",
    " !\"#$%&'()*+,-./",
    "0123456789:;<=>?",
    "@ABCDEFGHIJKLMNO",
    "PQRSTUVWXYZ[\\]^_",
    "`abcdefghijklmno",
    "pqrstuvwxyz{|}~\u{0000}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{00A3}\u{0000}\u{0000}\u{0192}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{00AA}\u{00BA}\u{0000}\u{0000}\u{00AC}\u{0000}\u{0000}\u{0000}\u{00AB}\u{00BB}",
    "\u{2591}\u{2592}\u{2593}\u{2502}\u{2524}\u{2561}\u{2562}\u{2556}\u{2555}\u{2563}\u{2551}\u{2557}\u{255D}\u{255C}\u{255B}\u{2510}",
    "\u{2514}\u{2534}\u{252C}\u{251C}\u{2500}\u{253C}\u{255E}\u{255F}\u{255A}\u{2554}\u{2569}\u{2566}\u{2560}\u{2550}\u{256C}\u{2567}",
    "\u{2568}\u{2564}\u{2565}\u{2559}\u{2558}\u{2552}\u{2553}\u{256B}\u{256A}\u{2518}\u{250C}\u{2588}\u{2584}\u{258C}\u{2590}\u{2580}",
    "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{2205}\u{2208}\u{0000}",
    "\u{2261}\u{00B1}\u{2265}\u{2264}\u{2320}\u{2321}\u{00F7}\u{2248}\u{00B0}\u{2219}\u{0000}\u{221A}\u{207F}\u{00B2}\u{25A0}\u{0000}",
];

const EMPTY_BITMAP_ROW: &str = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";

/// Sparse copy of Minecraft's `nonlatin_european.png` bitmap grid containing
/// the mcutils-style small caps and Window's standard UI markers. Keeping the
/// provider grid and [`crate::vanilla`] metrics in lockstep is required for
/// net-zero title segments.
pub(super) fn vanilla_window_text_nonlatin_rows() -> Vec<&'static str> {
    let mut rows = vec![EMPTY_BITMAP_ROW; 67];
    rows[0] = "\u{0000}\u{0000}\u{0000}\u{00B7}\u{0000}\u{0000}\u{0000}\u{00D7}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows[8] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0455}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows[9] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{026A}\u{0000}\u{0000}\u{0000}";
    rows[29] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{25B2}\u{0000}\u{25BC}\u{0000}\u{25CF}\u{0000}\u{0000}";
    rows[37] = "\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{1D00}\u{0299}\u{1D04}\u{1D05}\u{1D07}\u{A730}\u{0262}\u{029C}\u{1D0A}";
    rows[38] = "\u{1D0B}\u{029F}\u{1D0D}\u{0274}\u{1D0F}\u{1D18}\u{0000}\u{0280}\u{A731}\u{1D1B}\u{1D1C}\u{1D20}\u{1D21}\u{028F}\u{1D22}\u{0000}";
    rows[56] = "\u{0000}\u{0000}\u{25C6}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows
}

/// Sparse copy of Minecraft's `accented.png` bitmap grid containing the
/// mcutils-style small-cap Q (`U+01EB`).
pub(super) fn vanilla_small_text_accented_rows() -> Vec<&'static str> {
    let mut rows = vec![EMPTY_BITMAP_ROW; 75];
    rows[16] = "\u{0000}\u{0000}\u{01EB}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}\u{0000}";
    rows
}
