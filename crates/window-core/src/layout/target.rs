use crate::error::Error;
use crate::geometry::{Point, Rect, Size};
use crate::surface::ContainerKind;

/// The window or HUD being laid out: its bounds, capabilities, and error wording.
pub(super) trait LayoutTarget {
    fn name(&self) -> &str;
    fn kind(&self) -> &'static str;
    fn bounds(&self) -> Rect;
    fn visual_bounds(&self) -> Rect;
    fn min_text_y(&self) -> i32;
    fn allows_interaction(&self) -> bool;
    fn allows_sprite_slots(&self) -> bool;
    fn container_kind(&self) -> Option<ContainerKind>;
    fn duplicate_namespace(&self) -> &'static str;

    fn layout_err(&self, message: impl Into<String>) -> Error {
        Error::Layout { window: self.name().to_string(), message: message.into() }
    }

    fn missing_texture_err(&self, texture: &str, referenced_by: &str) -> Error {
        Error::Validation(format!(
            "missing texture `{texture}` referenced by {referenced_by} in {} `{}`",
            self.kind(),
            self.name()
        ))
    }

    fn frame_too_small_err(&self, frame: &str, texture: &str, size: Size, min_w: u32, min_h: u32) -> Error {
        Error::Validation(format!(
            "frame `{frame}` texture `{texture}` is {}x{}, too small for its insets \
             (need at least {min_w}x{min_h}) in {} `{}`",
            size.width,
            size.height,
            self.kind(),
            self.name()
        ))
    }

    fn overflow_warning(&self, rect: Rect, what: &str) -> String {
        let bounds = self.visual_bounds();
        format!(
            "{} `{}`: {what} draw {rect:?} extends outside the {}x{} visual {} rect",
            self.kind(),
            self.name(),
            bounds.width,
            bounds.height,
            self.bounds_label()
        )
    }

    fn outside_err(&self, rect: Rect, what: &str) -> Error {
        let bounds = self.bounds();
        self.layout_err(format!(
            "{what} rect {rect:?} is outside the {}x{} {} rect",
            bounds.width,
            bounds.height,
            self.bounds_label()
        ))
    }

    fn text_limit_err(&self, kind: &str, name: &str, y: i32) -> Error {
        self.layout_err(format!(
            "{kind} `{name}` y={y} is above the minimum {} text y of {} \
             (shifted-font ascent limit: y must be >= {})",
            self.kind(),
            self.min_text_y(),
            self.min_text_y()
        ))
    }

    fn interaction_err(&self, name: &str) -> Error {
        self.layout_err(format!("{} `{}` cannot contain interactive element `{name}`", self.kind(), self.name()))
    }

    fn sprite_slot_err(&self, name: &str) -> Error {
        self.layout_err(format!("{} `{}` cannot contain runtime sprite slot `{name}`", self.kind(), self.name()))
    }

    fn slot_pattern_err(&self, name: &str) -> Error {
        self.layout_err(format!("{} `{}` cannot contain slot pattern element `{name}`", self.kind(), self.name()))
    }

    fn bounds_label(&self) -> &'static str {
        match self.kind() {
            "window" => "GUI",
            "hud" => "HUD",
            _ => "layout",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct WindowTarget<'a> {
    pub(super) name: &'a str,
    pub(super) container: ContainerKind,
    pub(super) bounds: Rect,
    pub(super) visual_bounds: Rect,
    pub(super) title_origin: Point,
}

impl LayoutTarget for WindowTarget<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> &'static str {
        "window"
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn visual_bounds(&self) -> Rect {
        self.visual_bounds
    }

    fn min_text_y(&self) -> i32 {
        self.title_origin.y - 1
    }

    fn allows_interaction(&self) -> bool {
        true
    }

    fn allows_sprite_slots(&self) -> bool {
        true
    }

    fn container_kind(&self) -> Option<ContainerKind> {
        Some(self.container)
    }

    fn duplicate_namespace(&self) -> &'static str {
        "slot/sprite/button/label names share one namespace"
    }
}

#[derive(Clone, Copy)]
pub(super) struct HudTarget<'a> {
    pub(super) name: &'a str,
    pub(super) bounds: Rect,
    pub(super) visual_bounds: Rect,
}

impl LayoutTarget for HudTarget<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> &'static str {
        "hud"
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn visual_bounds(&self) -> Rect {
        self.visual_bounds
    }

    fn min_text_y(&self) -> i32 {
        -1
    }

    fn allows_interaction(&self) -> bool {
        false
    }

    fn allows_sprite_slots(&self) -> bool {
        false
    }

    fn container_kind(&self) -> Option<ContainerKind> {
        None
    }

    fn duplicate_namespace(&self) -> &'static str {
        "slot/label names share one namespace"
    }
}
