# Step 2: FnBrand Refactor

This step refactors the function brands to use the new pointer hierarchy, enabling generic implementations over any pointer brand that supports unsized coercion.

## Goals

1.  Add `FnBrand<PtrBrand: RefCountedPointer>` struct to `fp-library/src/brands.rs`.
2.  Add `RcFnBrand` and `ArcFnBrand` type aliases.
3.  Create `fp-library/src/types/fn_brand.rs` with blanket implementations.
4.  Remove old `fp-library/src/types/rc_fn.rs` and `arc_fn.rs`.
5.  Update all code that referenced old brands.

## Technical Design

### Refactored CloneableFn Using RefCountedPointer

`CloneableFn` will be refactored to use `RefCountedPointer` as its foundation.

#### The Unsized Coercion Problem

**Problem**: `RefCountedPointer::cloneable_new` accepts `T` (sized), but `CloneableFn` needs to create `CloneableOf<dyn Fn(A) -> B>` (unsized).

**Solution**: We use the `UnsizedCoercible` and `SendUnsizedCoercible` traits defined in Step 1. These traits abstract the unsized coercion that Rust can only perform with concrete types.

### Implementation

```rust
// fp-library/src/types/fn_brand.rs

use crate::{
	brands::{FnBrand, RcBrand, ArcBrand},
	classes::{
		category::Category,
		cloneable_fn::CloneableFn,
		function::Function,
		semigroupoid::Semigroupoid,
		send_cloneable_fn::SendCloneableFn,
		pointer::{UnsizedCoercible, SendUnsizedCoercible},
	},
};

/// Blanket implementation of CloneableFn for any FnBrand<P> where P: UnsizedCoercible.
///
/// This enables third-party pointer brands to automatically get FnBrand support
/// by implementing the UnsizedCoercible trait.
///
/// Note: UnsizedCoercible requires 'static bound to satisfy Semigroupoid lifetime requirements.
impl<P: UnsizedCoercible> Function for FnBrand<P> {
	type Of<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B>;

	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
		P::coerce_fn(f)
	}
}

impl<P: UnsizedCoercible> CloneableFn for FnBrand<P> {
	type Of<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B>;

	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
		P::coerce_fn(f)
	}
}

impl<P: UnsizedCoercible> Semigroupoid for FnBrand<P> {
	fn compose<'a, B: 'a, D: 'a, C: 'a>(
		f: Self::Of<'a, C, D>,
		g: Self::Of<'a, B, C>,
	) -> Self::Of<'a, B, D> {
		P::coerce_fn(move |b| f(g(b)))
	}
}

impl<P: UnsizedCoercible> Category for FnBrand<P> {
	fn identity<'a, A>() -> Self::Of<'a, A, A> {
		P::coerce_fn(|a| a)
	}
}

// SendCloneableFn only for SendUnsizedCoercible (which extends UnsizedCoercible + SendRefCountedPointer)
impl<P: SendUnsizedCoercible> SendCloneableFn for FnBrand<P> {
	type SendOf<'a, A, B> = P::SendOf<dyn 'a + Fn(A) -> B + Send + Sync>;

	fn send_cloneable_fn_new<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<'a, A, B> {
		P::coerce_send_fn(f)
	}
}
```

### Relationship to RefCountedPointer

The `FnBrand<PtrBrand>` pattern demonstrates how library types build on `RefCountedPointer`:

```
RefCountedPointer (trait)
	│
	├── RcBrand (impl)
	│      └── FnBrand<RcBrand> → CloneableFn using Rc<dyn Fn>
	│
	└── ArcBrand (impl SendRefCountedPointer)
		   └── FnBrand<ArcBrand> → CloneableFn + SendCloneableFn using Arc<dyn Fn>
```

The `FnBrand` constraint requires `PtrBrand: RefCountedPointer` because:

1. **Clonability**: `CloneableFn::Of` must be `Clone` (satisfied by `CloneableOf`)
2. **Deref**: Function wrappers must deref to `dyn Fn` (satisfied by `Deref`)
3. **new factory**: Creating wrapped functions requires `cloneable_new` (via `UnsizedCoercible`)

## Checklist

- [x] Add `FnBrand<PtrBrand: RefCountedPointer>` struct to `fp-library/src/brands.rs`
- [x] Add `RcFnBrand` and `ArcFnBrand` type aliases
- [x] Create `fp-library/src/types/fn_brand.rs`
   - Implement blanket `Function`, `CloneableFn`, `Semigroupoid`, `Category` for `FnBrand<P: UnsizedCoercible>`
   - Implement blanket `SendCloneableFn` for `FnBrand<P: SendUnsizedCoercible>`
- [x] Remove old `fp-library/src/types/rc_fn.rs` and `arc_fn.rs`
- [x] Update all code that referenced old brands

### Phase 2 Tests

- [x] All existing `RcFnBrand` tests still pass
- [x] All existing `ArcFnBrand` tests still pass
- [x] `SendCloneableFn` tests pass for `FnBrand<ArcBrand>`
- [x] Compile-fail: `FnBrand<RcBrand>` cannot be used with `SendCloneableFn`
- [x] Semigroupoid associativity law
- [x] Category identity laws
