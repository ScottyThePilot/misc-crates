use std::collections::hash_map;
use std::ops::Index;
use std::fmt;

use nohash_hasher::{IntMap as NoHashMap, BuildNoHashHasher, IsEnabled};

use super::{IdContext, Id};



pub type IdMapBuildHasher<T> = BuildNoHashHasher<Id<T>>;

impl<F: ?Sized> IsEnabled for Id<F> {}

/// A hashmap with [`Id`]s as opaque keys.
///
/// If you would like an [`IdContext`] to be paired with this, use [`IdMap`] instead.
pub type IdHashMap<T> = NoHashMap<Id<T>, T>;

/// A hashmap with [`Id`]s as opaque keys alongside an [`IdContext`] for producing them.
#[derive(Clone)]
pub struct IdMap<T> {
  context: IdContext<T>,
  map: NoHashMap<Id<T>, T>
}

impl<T> IdMap<T> {
  pub fn new() -> Self {
    IdMap::from_map_raw(NoHashMap::with_hasher(Default::default()))
  }

  pub fn with_capacity(capacity: usize) -> Self {
    IdMap::from_map_raw(NoHashMap::with_capacity_and_hasher(capacity, Default::default()))
  }

  #[inline]
  fn from_map_raw(map: NoHashMap<Id<T>, T>) -> Self {
    IdMap { context: IdContext::new(), map }
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    self.map.capacity()
  }

  #[inline]
  pub fn ids(&self) -> Ids<'_, T> {
    Ids { inner: self.map.keys() }
  }

  #[inline]
  pub fn values(&self) -> Values<'_, T> {
    Values { inner: self.map.values() }
  }

  #[inline]
  pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
    ValuesMut { inner: self.map.values_mut() }
  }

  #[inline]
  pub fn into_values(self) -> IntoValues<T> {
    IntoValues { inner: self.map.into_values() }
  }

  #[inline]
  pub fn iter(&self) -> Iter<'_, T> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    self.into_iter()
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.map.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  #[inline]
  pub fn drain(&mut self) -> Drain<'_, T> {
    self.context.current_id = 0;
    Drain { inner: self.map.drain() }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.context.current_id = 0;
    self.map.clear();
  }

  #[inline]
  pub fn hasher(&self) -> &IdMapBuildHasher<T> {
    self.map.hasher()
  }

  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    self.map.reserve(additional);
  }

  #[inline]
  pub fn shrink_to_fit(&mut self) {
    self.map.shrink_to_fit()
  }

  // TODO: Entry

  #[inline]
  pub fn get(&self, id: Id<T>) -> Option<&T> {
    self.map.get(&id)
  }

  #[inline]
  pub fn contains_id(&self, id: Id<T>) -> bool {
    self.map.contains_key(&id)
  }

  #[inline]
  pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
    self.map.get_mut(&id)
  }

  #[inline]
  pub fn next_id(&mut self) -> Id<T> {
    self.context.next_id()
  }

  pub fn insert_new(&mut self, value: T) -> Id<T> {
    let id = self.next_id();
    let result = self.map.insert(id, value);
    debug_assert!(result.is_none());
    id
  }

  pub fn insert_new_with<F>(&mut self, f: F) -> Id<T>
  where F: FnOnce(Id<T>) -> T {
    let id = self.next_id();
    let result = self.map.insert(id, f(id));
    debug_assert!(result.is_none());
    id
  }

  #[inline]
  pub fn insert(&mut self, id: Id<T>, value: T) -> Option<T> {
    self.map.insert(id, value)
  }

  #[inline]
  pub fn remove(&mut self, id: Id<T>) -> Option<T> {
    self.map.remove(&id)
  }

  #[inline]
  pub fn retain<F>(&mut self, mut f: F)
  where F: FnMut(Id<T>, &mut T) -> bool {
    self.map.retain(move |&id, v| f(id, v));
  }
}

impl<T: fmt::Debug> fmt::Debug for IdMap<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_map().entries(self.iter()).finish()
  }
}

impl<T> Default for IdMap<T> {
  #[inline]
  fn default() -> Self {
    IdMap::new()
  }
}

impl<T> Index<Id<T>> for IdMap<T> {
  type Output = T;

  #[inline]
  fn index(&self, id: Id<T>) -> &Self::Output {
    self.get(id).expect("no entry found for id")
  }
}

impl<'a, T> IntoIterator for &'a IdMap<T> {
  type Item = (Id<T>, &'a T);
  type IntoIter = Iter<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    Iter { inner: self.map.iter() }
  }
}

impl<'a, T> IntoIterator for &'a mut IdMap<T> {
  type Item = (Id<T>, &'a mut T);
  type IntoIter = IterMut<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    IterMut { inner: self.map.iter_mut() }
  }
}

impl<T> IntoIterator for IdMap<T> {
  type Item = (Id<T>, T);
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    IntoIter { inner: self.map.into_iter() }
  }
}

// TODO: `IntoIterator` implementations

#[cfg(feature = "serde")]
impl<T> serde::Serialize for IdMap<T>
where T: serde::Serialize {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.map.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for IdMap<T>
where T: serde::Deserialize<'de> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    struct IdMapVisitor<T>(std::marker::PhantomData<IdMap<T>>);

    impl<'de, T> serde::de::Visitor<'de> for IdMapVisitor<T>
    where T: serde::Deserialize<'de> {
      type Value = IdMap<T>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a map")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where A: serde::de::MapAccess<'de> {
        let size = map.size_hint().unwrap_or(0).min(4096);
        let mut values = NoHashMap::with_capacity_and_hasher(size, Default::default());
        let mut current_id: u64 = 0;

        while let Some((key, value)) = map.next_entry::<Id<T>, T>()? {
          current_id = current_id.max(key.id);
          values.insert(key, value);
        };

        current_id += 1;

        Ok(IdMap {
          context: IdContext::with_current_id(current_id),
          map: values
        })
      }
    }

    deserializer.deserialize_map(IdMapVisitor(std::marker::PhantomData))
  }
}

macro_rules! impl_iterator {
  {
    $(#[$attr:meta])*
    $vis:vis struct $Type:ident <$($lt:lifetime),* $(,)? $($gn:ident),* $(,)?>,
    $inner:ident: $InnerType:ty, $Item:ty, $map:expr
    $(, where $($w:tt)*)? $(,)?
  } => {
    $(#[$attr])*
    $vis struct $Type<$($lt,)* $($gn,)*> {
      $inner: $InnerType
    }

    impl<$($lt,)* $($gn,)*> std::iter::Iterator for $Type<$($lt,)* $($gn,)*> $(where $($w)*)? {
      type Item = $Item;

      #[inline]
      fn next(&mut self) -> Option<Self::Item> {
        self.$inner.next().map($map)
      }

      #[inline]
      fn size_hint(&self) -> (usize, Option<usize>) {
        self.$inner.size_hint()
      }
    }

    impl<$($lt,)* $($gn,)*> std::iter::ExactSizeIterator for $Type<$($lt,)* $($gn,)*> $(where $($w)*)? {
      #[inline]
      fn len(&self) -> usize {
        self.$inner.len()
      }
    }

    impl<$($lt,)* $($gn,)*> std::iter::FusedIterator for $Type<$($lt,)* $($gn,)*> $(where $($w)*)? {}
  };
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct Ids<'a, T>,
  inner: hash_map::Keys<'a, Id<T>, T>,
  Id<T>, |id| *id
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct Values<'a, T>,
  inner: hash_map::Values<'a, Id<T>, T>,
  &'a T, std::convert::identity
}

impl_iterator! {
  #[derive(Debug)] pub struct ValuesMut<'a, T>,
  inner: hash_map::ValuesMut<'a, Id<T>, T>,
  &'a mut T, std::convert::identity
}

impl_iterator! {
  #[derive(Debug)] pub struct IntoValues<T>,
  inner: hash_map::IntoValues<Id<T>, T>,
  T, std::convert::identity
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct Iter<'a, T>,
  inner: hash_map::Iter<'a, Id<T>, T>,
  (Id<T>, &'a T), |(id, v)| (*id, v)
}

impl_iterator! {
  #[derive(Debug)] pub struct IterMut<'a, T>,
  inner: hash_map::IterMut<'a, Id<T>, T>,
  (Id<T>, &'a mut T), |(id, v)| (*id, v)
}

impl_iterator! {
  #[derive(Debug)] pub struct IntoIter<T>,
  inner: hash_map::IntoIter<Id<T>, T>,
  (Id<T>, T), std::convert::identity
}

impl_iterator! {
  #[derive(Debug)] pub struct Drain<'a, T>,
  inner: hash_map::Drain<'a, Id<T>, T>,
  T, |(_, v)| v
}
