pub trait SliceExt<T> {
  fn to_array<const N: usize>(&self) -> Option<[T; N]> where T: Clone;
  fn to_ref_array<const N: usize>(&self) -> Option<&[T; N]>;
  fn to_mut_array<const N: usize>(&mut self) -> Option<&mut [T; N]>;
  fn windows_mut_each<F, const N: usize>(&mut self, f: F) where F: FnMut(&mut [T; N]);
}

impl<T> SliceExt<T> for [T] {
  #[inline]
  fn to_array<const N: usize>(&self) -> Option<[T; N]> where T: Clone {
    <&[T; N]>::try_from(self).ok().cloned()
  }

  #[inline]
  fn to_ref_array<const N: usize>(&self) -> Option<&[T; N]> {
    <&[T; N]>::try_from(self).ok()
  }

  #[inline]
  fn to_mut_array<const N: usize>(&mut self) -> Option<&mut [T; N]> {
    <&mut [T; N]>::try_from(self).ok()
  }

  fn windows_mut_each<F, const N: usize>(&mut self, mut f: F)
  where F: FnMut(&mut [T; N]) {
    if self.len() < N { return };
    for i in 0..(self.len() + 1 - N) {
      match self[i..(i + N)].to_mut_array::<N>() {
        Some(slice) => f(slice),
        None => unreachable!()
      };
    };
  }
}
