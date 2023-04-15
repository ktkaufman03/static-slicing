#![cfg_attr(not(feature = "std"), no_std)]
// Disable Clippy's let_unit_value warning because if we take its suggestion
// we just get a different warning.
#![allow(clippy::let_unit_value)]

//! Provides utilities for emulating statically-checked array slicing and copying.
//!
//! The [`StaticRangeIndex`] type can be used as an index into fixed-size arrays
//! to get or set a fixed-size slice. Likewise, the [`StaticIndex`] type can be used 
//! to get or set the element at a given index.
//! 
//! The static index types can be used to index other collections, such as slices and [`Vec`]s,
//! but only when they are wrapped in a [`SliceWrapper`]. Due to Rust's orphan rule and lack of
//! specialization support, this seems to be the best we can do - it's apparently impossible to
//! implement [`Index`] for both `[T; N]` and `[T]`.
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
//! let arr = [3, 5, 7, 9];
//! let value = arr[StaticIndex::<2>];
//! assert_eq!(value, 7);
//! ```
//! 
//! This example demonstrates how to obtain a 2-element slice of a 4-element [`Vec`] 
//! starting from index 0.
//! ```
//! use static_slicing::{SliceWrapper, StaticRangeIndex};
//! let x = vec![1, 2, 3];
//! let x = SliceWrapper::new(x);
//! assert_eq!(x[StaticRangeIndex::<0, 2>], [1, 2]);
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
//! 
//! The following examples demonstrate the runtime safety guarantees of the static slicing framework.
//! Note that when fixed-size arrays are used, there is zero cost at runtime, since all checks are done
//! at compile time.
//! 
//! ```should_panic
//! use static_slicing::{SliceWrapper, StaticIndex};
//! let x = SliceWrapper::new(vec![1, 2, 3]);
//! let _ = x[StaticIndex::<3>];
//! ```
//! ```should_panic
//! use static_slicing::{SliceWrapper, StaticIndex};
//! let mut x = SliceWrapper::new(vec![1, 2, 3]);
//! x[StaticIndex::<3>] = 5;
//! ```
use core::{
    marker::PhantomData, 
    ops::{Index, IndexMut, Deref, DerefMut},
};

/// Internal helper trait for static indexing.
///
/// [`IsValidIndex::RESULT`] must evaluate to `()` if the index is valid,
/// or panic otherwise.
trait IsValidIndex<const INDEX: usize> {
    const RESULT: ();
}

/// A single-element index that exists entirely at compile time.
pub struct StaticIndex<const INDEX: usize>;

impl<T, const INDEX: usize, const N: usize> IsValidIndex<INDEX> for [T; N] {
    const RESULT: () = assert!(N > INDEX, "Index is out of bounds!");
}

impl<T, const INDEX: usize, const N: usize> Index<StaticIndex<INDEX>> for [T; N] {
    type Output = T;

    fn index(&self, _: StaticIndex<INDEX>) -> &Self::Output {
        let _ = <[T; N] as IsValidIndex<INDEX>>::RESULT;

        // SAFETY: We've verified bounds at compile time.
        unsafe { &*(self.as_ptr().add(INDEX) as *const T) }
    }
}

impl<T, const INDEX: usize, const N: usize> IndexMut<StaticIndex<INDEX>> for [T; N] {
    fn index_mut(&mut self, _: StaticIndex<INDEX>) -> &mut Self::Output {
        let _ = <[T; N] as IsValidIndex<INDEX>>::RESULT;

        // SAFETY: We've verified bounds at compile time.
        unsafe { &mut *(self.as_mut_ptr().add(INDEX) as *mut T) }
    }
}

/// Internal helper trait for static range indexing.
///
/// [`IsValidRangeIndex::RESULT`] must evaluate to `()` if the range is valid,
/// or panic otherwise.
trait IsValidRangeIndex<const START: usize, const LENGTH: usize> {
    const RESULT: ();
}

/// A range index that exists entirely at compile time.
/// Range indexes can be used to obtain fixed-size slices.
/// For any pair of `(START, LENGTH)`, the range covered is `[START, START+LENGTH)`.
pub struct StaticRangeIndex<const START: usize, const LENGTH: usize>;

impl<T, const START: usize, const LENGTH: usize, const N: usize> IsValidRangeIndex<START, LENGTH>
    for [T; N]
{
    const RESULT: () = {
        assert!(N > START, "Starting index is out of bounds!");
        assert!(N - START >= LENGTH, "Ending index is out of bounds! Check your range index's length.");
    };
}

impl<T, const START: usize, const LENGTH: usize, const N: usize>
    Index<StaticRangeIndex<START, LENGTH>> for [T; N]
{
    type Output = [T; LENGTH];

    fn index(&self, _: StaticRangeIndex<START, LENGTH>) -> &Self::Output {
        let _ = <[T; N] as IsValidRangeIndex<START, LENGTH>>::RESULT;

        // SAFETY: We've verified bounds at compile time.
        unsafe { &*(self.as_ptr().add(START) as *const [T; LENGTH]) }
    }
}

impl<T, const START: usize, const LENGTH: usize, const N: usize>
    IndexMut<StaticRangeIndex<START, LENGTH>> for [T; N]
{
    fn index_mut(&mut self, _: StaticRangeIndex<START, LENGTH>) -> &mut Self::Output {
        let _ = <[T; N] as IsValidRangeIndex<START, LENGTH>>::RESULT;

        // SAFETY: We've verified bounds at compiile time.
        unsafe { &mut *(self.as_mut_ptr().add(START) as *mut [T; LENGTH]) }
    }
}

/// Wrapper around slice references to add support for
/// the static index types.
/// 
/// Due to language weirdness, we can't implement [`Index`] and [`IndexMut`]
/// with static indexes for both `[T]` and `[T; N]`. As a result, we need this 
/// wrapper type.
/// 
/// For convenience, [`SliceWrapper`] also implements [`Deref`] and [`DerefMut`].
/// This allows [`SliceWrapper`] instances to be converted to `&[T]`, in addition to
/// allowing all the underlying functions of `T` to be used.
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

impl<'a, I, T> Deref for SliceWrapper<'a, I, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, I, T> DerefMut for SliceWrapper<'a, I, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I, S: AsRef<[I]>, const START: usize, const LENGTH: usize> Index<StaticRangeIndex<START, LENGTH>> for SliceWrapper<'_, I, S> {
    type Output = [I; LENGTH];

    fn index(&self, _: StaticRangeIndex<START, LENGTH>) -> &Self::Output {
        let inner: &[I] = self.0.as_ref();

        assert!(inner.len() > START, "Starting index {} is out of bounds", START);
        assert!(inner.len() - START >= LENGTH, "Not enough items after index {} (requested {}; length: {})", START, LENGTH, inner.len());

        // SAFETY: We've verified bounds at runtime.
        unsafe { &*(inner.as_ptr().add(START) as *const [I; LENGTH]) }
    }
}

impl<I, S: AsRef<[I]>, const INDEX: usize> Index<StaticIndex<INDEX>> for SliceWrapper<'_, I, S> {
    type Output = I;

    fn index(&self, _: StaticIndex<INDEX>) -> &Self::Output {
        self.0.as_ref().index(INDEX)
    }
}

impl<I, S: AsRef<[I]> + AsMut<[I]>, const START: usize, const LENGTH: usize> IndexMut<StaticRangeIndex<START, LENGTH>> for SliceWrapper<'_, I, S> {
    fn index_mut(&mut self, _: StaticRangeIndex<START, LENGTH>) -> &mut Self::Output {
        let inner: &mut [I] = self.0.as_mut();

        assert!(inner.len() > START, "Starting index {} is out of bounds", START);
        assert!(inner.len() - START >= LENGTH, "Not enough items after index {} (requested {}; length: {})", START, LENGTH, inner.len());

        // SAFETY: We've verified bounds at runtime.
        unsafe { &mut *(inner.as_mut_ptr().add(START) as *mut [I; LENGTH]) }
    }
}

impl<I, S: AsRef<[I]> + AsMut<[I]>, const INDEX: usize> IndexMut<StaticIndex<INDEX>> for SliceWrapper<'_, I, S> {
    fn index_mut(&mut self, _: StaticIndex<INDEX>) -> &mut Self::Output {
        self.0.as_mut().index_mut(INDEX)
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
            assert_eq!(x.len(), 3);
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
            assert_eq!(x.len(), 3);
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