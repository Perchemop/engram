//! App-level theme preference (domain side).

use serde::{Deserialize, Serialize};

/// Colour theme of the application shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// All themes in toggle order.
    pub const ALL: [Theme; 2] = [Theme::Dark, Theme::Light];

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_serde_roundtrip() {
        for t in Theme::ALL {
            let json = serde_json::to_string(&t).unwrap();
            let back: Theme = serde_json::from_str(&json).unwrap();
            assert_eq!(back, t);
        }
    }
}
