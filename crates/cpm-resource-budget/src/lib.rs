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
use std::collections::BTreeMap;
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub fuel: u64,
    pub memory_bytes: u64,
    pub output_bytes: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Usage {
    pub fuel: u64,
    pub memory_bytes: u64,
    pub output_bytes: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reservation {
    pub id: String,
    pub usage: Usage,
    pub committed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetLedger {
    capacity: Budget,
    available: Budget,
    reservations: BTreeMap<String, Reservation>,
    consumed: Usage,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BudgetError {
    #[error("fuel budget exceeded")]
    Fuel,
    #[error("memory budget exceeded")]
    Memory,
    #[error("output budget exceeded")]
    Output,
    #[error("reservation id is invalid")]
    InvalidId,
    #[error("reservation {0} already exists")]
    Duplicate(String),
    #[error("reservation {0} does not exist")]
    Missing(String),
    #[error("reservation {0} is already committed")]
    Committed(String),
    #[error("budget arithmetic overflow")]
    Overflow,
}
impl Budget {
    pub fn consume(
        &mut self,
        fuel: u64,
        memory_bytes: u64,
        output_bytes: u64,
    ) -> Result<(), BudgetError> {
        let usage = Usage {
            fuel,
            memory_bytes,
            output_bytes,
        };
        ensure(*self, usage)?;
        self.fuel -= fuel;
        self.memory_bytes -= memory_bytes;
        self.output_bytes -= output_bytes;
        Ok(())
    }
    pub fn can_fit(&self, u: Usage) -> bool {
        ensure(*self, u).is_ok()
    }
    pub fn remaining_ratio(&self, initial: Budget) -> [f64; 3] {
        [
            ratio(self.fuel, initial.fuel),
            ratio(self.memory_bytes, initial.memory_bytes),
            ratio(self.output_bytes, initial.output_bytes),
        ]
    }
    pub fn saturating_usage(&self, remaining: Budget) -> Usage {
        Usage {
            fuel: self.fuel.saturating_sub(remaining.fuel),
            memory_bytes: self.memory_bytes.saturating_sub(remaining.memory_bytes),
            output_bytes: self.output_bytes.saturating_sub(remaining.output_bytes),
        }
    }
}
impl BudgetLedger {
    pub fn new(capacity: Budget) -> Self {
        Self {
            capacity,
            available: capacity,
            reservations: BTreeMap::new(),
            consumed: Usage::default(),
        }
    }
    pub fn capacity(&self) -> Budget {
        self.capacity
    }
    pub fn available(&self) -> Budget {
        self.available
    }
    pub fn consumed(&self) -> Usage {
        self.consumed
    }
    pub fn reserve(&mut self, id: impl Into<String>, usage: Usage) -> Result<(), BudgetError> {
        let id = id.into();
        if id.is_empty() || id.len() > 96 {
            return Err(BudgetError::InvalidId);
        }
        if self.reservations.contains_key(&id) {
            return Err(BudgetError::Duplicate(id));
        }
        ensure(self.available, usage)?;
        self.available
            .consume(usage.fuel, usage.memory_bytes, usage.output_bytes)?;
        self.reservations.insert(
            id.clone(),
            Reservation {
                id,
                usage,
                committed: false,
            },
        );
        Ok(())
    }
    pub fn commit(&mut self, id: &str) -> Result<(), BudgetError> {
        let r = self
            .reservations
            .get_mut(id)
            .ok_or_else(|| BudgetError::Missing(id.into()))?;
        if r.committed {
            return Err(BudgetError::Committed(id.into()));
        }
        r.committed = true;
        self.consumed.fuel = self
            .consumed
            .fuel
            .checked_add(r.usage.fuel)
            .ok_or(BudgetError::Overflow)?;
        self.consumed.memory_bytes = self
            .consumed
            .memory_bytes
            .checked_add(r.usage.memory_bytes)
            .ok_or(BudgetError::Overflow)?;
        self.consumed.output_bytes = self
            .consumed
            .output_bytes
            .checked_add(r.usage.output_bytes)
            .ok_or(BudgetError::Overflow)?;
        Ok(())
    }
    pub fn release(&mut self, id: &str) -> Result<Usage, BudgetError> {
        let r = self
            .reservations
            .get(id)
            .ok_or_else(|| BudgetError::Missing(id.into()))?;
        if r.committed {
            return Err(BudgetError::Committed(id.into()));
        }
        let usage = r.usage;
        self.available.fuel = self
            .available
            .fuel
            .checked_add(usage.fuel)
            .ok_or(BudgetError::Overflow)?;
        self.available.memory_bytes = self
            .available
            .memory_bytes
            .checked_add(usage.memory_bytes)
            .ok_or(BudgetError::Overflow)?;
        self.available.output_bytes = self
            .available
            .output_bytes
            .checked_add(usage.output_bytes)
            .ok_or(BudgetError::Overflow)?;
        self.reservations.remove(id);
        Ok(usage)
    }
    pub fn reservation(&self, id: &str) -> Option<&Reservation> {
        self.reservations.get(id)
    }
    pub fn active(&self) -> impl Iterator<Item = &Reservation> {
        self.reservations.values().filter(|r| !r.committed)
    }
    pub fn invariant_holds(&self) -> bool {
        let active = self.active().fold(Usage::default(), |mut a, r| {
            a.fuel = a.fuel.saturating_add(r.usage.fuel);
            a.memory_bytes = a.memory_bytes.saturating_add(r.usage.memory_bytes);
            a.output_bytes = a.output_bytes.saturating_add(r.usage.output_bytes);
            a
        });
        self.available
            .fuel
            .saturating_add(active.fuel)
            .saturating_add(self.consumed.fuel)
            == self.capacity.fuel
            && self
                .available
                .memory_bytes
                .saturating_add(active.memory_bytes)
                .saturating_add(self.consumed.memory_bytes)
                == self.capacity.memory_bytes
            && self
                .available
                .output_bytes
                .saturating_add(active.output_bytes)
                .saturating_add(self.consumed.output_bytes)
                == self.capacity.output_bytes
    }
}
fn ensure(b: Budget, u: Usage) -> Result<(), BudgetError> {
    if u.fuel > b.fuel {
        return Err(BudgetError::Fuel);
    }
    if u.memory_bytes > b.memory_bytes {
        return Err(BudgetError::Memory);
    }
    if u.output_bytes > b.output_bytes {
        return Err(BudgetError::Output);
    }
    Ok(())
}
fn ratio(v: u64, total: u64) -> f64 {
    if total == 0 {
        if v == 0 { 1.0 } else { 0.0 }
    } else {
        v as f64 / total as f64
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn budget() -> Budget {
        Budget {
            fuel: 100,
            memory_bytes: 200,
            output_bytes: 50,
        }
    }
    fn usage() -> Usage {
        Usage {
            fuel: 20,
            memory_bytes: 30,
            output_bytes: 10,
        }
    }
    #[test]
    fn consume_is_atomic_on_failure() {
        let mut b = budget();
        assert_eq!(b.consume(20, 300, 1), Err(BudgetError::Memory));
        assert_eq!(b, budget());
    }
    #[test]
    fn reserve_and_commit_account_exactly() {
        let mut l = BudgetLedger::new(budget());
        l.reserve("trial", usage()).unwrap();
        assert_eq!(l.available().fuel, 80);
        l.commit("trial").unwrap();
        assert_eq!(l.consumed().fuel, 20);
        assert!(l.invariant_holds());
    }
    #[test]
    fn release_restores_capacity() {
        let mut l = BudgetLedger::new(budget());
        l.reserve("trial", usage()).unwrap();
        assert_eq!(l.release("trial").unwrap(), usage());
        assert_eq!(l.available(), budget());
        assert!(l.invariant_holds());
    }
    #[test]
    fn duplicate_reservation_is_rejected_without_second_charge() {
        let mut l = BudgetLedger::new(budget());
        l.reserve("trial", usage()).unwrap();
        assert!(matches!(
            l.reserve("trial", usage()),
            Err(BudgetError::Duplicate(_))
        ));
        assert_eq!(l.available().fuel, 80);
    }
    #[test]
    fn committed_reservation_cannot_be_released() {
        let mut l = BudgetLedger::new(budget());
        l.reserve("trial", usage()).unwrap();
        l.commit("trial").unwrap();
        assert!(matches!(l.release("trial"), Err(BudgetError::Committed(_))));
    }
    #[test]
    fn reservation_over_capacity_is_atomic() {
        let mut l = BudgetLedger::new(budget());
        let large = Usage {
            fuel: 101,
            ..Usage::default()
        };
        assert_eq!(l.reserve("x", large), Err(BudgetError::Fuel));
        assert_eq!(l.available(), budget());
    }
    #[test]
    fn active_excludes_committed() {
        let mut l = BudgetLedger::new(budget());
        l.reserve("a", usage()).unwrap();
        l.reserve("b", usage()).unwrap();
        l.commit("a").unwrap();
        assert_eq!(l.active().count(), 1);
    }
    #[test]
    fn ratios_report_remaining_capacity() {
        let mut b = budget();
        b.consume(50, 100, 25).unwrap();
        assert_eq!(b.remaining_ratio(budget()), [0.5, 0.5, 0.5]);
    }
}
