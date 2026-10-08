//! Error type shared across the engine.

use thiserror::Error;

/// Any failure produced while compiling a Window project.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An authored project document failed to parse or had an invalid shape.
    #[error("{path}: {message}")]
    Parse {
        /// Pack-source-relative path of the offending document.
        path: String,
        /// Human-readable description, ideally with span context.
        message: String,
    },

    /// A semantic validation failure (duplicate names, bad references, …).
    #[error("{0}")]
    Validation(String),

    /// Layout could not be solved for a window.
    #[error("window `{window}`: {message}")]
    Layout {
        /// The window being laid out.
        window: String,
        /// What went wrong.
        message: String,
    },

    /// A texture is missing or could not be decoded.
    #[error("texture `{path}`: {message}")]
    Texture {
        /// Pack-source-relative texture path.
        path: String,
        /// What went wrong.
        message: String,
    },

    /// Font generation failed (codepoint exhaustion, provider limits, …).
    #[error("font generation: {0}")]
    Font(String),

    /// Manifest serialization failed.
    #[error("manifest: {0}")]
    Manifest(String),
}

impl Error {
    /// This error, naming the authored element `debug_name` it comes from.
    pub(crate) fn with_debug_name(self, debug_name: Option<&str>) -> Self {
        let Some(debug_name) = debug_name else {
            return self;
        };
        let suffix = format!(" (in `{debug_name}`)");
        let append = |message: String| if message.ends_with(&suffix) { message } else { message + &suffix };
        match self {
            Error::Validation(message) => Error::Validation(append(message)),
            Error::Layout { window, message } => Error::Layout { window, message: append(message) },
            Error::Texture { path, message } => Error::Texture { path, message: append(message) },
            other => other,
        }
    }
}
