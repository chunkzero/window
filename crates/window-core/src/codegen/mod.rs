//! Kotlin source generation from compiled Window UI data.

use crate::Result;
use crate::manifest::Manifest;
use crate::pipeline::OutputFile;

mod definitions;
mod entries;
mod literals;
mod naming;
mod pack;
mod view;
mod writer;

#[cfg(test)]
mod tests;

/// Generate typed Kotlin view subclasses, one file per manifest window/HUD.
pub fn generate_kotlin(manifest: &Manifest, package_name: &str) -> Result<Vec<OutputFile>> {
    naming::validate_package(package_name)?;
    let mut files = Vec::with_capacity(6 + manifest.windows.len() + manifest.huds.len());
    files.push(pack::generate_pack(manifest, package_name));
    files.push(pack::generate_spacers(manifest, package_name));
    files.push(pack::generate_fonts(manifest, package_name));
    files.push(pack::generate_sprites(manifest, package_name));
    files.push(definitions::generate_window_definitions(manifest, package_name));
    files.push(definitions::generate_hud_definitions(manifest, package_name));
    for (name, window) in &manifest.windows {
        files.push(view::generate_window(name, window, package_name)?);
    }
    for (name, hud) in &manifest.huds {
        files.push(view::generate_hud(name, hud, package_name)?);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}
