use std::cmp::Ordering;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::fmt;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UOrd<T> {
  min: T,
  max: T
}

impl<T> UOrd<T> {
  pub fn new(a: T, b: T) -> Self where T: Ord {
    let (min, max) = match Ord::cmp(&a, &b) {
      Ordering::Less | Ordering::Equal => (a, b),
      Ordering::Greater => (b, a)
    };

    UOrd { min, max }
  }

  /// Returns the lesser of the two elements, based on `T`'s `Ord` implementation.
  #[inline(always)]
  pub fn min(&self) -> &T {
    &self.min
  }

  /// Returns the greater of the two elements, based on `T`'s `Ord` implementation.
  #[inline(always)]
  pub fn max(&self) -> &T {
    &self.max
  }

  /// Returns `true` if one of the two contained elements equals `x`.
  #[inline]
  pub fn contains<Q: ?Sized>(&self, x: &Q) -> bool
  where T: Borrow<Q>, Q: Eq {
    self.min.borrow() == x || self.max.borrow() == x
  }

  /// Returns the opposite elements when one of the contained elements matches the given value.
  #[inline]
  pub fn other<Q: ?Sized>(&self, x: &Q) -> Option<&T>
  where T: Borrow<Q>, Q: Eq {
    if self.max.borrow() == x {
      Some(&self.min)
    } else if self.min.borrow() == x {
      Some(&self.max)
    } else {
      None
    }
  }

  /// Returns true if the two elements of this pair are distinct (not equal).
  #[inline]
  pub fn is_distinct(&self) -> bool
  where T: Eq {
    self.min != self.max
  }

  /// Replaces one or both elements of this pair with a new value.
  pub fn replace<Q: ?Sized>(&self, from: &Q, to: T) -> Self
  where T: Ord + Borrow<Q> + Clone, Q: Eq {
    self.map_ref(|v| if v.borrow() == from { to.clone() } else { v.clone() })
  }

  pub fn as_ref(&self) -> UOrd<&T>
  where T: Ord {
    UOrd::new(&self.min, &self.max)
  }

  #[inline(always)]
  pub fn as_tuple(&self) -> (&T, &T) {
    (&self.min, &self.max)
  }

  #[inline(always)]
  pub fn into_tuple(self) -> (T, T) {
    (self.min, self.max)
  }

  #[inline(always)]
  pub fn as_array(&self) -> [&T; 2] {
    [&self.min, &self.max]
  }

  #[inline(always)]
  pub fn into_array(self) -> [T; 2] {
    [self.min, self.max]
  }

  pub fn map<U, F>(self, mut f: F) -> UOrd<U>
  where U: Ord, F: FnMut(T) -> U {
    UOrd::new(f(self.min), f(self.max))
  }

  pub fn map_ref<U, F>(&self, mut f: F) -> UOrd<U>
  where U: Ord, F: FnMut(&T) -> U {
    UOrd::new(f(&self.min), f(&self.max))
  }

  pub fn try_map<U, F>(self, mut f: F) -> Option<UOrd<U>>
  where U: Ord, F: FnMut(T) -> Option<U> {
    Some(UOrd::new(f(self.min)?, f(self.max)?))
  }

  #[inline]
  pub fn iter(&self) -> UOrdIter<T> {
    self.into_iter()
  }
}

type UOrdIter<'a, T> = std::array::IntoIter<&'a T, 2>;
type UOrdIntoIter<T> = std::array::IntoIter<T, 2>;

impl<T> IntoIterator for UOrd<T> {
  type Item = T;
  type IntoIter = UOrdIntoIter<T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.into_array().into_iter()
  }
}

impl<'a, T> IntoIterator for &'a UOrd<T> {
  type Item = &'a T;
  type IntoIter = UOrdIter<'a, T>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.as_array().into_iter()
  }
}

impl<T: fmt::Debug> fmt::Debug for UOrd<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_tuple("UOrd")
      .field(&self.min)
      .field(&self.max)
      .finish()
  }
}

impl<T: Ord> From<(T, T)> for UOrd<T> {
  #[inline(always)]
  fn from(value: (T, T)) -> UOrd<T> {
    UOrd::new(value.0, value.1)
  }
}

impl<T: Ord> Into<(T, T)> for UOrd<T> {
  #[inline(always)]
  fn into(self) -> (T, T) {
    self.into_tuple()
  }
}

impl<T: PartialEq> PartialEq for UOrd<T> {
  #[inline]
  fn eq(&self, other: &UOrd<T>) -> bool {
    (self.min == other.min && self.max == other.max) ||
    (self.min == other.max && self.max == other.min)
  }
}

impl<T: Eq> Eq for UOrd<T> {}

impl<T: Hash> Hash for UOrd<T> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_tuple().hash(state);
  }
}



#[cfg(feature = "serde")]
impl<T> serde::Serialize for UOrd<T>
where T: serde::Serialize {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.as_tuple().serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for UOrd<T>
where T: Ord + serde::Deserialize<'de> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    struct UOrdVisitor<T>(std::marker::PhantomData<UOrd<T>>);

    impl<'de, T> serde::de::Visitor<'de> for UOrdVisitor<T>
    where T: Ord + serde::Deserialize<'de> {
      type Value = UOrd<T>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a tuple of size 2")
      }

      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where A: serde::de::SeqAccess<'de> {
        let a = match seq.next_element()? {
          Some(value) => value,
          None => return Err(serde::de::Error::invalid_length(2, &self))
        };

        let b = match seq.next_element()? {
          Some(value) => value,
          None => return Err(serde::de::Error::invalid_length(2, &self))
        };

        Ok(UOrd::new(a, b))
      }
    }

    deserializer.deserialize_tuple(2, UOrdVisitor(std::marker::PhantomData))
  }
}
