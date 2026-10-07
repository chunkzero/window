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

/// The server API generated window views are bound to. HUD views are the same for every target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KotlinTarget {
    /// Window views take any `WindowHost<I>`; no server imports.
    Agnostic,
    /// Window views take a Minestom `Player` and bind it through `MinestomHost`.
    Minestom,
    /// Window views take a Multistom `Player` and bind it through `MultistomHost`.
    Multistom,
}

impl KotlinTarget {
    /// The fully qualified host class built from a `Player`, or `None` when the caller supplies the host.
    fn player_host(self) -> Option<&'static str> {
        match self {
            Self::Agnostic => None,
            Self::Minestom => Some("com.chunkzero.window.minestom.MinestomHost"),
            Self::Multistom => Some("com.chunkzero.window.multistom.MultistomHost"),
        }
    }

    /// The Kotlin item type generated window members use.
    fn item_type(self) -> &'static str {
        if self.player_host().is_some() { "ItemStack" } else { "I" }
    }
}

/// Generate typed Kotlin view subclasses, one file per manifest window/HUD.
pub fn generate_kotlin(manifest: &Manifest, package_name: &str, target: KotlinTarget) -> Result<Vec<OutputFile>> {
    naming::validate_package(package_name)?;
    let mut files = Vec::with_capacity(8 + manifest.windows.len() + manifest.huds.len());
    files.push(pack::generate_pack_data(manifest, package_name));
    files.push(pack::generate_spacers(manifest, package_name));
    files.push(pack::generate_fonts(manifest, package_name));
    files.push(pack::generate_sprites(manifest, package_name));
    files.extend(pack::generate_sprite_ids(manifest, package_name));
    files.extend(pack::generate_colors(manifest, package_name)?);
    files.push(definitions::generate_window_entries(manifest, package_name));
    files.push(definitions::generate_hud_entries(manifest, package_name));
    files.push(definitions::generate_window_definitions(manifest, package_name)?);
    files.push(definitions::generate_hud_definitions(manifest, package_name)?);
    for (name, window) in &manifest.windows {
        files.push(view::generate_window(name, window, package_name, target)?);
    }
    for (name, hud) in &manifest.huds {
        files.push(view::generate_hud(name, hud, package_name, target)?);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}
