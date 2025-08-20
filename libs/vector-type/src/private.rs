pub extern crate core;

use core::mem::{ManuallyDrop, align_of, size_of};
use core::marker::Sized;



macro_rules! array_binary_op_fn {
  ($vis:vis fn $fn_name:ident($Type:ty, $default:expr, [$op:tt])) => (
    $vis const fn $fn_name<const N: usize>(lhs: [$Type; N], rhs: [$Type; N]) -> [$Type; N] {
      let mut out = [$default; N];

      let mut i: usize = 0;
      while i < N {
        out[i] = lhs[i] $op rhs[i];
        i += 1;
      };

      out
    }
  );
}

macro_rules! array_unary_op_fn {
  ($vis:vis fn $fn_name:ident($Type:ty, $default:expr, [$op:tt])) => (
    $vis const fn $fn_name<const N: usize>(value: [$Type; N]) -> [$Type; N] {
      let mut out = [$default; N];

      let mut i: usize = 0;
      while i < N {
        out[i] = $op value[i];
        i += 1;
      };

      out
    }
  );
}

array_binary_op_fn!(pub fn array_bool_and(bool, false, [&]));
array_binary_op_fn!(pub fn array_bool_xor(bool, false, [^]));
array_binary_op_fn!(pub fn array_bool_or(bool, false, [|]));
array_unary_op_fn!(pub fn array_bool_not(bool, false, [!]));



macro_rules! assert_size_eq {
  ($Left:ty, $Right:ty) => (const {
    assert!(size_of::<$Left>() == size_of::<$Right>());
  });
}

macro_rules! assert_align_eq {
  ($Left:ty, $Right:ty) => (const {
    assert!(align_of::<$Left>() == align_of::<$Right>());
  });
}



pub const unsafe fn transmute_ref<T: ?Sized, U: ?Sized>(ptr: &T) -> &U {
  assert_size_eq!(*const T, *const U);

  unsafe { &*transmute::<*const T, *const U>(ptr as *const T) }
}

pub const unsafe fn transmute_ref_mut<T: ?Sized, U: ?Sized>(ptr: &mut T) -> &mut U {
  assert_size_eq!(*mut T, *mut U);

  unsafe { &mut *transmute::<*mut T, *mut U>(ptr as *mut T) }
}

pub const unsafe fn transmute_slice<T, U>(slice: &[T]) -> &[U] {
  assert_size_eq!(T, U);
  assert_align_eq!(T, U);

  unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const U, slice.len()) }
}

pub const unsafe fn transmute_slice_mut<T, U>(slice: &mut [T]) -> &mut [U] {
  assert_size_eq!(T, U);
  assert_align_eq!(T, U);

  unsafe { core::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut U, slice.len()) }
}

pub const unsafe fn transmute<T, U>(value: T) -> U {
  assert_size_eq!(T, U);
  assert_align_eq!(T, U);

  #[repr(C)]
  union TransmuteCast<T, U> {
    t: ManuallyDrop<T>,
    u: ManuallyDrop<U>
  }

  let value = ManuallyDrop::new(value);

  ManuallyDrop::into_inner(unsafe {
    TransmuteCast { t: value }.u
  })
}
