//! Fuel, memory, output, and elapsed execution budgets.
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub fuel: u64,
    pub memory_bytes: u64,
    pub output_bytes: u64,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BudgetError {
    #[error("fuel exhausted")]
    Fuel,
    #[error("memory exhausted")]
    Memory,
    #[error("output exhausted")]
    Output,
}
impl Budget {
    pub fn consume(&mut self, fuel: u64, memory: u64, output: u64) -> Result<(), BudgetError> {
        if fuel > self.fuel {
            return Err(BudgetError::Fuel);
        }
        if memory > self.memory_bytes {
            return Err(BudgetError::Memory);
        }
        if output > self.output_bytes {
            return Err(BudgetError::Output);
        }
        self.fuel -= fuel;
        self.memory_bytes -= memory;
        self.output_bytes -= output;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accounts() {
        let mut b = Budget {
            fuel: 10,
            memory_bytes: 20,
            output_bytes: 5,
        };
        b.consume(3, 4, 1).unwrap();
        assert_eq!(b.fuel, 7)
    }
    #[test]
    fn traps() {
        assert_eq!(
            Budget {
                fuel: 1,
                memory_bytes: 1,
                output_bytes: 1
            }
            .consume(2, 0, 0),
            Err(BudgetError::Fuel)
        )
    }
}
