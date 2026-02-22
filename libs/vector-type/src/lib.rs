#![no_std]
#![no_implicit_prelude]

#[doc(hidden)]
pub mod private;

/// Defines a 'Vector Struct' and a 'Vector Enum'.
///
/// The vector struct is essentially a generic array type,
/// where all fields are the same type. Each field corresponds to a
/// variant in the vector enum, and the vector enum can be used to index
/// fields of the vector struct.
///
/// The bulk of the utility you will get from this macro are the functions, constants,
/// and traits automatically implemented for the vector struct and vector enum.
/// See the 'Implementation' section for a list of items that are implemented by this macro.
///
/// The vector struct will always be `repr(C)`, and this cannot be changed.
/// The vector enum must specify a `repr` type via the macro, which should be an integer type.
/// Enum discriminants may be specified after the field names.
///
/// # Examples
///
/// The following code:
/// ```rust
/// vector_type!{
///   // The vector struct's name
///   pub struct MyVectorStruct;
///
///   // Attributes are allowed, though many traits are already derived
///   #[derive(Default)]
///   // The vector enum's name, and its repr type
///   pub enum MyVectorEnum as u8;
///
///   // Each field, and its respective enum variant
///   abstract {
///     field1: #[default] Field1,
///     field2: Field2,
///     field3: Field3
///   }
/// }
/// ```
///
/// Will expand to something (roughly) like this:
/// ```rust
/// #[repr(C)]
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// pub struct MyVectorStruct<T> {
///   field1: T,
///   field2: T,
///   field3: T
/// }
///
/// #[derive(Default)]
/// #[repr(u8)]
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// pub enum MyVectorEnum {
///   #[default]
///   Field1,
///   Field2,
///   Field3
/// }
/// ```
///
/// # Implementation
/// The following items are implemented for the vector struct and the vector enum:
/// ```rust
/// impl<T> $VectorStruct<T> {
///   const fn splat(value: T) -> Self where T: Copy;
///   const fn get(&self, variant: $VectorEnum) -> &T;
///   const fn get_mut(&mut self, variant: $VectorEnum) -> &mut T;
///   const fn zip<U>(self, other: $VectorStruct<U>) -> $VectorStruct<(T, U)>;
///   fn zip_with<U, V>(self, other: $VectorStruct<U>, mut f: impl FnMut(T, U) -> V) -> $VectorStruct<V>;
///   fn map<U>(self, mut f: impl FnMut(T) -> U) -> $VectorStruct<U>;
///   fn map_tagged<U>(self, mut f: impl FnMut($VectorEnum, T) -> U) -> $VectorStruct<U>;
///   fn try_map_opt<U>(self, mut f: impl FnMut(T) -> Option<U>) -> Option<$VectorStruct<U>>;
///   fn try_map_res<U, E>(self, mut f: impl FnMut(T) -> Result<U, E>) -> Result<$VectorStruct<U>, E>;
///   fn convert<U>(self) -> $VectorStruct<U> where T: Into<U>;
///   fn try_convert<U>(self) -> Result<$VectorStruct<U>, T::Error> where T: TryInto<U>;
///   const fn from_array(array: [T; $VectorEnum::VARIANTS_COUNT]) -> Self;
///   const fn from_array_ref(array: &[T; $VectorEnum::VARIANTS_COUNT]) -> &Self;
///   const fn from_array_ref_mut(array: &mut [T; $VectorEnum::VARIANTS_COUNT]) -> &mut Self;
///   const fn into_array(self) -> [T; $VectorEnum::VARIANTS_COUNT];
///   const fn as_array_ref(&self) -> &[T; $VectorEnum::VARIANTS_COUNT];
///   const fn as_array_ref_mut(&mut self) -> &mut [T; $VectorEnum::VARIANTS_COUNT];
///   const fn as_slice(&self) -> &[T];
///   const fn as_slice_mut(&mut self) -> &mut [T];
///   const fn each_ref(&self) -> $VectorStruct<&T>;
///   const fn each_mut(&mut self) -> $VectorStruct<&mut T>;
///   const fn each_ref_array(&self) -> [&T; $VectorEnum::VARIANTS_COUNT];
///   const fn each_mut_array(&mut self) -> [&mut T; $VectorEnum::VARIANTS_COUNT];
///   fn test_all(&self, f: impl FnMut(&T) -> bool) -> bool;
///   fn test_any(&self, f: impl FnMut(&T) -> bool) -> bool;
///   fn iter(&self) -> impl Iterator;
///   fn iter_mut(&mut self) -> impl Iterator;
/// }
///
/// impl $VectorStruct<bool> {
///   const ALL_TRUE: Self;
///   const ALL_FALSE: Self;
///   const fn just(which: $VectorEnum) -> Self;
///   const fn is_all_true(self) -> bool;
///   const fn is_all_false(self) -> bool;
///   const fn is_any_true(self) -> bool;
///   const fn is_any_false(self) -> bool;
///   const fn bool_and(self, other: Self) -> Self;
///   const fn bool_xor(self, other: Self) -> Self;
///   const fn bool_or(self, other: Self) -> Self;
///   const fn bool_not(self) -> Self;
/// }
///
/// impl<T> Debug for $VectorStruct<T> where T: Debug;
/// impl<T> Clone for $VectorStruct<T> where T: Clone;
/// impl<T> Copy for $VectorStruct<T> where T: Copy;
/// impl<T> PartialEq for $VectorStruct<T> where T: PartialEq;
/// impl<T> Eq for $VectorStruct<T> where T: Eq;
/// impl<T> Hash for $VectorStruct<T> where T: Hash;
///
/// impl<T> AsRef<[T; $VectorEnum::VARIANTS_COUNT]> for $VectorStruct<T>;
/// impl<T> AsMut<[T; $VectorEnum::VARIANTS_COUNT]> for $VectorStruct<T>;
/// impl<T> From<[T; $VectorEnum::VARIANTS_COUNT]> for $VectorStruct<T>;
/// impl<T> From<$VectorStruct<T>> for [T; $VectorEnum::VARIANTS_COUNT];
/// impl<T> Index<$VectorEnum> for $VectorStruct<T>;
/// impl<T> IndexMut<$VectorEnum> for $VectorStruct<T>;
/// impl<T> IntoIterator for $VectorStruct<T>;
/// impl<'a, T> IntoIterator for &'a $VectorStruct<T>;
/// impl<'a, T> IntoIterator for &'a mut $VectorStruct<T>;
///
/// impl<Lhs, Rhs> Add<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> Sub<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> Mul<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> Div<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> Rem<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> AddAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> SubAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> MulAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> DivAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> RemAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<T> Neg for $VectorStruct<T> where T: Neg;
///
/// impl<Lhs, Rhs> BitAnd<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> BitXor<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> BitOr<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> BitAndAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> BitXorAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> BitOrAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<T> Not for $VectorStruct<T> where T: Not;
///
/// impl<Lhs, Rhs> Shl<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> Shr<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: Add<Rhs>;
/// impl<Lhs, Rhs> ShlAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
/// impl<Lhs, Rhs> ShrAssign<$VectorStruct<Rhs>> for $VectorStruct<Lhs> where Lhs: AddAssign<Rhs>;
///
/// impl $VectorEnum {
///   const VARIANTS_COUNT: usize;
///   const VARIANTS_ARRAY: [Self; $VectorEnum::VARIANTS_COUNT];
///   const VARIANTS: $VectorStruct<Self>;
///   const fn to_num(self) -> $repr_type;
/// }
///
/// impl Debug for $VectorEnum;
/// impl Clone for $VectorEnum;
/// impl Copy for $VectorEnum;
/// impl PartialEq for $VectorEnum;
/// impl Eq for $VectorEnum;
/// impl PartialOrd for $VectorEnum;
/// impl Ord for $VectorEnum;
/// impl Hash for $VectorEnum;
/// ```
#[macro_export]
macro_rules! vector_type {
  (
    $(#[$attr_struct:meta])*
    $vis_struct:vis struct $VectorStruct:ident;

    $(#[$attr_enum:meta])*
    $vis_enum:vis enum $VectorEnum:ident as $repr_type:ident;

    abstract {
      $(
        $(#[$attr_field:meta])* $field:ident:
        $(#[$attr_variant:meta])* $Variant:ident
        $(= $discrim:expr)?
      ),* $(,)?
    }
  ) => (
    $(#[$attr_struct])*
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    $vis_struct struct $VectorStruct<T> {
      $($(#[$attr_field])* pub $field: T),*
    }

    $(#[$attr_enum])*
    #[repr($repr_type)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    $vis_enum enum $VectorEnum {
      $($(#[$attr_variant])* $Variant $(= $discrim)?),*
    }

    impl<T> $VectorStruct<T> {
      $vis_struct const fn splat(value: T) -> Self where T: core::marker::Copy {
        Self::from_array([value; $VectorEnum::VARIANTS_COUNT])
      }

      $vis_struct const fn get(&self, variant: $VectorEnum) -> &T {
        match variant { $($VectorEnum::$Variant => &self.$field),* }
      }

      $vis_struct const fn get_mut(&mut self, variant: $VectorEnum) -> &mut T {
        match variant { $($VectorEnum::$Variant => &mut self.$field),* }
      }

      $vis_struct const fn zip<U>(self, other: $VectorStruct<U>) -> $VectorStruct<(T, U)> {
        let this = unsafe { $crate::private::transmute::<$VectorStruct<T>, $VectorStruct<core::mem::ManuallyDrop<T>>>(self) };
        let other = unsafe { $crate::private::transmute::<$VectorStruct<U>, $VectorStruct<core::mem::ManuallyDrop<U>>>(other) };

        $VectorStruct {
          $($field: (
            core::mem::ManuallyDrop::into_inner(this.$field),
            core::mem::ManuallyDrop::into_inner(other.$field)
          )),*
        }
      }

      $vis_struct fn zip_with<U, V>(self, other: $VectorStruct<U>, mut f: impl core::ops::FnMut(T, U) -> V) -> $VectorStruct<V> {
        self.zip(other).map(|(t, u)| f(t, u))
      }

      $vis_struct fn map<U>(self, mut f: impl core::ops::FnMut(T) -> U) -> $VectorStruct<U> {
        $VectorStruct { $($field: f(self.$field)),* }
      }

      $vis_struct fn map_tagged<U>(self, mut f: impl core::ops::FnMut($VectorEnum, T) -> U) -> $VectorStruct<U> {
        $VectorStruct { $($field: f($VectorEnum::$Variant, self.$field)),* }
      }

      $vis_struct fn try_map_opt<U>(self, mut f: impl core::ops::FnMut(T) -> core::option::Option<U>) -> core::option::Option<$VectorStruct<U>> {
        core::option::Option::Some($VectorStruct { $($field: f(self.$field)?),* })
      }

      $vis_struct fn try_map_res<U, E>(self, mut f: impl core::ops::FnMut(T) -> core::result::Result<U, E>) -> core::result::Result<$VectorStruct<U>, E> {
        core::result::Result::Ok($VectorStruct { $($field: f(self.$field)?),* })
      }

      $vis_struct fn convert<U>(self) -> $VectorStruct<U> where T: core::convert::Into<U> {
        self.map(T::into)
      }

      $vis_struct fn try_convert<U>(self) -> core::result::Result<$VectorStruct<U>, T::Error> where T: core::convert::TryInto<U> {
        self.try_map_res(T::try_into)
      }

      $vis_struct const fn from_array(array: [T; $VectorEnum::VARIANTS_COUNT]) -> Self {
        unsafe { $crate::private::transmute::<[T; $VectorEnum::VARIANTS_COUNT], $VectorStruct<T>>(array) }
      }

      $vis_struct const fn from_array_ref(array: &[T; $VectorEnum::VARIANTS_COUNT]) -> &Self {
        unsafe { $crate::private::transmute_ref::<[T; $VectorEnum::VARIANTS_COUNT], $VectorStruct<T>>(array) }
      }

      $vis_struct const fn from_array_ref_mut(array: &mut [T; $VectorEnum::VARIANTS_COUNT]) -> &mut Self {
        unsafe { $crate::private::transmute_ref_mut::<[T; $VectorEnum::VARIANTS_COUNT], $VectorStruct<T>>(array) }
      }

      $vis_struct const fn into_array(self) -> [T; $VectorEnum::VARIANTS_COUNT] {
        unsafe { $crate::private::transmute::<$VectorStruct<T>, [T; $VectorEnum::VARIANTS_COUNT]>(self) }
      }

      $vis_struct const fn as_array_ref(&self) -> &[T; $VectorEnum::VARIANTS_COUNT] {
        unsafe { $crate::private::transmute_ref::<$VectorStruct<T>, [T; $VectorEnum::VARIANTS_COUNT]>(self) }
      }

      $vis_struct const fn as_array_ref_mut(&mut self) -> &mut [T; $VectorEnum::VARIANTS_COUNT] {
        unsafe { $crate::private::transmute_ref_mut::<$VectorStruct<T>, [T; $VectorEnum::VARIANTS_COUNT]>(self) }
      }

      $vis_struct const fn as_slice(&self) -> &[T] {
        self.as_array_ref()
      }

      $vis_struct const fn as_slice_mut(&mut self) -> &mut [T] {
        self.as_array_ref_mut()
      }

      $vis_struct const fn each_ref(&self) -> $VectorStruct<&T> {
        $VectorStruct { $($field: &self.$field),* }
      }

      $vis_struct const fn each_mut(&mut self) -> $VectorStruct<&mut T> {
        $VectorStruct { $($field: &mut self.$field),* }
      }

      $vis_struct const fn each_ref_array(&self) -> [&T; $VectorEnum::VARIANTS_COUNT] {
        self.each_ref().into_array()
      }

      $vis_struct const fn each_mut_array(&mut self) -> [&mut T; $VectorEnum::VARIANTS_COUNT] {
        self.each_mut().into_array()
      }

      $vis_struct fn test_all(&self, f: impl core::ops::FnMut(&T) -> bool) -> bool {
        core::iter::Iterator::all(&mut self.iter(), f)
      }

      $vis_struct fn test_any(&self, f: impl core::ops::FnMut(&T) -> bool) -> bool {
        core::iter::Iterator::any(&mut self.iter(), f)
      }

      $vis_struct fn iter(&self) -> <&Self as core::iter::IntoIterator>::IntoIter {
        core::iter::IntoIterator::into_iter(self)
      }

      $vis_struct fn iter_mut(&mut self) -> <&mut Self as core::iter::IntoIterator>::IntoIter {
        core::iter::IntoIterator::into_iter(self)
      }
    }

    impl $VectorStruct<bool> {
      $vis_struct const ALL_TRUE: Self = Self::splat(true);
      $vis_struct const ALL_FALSE: Self = Self::splat(false);

      $vis_struct const fn just(which: $VectorEnum) -> Self {
        let mut this = Self::ALL_FALSE;
        *this.get_mut(which) = true;
        this
      }

      $vis_struct const fn is_all_true(self) -> bool {
        core::matches!(self, Self::ALL_TRUE)
      }

      $vis_struct const fn is_all_false(self) -> bool {
        core::matches!(self, Self::ALL_FALSE)
      }

      $vis_struct const fn is_any_true(self) -> bool {
        !core::matches!(self, Self::ALL_FALSE)
      }

      $vis_struct const fn is_any_false(self) -> bool {
        !core::matches!(self, Self::ALL_TRUE)
      }

      $vis_struct const fn bool_and(self, other: Self) -> Self {
        Self::from_array($crate::private::array_bool_and(self.into_array(), other.into_array()))
      }

      $vis_struct const fn bool_xor(self, other: Self) -> Self {
        Self::from_array($crate::private::array_bool_or(self.into_array(), other.into_array()))
      }

      $vis_struct const fn bool_or(self, other: Self) -> Self {
        Self::from_array($crate::private::array_bool_xor(self.into_array(), other.into_array()))
      }

      $vis_struct const fn bool_not(self) -> Self {
        Self::from_array($crate::private::array_bool_not(self.into_array()))
      }
    }

    impl<T> core::convert::AsRef<[T; $VectorEnum::VARIANTS_COUNT]> for $VectorStruct<T> {
      fn as_ref(&self) -> &[T; $VectorEnum::VARIANTS_COUNT] {
        self.as_array_ref()
      }
    }

    impl<T> core::convert::AsRef<[T]> for $VectorStruct<T> {
      fn as_ref(&self) -> &[T] {
        self.as_array_ref()
      }
    }

    impl<T> core::convert::AsMut<[T; $VectorEnum::VARIANTS_COUNT]> for $VectorStruct<T> {
      fn as_mut(&mut self) -> &mut [T; $VectorEnum::VARIANTS_COUNT] {
        self.as_array_ref_mut()
      }
    }

    impl<T> core::convert::AsMut<[T]> for $VectorStruct<T> {
      fn as_mut(&mut self) -> &mut [T] {
        self.as_array_ref_mut()
      }
    }

    impl<T> core::convert::From<[T; $VectorEnum::VARIANTS_COUNT]> for $VectorStruct<T> {
      fn from(value: [T; $VectorEnum::VARIANTS_COUNT]) -> $VectorStruct<T> {
        $VectorStruct::from_array(value)
      }
    }

    impl<T> core::convert::From<$VectorStruct<T>> for [T; $VectorEnum::VARIANTS_COUNT] {
      fn from(value: $VectorStruct<T>) -> [T; $VectorEnum::VARIANTS_COUNT] {
        $VectorStruct::into_array(value)
      }
    }

    impl<T> core::ops::Index<$VectorEnum> for $VectorStruct<T> {
      type Output = T;

      fn index(&self, variant: $VectorEnum) -> &T {
        self.get(variant)
      }
    }

    impl<T> core::ops::IndexMut<$VectorEnum> for $VectorStruct<T> {
      fn index_mut(&mut self, variant: $VectorEnum) -> &mut T {
        self.get_mut(variant)
      }
    }

    impl<T> core::iter::IntoIterator for $VectorStruct<T> {
      type Item = T;
      type IntoIter = <[T; $VectorEnum::VARIANTS_COUNT] as core::iter::IntoIterator>::IntoIter;

      fn into_iter(self) -> Self::IntoIter {
        self.into_array().into_iter()
      }
    }

    impl<'a, T> core::iter::IntoIterator for &'a $VectorStruct<T> {
      type Item = &'a T;
      type IntoIter = <&'a [T; $VectorEnum::VARIANTS_COUNT] as core::iter::IntoIterator>::IntoIter;

      fn into_iter(self) -> Self::IntoIter {
        self.as_array_ref().into_iter()
      }
    }

    impl<'a, T> core::iter::IntoIterator for &'a mut $VectorStruct<T> {
      type Item = &'a mut T;
      type IntoIter = <&'a mut [T; $VectorEnum::VARIANTS_COUNT] as core::iter::IntoIterator>::IntoIter;

      fn into_iter(self) -> Self::IntoIter {
        self.as_array_ref_mut().into_iter()
      }
    }

    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Add, add, AddAssign, add_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Sub, sub, SubAssign, sub_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Mul, mul, MulAssign, mul_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Div, div, DivAssign, div_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Rem, rem, RemAssign, rem_assign);
    $crate::vector_type_unary_op!(struct $VectorStruct, enum $VectorEnum, Neg, neg);

    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, BitAnd, bitand, BitAndAssign, bitand_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, BitXor, bitxor, BitXorAssign, bitxor_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, BitOr, bitor, BitOrAssign, bitor_assign);
    $crate::vector_type_unary_op!(struct $VectorStruct, enum $VectorEnum, Not, not);

    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Shl, shl, ShlAssign, shl_assign);
    $crate::vector_type_binary_op!(struct $VectorStruct, enum $VectorEnum, Shr, shr, ShrAssign, shr_assign);

    impl $VectorEnum {
      $vis_enum const VARIANTS_COUNT: usize = core::mem::size_of::<$VectorStruct<u8>>();
      $vis_enum const VARIANTS_ARRAY: [Self; $VectorEnum::VARIANTS_COUNT] = Self::VARIANTS.into_array();
      $vis_enum const VARIANTS: $VectorStruct<Self> = $VectorStruct { $($field: Self::$Variant),* };

      $vis_enum const fn to_num(self) -> $repr_type {
        self as $repr_type
      }
    }
  );
}

#[doc(hidden)]
#[macro_export]
macro_rules! vector_type_binary_op {
  (struct $VectorStruct:ident, enum $VectorEnum:ident, $OpTrait:ident, $op_fn:ident, $OpAssignTrait:ident, $op_assign_fn:ident) => (
    impl<Lhs, Rhs> core::ops::$OpTrait<$VectorStruct<Rhs>> for $VectorStruct<Lhs>
    where Lhs: core::ops::$OpTrait<Rhs> {
      type Output = $VectorStruct<<Lhs as core::ops::$OpTrait<Rhs>>::Output>;

      fn $op_fn(self, rhs: $VectorStruct<Rhs>) -> $VectorStruct<<Lhs as core::ops::$OpTrait<Rhs>>::Output> {
        self.zip_with(rhs, <Lhs as core::ops::$OpTrait<Rhs>>::$op_fn)
      }
    }

    impl<Lhs, Rhs> core::ops::$OpAssignTrait<$VectorStruct<Rhs>> for $VectorStruct<Lhs>
    where Lhs: core::ops::$OpAssignTrait<Rhs> {
      fn $op_assign_fn(&mut self, rhs: $VectorStruct<Rhs>) {
        self.each_mut().zip_with(rhs, <Lhs as core::ops::$OpAssignTrait<Rhs>>::$op_assign_fn);
      }
    }
  );
}

#[doc(hidden)]
#[macro_export]
macro_rules! vector_type_unary_op {
  (struct $VectorStruct:ident, enum $VectorEnum:ident, $OpTrait:ident, $op_fn:ident) => (
    impl<T> core::ops::$OpTrait for $VectorStruct<T>
    where T: core::ops::$OpTrait {
      type Output = $VectorStruct<<T as core::ops::$OpTrait>::Output>;

      fn $op_fn(self) -> $VectorStruct<<T as core::ops::$OpTrait>::Output> {
        self.map(<T as core::ops::$OpTrait>::$op_fn)
      }
    }
  );
}

#[cfg(test)]
mod tests {
  pub extern crate core;

  crate::vector_type!{
    pub struct Vector;

    #[derive(Default)]
    pub enum VectorField as u8;

    abstract {
      field1: #[default] Field1,
      field2: Field2,
      field3: Field3
    }
  }
}
