//! Stable value objects shared by Capability Provenance Mesh contexts.

use std::{fmt, str::FromStr};

/// Error returned when a boundary value violates the published contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// A content digest was not exactly 64 lowercase hexadecimal characters.
    InvalidDigest,
    /// A capability name was empty, too long, or contained unsupported characters.
    InvalidCapability,
    /// A monotonically increasing revision overflowed.
    RevisionOverflow,
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDigest => "digest must be 64 lowercase hexadecimal characters",
            Self::InvalidCapability => {
                "capability must be 1..=96 lowercase ASCII letters, digits, '.', '-', or ':'"
            }
            Self::RevisionOverflow => "revision cannot advance beyond u64::MAX",
        })
    }
}

impl std::error::Error for ContractError {}

/// SHA-256 content identity represented as validated bytes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentDigest([u8; 32]);

impl ContentDigest {
    /// Returns the validated digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl fmt::Debug for ContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for ContentDigest {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ContractError::InvalidDigest);
        }
        if value.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(ContractError::InvalidDigest);
        }

        let mut digest = [0_u8; 32];
        let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
        debug_assert_eq!(remainder, []);
        for (index, pair) in pairs.iter().enumerate() {
            digest[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
        }
        Ok(Self(digest))
    }
}

fn nibble(byte: u8) -> Result<u8, ContractError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ContractError::InvalidDigest),
    }
}

/// Canonical capability vocabulary entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityName(String);

impl CapabilityName {
    /// Validates and constructs a capability name.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError::InvalidCapability`] when the value is empty,
    /// longer than 96 bytes, or outside the canonical character set.
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 96
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b':')
            });
        valid
            .then_some(Self(value))
            .ok_or(ContractError::InvalidCapability)
    }

    /// Returns the canonical name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Optimistic aggregate revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Revision(u64);

impl Revision {
    /// Creates a revision from its persisted integer.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the persisted integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Advances the revision without wrapping.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError::RevisionOverflow`] at [`u64::MAX`].
    pub fn next(self) -> Result<Self, ContractError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(ContractError::RevisionOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::{CapabilityName, ContentDigest, ContractError, Revision};
    use std::str::FromStr;

    #[test]
    fn digest_round_trips_canonical_hex() {
        let text = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let digest = ContentDigest::from_str(text).expect("valid digest");
        assert_eq!(digest.to_string(), text);
    }

    #[test]
    fn digest_rejects_uppercase_and_wrong_length() {
        assert_eq!(
            ContentDigest::from_str(&"A".repeat(64)),
            Err(ContractError::InvalidDigest)
        );
        assert_eq!(
            ContentDigest::from_str("00"),
            Err(ContractError::InvalidDigest)
        );
    }

    #[test]
    fn capability_accepts_namespaced_value_and_rejects_ambient_text() {
        let capability = CapabilityName::parse("blob.read:tenant-7").expect("valid name");
        assert_eq!(capability.as_str(), "blob.read:tenant-7");
        assert_eq!(
            CapabilityName::parse("Network Access"),
            Err(ContractError::InvalidCapability)
        );
    }

    #[test]
    fn revision_never_wraps() {
        assert_eq!(Revision::new(7).next().expect("next").get(), 8);
        assert_eq!(
            Revision::new(u64::MAX).next(),
            Err(ContractError::RevisionOverflow)
        );
    }
}
