pub extern crate bytemuck;

use bytemuck::TransparentWrapper;

use std::ffi::{OsString, OsStr};
use std::path::{PathBuf, Path};
use std::hash::{Hash, Hasher};
use std::borrow::{Borrow, BorrowMut, Cow};



#[repr(transparent)]
#[derive(Debug, Clone, Copy, TransparentWrapper, Default)]
pub struct UniCase<T: ?Sized>(pub T);

impl<T: ?Sized> UniCase<T> {
  pub fn borrowed<U: ?Sized>(&self) -> &UniCase<U> where T: Borrow<U> {
    UniCase::wrap_ref(self.0.borrow())
  }

  pub fn borrowed_mut<U: ?Sized>(&mut self) -> &mut UniCase<U> where T: BorrowMut<U> {
    UniCase::wrap_mut(self.0.borrow_mut())
  }
}

impl<T: ?Sized> AsRef<T> for UniCase<T> {
  fn as_ref(&self) -> &T {
    &self.0
  }
}

impl AsRef<[u8]> for UniCase<Vec<u8>> {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}

impl AsRef<str> for UniCase<String> {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl AsRef<OsStr> for UniCase<OsString> {
  fn as_ref(&self) -> &OsStr {
    &self.0
  }
}

impl AsRef<Path> for UniCase<PathBuf> {
  fn as_ref(&self) -> &Path {
    &self.0
  }
}

macro_rules! impl_borrow {
  ($($OwnedType:ty => $BorrowedType:ty),* $(,)?) => ($(
    impl Borrow<UniCase<$BorrowedType>> for UniCase<$OwnedType> {
      #[inline]
      fn borrow(&self) -> &UniCase<$BorrowedType> {
        self.borrowed()
      }
    }
  )*);
}

impl_borrow!(
  Vec<u8> => [u8],
  String => str,
  PathBuf => Path,
  OsString => OsStr
);



impl Ord for UniCase<u8> {
  #[inline]
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.to_ascii_lowercase().cmp(&other.0.to_ascii_lowercase())
  }
}

impl PartialOrd for UniCase<u8> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for UniCase<u8> {}

impl PartialEq for UniCase<u8> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0.eq_ignore_ascii_case(&other.0)
  }
}

impl Hash for UniCase<u8> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.to_ascii_lowercase().hash(state);
  }
}



impl<T: ?Sized> Ord for UniCase<T> where T: UniCaseEnabled {
  #[inline]
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.cmp_ignore_ascii_case(&other.0)
  }
}

impl<T: ?Sized> PartialOrd for UniCase<T> where T: UniCaseEnabled {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<T: ?Sized> Eq for UniCase<T> where T: UniCaseEnabled {}

impl<T: ?Sized> PartialEq for UniCase<T> where T: UniCaseEnabled {
  #[inline]
  fn eq(&self, other: &UniCase<T>) -> bool {
    self.0.eq_ignore_ascii_case(&other.0)
  }
}

impl<T: ?Sized> Hash for UniCase<T> where T: UniCaseEnabled {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash_ignore_ascii_case(state);
  }
}



pub trait UniCaseEnabled {
  fn as_bytes(&self) -> &[u8];

  #[inline]
  fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
    self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
  }

  fn cmp_ignore_ascii_case(&self, other: &Self) -> std::cmp::Ordering {
    let self_iter = self.as_bytes().iter().map(u8::to_ascii_lowercase);
    let other_iter = other.as_bytes().iter().map(u8::to_ascii_lowercase);
    self_iter.cmp(other_iter)
  }

  #[inline]
  fn hash_ignore_ascii_case<H: Hasher>(&self, state: &mut H) {
    UniCase::<u8>::wrap_slice(self.as_bytes()).hash(state);
  }
}

macro_rules! impl_unicase_enabled {
  ($($([$($tt:tt)*])? for $Type:ty => |$ident:ident| $expr:expr),* $(,)?) => ($(
    impl $(<$($tt)*>)? UniCaseEnabled for $Type {
      #[inline]
      fn as_bytes(&self) -> &[u8] {
        let $ident = self;
        $expr
      }
    }
  )*);
}

impl_unicase_enabled!(
  [T: UniCaseEnabled + ?Sized] for &T => |i| T::as_bytes(i),
  [T: UniCaseEnabled + ?Sized] for &mut T => |i| T::as_bytes(i),
  [T: UniCaseEnabled + ?Sized] for Box<T> => |i| T::as_bytes(i),
  [T: UniCaseEnabled + ?Sized] for std::rc::Rc<T> => |i| T::as_bytes(i),
  [T: UniCaseEnabled + ?Sized] for std::sync::Arc<T> => |i| T::as_bytes(i),
  [T: UniCaseEnabled + ToOwned + ?Sized] for Cow<'_, T> => |i| i.as_ref().as_bytes(),
  for [u8] => |i| i,
  for Vec<u8> => |i| i.as_slice(),
  for str => |i| i.as_bytes(),
  for String => |i| i.as_bytes(),
  for Path => |i| i.as_os_str().as_encoded_bytes(),
  for PathBuf => |i| i.as_os_str().as_encoded_bytes(),
  for OsStr => |i| i.as_encoded_bytes(),
  for OsString => |i| i.as_encoded_bytes()
);
