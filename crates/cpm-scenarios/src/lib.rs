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

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioPack {
    pub name: String,
    pub inputs: Vec<Vec<u8>>,
    pub digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedOutcome {
    Allow,
    Deny,
    Trap,
    Timeout,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCase {
    pub id: String,
    pub input: String,
    pub expected: ExpectedOutcome,
    pub required_evidence: Vec<String>,
    pub tags: BTreeSet<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioSuite {
    pub name: String,
    pub cases: Vec<ScenarioCase>,
    pub digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub actual: ExpectedOutcome,
    pub evidence: Vec<String>,
    pub duration_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteEvaluation {
    pub passed: usize,
    pub failed: Vec<String>,
    pub missing_evidence: BTreeMap<String, Vec<String>>,
    pub total_duration_ms: u64,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScenarioError {
    #[error("scenario name is empty or invalid")]
    Name,
    #[error("scenario pack is empty")]
    Empty,
    #[error("scenario digest mismatch")]
    Digest,
    #[error("duplicate scenario id {0}")]
    Duplicate(String),
    #[error("unknown result id {0}")]
    UnknownResult(String),
    #[error("missing result for {0}")]
    MissingResult(String),
    #[error("scenario evidence digest is invalid")]
    Evidence,
    #[error("duration overflow")]
    DurationOverflow,
}
impl ScenarioPack {
    pub fn freeze(name: String, inputs: Vec<Vec<u8>>) -> Result<Self, ScenarioError> {
        validate_name(&name)?;
        if inputs.is_empty() {
            return Err(ScenarioError::Empty);
        }
        let digest = hash_bytes(&name, &inputs);
        Ok(Self {
            name,
            inputs,
            digest,
        })
    }
    pub fn verify(&self) -> Result<(), ScenarioError> {
        validate_name(&self.name)?;
        if self.inputs.is_empty() {
            return Err(ScenarioError::Empty);
        }
        if hash_bytes(&self.name, &self.inputs) != self.digest {
            return Err(ScenarioError::Digest);
        }
        Ok(())
    }
    pub fn mutation_index(&self, other: &Self) -> Result<Vec<usize>, ScenarioError> {
        self.verify()?;
        other.verify()?;
        let n = self.inputs.len().max(other.inputs.len());
        Ok((0..n)
            .filter(|i| self.inputs.get(*i) != other.inputs.get(*i))
            .collect())
    }
}
impl ScenarioSuite {
    pub fn freeze(
        name: impl Into<String>,
        mut cases: Vec<ScenarioCase>,
    ) -> Result<Self, ScenarioError> {
        let name = name.into();
        validate_name(&name)?;
        if cases.is_empty() {
            return Err(ScenarioError::Empty);
        }
        let mut ids = BTreeSet::new();
        for c in &mut cases {
            validate_case(c)?;
            if !ids.insert(c.id.clone()) {
                return Err(ScenarioError::Duplicate(c.id.clone()));
            }
            c.required_evidence.sort();
        }
        cases.sort_by(|a, b| a.id.cmp(&b.id));
        let parts: Vec<_> = cases.iter().map(canonical_case).collect();
        let digest = hash_text(&name, &parts);
        Ok(Self {
            name,
            cases,
            digest,
        })
    }
    pub fn verify(&self) -> Result<(), ScenarioError> {
        let rebuilt = Self::freeze(self.name.clone(), self.cases.clone())?;
        if rebuilt.digest != self.digest {
            return Err(ScenarioError::Digest);
        }
        Ok(())
    }
    pub fn select_tag(&self, tag: &str) -> Vec<&ScenarioCase> {
        self.cases.iter().filter(|c| c.tags.contains(tag)).collect()
    }
    pub fn evaluate(&self, results: &[CaseResult]) -> Result<SuiteEvaluation, ScenarioError> {
        self.verify()?;
        let expected: BTreeMap<_, _> = self.cases.iter().map(|c| (c.id.as_str(), c)).collect();
        let mut actual = BTreeMap::new();
        for r in results {
            if !expected.contains_key(r.id.as_str()) {
                return Err(ScenarioError::UnknownResult(r.id.clone()));
            }
            if actual.insert(r.id.as_str(), r).is_some() {
                return Err(ScenarioError::UnknownResult(r.id.clone()));
            }
        }
        let mut passed = 0;
        let mut failed = Vec::new();
        let mut missing = BTreeMap::new();
        let mut duration = 0u64;
        for case in &self.cases {
            let result = actual
                .get(case.id.as_str())
                .ok_or_else(|| ScenarioError::MissingResult(case.id.clone()))?;
            duration = duration
                .checked_add(result.duration_ms)
                .ok_or(ScenarioError::DurationOverflow)?;
            let absent: Vec<_> = case
                .required_evidence
                .iter()
                .filter(|e| !result.evidence.contains(e))
                .cloned()
                .collect();
            if !absent.is_empty() {
                missing.insert(case.id.clone(), absent);
            }
            if result.actual == case.expected && !missing.contains_key(&case.id) {
                passed += 1
            } else {
                failed.push(case.id.clone());
            }
        }
        Ok(SuiteEvaluation {
            passed,
            failed,
            missing_evidence: missing,
            total_duration_ms: duration,
        })
    }
}
fn validate_name(s: &str) -> Result<(), ScenarioError> {
    if s.is_empty()
        || s.len() > 96
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        Err(ScenarioError::Name)
    } else {
        Ok(())
    }
}
fn validate_case(c: &ScenarioCase) -> Result<(), ScenarioError> {
    validate_name(&c.id)?;
    if c.input.is_empty() || c.input.len() > 4096 {
        return Err(ScenarioError::Empty);
    }
    if c.required_evidence.iter().any(|e| {
        e.len() != 64
            || !e
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    }) {
        return Err(ScenarioError::Evidence);
    }
    Ok(())
}
fn canonical_case(c: &ScenarioCase) -> String {
    format!(
        "{}\0{}\0{:?}\0{}\0{}",
        c.id,
        c.input,
        c.expected,
        c.required_evidence.join(","),
        c.tags.iter().cloned().collect::<Vec<_>>().join(",")
    )
}
fn hash_text(name: &str, parts: &[String]) -> String {
    let bytes: Vec<_> = parts.iter().map(|v| v.as_bytes().to_vec()).collect();
    hash_bytes(name, &bytes)
}
fn hash_bytes(name: &str, parts: &[Vec<u8>]) -> String {
    let mut h = Sha256::new();
    h.update((name.len() as u64).to_be_bytes());
    h.update(name);
    for p in parts {
        h.update((p.len() as u64).to_be_bytes());
        h.update(p);
    }
    format!("{:x}", h.finalize())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn case(id: &str, outcome: ExpectedOutcome) -> ScenarioCase {
        ScenarioCase {
            id: id.into(),
            input: format!("input-{id}"),
            expected: outcome,
            required_evidence: vec!["a".repeat(64)],
            tags: BTreeSet::from(["core".into()]),
        }
    }
    #[test]
    fn pack_detects_mutation() {
        let mut p = ScenarioPack::freeze("base".into(), vec![b"a".to_vec()]).unwrap();
        p.inputs[0] = b"b".to_vec();
        assert_eq!(p.verify(), Err(ScenarioError::Digest));
    }
    #[test]
    fn length_prefix_prevents_ambiguous_digest() {
        let a = ScenarioPack::freeze("x".into(), vec![b"ab".to_vec(), b"c".to_vec()]).unwrap();
        let b = ScenarioPack::freeze("x".into(), vec![b"a".to_vec(), b"bc".to_vec()]).unwrap();
        assert_ne!(a.digest, b.digest);
    }
    #[test]
    fn suite_evaluates_outcomes_and_evidence() {
        let s =
            ScenarioSuite::freeze("suite", vec![case("allow", ExpectedOutcome::Allow)]).unwrap();
        let e = s
            .evaluate(&[CaseResult {
                id: "allow".into(),
                actual: ExpectedOutcome::Allow,
                evidence: vec!["a".repeat(64)],
                duration_ms: 3,
            }])
            .unwrap();
        assert_eq!(e.passed, 1);
    }
    #[test]
    fn missing_evidence_fails_case() {
        let s = ScenarioSuite::freeze("suite", vec![case("deny", ExpectedOutcome::Deny)]).unwrap();
        let e = s
            .evaluate(&[CaseResult {
                id: "deny".into(),
                actual: ExpectedOutcome::Deny,
                evidence: vec![],
                duration_ms: 1,
            }])
            .unwrap();
        assert_eq!(e.failed, vec!["deny"]);
    }
    #[test]
    fn duplicate_ids_are_rejected() {
        assert!(matches!(
            ScenarioSuite::freeze(
                "suite",
                vec![
                    case("a", ExpectedOutcome::Allow),
                    case("a", ExpectedOutcome::Deny)
                ]
            ),
            Err(ScenarioError::Duplicate(_))
        ));
    }
    #[test]
    fn unknown_result_is_rejected() {
        let s = ScenarioSuite::freeze("suite", vec![case("a", ExpectedOutcome::Allow)]).unwrap();
        assert!(matches!(
            s.evaluate(&[CaseResult {
                id: "b".into(),
                actual: ExpectedOutcome::Allow,
                evidence: vec![],
                duration_ms: 1
            }]),
            Err(ScenarioError::UnknownResult(_))
        ));
    }
    #[test]
    fn tag_selection_is_domain_specific() {
        let s = ScenarioSuite::freeze("suite", vec![case("a", ExpectedOutcome::Allow)]).unwrap();
        assert_eq!(s.select_tag("core").len(), 1);
    }
    #[test]
    fn mutation_index_tracks_changed_inputs() {
        let a = ScenarioPack::freeze("x".into(), vec![b"a".to_vec(), b"b".to_vec()]).unwrap();
        let b = ScenarioPack::freeze(
            "x".into(),
            vec![b"a".to_vec(), b"c".to_vec(), b"d".to_vec()],
        )
        .unwrap();
        assert_eq!(a.mutation_index(&b).unwrap(), vec![1, 2]);
    }
}
