use crate::ir::Rgb;

/// Procedural styles that Window can rasterize into UI sprites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedStyle {
    /// Shape family.
    pub kind: GeneratedKind,
    /// Main fill color.
    pub fill: Rgb,
    /// Border color.
    pub border_color: Rgb,
    /// Border width in pixels.
    pub border_width: u32,
    /// Corner radius in pixels.
    pub radius: u32,
    /// Inner bevel/depth in pixels.
    pub inset_depth: u32,
    /// Optional top/left highlight color.
    pub highlight_color: Option<Rgb>,
    /// Optional bottom/right shadow color.
    pub shadow_color: Option<Rgb>,
    /// Optional accent color for vents/badges.
    pub accent_color: Option<Rgb>,
    /// Optional 2px indicator bar along the inner bottom edge of panels, buttons, and slots.
    pub indicator_color: Option<Rgb>,
    /// Optional diagonal stripe color.
    pub stripe_color: Option<Rgb>,
    /// Optional diagonal stripe shadow color.
    pub stripe_shadow_color: Option<Rgb>,
    /// Diagonal stripe width in pixels.
    pub stripe_width: u32,
}

impl GeneratedStyle {
    /// Defaults for a named generated shape.
    pub fn defaults(kind: GeneratedKind) -> Self {
        match kind {
            GeneratedKind::Panel => Self::panel(),
            GeneratedKind::Button => Self::button(),
            GeneratedKind::Slot => Self::slot(),
            GeneratedKind::HazardBar => Self::hazard_bar(),
            GeneratedKind::Vent => Self::vent(),
            GeneratedKind::Badge => Self::badge(),
        }
    }

    fn panel() -> Self {
        Self {
            kind: GeneratedKind::Panel,
            fill: Rgb::new(0x0a, 0x68, 0x84),
            border_color: Rgb::new(0x0d, 0x5a, 0x70),
            border_width: 1,
            radius: 2,
            inset_depth: 1,
            highlight_color: Some(Rgb::new(0x2f, 0x9d, 0xb4)),
            shadow_color: Some(Rgb::new(0x04, 0x26, 0x33)),
            accent_color: Some(Rgb::new(0xc3, 0x5f, 0x00)),
            indicator_color: None,
            stripe_color: None,
            stripe_shadow_color: None,
            stripe_width: 8,
        }
    }

    fn button() -> Self {
        Self {
            kind: GeneratedKind::Button,
            fill: Rgb::new(0x0b, 0x6f, 0x90),
            border_color: Rgb::new(0x0d, 0x5a, 0x70),
            border_width: 1,
            radius: 2,
            inset_depth: 1,
            highlight_color: Some(Rgb::new(0x2f, 0x9d, 0xb4)),
            shadow_color: Some(Rgb::new(0x04, 0x26, 0x33)),
            accent_color: Some(Rgb::new(0xc3, 0x5f, 0x00)),
            indicator_color: None,
            stripe_color: None,
            stripe_shadow_color: None,
            stripe_width: 8,
        }
    }

    fn slot() -> Self {
        Self {
            kind: GeneratedKind::Slot,
            fill: Rgb::new(0x08, 0x2f, 0x3c),
            border_color: Rgb::new(0x0d, 0x5a, 0x70),
            border_width: 1,
            radius: 1,
            inset_depth: 1,
            highlight_color: Some(Rgb::new(0x2f, 0x9d, 0xb4)),
            shadow_color: Some(Rgb::new(0x04, 0x26, 0x33)),
            accent_color: None,
            indicator_color: None,
            stripe_color: None,
            stripe_shadow_color: None,
            stripe_width: 8,
        }
    }

    fn hazard_bar() -> Self {
        Self {
            kind: GeneratedKind::HazardBar,
            fill: Rgb::new(0xef, 0xb3, 0x1a),
            border_color: Rgb::new(0x7c, 0x2d, 0x00),
            border_width: 1,
            radius: 0,
            inset_depth: 0,
            highlight_color: Some(Rgb::new(0xef, 0xb3, 0x1a)),
            shadow_color: Some(Rgb::new(0x5c, 0x21, 0x00)),
            accent_color: None,
            indicator_color: None,
            stripe_color: Some(Rgb::new(0xc3, 0x5f, 0x00)),
            stripe_shadow_color: None,
            stripe_width: 10,
        }
    }

    fn vent() -> Self {
        Self {
            kind: GeneratedKind::Vent,
            fill: Rgb::new(0x08, 0x2f, 0x3c),
            border_color: Rgb::new(0x0d, 0x5a, 0x70),
            border_width: 0,
            radius: 1,
            inset_depth: 0,
            highlight_color: Some(Rgb::new(0x2f, 0x9d, 0xb4)),
            shadow_color: Some(Rgb::new(0x04, 0x26, 0x33)),
            accent_color: Some(Rgb::new(0x06, 0x1b, 0x26)),
            indicator_color: None,
            stripe_color: None,
            stripe_shadow_color: None,
            stripe_width: 4,
        }
    }

    fn badge() -> Self {
        Self {
            kind: GeneratedKind::Badge,
            fill: Rgb::new(0xc3, 0x5f, 0x00),
            border_color: Rgb::new(0x7c, 0x2d, 0x00),
            border_width: 1,
            radius: 2,
            inset_depth: 1,
            highlight_color: Some(Rgb::new(0xef, 0xb3, 0x1a)),
            shadow_color: Some(Rgb::new(0x5c, 0x21, 0x00)),
            accent_color: Some(Rgb::new(0x0b, 0x87, 0x9c)),
            indicator_color: None,
            stripe_color: None,
            stripe_shadow_color: None,
            stripe_width: 6,
        }
    }
}

/// Built-in generated shape families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratedKind {
    /// Rounded panel with border and inset depth.
    Panel,
    /// Compact button/chip surface.
    Button,
    /// Recessed square/slot cell.
    Slot,
    /// Orange caution strip with diagonal stripes.
    HazardBar,
    /// Decorative horizontal vent.
    Vent,
    /// Small decorative badge/corner cap.
    Badge,
}
