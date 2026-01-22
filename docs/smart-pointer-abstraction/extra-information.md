# Pointer Abstraction Extra Information

This document contains extra information about the Pointer Abstraction implementation.

## Design Decisions

### Trait Hierarchy: Why Pointer → RefCountedPointer → SendRefCountedPointer

The three-level trait hierarchy was chosen after careful analysis of naming and extensibility concerns.

#### Why Three Levels?

1. **`Pointer` (base)**: Minimal abstraction for any heap-allocated pointer — just `Deref<Target=T>` and `new`. This allows future `BoxBrand` support without reference counting.

2. **`RefCountedPointer` (extends Pointer)**: Adds `CloneableOf` associated type with `Clone` bound. This captures the key property of Rc/Arc: unconditional cheap cloning with shared state.

3. **`SendRefCountedPointer` (extends RefCountedPointer)**: Indicates thread safety. Like `SendClonableFn` which adds `SendOf`, this trait adds a `SendOf` associated type with explicit `Send + Sync` bounds. This is required because Rust's `for<T: Trait>` higher-ranked bounds syntax doesn't exist (only `for<'a>` works).

#### Naming Decision: `Pointer` + `RefCountedPointer`

After considering multiple options, the final names were chosen for:

| Name                    | Rationale                                        |
| ----------------------- | ------------------------------------------------ |
| `Pointer`               | Minimal, accurate descriptor for `new` + `Deref` |
| `RefCountedPointer`     | Precise — describes Rc/Arc's reference counting  |
| `SendRefCountedPointer` | Follows `SendClonableFn` naming pattern          |

### Why Additional Associated Type (CloneableOf) Instead of Marker Trait?

**Pattern**: Following `SendClonableFn`'s approach where subtraits add NEW associated types rather than marker traits.

**Reason**: Rust doesn't allow subtraits to strengthen bounds on inherited associated types:

```rust
// This DOES NOT work:
trait Pointer { type Of<T>: Deref; }
trait RefCountedPointer: Pointer { /* cannot add Clone to Of<T> */ }
```

By adding `CloneableOf`, `RefCountedPointer` can express "Clone + Deref" without modifying `Pointer::Of`.

## Challenges & Solutions

### Challenge 1: Unsized Coercion in ClonableFn

**Problem**: `RefCountedPointer::cloneable_new` accepts `T` (sized), but `ClonableFn` needs to create `CloneableOf<dyn Fn(A) -> B>` (unsized).

**Why this happens**: When you write `Rc::new(closure)`, Rust performs implicit unsized coercion because it knows the target type. But `cloneable_new` is generic and can't know the target type.

**Solution**: Use a macro to implement `ClonableFn` for `FnBrand<RcBrand>` and `FnBrand<ArcBrand>` separately. The macro explicitly calls `Rc::new` or `Arc::new`, allowing the coercion to happen.

```rust
macro_rules! impl_fn_brand {
	($ptr_brand:ty, $ptr_type:ident) => {
		impl ClonableFn for FnBrand<$ptr_brand> {
			type Of<'a, A, B> = $ptr_type<dyn 'a + Fn(A) -> B>;
			fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
				$ptr_type::new(f)  // Unsized coercion happens here
			}
		}
	};
}

impl_fn_brand!(RcBrand, Rc);
impl_fn_brand!(ArcBrand, Arc);
```

**Why not other solutions?**

- **nightly `CoerceUnsized`**: Would work but limits to nightly Rust
- **`cloneable_new_unsized` method**: Can't pass unsized values by value
- **Specialization**: Also nightly-only

### Challenge 2: Thread Safety Bounds

**Problem**: `Arc<T>` is `Send + Sync` when `T: Send + Sync`. But `RefCountedPointer` is generic and can't enforce this at the trait level. Rust's `for<T: Trait>` syntax does **not exist** (only `for<'a>` works for lifetimes).

**Solution**: Use `SendRefCountedPointer` with an explicit `SendOf` associated type, following the same pattern as `SendClonableFn` which adds `SendOf`:

```rust
/// Extension trait for thread-safe reference-counted pointers.
/// Adds SendOf associated type with explicit Send + Sync bounds.
pub trait SendRefCountedPointer: RefCountedPointer {
	type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;

	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
	where
		Self::SendOf<T>: Sized;
}

// Only ArcBrand implements this
impl SendRefCountedPointer for ArcBrand {
	type SendOf<T: ?Sized + Send + Sync> = Arc<T>;

	fn send_new<T: Send + Sync>(value: T) -> Arc<T> {
		Arc::new(value)
	}
}

// RcBrand does NOT implement SendRefCountedPointer
```

**Why this pattern?**

- Rust's `for<T: Trait>` syntax doesn't exist (only `for<'a>` works)
- Follows the established `SendClonableFn` pattern in this codebase
- The `T: Send + Sync` bound and `SendOf: Send + Sync` bound make the contract explicit

**Usage in constraints**:

```rust
// Require thread-safe reference-counted pointer
fn parallel_operation<P: SendRefCountedPointer, T: Send + Sync>(ptr: P::SendOf<T>) {
	std::thread::spawn(move || {
		// ptr is guaranteed Send + Sync
	});
}
```

### Challenge 3: Interaction with Once Brands and Function Brands

**Problem**: `OnceCellBrand` uses `std::cell::OnceCell` (not `Send`). `OnceLockBrand` uses `std::sync::OnceLock` (`Send + Sync`). Additionally, the function brand must match the pointer brand for thread safety. Invalid combinations would cause surprising behavior or silent performance issues:

- `Lazy<ArcBrand, OnceCellBrand, _, _>` — Arc is Send but OnceCell is not, defeating the purpose
- `Lazy<RcBrand, OnceLockBrand, _, _>` — Wastes OnceLock's synchronization overhead
- `Lazy<ArcBrand, OnceLockBrand, RcFnBrand, _>` — Pointer/function brand mismatch breaks thread safety

**Solution**: Enforce valid combinations at compile time with a 3-parameter marker trait:

```rust
/// Marker trait for valid Lazy pointer/once-cell/function-brand combinations.
pub trait ValidLazyCombination<PtrBrand, OnceBrand, FnBrand> {}

impl ValidLazyCombination<RcBrand, OnceCellBrand, RcFnBrand> for () {}
impl ValidLazyCombination<ArcBrand, OnceLockBrand, ArcFnBrand> for () {}
```

The `Lazy` struct includes this in its where clause:

```rust
pub struct Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>(...)
where
	PtrBrand: RefCountedPointer + ThunkWrapper,
	OnceBrand: Once,
	FnBrand: ClonableFn,
	(): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>;  // Compile-time enforcement
```

**Benefits**:

1. Invalid combinations fail immediately at `Lazy::new` with clear error
2. Users cannot accidentally create suboptimal configurations
3. The marker trait explicitly documents valid combinations
4. Third-party crates can add their own valid combinations if needed
5. Function brand is explicitly part of the type, enabling generic code over both local and thread-safe variants

### Challenge 4: SendClonableFn Integration

**Problem**: The existing `SendClonableFn` trait has a separate `SendOf` associated type. How does this integrate with `FnBrand<PtrBrand>`?

**Solution**: `SendClonableFn` is only implemented for `FnBrand<ArcBrand>`:

```rust
impl SendClonableFn for FnBrand<ArcBrand> {
	type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

	fn send_clonable_fn_new<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<'a, A, B> {
		Arc::new(f)
	}
}

// FnBrand<RcBrand> does NOT implement SendClonableFn
// because RcBrand does NOT implement SendRefCountedPointer
```

This maintains the same pattern: the extension trait is only implemented for thread-safe variants.

### Challenge 5: Why Not Just Use Pointer::Of for Lazy?

**Problem**: Why does `Lazy` use `RefCountedPointer::CloneableOf` instead of `Pointer::Of`?

**Reasoning**:

1. **Clone requirement**: `Lazy::clone()` must work unconditionally. The `CloneableOf` associated type guarantees `Clone` without requiring `T: Clone`.

2. **FnBrand constraint**: `FnBrand<P>` requires `P: RefCountedPointer`, not just `P: Pointer`. The thunk stored in `Lazy` must be clonable.

3. **Semantic consistency**: Using `CloneableOf` for both the outer wrapper and the thunk storage (via `FnBrand`) ensures both use the same pointer type (Rc or Arc).

**Alternative considered**: Could `Lazy` use `Pointer::Of` with a `where P::Of<T>: Clone` bound? Yes, but this would be more verbose and less clear about intent. The `RefCountedPointer` bound directly expresses "I need shared ownership semantics."

### Challenge 6: FnBrand Extensibility for Third-Party Pointer Brands

**Problem**: The `impl_fn_brand!` macro handles unsized coercion by explicitly calling `Rc::new` or `Arc::new`. Third-party crates implementing custom `RefCountedPointer` brands cannot automatically get `FnBrand<CustomBrand>` implementations.

**Why this happens**: Rust's unsized coercion (`impl Fn` → `dyn Fn`) requires the compiler to know the concrete target type at the call site. In generic code like `P::cloneable_new(f)`, the compiler can't perform this coercion.

**Solution**: Introduce a two-level trait hierarchy for unsized coercion: `UnsizedCoercible` for basic function coercion, and `SendUnsizedCoercible` (extending it) for thread-safe function coercion. This follows the same pattern as `ClonableFn` → `SendClonableFn` and eliminates the runtime panic for non-Send brands:

```rust
/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
pub trait UnsizedCoercible: RefCountedPointer {
	fn coerce_fn<'a, A, B>(
		f: impl 'a + Fn(A) -> B
	) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
}

/// Extension trait for thread-safe function coercion.
pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer {
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}

impl UnsizedCoercible for RcBrand {
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Rc<dyn 'a + Fn(A) -> B> {
		Rc::new(f)
	}
}
// Note: RcBrand does NOT implement SendUnsizedCoercible (Rc is !Send)

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

Now `FnBrand` can have a blanket implementation:

```rust
/// Blanket implementation of ClonableFn for any FnBrand<P> where P: UnsizedCoercible.
impl<P: UnsizedCoercible> ClonableFn for FnBrand<P> {
	type Of<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B>;

	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B> {
		P::coerce_fn(f)
	}
}

// SendClonableFn only for SendUnsizedCoercible
impl<P: SendUnsizedCoercible> SendClonableFn for FnBrand<P> {
	type SendOf<'a, A, B> = P::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;

	fn send_clonable_fn_new<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<'a, A, B> {
		P::coerce_fn_send(f)
	}
}
```

**Benefits of `UnsizedCoercible` over macro approach**:

1. **Automatic support**: Third-party brands just implement `UnsizedCoercible` and get `FnBrand` for free
2. **Type-safe**: The trait enforces correct method signatures
3. **Discoverable**: Users can see what they need to implement via trait bounds
4. **Composable**: Works with existing `RefCountedPointer` hierarchy

**Third-party usage**:

```rust
// In third-party crate:
use fp_library::{brands::*, classes::*, functions::*};

// Example 1: Single-threaded custom Rc (like RcBrand)
pub struct MyRcBrand;

impl Pointer for MyRcBrand { ... }
impl RefCountedPointer for MyRcBrand { ... }

// Just implement UnsizedCoercible to get FnBrand<MyRcBrand> support!
impl UnsizedCoercible for MyRcBrand {
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> MyRc<dyn 'a + Fn(A) -> B> {
		MyRc::new(f)
	}
}
// FnBrand<MyRcBrand> now implements ClonableFn (but NOT SendClonableFn)!

// Example 2: Thread-safe custom Arc (like ArcBrand)
pub struct MyArcBrand;

impl Pointer for MyArcBrand { ... }
impl RefCountedPointer for MyArcBrand { ... }
impl SendRefCountedPointer for MyArcBrand { ... }

// Implement both traits for thread-safe brands
impl UnsizedCoercible for MyArcBrand {
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> MyArc<dyn 'a + Fn(A) -> B> {
		MyArc::new(f)
	}
}

impl SendUnsizedCoercible for MyArcBrand {
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> MyArc<dyn 'a + Fn(A) -> B + Send + Sync> {
		MyArc::new(f)
	}
}
// FnBrand<MyArcBrand> now implements BOTH ClonableFn AND SendClonableFn!
```

## Efficiency Analysis

### Lazy Performance Characteristics

| Scenario           | Cost                                                 |
| ------------------ | ---------------------------------------------------- |
| Create             | `Rc/Arc::new(...)` ~20ns (heap allocation)           |
| Clone              | `Rc/Arc` clone ~3-5ns (reference count increment)    |
| Force (first)      | `Rc/Arc` deref + `OnceCell::get_or_init` + `thunk()` |
| Force (subsequent) | `Rc/Arc` deref + `OnceCell::get` + `clone()`         |

### Comparison with Old Value-Semantic Lazy

| Operation       | Old Lazy (Value)     | New Lazy (Shared)     |
| --------------- | -------------------- | --------------------- |
| Clone unforced  | ~1ns (copy OnceCell) | ~3-5ns (Rc/Arc clone) |
| Clone forced    | O(size of A)         | ~3-5ns                |
| Force 1 clone   | O(thunk)             | O(thunk)              |
| Force 2nd clone | O(thunk) again!      | O(1) - cached         |
| Force nth clone | O(n × thunk) total   | O(1) - all share      |

**Conclusion**: New shared semantics is more efficient when multiple clones exist, the thunk is expensive, or the value is large.

## Known Limitations

This section documents inherent limitations of the design that cannot be fully resolved without significant tradeoffs.

### Limitation 1: LazyError Loses Original Panic Payload Type

**What**: When a thunk panics, the panic payload is converted to `Arc<str>` and stored in `LazyError`. The original payload type (e.g., custom panic types) is lost.

**Why this happens**: The raw panic payload from `catch_unwind` is `Box<dyn Any + Send>`, which is `Send` but **not `Sync`**. Storing it directly would make `LazyError` `!Sync`, which would propagate to make `ArcLazy` `!Send` (since `Arc<T>` requires `T: Send + Sync` to be `Send`). Thread safety is essential for `ArcLazy`.

**What is lost**:

- The ability to re-panic with the original payload via `resume_unwind`
- Custom panic types that carry structured error information
- The ability to downcast to the original panic type

**What is preserved**:

- The panic message string (if the payload was `&str` or `String`)
- A generic message for non-string payloads ("non-string panic payload")
- Thread-safe access to error information via `ArcLazy`

**Workarounds**:

1. **Use string panic messages**: `panic!("descriptive message")` works best
2. **Include structured info in message**: `panic!("error code: {}, details: {}", code, details)`
3. **Log before panicking**: Log detailed error info before panicking if needed for debugging
4. **Avoid panics in thunks**: Return `Result<A, E>` from thunks instead of panicking

**Why this tradeoff was made**: Thread safety is a hard requirement for `ArcLazy` to be useful in concurrent code. Losing the original panic type affects debugging but doesn't affect correctness. Users who need rich error information should use `Result` types rather than panics.

### Limitation 2: `Lazy::force_cloned` Requires `A: Clone`

**What**: The `force_cloned(&self) -> Result<A, LazyError>` method requires `A: Clone` because it returns an owned value while keeping the cached value in the `Lazy`.

**Why this happens**: Shared memoization means multiple callers may need the value simultaneously. The `OnceCell` keeps the canonical value; callers receive clones.

**Impact**:

- Types that are expensive to clone (large `Vec`, complex structs) incur clone overhead
- Types that cannot be cloned (`!Clone` types) cannot use `force_cloned` (only `force`)

**Workarounds**:

1. **Use `force`**: Returns `Result<&A, LazyError>` without cloning
2. **Wrap in `Rc`/`Arc`**: `Lazy<..., Arc<ExpensiveType>>` makes cloning cheap
3. **Use `try_into_result`**: Extracts owned value when `Lazy` has single owner (does NOT require `A: Clone`)

**Example for expensive types**:

```rust
// Instead of:
use fp_library::{brands::*, classes::*, functions::*};
let lazy: RcLazy<Vec<u8>> = lazy_new::<RcLazyConfig, _>(...);
let vec = lazy_force_cloned::<RcLazyConfig, _>(&lazy)?;  // Clones the entire Vec

// Do this:
use fp_library::{brands::*, classes::*, functions::*};
let lazy: RcLazy<Arc<Vec<u8>>> = lazy_new::<RcLazyConfig, _>(
	clonable_fn_new::<RcFnBrand, _, _>(|_| Arc::new(vec![...]))
);
let arc_vec = lazy_force_cloned::<RcLazyConfig, _>(&lazy)?;  // Only clones the Arc (cheap)
```

**Why this tradeoff was made**: Shared memoization is the core semantic of `Lazy`. Removing `Clone` from `force_cloned` would require either:

- Taking `self` by value (destroying the `Lazy`, not shared)
- Returning `&A` only (covered by `force`)
- Unsafe transmutation (unsound)

The `Clone` requirement is explicit in the type signature, making the cost visible to users.

### Limitation 3: Recursive Lazy Evaluation Deadlocks (ArcLazy)

**What**: If a thunk recursively forces the same `ArcLazy` value, the program will **deadlock**. For `RcLazy`, this causes a panic instead.

**Why this happens**:

- `OnceLock::get_or_init` (used by `ArcLazy`) blocks on re-entry waiting for initialization
- `OnceCell::get_or_init` (used by `RcLazy`) panics on re-entry

**Example of problematic code**:

```rust
let lazy: Arc<ArcLazy<i32>> = Arc::new_cyclic(|weak| {
	let weak = weak.clone();
	lazy_new::<ArcLazyConfig, _>(send_clonable_fn_new::<ArcFnBrand, _, _>(move |_| {
		// Recursive force - DEADLOCK!
		let self_ref = weak.upgrade().unwrap();
		lazy_force::<ArcLazyConfig, _>(&*self_ref).unwrap_or(0) + 1
	}))
});
```

**Workarounds**:

1. **Avoid cyclic dependencies**: Structure code to avoid self-referential lazy values
2. **Use intermediate values**: Break cycles with non-lazy intermediate computations
3. **Explicit cycle detection**: Track forcing state manually if cycles are unavoidable

**Why this tradeoff was made**: Detecting cycles at runtime would require additional state (e.g., thread-local "currently forcing" set), adding overhead to every `force` call. Since recursive lazy evaluation is a programmer error (violates referential transparency), the design prioritizes performance for correct usage over error messages for incorrect usage.

## Alternatives Considered

### Alternative 1: Separate SendRefCountedPointer with Own Associated Type

**Description**: Like `SendClonableFn`, have `SendRefCountedPointer` define its own `SendOf` type.

```rust
trait SendRefCountedPointer: RefCountedPointer {
	type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;
	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>;
}
```

**Pros**:

- More explicit about thread-safe type
- Consistent with `SendClonableFn` pattern
- Type bounds are clear in the trait definition
- Required because `for<T: Trait>` syntax doesn't exist in Rust

**Cons**:

- `SendOf<T>` and `CloneableOf<T>` are the same type for Arc (just with different bounds)
- Slightly more API surface

**Decision**: ✅ Adopted - necessary because Rust's `for<T: Trait>` higher-ranked bounds don't exist. A marker trait with the invalid syntax would not compile.

### Alternative 2: No Pointer Trait, Just Refactor ClonableFn

**Description**: Keep separate `RcFnBrand`/`ArcFnBrand` but use them directly in `Lazy`.

**Pros**: Less new abstraction

**Cons**: Doesn't solve the problem of sharing semantics; FnBrand can't wrap arbitrary types

**Decision**: Rejected - need `RefCountedPointer` for `Lazy` to wrap `(OnceCell, Thunk)`

### Alternative 3: Keep Both Lazy Implementations

**Description**: Have `ValueLazy` (current) and `SharedLazy` (new).

**Pros**: No breaking change, both options available

**Cons**: Confusing API, maintenance burden, value semantics rarely useful

**Decision**: Rejected - clean break is better since backward compat isn't a goal

#### Detailed Analysis: Why Value-Semantic Lazy Has No Merit

The current implementation has a peculiar hybrid structure:

```rust
pub struct Lazy<'a, OnceBrand: Once, FnBrand: ClonableFn, A>(
	pub <OnceBrand as Once>::Of<A>,              // OnceCell<A> - DEEP cloned
	pub <FnBrand as ClonableFn>::Of<'a, (), A>,  // Rc<dyn Fn> - SHALLOW cloned (shared!)
);
```

This means cloning shares the thunk but not the memoization — the worst of both worlds:

| Behavior            | Value-Semantic Lazy    | Shared Lazy         | Direct Function Call |
| ------------------- | ---------------------- | ------------------- | -------------------- |
| Clone + force both  | Thunk runs **twice**   | Thunk runs **once** | N/A                  |
| Memory per clone    | OnceCell + Rc refcount | Rc refcount only    | None                 |
| Computation sharing | **None**               | Full                | None                 |

**Potential use cases examined:**

1. **"I want independent computations per clone"** → Just call the function directly. Lazy adds OnceCell overhead without benefit.

2. **"I want snapshot isolation for impure thunks"** → Side-effectful thunks violate referential transparency. This is a bug, not a feature.

3. **"I want to avoid Rc/Arc overhead"** → The thunk is already wrapped in Rc via ClonableFn, so you pay the overhead anyway.

4. **"I want thread-local memoization"** → Use `thread_local!` with `OnceCell` directly, or use shared `Lazy` with thread-local access patterns.

5. **"I want deterministic destruction order"** → Use explicit resource management or `Drop` guards.

**Conclusion**: Every legitimate use case is better served by either:

- Shared `Lazy` (for memoization with sharing)
- `OnceCell` directly (for simple one-time initialization)
- Direct function application (for independent computation)

The value-semantic `Lazy` is an accidental design — not useful, just confusing.

## Thread Safety Analysis

| Type Alias   | Pointer | OnceCell   | Send | Sync | Use Case        |
| ------------ | ------- | ---------- | ---- | ---- | --------------- |
| `RcLazy<A>`  | `Rc`    | `OnceCell` | ❌   | ❌   | Single-threaded |
| `ArcLazy<A>` | `Arc`   | `OnceLock` | ✅\* | ✅\* | Multi-threaded  |

\*When `A: Send + Sync`

**Invalid combinations** (would compile but not be thread-safe):

- `Lazy<ArcBrand, OnceCellBrand, _>` — Arc is Send but OnceCell is not
- `Lazy<RcBrand, OnceLockBrand, _>` — Wastes OnceLock's thread-safety

The type aliases and `LazyConfig` trait enforce valid combinations by design.

## Extensibility Strategy

The design explicitly supports future extensibility:

### What Works Now

| Brand      | `Pointer`   | `RefCountedPointer` | `SendRefCountedPointer` |
| ---------- | ----------- | ------------------- | ----------------------- |
| `RcBrand`  | ✅          | ✅                  | ❌                      |
| `ArcBrand` | ✅          | ✅                  | ✅                      |
| `BoxBrand` | ✅ (future) | ❌                  | N/A                     |

### Future Extensions (Out of Scope)

1. **`BoxBrand`**: Can implement `Pointer` only — `Box::clone()` requires `T: Clone`
2. **Custom allocators**: Third-party crates can implement the traits
3. **Weak reference support**: Could add `RefCountedPointer::downgrade()` later

```rust
// Example: future custom allocator support
impl Pointer for MyCustomRcBrand {
	type Of<T: ?Sized> = my_crate::CustomRc<T, MyAllocator>;

	fn new<T>(value: T) -> Self::Of<T> {
		my_crate::CustomRc::new_in(value, MyAllocator::default())
	}
}

impl RefCountedPointer for MyCustomRcBrand {
	type CloneableOf<T: ?Sized> = my_crate::CustomRc<T, MyAllocator>;

	fn cloneable_new<T>(value: T) -> Self::CloneableOf<T> {
		my_crate::CustomRc::new_in(value, MyAllocator::default())
	}
}
```

This allows the FP library's abstractions (Lazy, FnBrand, etc.) to work with custom allocators without library changes.

## References

- [Haskell's Data.Lazy](https://hackage.haskell.org/package/lazy)
- [PureScript's Data.Lazy](https://pursuit.purescript.org/packages/purescript-lazy)
- [std::rc::Rc documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html)
- [std::sync::Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [std::boxed::Box documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html)
- [std::borrow::Cow documentation](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [Existing SendClonableFn trait](../fp-library/src/classes/send_clonable_fn.rs)
- [Existing ClonableFn trait](../fp-library/src/classes/clonable_fn.rs)
- [Current Lazy implementation](../fp-library/src/types/lazy.rs)
