//! Core engine for Window: compiles TypeScript-authored UI definitions into Minecraft
//! resource-pack font artifacts (glyph textures, font providers, and the
//! compiled pack definition consumed by generated server runtimes).
//!
//! This crate is pure: no I/O assumptions, no host bindings. Inputs arrive as
//! bytes (see [`pipeline::CompileInput`]) and outputs leave as bytes (see
//! [`pipeline::CompileOutput`]). The architectural contract lives in
//! `docs/ARCHITECTURE.md`; the compiled definition schema in `docs/MANIFEST.md`; the
//! authoring format in `docs/AUTHORING.md`.

pub mod authoring;
pub mod bake;
pub mod codegen;
pub mod compose;
pub mod debug;
pub mod error;
pub mod font;
pub mod geometry;
mod hud;
pub mod inventory;
pub mod ir;
pub mod layout;
pub mod manifest;
pub mod model;
pub mod pipeline;
pub mod raster;
pub mod surface;
pub mod text_font;
pub mod validation;
pub mod vanilla;

pub use error::Error;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, Error>;
