Utilities for enhanced slicing and indexing
===

## Introduction
The `static_slicing` library provides a set of helpful utilities for compile-time checked slicing and indexing.

## Background

Not interested in reading all this? Here's a TL;DR: slicing and indexing can be weird in some fairly common situations, so this library makes those things less weird. Now you can [skip to the Installation section.](https://github.com/ktkaufman03/static-slicing#installation)

I initially developed this library for use by the [Worcester Polytechnic Institute](https://wpi.edu)'s team in the 2023 [MITRE Embedded Capture the Flag](https://ectf.mitre.org) competition.

During development of the firmware that was to be installed on an embedded system, an interesting language pain point was identified: insufficient compile-time inference surrounding array slicing and indexing.

For example, this function fails to compile ([try it yourself](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=492f9d73611549f07b99d2b9df218791)):
```rs
fn test() {
	let a = [0u8, 1u8, 2u8, 3u8];
	let x = a[4];	// there are only 4 elements in `a`! no good...
}
```
Thankfully, the compiler knows that accessing index 4 of a 4-element _array_ will never succeed, and raises a `unconditional_panic` warning that is turned into an error by default.

However, this nearly-identical function compiles just fine, and crashes at runtime (again, [try it yourself](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=301fc4147a9fc2d4716c7bc6158fe4a1)):
```rs
fn test() {
	let a = &[0u8, 1u8, 2u8, 3u8];
	let x = a[4];	// there are only 4 elements in `a`! no good...
}
```

The compiler _knows_ that `a` is a reference to a 4-element array of `u8` - in other words, `&[u8; 4]` - yet is no longer able to indicate that anything is potentially wrong.

The compiler's analysis (or lack thereof) of slicing can also be problematic. This function, which a new user might expect to compile, actually _doesn't_ compile:
```rs
fn test() {
	let a = &[0u8, 1u8, 2u8, 3u8];
	let x: &[u8; 2] = &a[1..3];	// *x should be [a[1], a[2]], which is a 2-element array.
}
```

The compiler's explanation:
```
3 |     let x: &[u8; 2] = &a[1..3];
  |            --------   ^^^^^^^^ expected array `[u8; 2]`, found slice `[u8]`
  |            |
  |            expected due to this
  ```

This doesn't work, either:
```rs
fn test() {
	let a = &[0u8, 1u8, 2u8, 3u8];
	let x: &[u8; 2] = &a[1..3].into();
}
```

However, _this_ does:
```rs
fn test() {
	let a = &[0u8, 1u8, 2u8, 3u8];
	let x: &[u8; 2] = &a[1..3].try_into().unwrap();
}
```

I am not a huge fan of throwing `.try_into().unwrap()` at the end of everything, but I _am_ a huge fan of solving weird problems like this - that's why this library exists!

## Installation

To install the current version of `static_slicing`, add the following to the `dependencies` section of your `Cargo.toml` file:

```toml
static-slicing = "0.2.0"
```

`no_std` support can be enabled by disabling default features:

```toml
static-slicing = { version = "0.2.0", default-features = false }
```

**Note: This library requires Rust 1.59 or later.**

## How does it work?

Don't want to read this? [Skip to the Examples section.](https://github.com/ktkaufman03/static-slicing#examples)

This library introduces two new index types that leverage the power of [const generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics):
- `StaticIndex<INDEX>` for getting/setting _individual_ items, and;
- `StaticRangeIndex<START, LENGTH>` for getting/setting _ranges_ of items.

For fixed-size arrays (i.e., `[T; N]` where `T` is the element type and `N` is the array length), the library provides implementations of `Index` and `IndexMut` that accept both of the static index types. With [const panic](https://rust-lang.github.io/rfcs/2345-const-panic.html) support (and a bit of type system hacking), invalid indexing operations can be prevented from compiling altogether!

_To see exactly how this is done, look at the implementations of `IsValidIndex` and `IsValidRangeIndex`._

For other types (slices, `Vec`s, etc.), use of `SliceWrapper` is necessary. `SliceWrapper` is an unfortunate workaround for some issues with Rust's orphan rules that I encountered during development. Regardless, `SliceWrapper` is designed to appear as "normal" as possible. It can be passed to functions that accept slice references, and essentially appears to be the data it's wrapping.

## Performance

[Criterion](https://github.com/bheisler/criterion.rs) benchmarks are included for fixed-size array indexing. On my computer, which has a 12-core AMD Ryzen 9 5900X, no significant _negative_ performance difference was observed between the built-in indexing facilities and those provided by this library. That is to say, this won't make your code 200000x slower - in fact, any performance impact should be negligible at worst.

Here are the raw numbers from a single benchmark run:

| benchmark type | compile-time checked | runtime checked |
| -------------- | -------------------- | --------------- |
| single index   | 428.87 ps            | 428.07 ps       |
| range index    | 212.19 ps            | 212.69 ps       |

(In some other runs, the compile-time checked single index was _slightly faster_ than the runtime-checked single index.)

## Examples

Here are some examples of how this library can be used.

### Example 1: getting an element and a slice from a fixed-size array
```rs
use static_slicing::{StaticIndex, StaticRangeIndex};

fn main() {
	let x = [513, 947, 386, 1234];

	// get the element at index 3
	let y = x[StaticIndex::<3>];

	// get 2 elements starting from index 1
	let z: &[i32; 2] = &x[StaticRangeIndex::<1, 2>];
	// this also works:
	let z: &[i32] = &x[StaticRangeIndex::<1, 2>];

	// prints: y = 1234
	println!("y = {}", y);

	// prints: z = [947, 386]
	println!("z = {:?}", z);
}
```

### Example 2: mutating using static indexes
```rs
use static_slicing::{StaticIndex, StaticRangeIndex};

fn main() {
	let mut x = [513, 947, 386, 1234];

	assert_eq!(x[StaticIndex::<1>], 947);
	x[StaticIndex::<1>] = 1337;
	assert_eq!(x[StaticIndex::<1>], 1337);
	assert_eq!(x, [513, 1337, 386, 1234]);

	x[StaticRangeIndex::<2, 2>] = [7331, 4040];

	assert_eq!(x[StaticRangeIndex::<2, 2>], [7331, 4040]);
	assert_eq!(x, [513, 1337, 7331, 4040]);
}
```

### Example 3: accidental out-of-bounds prevention
```rs
use static_slicing::{StaticIndex, StaticRangeIndex};

fn main() {
	// read Background to understand why 
	// `x` being an array reference is important!
	let x = &[513, 947, 386, 1234];

	// this block compiles...
	{
		let y = x[5];
		let z: &[i32; 2] = &x[2..5].try_into().unwrap();
	}

	// ...but not this one!
	{
		let y = x[StaticIndex::<5>];
		let z = x[StaticRangeIndex::<2, 3>];
	}
}
```

### Example 4: reading from a `SliceWrapper`
```rs
use static_slicing::{StaticIndex, StaticRangeIndex, SliceWrapper};

fn main() {
	let x = SliceWrapper::new(&[513, 947, 386, 1234][..]);
	
	{
		let y = x[StaticIndex::<3>];
		let z = x[StaticRangeIndex::<2, 2>];

		// prints: y = 1234
		println!("y = {}", y);

		// prints: z = [386, 1234]
		println!("z = {:?}", z);
	}

	{
		// both of these would panic at runtime 
		// since they're performing invalid operations.
		let _ = x[StaticIndex::<5>];
		let _ = x[StaticRangeIndex::<2, 4>];
	}
}
```

### Example 5: writing to a `SliceWrapper`
```rs
use static_slicing::{StaticIndex, StaticRangeIndex, SliceWrapper};

fn main() {
	// SliceWrappers are mutable under the following conditions:
	// 1. the wrapper itself has been declared as mutable, AND;
	// 2. the wrapper owns its wrapped data OR contains 
	//	  a mutable reference to the data.
	let mut x = SliceWrapper::new(vec![513, 947, 386, 1234]);
	
	assert_eq!(x[StaticIndex::<3>], 1234);
	x[StaticIndex::<3>] = 1337;
	assert_eq!(x[StaticIndex::<3>], 1337);

	assert_eq!(x[StaticRangeIndex::<1, 2>], [947, 386]);
	x[StaticRangeIndex::<1, 2>] = [5555, 6666];
	assert_eq!(x[StaticRangeIndex::<1, 2>], [5555, 6666]);
}
```

## Limitations

There are a few limitations to be aware of:
- `rust-analyzer` does not seem to be able to show the errors generated by the library's use of const panics.
- `SliceWrapper` cannot perform compile-time checks under any circumstances, even if given an array reference with known length. This _may_ be blocked until specialization is stabilized.
- The very existence of `SliceWrapper` is a limitation (in my mind, at least), but it's unlikely to go away unless radical changes are made to the orphan rules and coherence enforcement. (The fundamental problem is that I can't implement `Index(Mut)` for both `[T; N]` and `[T]`.)

Otherwise, this should be a pretty easy library to deal with - it certainly made things a lot easier for my teammates and I!