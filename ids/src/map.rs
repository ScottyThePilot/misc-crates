use std::collections::hash_map;
use std::iter::FusedIterator;
use std::ops::Index;
use std::fmt;

use nohash_hasher::{IntMap as NoHashMap, BuildNoHashHasher, IsEnabled};

use super::{IdContext, Id};



pub type IdMapBuildHasher<T> = BuildNoHashHasher<Id<T>>;

impl<T> IsEnabled for Id<T> {}

/// A hashmap with `Id<T>`s as opaque keys.
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
  pub fn iter(&self) -> Iter<'_, T> {
    Iter { inner: self.map.iter() }
  }

  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    IterMut { inner: self.map.iter_mut() }
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

  pub fn insert_new(&mut self, value: T) -> Id<T> {
    let id = self.context.next_id();
    let result = self.map.insert(id, value);
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

impl<T: Clone> Clone for IdMap<T> {
  fn clone(&self) -> Self {
    IdMap {
      context: self.context.duplicate(),
      map: self.map.clone()
    }
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
        let size = serde::__private::size_hint::cautious(map.size_hint());
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

    deserializer.deserialize_newtype_struct("Id", IdMapVisitor(std::marker::PhantomData))
  }
}



macro_rules! impl_iterator_struct {
  ($Iter:ident, $Item:ty) => {
    impl_iterator_struct!(@ $Iter, $Item, s -> s.inner.next());
  };
  ($Iter:ident, $Item:ty, $map:expr) => {
    impl_iterator_struct!(@ $Iter, $Item, s -> s.inner.next().map($map));
  };
  (@ $Iter:ident, $Item:ty, $self:ident -> $next:expr) => {
    impl<T: fmt::Debug> fmt::Debug for $Iter<'_, T> {
      #[inline]
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
      }
    }

    impl<'a, T> Iterator for $Iter<'a, T> {
      type Item = $Item;

      #[inline]
      fn next(&mut self) -> Option<$Item> {
        let $self = self;
        $next
      }

      #[inline]
      fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
      }
    }

    impl<T> ExactSizeIterator for $Iter<'_, T> {
      #[inline]
      fn len(&self) -> usize {
        self.inner.len()
      }
    }

    impl<T> FusedIterator for $Iter<'_, T> {}
  };
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Ids<'a, T: 'a> {
  inner: hash_map::Keys<'a, Id<T>, T>
}

impl_iterator_struct!(Ids, Id<T>, |id| *id);

#[repr(transparent)]
#[derive(Clone)]
pub struct Values<'a, T: 'a> {
  inner: hash_map::Values<'a, Id<T>, T>
}

impl_iterator_struct!(Values, &'a T);

#[repr(transparent)]
pub struct ValuesMut<'a, T: 'a> {
  inner: hash_map::ValuesMut<'a, Id<T>, T>
}

impl_iterator_struct!(ValuesMut, &'a mut T);

#[repr(transparent)]
#[derive(Clone)]
pub struct Iter<'a, T: 'a> {
  inner: hash_map::Iter<'a, Id<T>, T>
}

impl_iterator_struct!(Iter, (Id<T>, &'a T), |(id, v)| (*id, v));

#[repr(transparent)]
pub struct IterMut<'a, T: 'a> {
  inner: hash_map::IterMut<'a, Id<T>, T>
}

impl_iterator_struct!(IterMut, (Id<T>, &'a mut T), |(id, v)| (*id, v));

#[repr(transparent)]
pub struct Drain<'a, T: 'a> {
  inner: hash_map::Drain<'a, Id<T>, T>
}

impl_iterator_struct!(Drain, T, |(_, v)| v);
