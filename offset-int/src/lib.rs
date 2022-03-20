//! A set of functions that perform addition of signed integers with unsigned integers.
#![no_std]

macro_rules! define_fn {
  ($name:ident, $u:ty, $i:ty) => {
    #[inline]
    #[doc = concat!(
      "Performs addition of a `", stringify!($u), "` and an `", stringify!($i), "`, ",
      "returning `None` on underflow or overflow."
    )]
    pub fn $name(value: $u, o: $i) -> Option<$u> {
      let result = <$u>::wrapping_add(value, o as $u);
      // If the offset was positive, but we decreased, there has been an overflow
      // If the offset was negative, but we increased, there has been an underflow
      if (o > 0 && result <= value) || (o < 0 && result >= value) {
        None
      } else {
        Some(result)
      }
    }
  };
}

define_fn!(offset_u8_checked, u8, i8);
define_fn!(offset_u16_checked, u16, i16);
define_fn!(offset_u32_checked, u32, i32);
define_fn!(offset_u64_checked, u64, i64);
define_fn!(offset_u128_checked, u128, i128);
define_fn!(offset_usize_checked, usize, isize);

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! test_fn {
    ($name:ident, $u:ty, $i:ty) => {
      // Test overflow from MAX
      assert_eq!($name(<$u>::MAX, 1), None);
      // Test underflow from MIN
      assert_eq!($name(<$u>::MIN, -1), None);
      // Test regular functionality
      assert_eq!($name(0, 10), Some(10));
      assert_eq!($name(10, -10), Some(0));
    };
  }

  #[test]
  fn it_works() {
    test_fn!(offset_u8_checked, u8, i8);
    test_fn!(offset_u16_checked, u16, i16);
    test_fn!(offset_u32_checked, u32, i32);
    test_fn!(offset_u64_checked, u64, i64);
    test_fn!(offset_u128_checked, u128, i128);
    test_fn!(offset_usize_checked, usize, isize);
  }
}
