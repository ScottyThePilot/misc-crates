//! This crate provides opaque ID generators and types.
#[cfg(feature = "map")]
pub mod map;

#[cfg(feature = "map")]
pub use crate::map::IdMap;

use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::cmp::Ordering;
use std::fmt;

const ORDERING: AtomicOrdering = AtomicOrdering::SeqCst;



/// A context for spawning unique IDs.
/// The type parameter is to allow you to specify a "family" so that
/// IDs can be impossible to mix up, as they may cause issues if you
/// "cross the beams" so to speak. This is optional though, and it
/// defaults to the unit type, `()` as the default family.
#[derive(Debug)]
pub struct IdContext<F = ()> {
  current_id: u64,
  _family: P<F>
}

impl<F> IdContext<F> {
  #[cfg(feature = "serde")]
  const fn with_current_id(current_id: u64) -> IdContext<F> {
    IdContext { current_id, _family: P::new() }
  }

  /// Creates a new ID context.
  pub const fn new() -> IdContext<F> {
    IdContext {
      current_id: 0,
      _family: P::new()
    }
  }

  /// Spawns the next unique ID for this context.
  pub fn next_id(&mut self) -> Id<F> {
    let id = self.current_id;
    self.current_id += 1;
    Id { id, _family: P::new() }
  }

  fn duplicate(&self) -> IdContext<F> {
    IdContext {
      current_id: self.current_id,
      _family: P::new()
    }
  }
}

impl<F> Default for IdContext<F> {
  #[inline]
  fn default() -> IdContext<F> {
    IdContext::new()
  }
}



pub struct Id<F = ()> {
  id: u64,
  _family: P<F>
}

impl<F> fmt::Debug for Id<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("Id")
      .field(&self.id)
      .finish()
  }
}

impl<F> Clone for Id<F> {
  #[inline]
  fn clone(&self) -> Self {
    Id { id: self.id.clone(), _family: P::new() }
  }
}

impl<F> Copy for Id<F> {}

impl<F> PartialEq for Id<F> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<F> Eq for Id<F> {}

impl<F> PartialOrd for Id<F> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    PartialOrd::partial_cmp(&self.id, &other.id)
  }
}

impl<F> Ord for Id<F> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    Ord::cmp(&self.id, &other.id)
  }
}

impl<F> Hash for Id<F> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.id);
  }
}

#[cfg(feature = "serde")]
impl<F> serde::Serialize for Id<F> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    serializer.serialize_newtype_struct("Id", &self.id)
  }
}

#[cfg(feature = "serde")]
impl<'de, F> serde::Deserialize<'de> for Id<F> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    struct IdVisitor<F>(PhantomData<F>);

    impl<'de, F> serde::de::Visitor<'de> for IdVisitor<F> {
      type Value = Id<F>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("tuple struct Id or u64")
      }

      #[inline]
      fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
      where D: serde::Deserializer<'de>, {
        Ok(Id {
          id: <u64 as serde::Deserialize>::deserialize(deserializer)?,
          _family: P::new()
        })
      }

      #[inline]
      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where A: serde::de::SeqAccess<'de>, {
        match seq.next_element::<u64>()? {
          Some(id) => Ok(Id { id, _family: P::new() }),
          None => Err(serde::de::Error::invalid_length(0, &"tuple struct Id with 1 element"))
        }
      }

      #[inline]
      fn visit_u64<E>(self, id: u64) -> Result<Self::Value, E>
      where E: serde::de::Error {
        Ok(Id { id, _family: P::new() })
      }
    }

    deserializer.deserialize_newtype_struct("Id", IdVisitor(PhantomData))
  }
}



/// An atomic ID context.
/// This is just like `IdContext`, but operates atomically
/// and can be shared between threads.
#[derive(Debug)]
pub struct AtomicIdContext<F = ()> {
  current_id: AtomicU64,
  _family: P<F>
}

impl<F> AtomicIdContext<F> {
  pub const fn new() -> AtomicIdContext<F> {
    AtomicIdContext {
      current_id: AtomicU64::new(0),
      _family: P::new()
    }
  }

  pub fn next_id(&self) -> Id<F> {
    let id = self.current_id.fetch_add(1, ORDERING);
    Id { id, _family: P::new() }
  }
}

impl<F> Default for AtomicIdContext<F> {
  #[inline]
  fn default() -> AtomicIdContext<F> {
    AtomicIdContext::new()
  }
}



pub struct AtomicId<F = ()> {
  id: u64,
  _family: P<F>
}

impl<F> fmt::Debug for AtomicId<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("AtomicId")
      .field(&self.id)
      .finish()
  }
}

impl<F> Clone for AtomicId<F> {
  #[inline]
  fn clone(&self) -> Self {
    AtomicId { id: self.id.clone(), _family: P::new() }
  }
}

impl<F> Copy for AtomicId<F> {}

impl<F> PartialEq for AtomicId<F> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<F> Eq for AtomicId<F> {}

impl<F> PartialOrd for AtomicId<F> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    PartialOrd::partial_cmp(&self.id, &other.id)
  }
}

impl<F> Ord for AtomicId<F> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    Ord::cmp(&self.id, &other.id)
  }
}

impl<F> Hash for AtomicId<F> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.id);
  }
}



// This struct is necessary because `PhantomData` has stricter bounds
// on its `Send` and `Sync` implementations than are necessary.
#[repr(transparent)]
struct P<T>(PhantomData<T>);

impl<T> P<T> {
  #[inline]
  const fn new() -> Self {
    P(PhantomData)
  }
}

impl<F> fmt::Debug for P<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("Ph").finish()
  }
}

impl<T> Clone for P<T> {
  #[inline]
  fn clone(&self) -> Self {
    P(PhantomData)
  }
}

impl<T> Copy for P<T> {}

impl<T> Default for P<T> {
  #[inline]
  fn default() -> Self {
    P(PhantomData)
  }
}

// SAFETY: `P` is a zero-sized type
unsafe impl<T> Send for P<T> {}
unsafe impl<T> Sync for P<T> {}
