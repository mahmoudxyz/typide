//! The **only** crate in the workspace permitted to perform network I/O.
//!
//! Every request is gated by [`NetworkPolicy`]. In offline mode the policy
//! *fails closed*: no request is ever attempted. See `ARCHITECTURE.md` §20.
//!
//! Package downloading and raw HTTP fetches live here (via `typst-kit`'s
//! downloader). Everything is gated by an `online` flag: when offline the
//! request fails closed (ARCHITECTURE.md §20).
#![deny(missing_docs)]

use std::any::Any;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

use typst_kit::downloader::{Downloader, SystemDownloader};
use typst_kit::packages::SystemPackages;
use typst_syntax::package::PackageSpec;

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-net";

/// A [`Downloader`] that refuses every request — used to fail closed offline.
struct NoNet;
impl Downloader for NoNet {
    fn stream(&self, _key: &dyn Any, _url: &str) -> io::Result<(Option<usize>, Box<dyn Read>)> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "network disabled (offline)",
        ))
    }
}

/// Download a package (if not already cached) into the shared Typst package
/// cache and return the directory it was extracted to.
///
/// This is the **only** place package tarballs are fetched over the network
/// (ARCHITECTURE.md P1). When `online` is false only the local cache is
/// consulted (fail closed). It uses the same cache directories as the Typst CLI,
/// so the offline compiler in `typide-world` finds packages here afterwards.
pub fn obtain_package(spec_str: &str, online: bool) -> Result<PathBuf, String> {
    let spec = PackageSpec::from_str(spec_str)
        .map_err(|e| format!("bad package spec {spec_str:?}: {e}"))?;
    let root = if online {
        SystemPackages::new(SystemDownloader::new("typide/0.1")).obtain(&spec)
    } else {
        SystemPackages::new(NoNet).obtain(&spec)
    }
    .map_err(|e| format!("failed to obtain {spec_str}: {e:?}"))?;
    Ok(root.path().to_path_buf())
}

/// Fetch a URL and return its bytes (online only). Used for the Universe index.
pub fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let downloader = SystemDownloader::new("typide/0.1");
    downloader
        .download(&() as &dyn Any, url)
        .map_err(|e| format!("fetch {url} failed: {e}"))
}

/// Network access mode. Mirrors `policy.toml [network] mode` (ARCHITECTURE.md §8.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// Never touch the network. All requests fail closed.
    Offline,
    /// Prompt the user before each network operation.
    Ask,
    /// Allow downloads and update checks.
    Online,
}

/// Runtime network policy: the mode plus an optional host allowlist.
#[derive(Debug, Clone)]
pub struct NetworkPolicy {
    mode: NetworkMode,
    allow_hosts: Vec<String>,
}

/// Reason a request was refused by [`NetworkPolicy::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Denied {
    /// The policy is in offline mode.
    Offline,
    /// The host is not on the allowlist.
    HostNotAllowed(String),
}

impl NetworkPolicy {
    /// A fail-closed policy: offline, no hosts allowed. This is the safe default.
    pub fn offline() -> Self {
        Self {
            mode: NetworkMode::Offline,
            allow_hosts: Vec::new(),
        }
    }

    /// Build a policy from a mode and a host allowlist (empty = allow any host
    /// when not offline).
    pub fn new(mode: NetworkMode, allow_hosts: Vec<String>) -> Self {
        Self { mode, allow_hosts }
    }

    /// The current mode.
    pub fn mode(&self) -> NetworkMode {
        self.mode
    }

    /// Check whether a request to `host` is permitted. `Ask` is treated as
    /// permitted here; the *interactive consent* happens one layer up before
    /// this is called.
    pub fn check(&self, host: &str) -> Result<(), Denied> {
        match self.mode {
            NetworkMode::Offline => Err(Denied::Offline),
            NetworkMode::Ask | NetworkMode::Online => {
                if self.allow_hosts.is_empty() || self.allow_hosts.iter().any(|h| h == host) {
                    Ok(())
                } else {
                    Err(Denied::HostNotAllowed(host.to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_fails_closed() {
        let p = NetworkPolicy::offline();
        assert_eq!(p.check("packages.typst.org"), Err(Denied::Offline));
    }

    #[test]
    fn online_respects_allowlist() {
        let p = NetworkPolicy::new(NetworkMode::Online, vec!["mirror.uni.example".into()]);
        assert!(p.check("mirror.uni.example").is_ok());
        assert_eq!(
            p.check("evil.example"),
            Err(Denied::HostNotAllowed("evil.example".into()))
        );
    }

    #[test]
    fn online_empty_allowlist_permits_any() {
        let p = NetworkPolicy::new(NetworkMode::Online, vec![]);
        assert!(p.check("anywhere.example").is_ok());
    }
}

#[cfg(test)]
mod net_tests {
    #[test]
    #[ignore = "hits the network"]
    fn obtains_a_real_package() {
        let dir = super::obtain_package("@preview/oxifmt:0.2.1", true).expect("download");
        assert!(
            dir.join("typst.toml").exists(),
            "package extracted at {dir:?}"
        );
    }
}
