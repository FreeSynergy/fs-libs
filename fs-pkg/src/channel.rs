// channel.rs — ReleaseChannel: stable / testing / nightly distribution channels.
//
// Every package version is published to exactly one channel.
// Higher channels receive updates first; stable is the most conservative.

use serde::{Deserialize, Serialize};

/// Distribution channel for a package version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    /// Production-ready, thoroughly tested releases.
    #[default]
    Stable,
    /// Release candidates and preview builds.
    Testing,
    /// Latest development snapshots (may be unstable).
    Nightly,
}

impl ReleaseChannel {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Testing => "testing",
            Self::Nightly => "nightly",
        }
    }

    /// Parse from a string slice (case-insensitive).
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "stable" => Some(Self::Stable),
            "testing" => Some(Self::Testing),
            "nightly" => Some(Self::Nightly),
            _ => None,
        }
    }
}

impl std::fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}
