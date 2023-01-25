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
pub struct IdContext<F: ?Sized = ()> {
  current_id: u64,
  _family: P<F>
}

impl<F: ?Sized> IdContext<F> {
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
    Id::from_raw(id)
  }

  fn duplicate(&self) -> IdContext<F> {
    IdContext {
      current_id: self.current_id,
      _family: P::new()
    }
  }
}

impl<F: ?Sized> Default for IdContext<F> {
  #[inline]
  fn default() -> IdContext<F> {
    IdContext::new()
  }
}



pub struct Id<F: ?Sized = ()> {
  id: u64,
  _family: P<F>
}

impl<F: ?Sized> Id<F> {
  #[inline]
  pub const fn from_raw(id: u64) -> Self {
    Id { id, _family: P::new() }
  }

  #[inline]
  pub const fn into_raw(self) -> u64 {
    self.id
  }

  #[inline]
  pub const fn cast<U>(self) -> Id<U> {
    Id::from_raw(self.id)
  }
}

impl<F: ?Sized> fmt::Debug for Id<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("Id")
      .field(&self.id)
      .finish()
  }
}

impl<F: ?Sized> Clone for Id<F> {
  #[inline]
  fn clone(&self) -> Self {
    Id::from_raw(self.id)
  }
}

impl<F: ?Sized> Copy for Id<F> {}

impl<F: ?Sized> PartialEq for Id<F> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl<F: ?Sized> Eq for Id<F> {}

impl<F: ?Sized> PartialOrd for Id<F> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    PartialOrd::partial_cmp(&self.id, &other.id)
  }
}

impl<F: ?Sized> Ord for Id<F> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    Ord::cmp(&self.id, &other.id)
  }
}

impl<F: ?Sized> Hash for Id<F> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.id);
  }
}

#[cfg(feature = "serde")]
impl<F: ?Sized> serde::Serialize for Id<F> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.id.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, F: ?Sized> serde::Deserialize<'de> for Id<F> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    u64::deserialize(deserializer).map(Id::from_raw)
  }
}



/// An atomic ID context.
/// This is just like `IdContext`, but operates atomically
/// and can be shared between threads.
#[derive(Debug)]
pub struct AtomicIdContext<F: ?Sized = ()> {
  current_id: AtomicU64,
  _family: P<F>
}

impl<F: ?Sized> AtomicIdContext<F> {
  pub const fn new() -> AtomicIdContext<F> {
    AtomicIdContext {
      current_id: AtomicU64::new(0),
      _family: P::new()
    }
  }

  pub fn next_id(&self) -> Id<F> {
    let id = self.current_id.fetch_add(1, ORDERING);
    Id::from_raw(id)
  }
}

impl<F: ?Sized> Default for AtomicIdContext<F> {
  #[inline]
  fn default() -> AtomicIdContext<F> {
    AtomicIdContext::new()
  }
}



// This struct is necessary because `PhantomData` has stricter bounds
// on its `Send` and `Sync` implementations than are necessary.
#[repr(transparent)]
struct P<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> P<T> {
  #[inline]
  const fn new() -> Self {
    P(PhantomData)
  }
}

impl<F: ?Sized> fmt::Debug for P<F> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("Ph").finish()
  }
}

impl<T: ?Sized> Clone for P<T> {
  #[inline]
  fn clone(&self) -> Self {
    P(PhantomData)
  }
}

impl<T: ?Sized> Copy for P<T> {}

impl<T: ?Sized> Default for P<T> {
  #[inline]
  fn default() -> Self {
    P(PhantomData)
  }
}

// SAFETY: `P` is a zero-sized type
unsafe impl<T: ?Sized> Send for P<T> {}
unsafe impl<T: ?Sized> Sync for P<T> {}
