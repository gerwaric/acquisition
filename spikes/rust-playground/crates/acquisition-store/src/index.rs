//! The account index: `store/<provider>/accounts.json`, a small non-secret
//! list of the accounts this machine has logged into (GGG username,
//! `name#discriminator`), maintained by the daemon at login/logout and
//! read by frontends to resolve `--account`/`ACQ_ACCOUNT` without a
//! daemon. It is also how the daemon knows which keyring entries to load
//! at start (the keyring crate cannot enumerate). Secrets never live here.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountEntry {
    /// GGG username as the token response reports it: `name#discriminator`.
    pub username: String,
    /// Unix seconds of the most recent login.
    pub last_login: i64,
    /// A refresh token for this account is in the keyring. False for
    /// one-off sessions (`ACQ_NO_KEYRING`, a failed keyring save) and after
    /// logout; the account stays listed because its store file remains.
    pub persisted: bool,
    /// From `GET /profile`, recorded opportunistically; stable across name
    /// changes. Not used as a key yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct File {
    accounts: Vec<AccountEntry>,
}

#[derive(Debug)]
pub struct Index {
    path: PathBuf,
    entries: Vec<AccountEntry>,
}

/// How a selector failed to pick exactly one account.
#[derive(Debug, PartialEq, Eq)]
pub enum Resolve {
    /// No accounts are known at all.
    Empty,
    /// The selector matched nothing.
    NotFound(String),
    /// No selector, and more than one account is known.
    Ambiguous(Vec<String>),
}

impl std::fmt::Display for Resolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resolve::Empty => write!(f, "no accounts known (run `acq auth`)"),
            Resolve::NotFound(s) => write!(f, "no account matches {s:?} (see `acq accounts`)"),
            Resolve::Ambiguous(names) => write!(
                f,
                "several accounts are known; pick one with --account or ACQ_ACCOUNT: {}",
                names.join(", ")
            ),
        }
    }
}

impl std::error::Error for Resolve {}

/// The provider's store directory: `$ACQ_STORE_DIR/<provider>`, else
/// `~/.local/share/acquisition-playground/store/<provider>`. One
/// directory per provider so mock data never mixes with real.
pub fn store_dir(provider: &str) -> PathBuf {
    let base = match std::env::var_os("ACQ_STORE_DIR") {
        Some(d) => PathBuf::from(d),
        None => std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join(".local/share/acquisition-playground/store"),
    };
    base.join(provider)
}

/// `<dir>/<username>.db`, with the username made filename-safe
/// (`GERWARIC#7694` → `GERWARIC_7694.db`).
pub fn account_path(dir: &Path, username: &str) -> PathBuf {
    let safe: String = username
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    dir.join(format!("{safe}.db"))
}

pub fn index_path(dir: &Path) -> PathBuf {
    dir.join("accounts.json")
}

/// Whether a selector names this account: the exact username, the name
/// without its `#discriminator` (both case-insensitive), or the uuid.
/// Never a prefix.
pub fn account_matches(selector: &str, username: &str, uuid: Option<&str>) -> bool {
    let sel = selector.to_lowercase();
    let u = username.to_lowercase();
    u == sel || u.split_once('#').is_some_and(|(name, _)| name == sel) || uuid == Some(selector)
}

impl Index {
    /// A missing file is an empty index.
    pub fn load(dir: &Path) -> Result<Index> {
        let path = index_path(dir);
        let entries = match std::fs::read_to_string(&path) {
            Ok(text) => {
                serde_json::from_str::<File>(&text)
                    .with_context(|| format!("parsing {}", path.display()))?
                    .accounts
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
        };
        Ok(Index { path, entries })
    }

    pub fn entries(&self) -> &[AccountEntry] {
        &self.entries
    }

    pub fn get(&self, username: &str) -> Option<&AccountEntry> {
        self.entries.iter().find(|e| e.username == username)
    }

    /// Record a login. Rewrites the file.
    pub fn upsert(&mut self, username: &str, persisted: bool, at: i64) -> Result<()> {
        match self.entries.iter_mut().find(|e| e.username == username) {
            Some(e) => {
                e.last_login = at;
                e.persisted = persisted;
            }
            None => self.entries.push(AccountEntry {
                username: username.to_string(),
                last_login: at,
                persisted,
                uuid: None,
            }),
        }
        self.save()
    }

    /// Logout or a dead grant: the keyring no longer holds this account.
    pub fn set_persisted(&mut self, username: &str, persisted: bool) -> Result<()> {
        if let Some(e) = self.entries.iter_mut().find(|e| e.username == username) {
            e.persisted = persisted;
            self.save()?;
        }
        Ok(())
    }

    pub fn set_uuid(&mut self, username: &str, uuid: &str) -> Result<()> {
        if let Some(e) = self.entries.iter_mut().find(|e| e.username == username) {
            e.uuid = Some(uuid.to_string());
            self.save()?;
        }
        Ok(())
    }

    /// Accounts with a keyring entry, most recent login first: the order
    /// a daemon restores them in.
    pub fn persisted(&self) -> Vec<&AccountEntry> {
        let mut v: Vec<&AccountEntry> = self.entries.iter().filter(|e| e.persisted).collect();
        v.sort_by_key(|e| std::cmp::Reverse(e.last_login));
        v
    }

    /// Pick one account. A selector matches the exact username, the
    /// username without its `#discriminator`, or the exact uuid — never a
    /// prefix. No selector picks the sole account, or fails as ambiguous.
    pub fn resolve(&self, selector: Option<&str>) -> Result<&AccountEntry, Resolve> {
        if self.entries.is_empty() {
            return Err(Resolve::Empty);
        }
        match selector {
            None => match self.entries.as_slice() {
                [only] => Ok(only),
                _ => Err(Resolve::Ambiguous(
                    self.entries.iter().map(|e| e.username.clone()).collect(),
                )),
            },
            Some(sel) => {
                let hits: Vec<&AccountEntry> = self
                    .entries
                    .iter()
                    .filter(|e| account_matches(sel, &e.username, e.uuid.as_deref()))
                    .collect();
                match hits.as_slice() {
                    [one] => Ok(one),
                    [] => Err(Resolve::NotFound(sel.to_string())),
                    many => Err(Resolve::Ambiguous(
                        many.iter().map(|e| e.username.clone()).collect(),
                    )),
                }
            }
        }
    }

    fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let file = File {
            accounts: self.entries.clone(),
        };
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(&file)?)
            .with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, &self.path)
            .with_context(|| format!("replacing {}", self.path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "acq-index-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trip_and_resolution() {
        let dir = tmp();
        let mut idx = Index::load(&dir).unwrap();
        assert_eq!(idx.resolve(None), Err(Resolve::Empty));
        idx.upsert("Alice#1234", true, 10).unwrap();
        assert_eq!(idx.resolve(None).unwrap().username, "Alice#1234");
        idx.upsert("Bob#0001", false, 20).unwrap();
        assert!(matches!(idx.resolve(None), Err(Resolve::Ambiguous(ref v)) if v.len() == 2));
        assert_eq!(idx.resolve(Some("alice")).unwrap().username, "Alice#1234");
        assert_eq!(
            idx.resolve(Some("Alice#1234")).unwrap().username,
            "Alice#1234"
        );
        assert_eq!(
            idx.resolve(Some("Ali")),
            Err(Resolve::NotFound("Ali".into()))
        );
        idx.set_uuid("Bob#0001", "u-bob").unwrap();
        assert_eq!(idx.resolve(Some("u-bob")).unwrap().username, "Bob#0001");
        // Reload from disk: same content; persisted order is by recency.
        let idx = Index::load(&dir).unwrap();
        assert_eq!(idx.entries().len(), 2);
        assert_eq!(
            idx.persisted()
                .iter()
                .map(|e| e.username.as_str())
                .collect::<Vec<_>>(),
            vec!["Alice#1234"]
        );
        let mut idx = idx;
        idx.upsert("Bob#0001", true, 30).unwrap();
        assert_eq!(idx.persisted()[0].username, "Bob#0001");
        idx.set_persisted("Bob#0001", false).unwrap();
        assert_eq!(idx.persisted().len(), 1);
        assert_eq!(
            account_path(&dir, "Alice#1234").file_name().unwrap(),
            "Alice_1234.db"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_index_is_an_error_not_a_reset() {
        let dir = tmp();
        std::fs::write(index_path(&dir), "{nope").unwrap();
        assert!(Index::load(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
