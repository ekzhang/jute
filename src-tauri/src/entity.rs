//! Types for generating and representing unique, labelled IDs.

use std::{array, fmt, str::FromStr};

use anyhow::bail;
use rand::Rng;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{EnumIter, IntoEnumIterator};

/// Entity category for generated IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Entity {
    /// Python virtual environments created by Jute.
    Venv,
}

impl Entity {
    /// Get the prefix for IDs generated for this entity.
    pub const fn id_prefix(&self) -> &'static str {
        match self {
            Entity::Venv => "ve-",
        }
    }
}

/// An entity ID generated for a specific category of object.
#[derive(Clone, Copy, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct EntityId {
    /// Which kind of entity this ID represents.
    pub kind: Entity,

    /// The unique ID for this entity.
    id: [u8; Self::ID_LENGTH],
}

impl EntityId {
    /// The length of the random portion of the ID in characters.
    pub const ID_LENGTH: usize = 12;

    /// Get the ID as a string.
    pub fn new(kind: Entity) -> Self {
        // Sample 12 characters, each one of 36 possibilities (lowercase letters and
        // numbers).
        let charset = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::thread_rng();
        let id = array::from_fn(|_| charset[rng.gen_range(0..charset.len())]);
        EntityId { kind, id }
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.kind.id_prefix(),
            String::from_utf8_lossy(&self.id)
        )
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EntityId")
            .field(&format_args!(
                "{}{}",
                self.kind.id_prefix(),
                String::from_utf8_lossy(&self.id)
            ))
            .finish()
    }
}

impl FromStr for EntityId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for kind in Entity::iter() {
            if let Some(maybe_id) = s.strip_prefix(kind.id_prefix()) {
                let id = maybe_id.as_bytes();
                if let Ok(id) = id.try_into() {
                    return Ok(EntityId { kind, id });
                } else {
                    bail!("invalid entity ID length {s}")
                }
            }
        }
        bail!("invalid entity prefix {s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id() {
        let id = EntityId::new(Entity::Venv);
        assert_eq!(id.to_string().len(), 12 + 3);
        assert_eq!(id.to_string().chars().count(), 12 + 3);
        assert!(id.to_string().starts_with("ve-"));

        assert_eq!(id, id);
        assert_ne!(id, EntityId::new(Entity::Venv));
    }

    #[test]
    fn test_entity_id_parse() {
        let id = EntityId::new(Entity::Venv);
        let parsed = id.to_string().parse::<EntityId>().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_entity_id_validation() {
        let parsed = "ve-1234567890".parse::<EntityId>();
        assert!(parsed.is_err());

        let parsed = "ve-1234567890ab".parse::<EntityId>();
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.kind, Entity::Venv);
        assert_eq!(parsed.to_string(), "ve-1234567890ab");
    }
}
