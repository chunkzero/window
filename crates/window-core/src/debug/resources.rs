//! Resource inventory, image facts, and content fingerprints.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use super::schema::{DebugAlphaMask, DebugFingerprint, DebugImage, DebugResource};
use crate::compose::Texture;
use crate::manifest::Manifest;
use crate::pipeline::OutputFile;
use crate::{Error, Result};

/// Describe every generated file, then add entries for referenced resources that were not generated.
pub(super) fn describe(
    generated: &[&OutputFile],
    referenced: BTreeSet<String>,
) -> Result<BTreeMap<String, DebugResource>> {
    let mut resources = BTreeMap::new();
    for file in generated {
        let resource_id = resource_id_from_path(&file.path);
        let image = if file.path.ends_with(".png") {
            Some(debug_image(
                &Texture::decode_png(&file.contents)
                    .map_err(|error| Error::Texture { path: file.path.clone(), message: error.to_string() })?,
            ))
        } else {
            None
        };
        resources.insert(
            file.path.clone(),
            DebugResource {
                resource_id,
                path: Some(file.path.clone()),
                generated: true,
                byte_length: Some(file.contents.len() as u64),
                fingerprint: Some(fingerprint(&file.contents)),
                image,
            },
        );
    }
    let generated_ids: BTreeSet<String> =
        resources.values().filter_map(|resource| resource.resource_id.clone()).collect();
    for id in referenced {
        if !generated_ids.contains(&id) {
            resources.insert(
                format!("resource:{id}"),
                DebugResource {
                    resource_id: Some(id),
                    path: None,
                    generated: false,
                    byte_length: None,
                    fingerprint: None,
                    image: None,
                },
            );
        }
    }
    Ok(resources)
}

fn fingerprint(bytes: &[u8]) -> DebugFingerprint {
    DebugFingerprint { algorithm: "sha256".into(), value: hex_digest(Sha256::digest(bytes)) }
}

pub(super) fn pack_fingerprint(manifest: &Manifest, files: &[&OutputFile]) -> Result<DebugFingerprint> {
    let manifest = manifest.to_json_bytes()?;
    let mut digest = Sha256::new();
    digest.update(b"window-pack-debug-v1\0");
    update_framed(&mut digest, b"manifest");
    update_framed(&mut digest, &manifest);
    for file in files {
        update_framed(&mut digest, file.path.as_bytes());
        update_framed(&mut digest, &file.contents);
    }
    Ok(DebugFingerprint { algorithm: "sha256".into(), value: hex_digest(digest.finalize()) })
}

fn update_framed(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    use std::fmt::Write;
    let bytes = bytes.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

fn debug_image(texture: &Texture) -> DebugImage {
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;
    let pixel_count = texture.width.saturating_mul(texture.height);
    let mut mask = vec![0u8; pixel_count.div_ceil(8) as usize];
    for index in 0..pixel_count {
        let alpha = texture.rgba[index as usize * 4 + 3];
        if alpha != 0 {
            let x = index % texture.width.max(1);
            let y = index / texture.width.max(1);
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            found = true;
            mask[index as usize / 8] |= 1 << (7 - index % 8);
        }
    }
    DebugImage {
        width: texture.width,
        height: texture.height,
        opaque_bounds: if found { Some([min_x, min_y, max_x - min_x + 1, max_y - min_y + 1]) } else { None },
        alpha_mask: DebugAlphaMask { encoding: "bitset_hex_msb0".into(), data: hex_digest(mask) },
    }
}

fn resource_id_from_path(path: &str) -> Option<String> {
    let rest = path.strip_prefix("assets/")?;
    let (namespace, path) = rest.split_once('/')?;
    let resource_path = if let Some(path) = path.strip_prefix("textures/") {
        path.to_string()
    } else {
        let path = path.strip_prefix("font/")?;
        path.strip_suffix(".json")?.to_string()
    };
    Some(format!("{namespace}:{resource_path}"))
}
