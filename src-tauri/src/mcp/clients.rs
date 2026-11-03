//! The clients the user has allowed or rejected. A client is known by the name
//! and the version it declares in `initialize`. The answer is remembered in
//! `clients.json`; a client that has not been answered yet is only in memory,
//! so after a restart it must ask again.
//!
//! The name is a label the client chose, not a proof of who it is. The real
//! protection is the token; this list is the user's control (see
//! `docs/ai-and-mcp.md`).

use super::token::write_private;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CLIENTS_FILE: &str = "clients.json";

/// How many clients may wait for an answer at the same time.
pub const MAX_PENDING: usize = 16;
/// How many answers are kept in `clients.json`.
pub const MAX_KNOWN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allowed,
    Rejected,
}

/// Where a client stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    Allowed,
    Rejected,
    /// Waiting for the user to answer.
    Pending,
    /// Never seen, or removed.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Known {
    name: String,
    version: String,
    decision: Decision,
}

/// A client as the UI lists it. `state` is `allowed`, `rejected` or `pending`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub state: &'static str,
    /// The user pressed Stop for this client and has not let it continue.
    pub stopped: bool,
}

pub struct Clients {
    file: PathBuf,
    known: Vec<Known>,
    pending: Vec<(String, String)>,
}

impl Clients {
    /// Reads `clients.json` from the data folder. A missing or damaged file
    /// means no client is known: the safe answer is to ask again.
    pub fn load(data_dir: &Path) -> Clients {
        let file = data_dir.join(CLIENTS_FILE);
        let known = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Clients { file, known, pending: Vec::new() }
    }

    fn save(&self) -> std::io::Result<()> {
        let text = serde_json::to_vec_pretty(&self.known)?;
        write_private(&self.file, &text)
    }

    pub fn standing(&self, name: &str, version: &str) -> Standing {
        if let Some(known) = self.known.iter().find(|k| k.name == name && k.version == version) {
            return match known.decision {
                Decision::Allowed => Standing::Allowed,
                Decision::Rejected => Standing::Rejected,
            };
        }
        if self.pending.iter().any(|(n, v)| n == name && v == version) {
            Standing::Pending
        } else {
            Standing::Unknown
        }
    }

    /// A client connects. A client that is not known becomes pending (or is
    /// allowed at once when `ask` is false). The flag is true when the client
    /// just became pending, so the user is told only once. `None` when the
    /// lists are full: a program must not be able to fill them with names.
    pub fn request(&mut self, name: &str, version: &str, ask: bool) -> Option<(Standing, bool)> {
        Some(match self.standing(name, version) {
            Standing::Unknown if ask => {
                if self.pending.len() >= MAX_PENDING {
                    return None;
                }
                self.pending.push((name.to_string(), version.to_string()));
                (Standing::Pending, true)
            }
            Standing::Unknown => {
                if self.known.len() >= MAX_KNOWN {
                    return None;
                }
                self.known.push(Known {
                    name: name.to_string(),
                    version: version.to_string(),
                    decision: Decision::Allowed,
                });
                // If it cannot be saved the client still works for this run.
                let _ = self.save();
                (Standing::Allowed, false)
            }
            standing => (standing, false),
        })
    }

    /// The answer of the user for a pending client. False if none waits.
    pub fn answer(&mut self, name: &str, version: &str, allow: bool) -> std::io::Result<bool> {
        let Some(at) = self.pending.iter().position(|(n, v)| n == name && v == version) else {
            return Ok(false);
        };
        self.pending.remove(at);
        self.known.retain(|k| !(k.name == name && k.version == version));
        self.known.push(Known {
            name: name.to_string(),
            version: version.to_string(),
            decision: if allow { Decision::Allowed } else { Decision::Rejected },
        });
        self.save()?;
        Ok(true)
    }

    /// Forgets a client (allowed, rejected or pending). It must ask again.
    pub fn remove(&mut self, name: &str, version: &str) -> std::io::Result<bool> {
        let before = self.known.len() + self.pending.len();
        self.known.retain(|k| !(k.name == name && k.version == version));
        self.pending.retain(|(n, v)| !(n == name && v == version));
        let removed = self.known.len() + self.pending.len() < before;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn list(&self) -> Vec<ClientInfo> {
        let known = self.known.iter().map(|k| ClientInfo {
            name: k.name.clone(),
            version: k.version.clone(),
            state: match k.decision {
                Decision::Allowed => "allowed",
                Decision::Rejected => "rejected",
            },
            stopped: false,
        });
        let pending = self.pending.iter().map(|(name, version)| ClientInfo {
            name: name.clone(),
            version: version.clone(),
            state: "pending",
            stopped: false,
        });
        known.chain(pending).collect()
    }
}
