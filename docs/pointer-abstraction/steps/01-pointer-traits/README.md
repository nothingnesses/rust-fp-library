# Step 1: Pointer Trait Foundation

This step establishes the core trait hierarchy and brand definitions that underpin the entire pointer abstraction.

## Goals

1.  Define the `Pointer` base trait.
2.  Define the `RefCountedPointer` extension trait.
3.  Define the `SendRefCountedPointer` marker trait.
4.  Define `UnsizedCoercible` and `SendUnsizedCoercible` traits for function coercion.
5.  Implement these traits for `RcBrand` and `ArcBrand`.
6.  Add necessary Kind traits for HKT integration.

## Technical Design

### Pointer Type Class Hierarchy

The design uses a three-level trait hierarchy following the "Additional Associated Type" pattern established by `CloneableFn` → `SendCloneableFn`:

```
Pointer                    (base: Of<T> with Deref)
	│
	└── RefCountedPointer  (adds: CloneableOf<T> with Clone + Deref)
			│
			└── SendRefCountedPointer  (marker: CloneableOf<T> is Send+Sync)
```

#### Trait Definitions

```rust
// fp-library/src/classes/pointer.rs

use std::ops::Deref;

/// Base type class for heap-allocated pointers.
///
/// This is the minimal abstraction: any type that can wrap a value and
/// dereference to it. Does NOT require Clone — that's added by subtraits.
pub trait Pointer {
	/// The pointer type constructor.
	/// For `RcBrand`, this is `Rc<T>`. For `BoxBrand`, this would be `Box<T>`.
	type Of<T: ?Sized>: Deref<Target = T>;

	/// Wraps a sized value in the pointer.
	fn new<T>(value: T) -> Self::Of<T>
	where
		Self::Of<T>: Sized;
}

/// Extension trait for reference-counted pointers with shared ownership.
///
/// Adds `CloneableOf` associated type which is Clone + Deref. This follows
/// the pattern of `SendCloneableFn` adding `SendOf` to `CloneableFn`.
pub trait RefCountedPointer: Pointer {
	/// The cloneable pointer type constructor.
	/// For Rc/Arc, this is the same as `Of<T>`.
	type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;

	/// Wraps a sized value in a cloneable pointer.
	fn cloneable_new<T>(value: T) -> Self::CloneableOf<T>
	where
		Self::CloneableOf<T>: Sized;

	/// Attempts to unwrap the inner value if this is the sole reference.
	fn try_unwrap<T>(ptr: Self::CloneableOf<T>) -> Result<T, Self::CloneableOf<T>>;
}

/// Extension trait for thread-safe reference-counted pointers.
///
/// This follows the same pattern as `SendCloneableFn` extends `CloneableFn`,
/// adding a `SendOf` associated type with explicit `Send + Sync` bounds.
pub trait SendRefCountedPointer: RefCountedPointer {
	/// The thread-safe pointer type constructor.
	/// For `ArcBrand`, this is `Arc<T>` where `T: Send + Sync`.
	type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;

	/// Wraps a sized value in a thread-safe pointer.
	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
	where
		Self::SendOf<T>: Sized;
}
```

#### Unsized Coercion Traits

To support `FnBrand` extensibility for third-party pointer brands, we introduce traits that handle unsized coercion (e.g., `impl Fn` -> `dyn Fn`).

```rust
// fp-library/src/classes/pointer.rs (continued)

/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
pub trait UnsizedCoercible: RefCountedPointer {
	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	fn coerce_fn<'a, A, B>(
		f: impl 'a + Fn(A) -> B
	) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
}

/// Extension trait for pointer brands that can coerce to thread-safe `dyn Fn + Send + Sync`.
pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}
```

#### Free Functions

```rust
// fp-library/src/classes/pointer.rs (continued)

pub fn pointer_new<P: Pointer, T>(value: T) -> P::Of<T>
where
	P::Of<T>: Sized,
{
	P::new(value)
}

pub fn ref_counted_new<P: RefCountedPointer, T>(value: T) -> P::CloneableOf<T>
where
	P::CloneableOf<T>: Sized,
{
	P::cloneable_new(value)
}

pub fn send_ref_counted_new<P: SendRefCountedPointer, T: Send + Sync>(value: T) -> P::SendOf<T>
where
	P::SendOf<T>: Sized,
{
	P::send_new(value)
}
```

### Brand Definitions

```rust
// fp-library/src/brands.rs

use std::marker::PhantomData;
use crate::classes::pointer::RefCountedPointer;

/// Brand for `std::rc::Rc` reference-counted pointer.
pub struct RcBrand;

/// Brand for `std::sync::Arc` atomic reference-counted pointer.
pub struct ArcBrand;

/// Brand for `std::boxed::Box` unique ownership pointer.
pub struct BoxBrand;

/// Generic function brand parameterized by reference-counted pointer choice.
pub struct FnBrand<PtrBrand: RefCountedPointer>(PhantomData<PtrBrand>);

/// Type alias for Rc-based function wrapper (convenience).
pub type RcFnBrand = FnBrand<RcBrand>;

/// Type alias for Arc-based function wrapper (convenience).
pub type ArcFnBrand = FnBrand<ArcBrand>;
```

### Pointer Implementations

#### RcBrand Implementation

```rust
// fp-library/src/types/rc_ptr.rs

use crate::{
	brands::RcBrand,
	classes::pointer::{Pointer, RefCountedPointer, UnsizedCoercible},
};
use std::rc::Rc;

impl Pointer for RcBrand {
	type Of<T: ?Sized> = Rc<T>;

	fn new<T>(value: T) -> Rc<T> {
		Rc::new(value)
	}
}

impl RefCountedPointer for RcBrand {
	type CloneableOf<T: ?Sized> = Rc<T>;

	fn cloneable_new<T>(value: T) -> Rc<T> {
		Rc::new(value)
	}

	fn try_unwrap<T>(ptr: Rc<T>) -> Result<T, Rc<T>> {
		Rc::try_unwrap(ptr)
	}
}

impl UnsizedCoercible for RcBrand {
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Rc<dyn 'a + Fn(A) -> B> {
		Rc::new(f)
	}
}
```

#### ArcBrand Implementation

```rust
// fp-library/src/types/arc_ptr.rs

use crate::{
	brands::ArcBrand,
	classes::pointer::{Pointer, RefCountedPointer, SendRefCountedPointer, UnsizedCoercible, SendUnsizedCoercible},
};
use std::sync::Arc;

impl Pointer for ArcBrand {
	type Of<T: ?Sized> = Arc<T>;

	fn new<T>(value: T) -> Arc<T> {
		Arc::new(value)
	}
}

impl RefCountedPointer for ArcBrand {
	type CloneableOf<T: ?Sized> = Arc<T>;

	fn cloneable_new<T>(value: T) -> Arc<T> {
		Arc::new(value)
	}

	fn try_unwrap<T>(ptr: Arc<T>) -> Result<T, Arc<T>> {
		Arc::try_unwrap(ptr)
	}
}

impl SendRefCountedPointer for ArcBrand {
	type SendOf<T: ?Sized + Send + Sync> = Arc<T>;

	fn send_new<T: Send + Sync>(value: T) -> Arc<T> {
		Arc::new(value)
	}
}

impl UnsizedCoercible for ArcBrand {
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Arc<dyn 'a + Fn(A) -> B> {
		Arc::new(f)
	}
}

impl SendUnsizedCoercible for ArcBrand {
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Arc<dyn 'a + Fn(A) -> B + Send + Sync> {
		Arc::new(f)
	}
}
```

### HKT Integration & Kind Traits

To ensure full integration with the library's Higher-Kinded Type (HKT) machinery, the new types rely on specific `Kind` traits defined in `fp-library/src/kinds.rs`.

1.  **`Kind_140eb1e35dc7afb3`** (Signature: `type Of<'a, A, B>`)
    *   **Used By:** `FnBrand<P>`

2.  **`Kind_ad6c20556a82a1f0`** (Signature: `type Of<A>`)
    *   **Used By:** `Pointer` brands (`RcBrand`, `ArcBrand`)

3.  **`Kind` with Signature `type Of<'a, A>`** (Currently **MISSING**)
    *   **Used By:** `LazyBrand` (specifically for `SendDefer`)
    *   **Action Required:** The current `kinds.rs` defines `type Of<'a>` and `type Of<'a, A: 'a>: 'a`, but **not** the unbounded `type Of<'a, A>`. This specific `Kind` trait must be added to `kinds.rs` and implemented by `LazyBrand`.

## Checklist

- [x] **Add missing Kind trait** to `fp-library/src/kinds.rs`:
   - Add `def_kind! { type Of<'a, A>; }` to support `SendDefer`.
- [x] Create `fp-library/src/classes/pointer.rs`
   - Define `Pointer` base trait with `Of<T>` and `new`
   - Define `RefCountedPointer` extension with `CloneableOf<T>` and `cloneable_new`
   - Define `SendRefCountedPointer` marker trait
   - Define `UnsizedCoercible` trait for basic function coercion
   - Define `SendUnsizedCoercible` trait for thread-safe function coercion
   - Add free functions `pointer_new` and `ref_counted_new`
- [x] Add `RcBrand` and `ArcBrand` to `fp-library/src/brands.rs`
- [x] Create `fp-library/src/types/rc_ptr.rs` with `Pointer`, `RefCountedPointer`, and `UnsizedCoercible` impls for `RcBrand`
- [x] Create `fp-library/src/types/arc_ptr.rs` with `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`, `UnsizedCoercible`, and `SendUnsizedCoercible` impls for `ArcBrand`
- [x] Update module re-exports

### Phase 1 Tests

- [x] Unit test: `RcBrand::new` creates `Rc<T>`
- [x] Unit test: `ArcBrand::new` creates `Arc<T>`
- [x] Unit test: `RcBrand::cloneable_new` creates `Rc<T>`
- [x] Unit test: `ArcBrand::cloneable_new` creates `Arc<T>`
- [x] Unit test: `Clone` works for `RcBrand::CloneableOf<T>`
- [x] Unit test: `Clone` works for `ArcBrand::CloneableOf<T>`
- [x] Unit test: `Deref` works correctly for both
- [x] Unit test: `RcBrand::try_unwrap` returns `Ok(value)` when sole reference
- [x] Unit test: `RcBrand::try_unwrap` returns `Err(ptr)` when shared
- [x] Unit test: `ArcBrand::try_unwrap` returns `Ok(value)` when sole reference
- [x] Unit test: `ArcBrand::try_unwrap` returns `Err(ptr)` when shared
- [x] Compile-fail test: `Rc<T>` is `!Send`
- [x] Compile-success test: `Arc<T: Send + Sync>` is `Send + Sync`
