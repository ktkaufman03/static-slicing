// Disable Clippy's let_unit_value warning because if we take its suggestion
// we just get a different warning.
#![allow(clippy::let_unit_value)]

//! Provides utilities for emulating statically-checked array slicing and copying.
//!
//! The [`StaticRangeIndex`] type can be used as an index into **fixed-size arrays** to get a fixed-size slice,
//! or "`n`-slice" where `n` is a constant.
//! The [`FixedSizeCopy`] trait provides a `copy_from` function that can be used for copies between statically-sized arrays
//! of types implementing [`Copy`].
//!
//! # Examples
//!
//! This example demonstrates how to obtain an 8-element slice of an array, starting from index 4.
//! ```
//! use static_slicing::StaticRangeIndex;
//! let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
//! let sub_arr = arr[StaticRangeIndex::<4, 8>];
//! assert_eq!(sub_arr, arr[4..12]);
//! ```
//!
//! This example demonstrates how to obtain a mutable 3-element slice of an array, starting from index 2.
//! ```
//! use static_slicing::StaticRangeIndex;
//! let mut arr = [3, 5, 7, 9, 11];
//! let sub_arr = &mut arr[StaticRangeIndex::<2, 3>];
//! sub_arr[1] = 13;
//! assert_eq!(arr[3], 13);
//! ```
//!
//! This example demonstrates how to obtain the item at index 2 of a 4-element array.
//! ```
//! use static_slicing::StaticIndex;
//! let mut arr = [3, 5, 7, 9];
//! let value = arr[StaticIndex::<2>];
//! assert_eq!(value, 7);
//! ```
//!
//! The following examples demonstrate the compile-time safety guarantees of the static slicing framework.
//! ```compile_fail
//! use static_slicing::StaticRangeIndex;
//! let arr = [1, 2, 3, 4, 5];
//! // error! we can't get 5 elements starting from index 1
//! let sub_arr = arr[StaticRangeIndex::<1, 5>];
//! ```
//!
//! ```compile_fail
//! use static_slicing::StaticIndex;
//! let arr = [1, 2, 3, 4, 5];
//! // error! we can't get the item at index 5, because there are only 5 items
//! let value = arr[StaticIndex::<5>];
//! ```
use core::ops::{Index, IndexMut};
use std::marker::PhantomData;

/// Internal helper trait for static indexing.
///
/// [`IsValidIndex::RESULT`] must evaluate to `()` if the index is valid,
/// or panic otherwise.
trait IsValidIndex<const INDEX: usize> {
    const RESULT: ();
}

/// An index that exists entirely at compile time.
///
/// This type can be used as an index into **fixed-size arrays** to get a value.
pub struct StaticIndex<const INDEX: usize>;

impl<const INDEX: usize, const N: usize, T> IsValidIndex<INDEX> for [T; N] {
    const RESULT: () = {
        if INDEX >= N {
            panic!("Index is out of bounds!");
        }
    };
}

impl<const INDEX: usize, const N: usize, T> Index<StaticIndex<INDEX>> for [T; N] {
    type Output = T;

    fn index(&self, _: StaticIndex<INDEX>) -> &Self::Output {
        let _ = <[T; N] as IsValidIndex<INDEX>>::RESULT;

        // SAFETY: We've verified bounds at compile time.
        unsafe { &*(self.as_ptr().add(INDEX) as *const T) }
    }
}

impl<const INDEX: usize, const N: usize, T> IndexMut<StaticIndex<INDEX>> for [T; N] {
    fn index_mut(&mut self, _: StaticIndex<INDEX>) -> &mut Self::Output {
        let _ = <[T; N] as IsValidIndex<INDEX>>::RESULT;

        // SAFETY: We've verified bounds at compile time.
        unsafe { &mut *(self.as_mut_ptr().add(INDEX) as *mut T) }
    }
}

/// Internal helper trait for static range indexing.
///
/// [`IsValidIndexRange::RESULT`] must evaluate to `()` if the range is valid,
/// or panic otherwise.
trait IsValidIndexRange<const START: usize, const LENGTH: usize> {
    const RESULT: ();
}

/// A range index that exists entirely at compile time.
///
/// This type can be used as an index into **fixed-size arrays** to get a fixed-size slice.
/// For any pair of `(START, LENGTH)`, the range covered is `[START, START+LENGTH)`.
pub struct StaticRangeIndex<const START: usize, const LENGTH: usize>;

impl<const START: usize, const LENGTH: usize, const N: usize, T> IsValidIndexRange<START, LENGTH>
    for [T; N]
{
    const RESULT: () = {
        if START >= N {
            panic!("Starting index is out-of-bounds, please see compile error for more info");
        } else if START + LENGTH > N {
            panic!("Ending index is out-of-bounds, please see compile error for more info");
        }
    };
}

impl<const START: usize, const LENGTH: usize, const N: usize, T>
    Index<StaticRangeIndex<START, LENGTH>> for [T; N]
{
    type Output = [T; LENGTH];

    fn index(&self, _: StaticRangeIndex<START, LENGTH>) -> &Self::Output {
        let _ = <[T; N] as IsValidIndexRange<START, LENGTH>>::RESULT;

        // SAFETY: We've verified bounds at compile time.
        unsafe { &*(self.as_ptr().add(START) as *const [T; LENGTH]) }
    }
}

impl<const START: usize, const LENGTH: usize, const N: usize, T>
    IndexMut<StaticRangeIndex<START, LENGTH>> for [T; N]
{
    fn index_mut(&mut self, _: StaticRangeIndex<START, LENGTH>) -> &mut Self::Output {
        let _ = <[T; N] as IsValidIndexRange<START, LENGTH>>::RESULT;

        // SAFETY: We've verified bounds at compiile time.
        unsafe { &mut *(self.as_mut_ptr().add(START) as *mut [T; LENGTH]) }
    }
}

/// Wrapper around slice references to add support for
/// the static index types.
/// 
/// Due to language weirdness, we can't implement Index(Mut)
/// for both \[T\] and \[T; N\]. As a result, we need this 
/// wrapper type.
#[repr(transparent)]
pub struct SliceWrapper<'a, I, T>(
    /// The actual data reference.
    T,

    /// Informs the compiler that the lifetime 'a
    /// is actually part of the type.
    PhantomData<&'a ()>,

    /// Informs the compiler that the type parameter I
    /// is actually part of the type. The reason we need
    /// this is so that we can use an AsRef bound without
    /// the compiler throwing an E0207 at us regarding I.
    PhantomData<I>
);

impl<'a, I, T> SliceWrapper<'a, I, T> where T: AsRef<[I]> {
    pub fn new(data: T) -> Self {
        Self(data, PhantomData, PhantomData)
    }
}

impl<const START: usize, const LENGTH: usize, I, S: AsRef<[I]>> Index<StaticRangeIndex<START, LENGTH>> for SliceWrapper<'_, I, S> {
    type Output = [I; LENGTH];

    fn index(&self, _: StaticRangeIndex<START, LENGTH>) -> &Self::Output {
        let inner: &[I] = self.0.as_ref();

        assert!(inner.len() > START, "Starting index {} is out of bounds", START);
        assert!(inner.len() - START >= LENGTH, "Not enough items after index {} (requested {}; length: {})", START, LENGTH, inner.len());

        // SAFETY: We've verified bounds at runtime.
        unsafe { &*(inner.as_ptr().add(START) as *const [I; LENGTH]) }
    }
}

impl<const INDEX: usize, I, S: AsRef<[I]>> Index<StaticIndex<INDEX>> for SliceWrapper<'_, I, S> {
    type Output = I;

    fn index(&self, _: StaticIndex<INDEX>) -> &Self::Output {
        self.0.as_ref().index(INDEX)
    }
}

impl<const START: usize, const LENGTH: usize, I, S: AsRef<[I]> + AsMut<[I]>> IndexMut<StaticRangeIndex<START, LENGTH>> for SliceWrapper<'_, I, S> {
    fn index_mut(&mut self, _: StaticRangeIndex<START, LENGTH>) -> &mut Self::Output {
        let inner: &mut [I] = self.0.as_mut();

        assert!(inner.len() > START, "Starting index {} is out of bounds", START);
        assert!(inner.len() - START >= LENGTH, "Not enough items after index {} (requested {}; length: {})", START, LENGTH, inner.len());

        // SAFETY: We've verified bounds at runtime.
        unsafe { &mut *(inner.as_mut_ptr().add(START) as *mut [I; LENGTH]) }
    }
}

impl<const INDEX: usize, I, S: AsRef<[I]> + AsMut<[I]>> IndexMut<StaticIndex<INDEX>> for SliceWrapper<'_, I, S> {
    fn index_mut(&mut self, _: StaticIndex<INDEX>) -> &mut Self::Output {
        self.0.as_mut().index_mut(INDEX)
    }
}

/// Fixed-size collections supporting copies from other fixed-size collections.
///
/// # Examples
///
/// This example demonstrates how to copy one 4-byte array to another 4-byte array.
/// ```
/// use static_slicing::FixedSizeCopy;
///
/// let a1 = [0u8, 1u8, 2u8, 3u8];
/// let mut a2 = [9u8; 4];
/// a2.copy_from(a1);
/// assert_eq!(a1, a2);
/// ```
///
/// This example demonstrates the compile-time safety guarantees of [`FixedSizeCopy`].
/// ```compile_fail
/// use static_slicing::FixedSizeCopy;
///
/// let a1 = [0u8, 1u8, 2u8, 3u8, 4u8];
/// let mut a2 = [9u8; 4];
/// // error! a1 has 5 elements but a2 only has room for 4
/// a2.copy_from(a1);
/// assert_eq!(a1, a2);
/// ```
pub trait FixedSizeCopy<T>
where
    T: Copy,
{
    fn copy_from(&mut self, input: Self);
}

impl<T, const N: usize> FixedSizeCopy<T> for [T; N]
where
    T: Copy,
{
    fn copy_from(&mut self, input: Self) {
        // SAFETY: Copying between fixed-size arrays of the same length and type `T` is guaranteed to be SAFE,
        //         in the sense that things won't explode right away if you do it, even if the data isn't really copyable.
        //
        //         It is not always well-defined, though. This is why `FixedSizeCopy` imposes an additional `Copy` bound
        //         on the type `T`. Any type that implements `Copy` can be considered safe to bit-copy around, as opposed to
        //         types that only implement `Clone` (or don't implement either!)
        unsafe {
            core::ptr::copy_nonoverlapping(input.as_ptr(), self.as_mut_ptr(), N);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod core_functionality {
        use super::*;

        #[test]
        fn test_immutable_static_slice() {
            let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
            let sub_arr = arr[StaticRangeIndex::<4, 8>];
    
            assert_eq!(sub_arr, arr[4..12]);
        }
    
        #[test]
        fn test_mutable_static_slice() {
            let mut arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
            let sub_arr = &mut arr[StaticRangeIndex::<4, 8>];
    
            sub_arr[0] = 1234;
            assert_eq!(arr[4], 1234);
        }
    
        #[test]
        fn test_full_immutable_static_slice() {
            let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
            let sub_arr = arr[StaticRangeIndex::<0, 12>];
    
            assert_eq!(arr, sub_arr);
        }
    
        #[test]
        fn test_full_mutable_static_slice() {
            let mut arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
            let sub_arr = &mut arr[StaticRangeIndex::<0, 12>];
    
            sub_arr[4] = 5;
            sub_arr[5] = 4;
            assert_eq!(arr[4], 5);
            assert_eq!(arr[5], 4);
        }
    
        #[test]
        fn test_fixed_size_copy() {
            let a1 = [1, 2, 3, 4, 5, 6];
            let mut a2 = [0; 6];
    
            a2.copy_from(a1);
            assert_eq!(a2, a1);
        }
    
        #[test]
        fn test_immutable_static_index() {
            let arr = [1, 2, 3, 4, 5];
            assert_eq!(arr[StaticIndex::<4>], 5);
        }
    
        #[test]
        fn test_mutable_static_index() {
            let mut arr = [1, 2, 3, 4, 5];
            arr[StaticIndex::<4>] = 6;
            assert_eq!(arr, [1, 2, 3, 4, 6]);
        }
    }

    mod wrapper_functionality {
        use super::*;

        #[test]
        fn test_wrapped_slice_read_single() {
            let x = SliceWrapper::new(&[1, 2, 3]);
            assert_eq!(x[StaticIndex::<2>], 3);
        }

        #[test]
        fn test_wrapped_slice_write_single() {
            let mut x =  [1, 2, 3];
            let mut y = SliceWrapper::new(&mut x);
            y[StaticIndex::<2>] = 5;
            assert_eq!(x[2], 5);
        }

        #[test]
        fn test_wrapped_slice_read_multi() {
            let x = SliceWrapper::new(&[1, 2, 3]);
            assert_eq!(x[StaticRangeIndex::<0, 2>], [1, 2]);
        }

        #[test]
        fn test_wrapped_slice_write_multi() {
            let mut x =  [1, 2, 3];
            let mut y = SliceWrapper::new(&mut x);
            y[StaticRangeIndex::<0, 2>] = [3, 4];
            assert_eq!(x, [3, 4, 3]);
        }

        #[test]
        fn test_wrapped_vec_read() {
            let x = vec![1, 2, 3];
            let x = SliceWrapper::new(x);
            assert_eq!(x[StaticRangeIndex::<0, 2>], [1, 2]);
        }

        #[test]
        fn test_wrapped_vec_write() {
            let mut x = vec![1, 2, 3];
            let mut y = SliceWrapper::new(&mut x);
            y[StaticRangeIndex::<1, 2>] = [4, 5];

            assert_eq!(y[StaticRangeIndex::<0, 3>], [1, 4, 5]);
            assert_eq!(x[0..3], [1, 4, 5]);
        }
    }

    mod wrapper_safety {
        use super::*;

        #[test]
        #[should_panic]
        fn wrapped_slice_oob_read_should_panic() {
            let x = SliceWrapper::new(&[1, 2, 3]);
            let _ = x[StaticIndex::<3>];
        }
    
        #[test]
        #[should_panic]
        fn wrapped_slice_oob_write_should_panic() {
            let mut x = [1, 2, 3];
            let mut x = SliceWrapper::new(&mut x);
            x[StaticIndex::<3>] = 1337;
        }
        
        #[test]
        #[should_panic]
        fn wrapped_slice_oob_range_read_should_panic() {
            let x = SliceWrapper::new(&[1, 2, 3]);
            let _ = x[StaticRangeIndex::<0, 5>];
        }
    
        #[test]
        #[should_panic]
        fn wrapped_slice_oob_range_write_should_panic() {
            let mut x = [1, 2, 3];
            let mut x = SliceWrapper::new(&mut x);
            x[StaticRangeIndex::<0, 5>] = [2, 3, 4, 5, 6];
        }
    
        #[test]
        #[should_panic]
        fn wrapped_vec_oob_read_should_panic() {
            let x = SliceWrapper::new(vec![1, 2, 3]);
            let _ = x[StaticIndex::<3>];
        }
    
        #[test]
        #[should_panic]
        fn wrapped_vec_oob_write_should_panic() {
            let mut x = SliceWrapper::new(vec![1, 2, 3]);
            x[StaticIndex::<3>] = 1337;
        }
    
        #[test]
        #[should_panic]
        fn wrapped_vec_oob_range_read_should_panic() {
            let x = SliceWrapper::new(vec![1, 2, 3]);
            let _ = x[StaticRangeIndex::<0, 5>];
        }
    
        #[test]
        #[should_panic]
        fn wrapped_vec_oob_range_write_should_panic() {
            let mut x = SliceWrapper::new(vec![1, 2, 3]);
            x[StaticRangeIndex::<0, 5>] = [2, 3, 4, 5, 6];
        }
    }
}