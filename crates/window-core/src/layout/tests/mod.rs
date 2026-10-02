mod controls;
mod draws;
mod flow;
mod patterns;
mod text;

use std::collections::HashMap;

use serde_json::{Value, json};

use super::solve;
use crate::authoring::{ParsedProject, project_from_json};
use crate::geometry::{Rect, Size};
use crate::ir::{Align, Draw, LaidOutWindow, Rgb};
use crate::vanilla;

/// Build a texture-size closure from a name→(w,h) map.
fn sizes(entries: &[(&str, u32, u32)]) -> impl Fn(&str) -> Option<Size> + 'static {
    let map: HashMap<String, Size> = entries.iter().map(|(p, w, h)| (p.to_string(), Size::new(*w, *h))).collect();
    move |p: &str| map.get(p).copied()
}

fn project(value: Value) -> ParsedProject {
    project_from_json(value.to_string().as_bytes()).expect("parse project")
}

fn window(children: Value) -> ParsedProject {
    project(json!({
        "windows": [{
            "name": "s",
            "container": "generic_9x3",
            "children": children,
        }],
    }))
}

fn themed(theme: Value, children: Value) -> ParsedProject {
    project(json!({
        "theme": theme,
        "windows": [{
            "name": "s",
            "container": "generic_9x6",
            "children": children,
        }],
    }))
}

fn solve_one(project: ParsedProject, tx: &dyn Fn(&str) -> Option<Size>) -> LaidOutWindow {
    solve(&project, tx).expect("solve").pop().expect("window")
}
