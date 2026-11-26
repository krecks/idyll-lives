use std::fmt;

/// Error raised while loading data or applying commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    /// What failed is damaged, not merely unreadable by this version: a save the loader's own checks
    /// found corrupt (a broken lot, a broken id, chemistry that isn't there). A save like that is not
    /// worth keeping for a later version to open, so the menu removes it (`sim-wasm`'s `LoadError`).
    damaged: bool,
}

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            damaged: false,
        }
    }

    /// An error about data that is damaged: see `damaged`.
    pub fn damaged(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            damaged: true,
        }
    }

    pub fn is_damaged(&self) -> bool {
        self.damaged
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(format!("invalid JSON: {e}"))
    }
}
