//! Output size accounting: which part of the compiler's output each emitted file belongs to, and the report an
//! author reads to decide what to cut.
//!
//! Sizes are the bytes of the compiler's pack files plus generated Kotlin, before the pack builder's later
//! optimization. Files are attributed whole; shared documents are reported as their own lines.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::pipeline::{CompileOutput, OutputFile};

/// The window or HUD a file draws.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pane<'a> {
    pub(crate) hud: bool,
    pub(crate) name: &'a str,
}

/// The part of the output one emitted file belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    /// A pane's static art texture.
    Static {
        hud: bool,
        pane: String,
    },
    /// One baked switch case of a pane.
    SwitchCase {
        hud: bool,
        pane: String,
    },
    /// A sprite font document, labelled by its font name such as `sprite_y12`.
    SpriteFont(String),
    /// A text-row font document (a shifted label font or a text font's offset document), labelled by its name.
    TextRowFont(String),
    SpriteTexture,
    TextSheet,
    MainFont,
    Shader,
    Vanilla,
    Hitbox,
    Debug,
    Kotlin,
}

impl Part {
    pub(crate) fn static_art(pane: Pane<'_>) -> Self {
        Self::Static { hud: pane.hud, pane: pane.name.into() }
    }

    pub(crate) fn switch_case(pane: Pane<'_>) -> Self {
        Self::SwitchCase { hud: pane.hud, pane: pane.name.into() }
    }
}

/// The byte size of every emitted file, tagged with its part.
#[derive(Clone, Debug, Default)]
pub struct SizeLedger {
    entries: Vec<(Part, u64)>,
}

impl SizeLedger {
    pub(crate) fn record(&mut self, part: Part, file: &OutputFile) {
        self.entries.push((part, file.contents.as_bytes().len() as u64));
    }
}

/// The size report of one build.
#[derive(Clone, Debug)]
pub struct SizeReport {
    /// Total bytes of the pack files and generated Kotlin.
    pub total: u64,
    budget: Option<u64>,
    text: String,
}

impl SizeReport {
    /// Summarizes `core`'s files and `kotlin` against `budget`.
    pub fn new(core: &CompileOutput, kotlin: &[OutputFile], budget: Option<u64>) -> Self {
        let mut ledger = core.sizes.clone();
        for file in kotlin {
            ledger.record(Part::Kotlin, file);
        }
        let total = core.files.iter().chain(kotlin).map(|file| file.contents.as_bytes().len() as u64).sum();
        let text = render(&ledger, total, budget);
        Self { total, budget, text }
    }

    /// The report: a header with the total, then one line per pane and shared part, largest first.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The error for an output above the budget, or `None` when it fits or there is no budget.
    pub fn budget_error(&self) -> Option<String> {
        let budget = self.budget.filter(|&budget| self.total > budget)?;
        Some(format!(
            "Window output is {} bytes, {} bytes over the size.budget of {budget} bytes\n{}",
            self.total,
            self.total - budget,
            self.text
        ))
    }
}

#[derive(Default)]
struct PaneSizes {
    static_bytes: u64,
    cases: u64,
    case_bytes: u64,
}

#[derive(Default)]
struct Group {
    count: u64,
    bytes: u64,
    largest: Option<(String, u64)>,
}

impl Group {
    fn add(&mut self, label: Option<&str>, bytes: u64) {
        self.count += 1;
        self.bytes += bytes;
        if let Some(label) = label
            && self.largest.as_ref().is_none_or(|(_, largest)| bytes > *largest)
        {
            self.largest = Some((label.into(), bytes));
        }
    }
}

/// One report line: its label, bytes, and trailing detail.
struct Row {
    label: String,
    bytes: u64,
    detail: String,
}

fn render(ledger: &SizeLedger, total: u64, budget: Option<u64>) -> String {
    let mut panes: BTreeMap<(bool, &str), PaneSizes> = BTreeMap::new();
    let mut sprite_fonts = Group::default();
    let mut text_row_fonts = Group::default();
    let mut shared: BTreeMap<&str, Group> = BTreeMap::new();
    let mut attributed = 0;
    for (part, bytes) in &ledger.entries {
        let bytes = *bytes;
        attributed += bytes;
        let category = match part {
            Part::Static { hud, pane } => {
                panes.entry((*hud, pane)).or_default().static_bytes += bytes;
                continue;
            }
            Part::SwitchCase { hud, pane } => {
                let sizes = panes.entry((*hud, pane)).or_default();
                sizes.cases += 1;
                sizes.case_bytes += bytes;
                continue;
            }
            Part::SpriteFont(label) => {
                sprite_fonts.add(Some(label), bytes);
                continue;
            }
            Part::TextRowFont(label) => {
                text_row_fonts.add(Some(label), bytes);
                continue;
            }
            Part::SpriteTexture => "sprite textures",
            Part::TextSheet => "text font sheets",
            Part::MainFont => "main font",
            Part::Shader => "shaders",
            Part::Vanilla => "vanilla overrides",
            Part::Hitbox => "hitbox item",
            Part::Debug => "debug descriptor",
            Part::Kotlin => "kotlin",
        };
        shared.entry(category).or_default().add(None, bytes);
    }

    let mut pane_rows: Vec<_> = panes.into_iter().collect();
    pane_rows.sort_by_key(|(_, sizes)| std::cmp::Reverse(sizes.static_bytes + sizes.case_bytes));
    let mut rows: Vec<Row> = pane_rows
        .into_iter()
        .map(|((hud, name), sizes)| Row {
            label: format!("{} {name}", if hud { "hud" } else { "win" }),
            bytes: sizes.static_bytes + sizes.case_bytes,
            detail: format!(
                "static {} · {} switch case{} {}",
                size(sizes.static_bytes),
                sizes.cases,
                if sizes.cases == 1 { "" } else { "s" },
                size(sizes.case_bytes)
            ),
        })
        .collect();
    for (label, group) in [("sprite fonts", sprite_fonts), ("text-row fonts", text_row_fonts)] {
        if group.count > 0 {
            rows.push(Row {
                label: format!("{label} ({})", group.count),
                bytes: group.bytes,
                detail: group
                    .largest
                    .map_or_else(String::new, |(name, bytes)| format!("largest {name} {}", size(bytes))),
            });
        }
    }
    let mut shared: Vec<_> = shared.into_iter().collect();
    shared.sort_by_key(|(_, group)| std::cmp::Reverse(group.bytes));
    for (label, group) in shared {
        let label = if group.count > 1 { format!("{label} ({})", group.count) } else { label.into() };
        rows.push(Row { label, bytes: group.bytes, detail: String::new() });
    }
    if total > attributed {
        rows.push(Row { label: "unattributed".into(), bytes: total - attributed, detail: String::new() });
    }

    let mut out = match budget {
        Some(budget) => format!("Window output: {} of {} budget", size(total), size(budget)),
        None => format!("Window output: {}", size(total)),
    };
    out.push_str(" (compiler output, before pack optimization)");
    let width = rows.iter().map(|row| row.label.chars().count()).max().unwrap_or(0);
    for Row { label, bytes, detail } in rows {
        let _ = write!(out, "\n  {label:<width$}  {:>9}", size(bytes));
        if !detail.is_empty() {
            let _ = write!(out, "   {detail}");
        }
    }
    out
}

fn size(bytes: u64) -> String {
    match bytes {
        0..1_000 => format!("{bytes} B"),
        1_000..1_000_000 => format!("{:.1} KB", bytes as f64 / 1_000.0),
        _ => format!("{:.2} MB", bytes as f64 / 1_000_000.0),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::pipeline::{CompileInput, compile_project_json};

    fn compile() -> CompileOutput {
        let project = r##"{
          "windows":[{"name":"shop","container":"generic_9x3","children":[
            {"type":"slot","handle":{"kind":"text","id":"title"},"width":30,"x":9,"y":5},
            {"type":"flex","x":8,"y":0,"style":{"width":8,"height":8},"frame":{"art":"shape","kind":"panel",
              "fill":"#123456","border_width":0,"radius":0,"inset_depth":0}},
            {"type":"switch","handle":{"kind":"flag","id":"lamp","shape":[1],"at":[0]},"x":8,"y":20,"children":[
              {"type":"case","value":"true","children":[{"type":"sprite","art":{"art":"shape","kind":"panel",
                "fill":"#123456","border_width":0,"radius":0,"inset_depth":0,"width":4,"height":4}}]},
              {"type":"case","value":"false","children":[]}
            ]}
          ]}]
        }"##;
        compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap()
    }

    fn bytes(output: &CompileOutput, suffix: &str) -> u64 {
        output.files.iter().find(|file| file.path.ends_with(suffix)).unwrap().contents.as_bytes().len() as u64
    }

    #[test]
    fn report_attributes_files_and_totals_the_output() {
        let output = compile();
        let kotlin = [OutputFile::text("ShopView.kt", "class ShopView".into())];
        let report = SizeReport::new(&output, &kotlin, None);

        let all: u64 = output.files.iter().map(|file| file.contents.as_bytes().len() as u64).sum();
        assert_eq!(report.total, all + 14);
        let text = report.text();
        let line = |label: &str| text.lines().find(|line| line.trim_start().starts_with(label)).unwrap_or(text);
        let case = bytes(&output, "shop_switch/lamp-0/true.png");
        assert!(line("win shop").contains(&format!("1 switch case {}", size(case))), "{text}");
        assert!(line("win shop").contains(&format!("static {}", size(bytes(&output, "textures/font/shop.png")))));
        assert!(line("text-row fonts (1)").contains("largest y"), "{text}");
        assert!(line("kotlin").contains("14 B"), "{text}");
        assert!(!text.contains("unattributed"), "{text}");
    }

    #[test]
    fn budget_error_reports_the_overage() {
        let output = compile();
        let total = SizeReport::new(&output, &[], None).total;
        assert!(SizeReport::new(&output, &[], Some(total)).budget_error().is_none());

        let error = SizeReport::new(&output, &[], Some(total - 10)).budget_error().unwrap();
        assert!(error.contains(&format!("{total} bytes, 10 bytes over the size.budget of {} bytes", total - 10)));
        assert!(error.contains("win shop"), "{error}");
    }
}
