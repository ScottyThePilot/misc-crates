#![allow(mismatched_lifetime_syntaxes)]
#![deprecated = "https://github.com/ScottyThePilot/uord"]
#![no_std]
use core::cmp::Ordering;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::fmt;

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
  pub const fn min(&self) -> &T {
    &self.min
  }

  /// Returns the greater of the two elements, based on `T`'s `Ord` implementation.
  #[inline(always)]
  pub const fn max(&self) -> &T {
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
    self.as_ref().map(|v| if v.borrow() == from { to.clone() } else { v.clone() })
  }

  #[inline]
  pub const fn as_ref(&self) -> UOrd<&T> {
    // &T's Ord impl is the same as T, so this should be OK
    UOrd {
      min: &self.min,
      max: &self.max
    }
  }

  #[inline(always)]
  pub const fn as_tuple(&self) -> (&T, &T) {
    (&self.min, &self.max)
  }

  #[inline(always)]
  pub fn into_tuple(self) -> (T, T) {
    (self.min, self.max)
  }

  #[inline(always)]
  pub const fn as_array(&self) -> [&T; 2] {
    [&self.min, &self.max]
  }

  #[inline(always)]
  pub const fn as_array_ref(&self) -> &[T; 2] {
    // SAFETY: UOrd<T> is repr(C) and contains 2 elements, so it is identical to [T; 2]
    debug_assert!(core::mem::size_of::<UOrd<T>>() == core::mem::size_of::<[T; 2]>());
    unsafe { &*(self as *const UOrd<T> as *const [T; 2]) }
  }

  #[inline(always)]
  pub fn into_array(self) -> [T; 2] {
    [self.min, self.max]
  }

  pub fn map<U, F>(self, mut f: F) -> UOrd<U>
  where U: Ord, F: FnMut(T) -> U {
    UOrd::new(f(self.min), f(self.max))
  }

  pub fn try_map_opt<U, F>(self, mut f: F) -> Option<UOrd<U>>
  where U: Ord, F: FnMut(T) -> Option<U> {
    Some(UOrd::new(f(self.min)?, f(self.max)?))
  }

  pub fn try_map<U, E, F>(self, mut f: F) -> Result<UOrd<U>, E>
  where U: Ord, F: FnMut(T) -> Result<U, E> {
    Ok(UOrd::new(f(self.min)?, f(self.max)?))
  }

  #[inline]
  pub fn convert<U>(self) -> UOrd<U>
  where U: Ord, T: Into<U> {
    self.map(T::into)
  }

  #[inline]
  pub fn try_convert<U>(self) -> Result<UOrd<U>, T::Error>
  where U: Ord, T: TryInto<U> {
    self.try_map(T::try_into)
  }

  pub fn both<F>(&self, mut f: F) -> bool
  where F: FnMut(&T) -> bool {
    f(&self.min) && f(&self.max)
  }

  pub fn either<F>(&self, mut f: F) -> bool
  where F: FnMut(&T) -> bool {
    f(&self.min) || f(&self.max)
  }

  #[inline]
  pub fn iter(&self) -> UOrdIter<T> {
    self.into_iter()
  }

  pub fn inspect<F>(&mut self, mut f: F)
  where T: Ord, F: FnMut(&mut T, &mut T) {
    f(&mut self.min, &mut self.max);
    if let Ordering::Greater = Ord::cmp(&self.min, &self.max) {
      core::mem::swap(&mut self.min, &mut self.max);
    };
  }
}

type UOrdIter<'a, T> = core::array::IntoIter<&'a T, 2>;
type UOrdIntoIter<T> = core::array::IntoIter<T, 2>;

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

impl<T: Ord> From<[T; 2]> for UOrd<T> {
  #[inline(always)]
  fn from(value: [T; 2]) -> UOrd<T> {
    let [a, b] = value;
    UOrd::new(a, b)
  }
}

impl<T: Ord> From<UOrd<T>> for (T, T) {
  fn from(value: UOrd<T>) -> Self {
    value.into_tuple()
  }
}

impl<T: Ord> From<UOrd<T>> for [T; 2] {
  fn from(value: UOrd<T>) -> Self {
    value.into_array()
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

impl<T: PartialOrd> PartialOrd for UOrd<T> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.as_array_ref().partial_cmp(other.as_array_ref())
  }
}

impl<T: Ord> Ord for UOrd<T> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_array_ref().cmp(other.as_array_ref())
  }
}

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
    struct UOrdVisitor<T>(core::marker::PhantomData<UOrd<T>>);

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

    deserializer.deserialize_tuple(2, UOrdVisitor(core::marker::PhantomData))
  }
}
