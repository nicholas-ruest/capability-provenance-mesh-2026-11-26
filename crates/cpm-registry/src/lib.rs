#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns,
    clippy::return_self_not_must_use,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::collapsible_if
)]
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub plugin: String,
    pub version: String,
    pub receipt: String,
    pub revoked: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryEvent {
    Activated {
        entry: Entry,
        sequence: u64,
    },
    Revoked {
        plugin: String,
        version: String,
        reason: String,
        sequence: u64,
    },
    RolledBack {
        plugin: String,
        from: String,
        to: String,
        sequence: u64,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Registry {
    entries: BTreeMap<(String, String), Entry>,
    active: BTreeMap<String, String>,
    events: Vec<RegistryEvent>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginHistory {
    pub plugin: String,
    pub versions: Vec<Entry>,
    pub active: Option<String>,
    pub revocations: usize,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("decision denied")]
    Denied,
    #[error("plugin is already active at this version")]
    Duplicate,
    #[error("plugin version not found")]
    Missing,
    #[error("plugin is revoked")]
    Revoked,
    #[error("plugin identifier is invalid")]
    Plugin,
    #[error("version is invalid")]
    Version,
    #[error("receipt digest is invalid")]
    Receipt,
    #[error("revocation reason is required")]
    Reason,
    #[error("cannot rollback to current version")]
    SameVersion,
    #[error("registry replay sequence mismatch at {0}")]
    Replay(u64),
}
impl Registry {
    pub fn activate(&mut self, entry: Entry, admitted: bool) -> Result<(), RegistryError> {
        if !admitted {
            return Err(RegistryError::Denied);
        }
        validate_entry(&entry)?;
        let key = (entry.plugin.clone(), entry.version.clone());
        if self.entries.contains_key(&key) {
            return Err(RegistryError::Duplicate);
        }
        if entry.revoked {
            return Err(RegistryError::Revoked);
        }
        let sequence = self.events.len() as u64 + 1;
        self.active
            .insert(entry.plugin.clone(), entry.version.clone());
        self.entries.insert(key, entry.clone());
        self.events
            .push(RegistryEvent::Activated { entry, sequence });
        Ok(())
    }
    pub fn get(&self, plugin: &str, version: &str) -> Result<&Entry, RegistryError> {
        self.entries
            .get(&(plugin.into(), version.into()))
            .ok_or(RegistryError::Missing)
    }
    pub fn active(&self, plugin: &str) -> Option<&Entry> {
        let version = self.active.get(plugin)?;
        self.get(plugin, version).ok()
    }
    pub fn revoke(&mut self, plugin: &str, version: &str) -> Result<(), RegistryError> {
        self.revoke_with_reason(plugin, version, "operator revocation")
    }
    pub fn revoke_with_reason(
        &mut self,
        plugin: &str,
        version: &str,
        reason: &str,
    ) -> Result<(), RegistryError> {
        if reason.trim().is_empty() || reason.len() > 256 {
            return Err(RegistryError::Reason);
        }
        let entry = self
            .entries
            .get_mut(&(plugin.into(), version.into()))
            .ok_or(RegistryError::Missing)?;
        entry.revoked = true;
        if self.active.get(plugin).is_some_and(|v| v == version) {
            self.active.remove(plugin);
        }
        let sequence = self.events.len() as u64 + 1;
        self.events.push(RegistryEvent::Revoked {
            plugin: plugin.into(),
            version: version.into(),
            reason: reason.into(),
            sequence,
        });
        Ok(())
    }
    pub fn rollback(&mut self, plugin: &str, target: &str) -> Result<(), RegistryError> {
        let from = self
            .active
            .get(plugin)
            .cloned()
            .ok_or(RegistryError::Missing)?;
        if from == target {
            return Err(RegistryError::SameVersion);
        }
        let target_entry = self.get(plugin, target)?;
        if target_entry.revoked {
            return Err(RegistryError::Revoked);
        }
        self.active.insert(plugin.into(), target.into());
        let sequence = self.events.len() as u64 + 1;
        self.events.push(RegistryEvent::RolledBack {
            plugin: plugin.into(),
            from,
            to: target.into(),
            sequence,
        });
        Ok(())
    }
    pub fn history(&self, plugin: &str) -> PluginHistory {
        let mut versions: Vec<_> = self
            .entries
            .values()
            .filter(|e| e.plugin == plugin)
            .cloned()
            .collect();
        versions.sort_by(|a, b| compare_version(&a.version, &b.version));
        let revocations = versions.iter().filter(|e| e.revoked).count();
        PluginHistory {
            plugin: plugin.into(),
            versions,
            active: self.active.get(plugin).cloned(),
            revocations,
        }
    }
    pub fn events(&self) -> &[RegistryEvent] {
        &self.events
    }
    pub fn replay(events: &[RegistryEvent]) -> Result<Self, RegistryError> {
        let mut registry = Self::default();
        for (index, event) in events.iter().enumerate() {
            let expected = index as u64 + 1;
            let actual = match event {
                RegistryEvent::Activated { sequence, .. }
                | RegistryEvent::Revoked { sequence, .. }
                | RegistryEvent::RolledBack { sequence, .. } => *sequence,
            };
            if actual != expected {
                return Err(RegistryError::Replay(actual));
            }
            match event {
                RegistryEvent::Activated { entry, .. } => registry.activate(entry.clone(), true)?,
                RegistryEvent::Revoked {
                    plugin,
                    version,
                    reason,
                    ..
                } => registry.revoke_with_reason(plugin, version, reason)?,
                RegistryEvent::RolledBack { plugin, to, .. } => registry.rollback(plugin, to)?,
            }
        }
        Ok(registry)
    }
    pub fn plugins(&self) -> BTreeSet<String> {
        self.entries.values().map(|e| e.plugin.clone()).collect()
    }
    pub fn receipt_in_use(&self, receipt: &str) -> bool {
        self.entries.values().any(|e| e.receipt == receipt)
    }
}
fn validate_entry(e: &Entry) -> Result<(), RegistryError> {
    if e.plugin.is_empty()
        || e.plugin.len() > 96
        || !e
            .plugin
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(RegistryError::Plugin);
    }
    let parts: Vec<_> = e.version.split('.').collect();
    if parts.is_empty()
        || parts.len() > 3
        || parts
            .iter()
            .any(|p| p.is_empty() || p.parse::<u32>().is_err())
    {
        return Err(RegistryError::Version);
    }
    if e.receipt.len() != 64
        || !e
            .receipt
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(RegistryError::Receipt);
    }
    Ok(())
}
fn compare_version(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| {
        v.split('.')
            .map(|p| p.parse::<u32>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(a).cmp(&parse(b))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entry(v: &str) -> Entry {
        Entry {
            plugin: "plugin-a".into(),
            version: v.into(),
            receipt: "a".repeat(64),
            revoked: false,
        }
    }
    #[test]
    fn activates_and_reads_current() {
        let mut r = Registry::default();
        r.activate(entry("1.0.0"), true).unwrap();
        assert_eq!(r.active("plugin-a").unwrap().version, "1.0.0");
    }
    #[test]
    fn duplicate_activation_is_rejected() {
        let mut r = Registry::default();
        r.activate(entry("1.0.0"), true).unwrap();
        assert_eq!(
            r.activate(entry("1.0.0"), true),
            Err(RegistryError::Duplicate)
        );
    }
    #[test]
    fn revoke_removes_active_pointer() {
        let mut r = Registry::default();
        r.activate(entry("1.0.0"), true).unwrap();
        r.revoke_with_reason("plugin-a", "1.0.0", "incident")
            .unwrap();
        assert!(r.active("plugin-a").is_none());
        assert!(r.get("plugin-a", "1.0.0").unwrap().revoked);
    }
    #[test]
    fn rollback_selects_known_unrevoked_version() {
        let mut r = Registry::default();
        r.activate(entry("1.0.0"), true).unwrap();
        r.activate(entry("2.0.0"), true).unwrap();
        r.rollback("plugin-a", "1.0.0").unwrap();
        assert_eq!(r.active("plugin-a").unwrap().version, "1.0.0");
    }
    #[test]
    fn rollback_to_revoked_version_fails() {
        let mut r = Registry::default();
        r.activate(entry("1.0.0"), true).unwrap();
        r.activate(entry("2.0.0"), true).unwrap();
        r.revoke("plugin-a", "1.0.0").unwrap();
        assert_eq!(r.rollback("plugin-a", "1.0.0"), Err(RegistryError::Revoked));
    }
    #[test]
    fn history_sorts_semantic_versions() {
        let mut r = Registry::default();
        r.activate(entry("10.0.0"), true).unwrap();
        r.activate(entry("2.0.0"), true).unwrap();
        let h = r.history("plugin-a");
        assert_eq!(
            h.versions
                .iter()
                .map(|e| e.version.as_str())
                .collect::<Vec<_>>(),
            vec!["2.0.0", "10.0.0"]
        );
    }
    #[test]
    fn replay_reconstructs_registry() {
        let mut r = Registry::default();
        r.activate(entry("1.0.0"), true).unwrap();
        r.activate(entry("2.0.0"), true).unwrap();
        r.rollback("plugin-a", "1.0.0").unwrap();
        let rebuilt = Registry::replay(r.events()).unwrap();
        assert_eq!(rebuilt.active("plugin-a"), r.active("plugin-a"));
    }
    #[test]
    fn malformed_receipt_fails_closed() {
        let mut e = entry("1.0.0");
        e.receipt = "branch".into();
        assert_eq!(
            Registry::default().activate(e, true),
            Err(RegistryError::Receipt)
        );
    }
}
