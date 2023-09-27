//! This crate provides opaque ID generators and types.

pub extern crate nohash_hasher;

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
#[repr(transparent)]
#[derive(Debug)]
pub struct IdContext<F: ?Sized = ()> {
  current_id: u64,
  family: PhantomData<F>
}

impl<F: ?Sized> IdContext<F> {
  /// Creates a new ID context with an from which it should start counting from.
  #[inline]
  pub const fn with_current_id(current_id: u64) -> IdContext<F> {
    IdContext {
      current_id,
      family: PhantomData
    }
  }

  /// Creates a new ID context.
  #[inline]
  pub const fn new() -> IdContext<F> {
    Self::with_current_id(0)
  }

  /// Spawns the next unique ID for this context.
  pub fn next_id(&mut self) -> Id<F> {
    let id = self.current_id;
    self.current_id += 1;
    Id::from_raw(id)
  }
}

impl<F: ?Sized> Clone for IdContext<F> {
  #[inline]
  fn clone(&self) -> Self {
    IdContext {
      current_id: self.current_id,
      family: PhantomData
    }
  }
}

impl<F: ?Sized> Default for IdContext<F> {
  #[inline]
  fn default() -> IdContext<F> {
    IdContext::new()
  }
}

unsafe impl<F: ?Sized> Send for IdContext<F> {}
unsafe impl<F: ?Sized> Sync for IdContext<F> {}



#[repr(transparent)]
pub struct Id<F: ?Sized = ()> {
  id: u64,
  family: PhantomData<F>
}

impl<F: ?Sized> Id<F> {
  #[inline]
  pub const fn from_raw(id: u64) -> Self {
    Id { id, family: PhantomData }
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

unsafe impl<F: ?Sized> Send for Id<F> {}
unsafe impl<F: ?Sized> Sync for Id<F> {}

#[cfg(feature = "serde")]
impl<F: ?Sized> serde::Serialize for Id<F> {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.id.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, F: ?Sized> serde::Deserialize<'de> for Id<F> {
  #[inline]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    u64::deserialize(deserializer).map(Id::from_raw)
  }
}



/// A atomic context for spawning unique IDs.
/// This is just like `IdContext`, but operates atomically and can be shared between threads.
#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicIdContext<F: ?Sized = ()> {
  current_id: AtomicU64,
  family: PhantomData<F>
}

impl<F: ?Sized> AtomicIdContext<F> {
  /// Creates a new ID context with an from which it should start counting from.
  #[inline]
  pub const fn with_current_id(current_id: u64) -> AtomicIdContext<F> {
    AtomicIdContext {
      current_id: AtomicU64::new(current_id),
      family: PhantomData
    }
  }

  /// Creates a new ID context.
  #[inline]
  pub const fn new() -> AtomicIdContext<F> {
    Self::with_current_id(0)
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

unsafe impl<F: ?Sized> Send for AtomicIdContext<F> {}
unsafe impl<F: ?Sized> Sync for AtomicIdContext<F> {}
