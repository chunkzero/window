//! Inline art: art values authored in place of theme names. Each is interned into the theme under a name derived
//! from a hash of its content, prefixed by its optional name, so identical art is shared and no art clashes with
//! another or with a theme name.

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::theme::{FrameDto, SpriteDto};
use crate::model::{SpriteDef, Theme};
use crate::{Error, Result};

/// The prefix of every interned art name. Theme names cannot contain `/`, so interned names never clash with them.
pub(crate) const ART_PREFIX: &str = "art/";

/// Shortest hash suffix of an interned name; longer suffixes only resolve hash prefix collisions.
const HASH_LEN: usize = 6;

/// A frame or sprite reference: a theme name, or an inline art value.
#[derive(Clone, Debug)]
pub(super) enum ArtRefDto {
    Named(String),
    Inline(InlineArtDto),
}

/// An inline art value: `texture(path, options)` or `shape(style, options)`.
#[derive(Clone, Debug)]
pub(super) struct InlineArtDto {
    /// `"texture"` or `"shape"`.
    art: String,
    name: Option<String>,
    /// The frame or sprite definition, as a theme declares it.
    def: serde_json::Map<String, serde_json::Value>,
}

impl<'de> Deserialize<'de> for ArtRefDto {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(name) => Ok(Self::Named(name)),
            serde_json::Value::Object(mut def) => {
                let art = match def.remove("art") {
                    Some(serde_json::Value::String(art)) if art == "texture" || art == "shape" => art,
                    _ => return Err(D::Error::custom("art value must set `art` to \"texture\" or \"shape\"")),
                };
                let name = match def.remove("name") {
                    None => None,
                    Some(serde_json::Value::String(name)) => Some(name),
                    Some(_) => return Err(D::Error::custom("art `name` must be a string")),
                };
                Ok(Self::Inline(InlineArtDto { art, name, def }))
            }
            _ => Err(D::Error::custom("expected a theme name or an art value")),
        }
    }
}

impl ArtRefDto {
    /// The theme or interned name; inline art must be interned first.
    pub(super) fn name(&self) -> Result<String> {
        match self {
            Self::Named(name) => Ok(name.clone()),
            Self::Inline(_) => Err(Error::Validation("inline art was not interned".into())),
        }
    }
}

/// How an art value is drawn, which decides the theme table it joins.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArtUse {
    /// Stretched over a laid-out rect.
    Frame,
    /// Drawn at its own size into static art.
    Image,
    /// Drawn at its own size by the runtime, as a fixed sprite slot or a collection selection.
    Sprite,
}

/// Replaces an inline art value in `art` with the name it is interned under in `theme`.
pub(super) fn intern(theme: &mut Theme, art: &mut Option<ArtRefDto>, usage: ArtUse) -> Result<()> {
    let Some(ArtRefDto::Inline(inline)) = art else {
        return Ok(());
    };
    let name = intern_inline(theme, inline, usage)?;
    *art = Some(ArtRefDto::Named(name));
    Ok(())
}

fn intern_inline(theme: &mut Theme, inline: &InlineArtDto, usage: ArtUse) -> Result<String> {
    let def = inline.def(usage)?;
    let table = if usage == ArtUse::Frame { "frame" } else { "sprite" };
    let content = serde_json::Value::Object(def.clone()).to_string();
    let digest = Sha256::digest(format!("{}\0{table}\0{content}", inline.art).as_bytes());
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let prefix = match &inline.name {
        Some(name) => format!("{ART_PREFIX}{}-", slug(name)?),
        None => ART_PREFIX.to_string(),
    };
    if usage == ArtUse::Frame {
        let frame = inline.parse::<FrameDto>(def)?.into_frame()?;
        let key = free_key(&prefix, &hash, |key| theme.frames.get(key).is_none_or(|existing| *existing == frame));
        theme.frames.insert(key.clone(), frame);
        return Ok(key);
    }
    let sprite = inline.parse::<SpriteDto>(def)?.into_sprite()?;
    let key = free_key(&prefix, &hash, |key| theme.sprites.get(key).is_none_or(|existing| *existing == sprite));
    theme.sprites.insert(key.clone(), sprite);
    if usage == ArtUse::Sprite {
        theme.runtime_art.insert(key.clone());
    }
    Ok(key)
}

impl InlineArtDto {
    /// The art as a runtime sprite definition.
    pub(super) fn sprite(&self) -> Result<SpriteDef> {
        self.parse::<SpriteDto>(self.def(ArtUse::Sprite)?)?.into_sprite()
    }

    /// The theme definition of this art drawn as `usage`.
    fn def(&self, usage: ArtUse) -> Result<serde_json::Map<String, serde_json::Value>> {
        let mut def = self.def.clone();
        if usage == ArtUse::Frame {
            // Frames are drawn at their laid-out size.
            def.remove("width");
            def.remove("height");
        } else {
            // Images and sprites are drawn whole, so nine-slice insets do not apply.
            def.remove("insets");
        }
        match (self.art.as_str(), def.contains_key("texture")) {
            ("texture", false) => Err(Error::Validation("texture art requires `texture`".into())),
            ("shape", true) => Err(Error::Validation("shape art does not accept `texture`".into())),
            _ => Ok(def),
        }
    }

    fn parse<T: serde::de::DeserializeOwned>(&self, def: serde_json::Map<String, serde_json::Value>) -> Result<T> {
        serde_json::from_value(serde_json::Value::Object(def))
            .map_err(|error| Error::Validation(format!("{} art: {error}", self.art)))
    }
}

/// The shortest `prefix` + hash prefix that `fits`: unused, or holding the same art.
fn free_key(prefix: &str, hash: &str, fits: impl Fn(&str) -> bool) -> String {
    (HASH_LEN..=hash.len())
        .map(|len| format!("{prefix}{}", &hash[..len]))
        .find(|key| fits(key))
        .unwrap_or_else(|| format!("{prefix}{hash}"))
}

/// An art name as one resource path segment: `industrial/button` becomes `industrial-button`.
fn slug(name: &str) -> Result<String> {
    let valid = name.bytes().next().is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'/'));
    if !valid {
        return Err(Error::Validation(format!("art name `{name}` is invalid; must match ^[a-z0-9][a-z0-9_/-]*$")));
    }
    Ok(name.replace(['/', '_'], "-"))
}

/// Whether `name` is an interned art name rather than a theme name.
pub(crate) fn is_inline(name: &str) -> bool {
    name.starts_with(ART_PREFIX)
}
