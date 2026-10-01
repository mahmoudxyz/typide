//! Settings merge with admin policy override.
//!
//! Precedence (ARCHITECTURE.md §8.5): `defaults < user < project (non-security)
//! < policy`. Security-relevant keys (network, AI, updates) can never be
//! *loosened* by project files. Milestone-0 skeleton models the merge order.
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-policy";

// ── Institutional policy file (ARCHITECTURE.md §8.5) ─────────────────────────

#[derive(Debug, Default, Deserialize)]
struct PolicyFile {
    network: Option<NetworkSection>,
    updates: Option<UpdatesSection>,
    ai: Option<AiSection>,
}
#[derive(Debug, Default, Deserialize)]
struct NetworkSection {
    mode: Option<String>,
    #[serde(default, rename = "allow-hosts")]
    allow_hosts: Vec<String>,
}
#[derive(Debug, Default, Deserialize)]
struct UpdatesSection {
    enabled: Option<bool>,
}
#[derive(Debug, Default, Deserialize)]
struct AiSection {
    enabled: Option<bool>,
}

/// The effective admin policy. Any `Some`/non-empty field **overrides** user
/// settings and is shown as locked in the UI. Security keys can never be
/// loosened by project files.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Policy {
    /// Enforced network mode: "offline" | "ask" | "online".
    pub network_mode: Option<String>,
    /// Host allowlist (when not offline).
    pub allow_hosts: Vec<String>,
    /// Whether app updates are permitted.
    pub updates_enabled: Option<bool>,
    /// Whether AI features are permitted.
    pub ai_enabled: Option<bool>,
    /// Path the policy was loaded from (if any).
    pub source: Option<String>,
}

impl Policy {
    /// Whether any security-relevant key is enforced by policy.
    pub fn is_active(&self) -> bool {
        self.network_mode.is_some() || self.updates_enabled.is_some() || self.ai_enabled.is_some()
    }
}

/// The OS-specific system policy location.
fn policy_path() -> PathBuf {
    if let Ok(p) = std::env::var("TYPIDE_POLICY_PATH") {
        return PathBuf::from(p);
    }
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into());
        PathBuf::from(base).join("typide").join("policy.toml")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/typide/policy.toml")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        PathBuf::from("/etc/typide/policy.toml")
    }
}

/// Load the system policy, or a default (no-op) policy if none is present.
pub fn load() -> Policy {
    let path = policy_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Policy::default();
    };
    parse(&text, Some(path.to_string_lossy().to_string()))
}

fn parse(text: &str, source: Option<String>) -> Policy {
    let pf: PolicyFile = toml::from_str(text).unwrap_or_default();
    Policy {
        network_mode: pf.network.as_ref().and_then(|n| n.mode.clone()),
        allow_hosts: pf
            .network
            .as_ref()
            .map(|n| n.allow_hosts.clone())
            .unwrap_or_default(),
        updates_enabled: pf.updates.and_then(|u| u.enabled),
        ai_enabled: pf.ai.and_then(|a| a.enabled),
        source,
    }
}

/// The layers that contribute to effective settings, lowest precedence first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// Built-in defaults.
    Defaults,
    /// User settings (`settings.toml`).
    User,
    /// Project settings (`typide.toml`) — only non-security keys apply.
    Project,
    /// Admin policy (`policy.toml`) — always wins.
    Policy,
}

impl Layer {
    /// Numeric precedence; higher wins.
    pub fn precedence(self) -> u8 {
        match self {
            Layer::Defaults => 0,
            Layer::User => 1,
            Layer::Project => 2,
            Layer::Policy => 3,
        }
    }

    /// Whether this layer may set security-relevant keys (network, AI, updates).
    pub fn may_set_security_keys(self) -> bool {
        !matches!(self, Layer::Project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_outranks_everything() {
        assert!(Layer::Policy.precedence() > Layer::Project.precedence());
        assert!(Layer::Project.precedence() > Layer::User.precedence());
        assert!(Layer::User.precedence() > Layer::Defaults.precedence());
    }

    #[test]
    fn project_cannot_set_security_keys() {
        assert!(!Layer::Project.may_set_security_keys());
        assert!(Layer::Policy.may_set_security_keys());
    }

    #[test]
    fn parses_enforced_network_mode() {
        let p = parse(
            "[network]\nmode = \"offline\"\nallow-hosts = [\"mirror.uni.example\"]\n\n[updates]\nenabled = false\n",
            None,
        );
        assert_eq!(p.network_mode.as_deref(), Some("offline"));
        assert_eq!(p.allow_hosts, vec!["mirror.uni.example".to_string()]);
        assert_eq!(p.updates_enabled, Some(false));
        assert!(p.is_active());
    }

    #[test]
    fn empty_policy_is_inactive() {
        assert!(!parse("", None).is_active());
    }
}
