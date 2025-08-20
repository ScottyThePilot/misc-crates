pub trait SliceExt<T> {
  fn to_array<const N: usize>(&self) -> Option<[T; N]> where T: Copy;
  fn to_ref_array<const N: usize>(&self) -> Option<&[T; N]>;
  fn to_mut_array<const N: usize>(&mut self) -> Option<&mut [T; N]>;
  fn windows_mut_each<F>(&mut self, len: usize, f: F) where F: FnMut(&mut [T]);
  fn array_windows_mut_each<F, const N: usize>(&mut self, f: F) where F: FnMut(&mut [T; N]);
}

impl<T> SliceExt<T> for [T] {
  /// Shortcut method to [`array::try_from`][<https://doc.rust-lang.org/std/primitive.array.html#method.try_from-3>].
  #[inline]
  fn to_array<const N: usize>(&self) -> Option<[T; N]> where T: Copy {
    self.try_into().ok()
  }

  /// Shortcut method to [`array::try_from`][<https://doc.rust-lang.org/std/primitive.array.html#method.try_from>].
  #[inline]
  fn to_ref_array<const N: usize>(&self) -> Option<&[T; N]> {
    self.try_into().ok()
  }

  /// Shortcut method to [`array::try_from`][<https://doc.rust-lang.org/std/primitive.array.html#method.try_from-2>].
  #[inline]
  fn to_mut_array<const N: usize>(&mut self) -> Option<&mut [T; N]> {
    self.try_into().ok()
  }

  /// Similar to [`slice::windows`][<https://doc.rust-lang.org/std/primitive.slice.html#method.windows>]
  /// but mutable and uses a function due to iterator limitations.
  fn windows_mut_each<F>(&mut self, len: usize, mut f: F)
  where F: FnMut(&mut [T]) {
    assert_ne!(len, 0);
    if self.len() < len { return };
    if self.len() == len { return f(self) };
    for i in 0..(self.len() + 1 - len) {
      let slice = &mut self[i..(i + len)];
      assert_eq!(slice.len(), len);
      f(slice);
    };
  }

  /// Similar to [`slice::windows`][<https://doc.rust-lang.org/std/primitive.slice.html#method.windows>]
  /// but mutable, takes a constant length parameter, and uses a function due to iterator limitations.
  fn array_windows_mut_each<F, const N: usize>(&mut self, mut f: F)
  where F: FnMut(&mut [T; N]) {
    assert_ne!(N, 0);
    if self.len() < N { return };
    if self.len() == N { return f(self.to_mut_array::<N>().unwrap()) };
    for i in 0..(self.len() + 1 - N) {
      let slice = &mut self[i..(i + N)];
      f(slice.to_mut_array().unwrap());
    };
  }
}
