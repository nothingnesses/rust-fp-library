# Pointer Abstraction Implementation Guide

This guide outlines the implementation plan for introducing a unified pointer type class hierarchy to the library. This is a breaking change designed to allow library types to be parameterized over the choice of smart pointer (such as Rc/Arc) and to enable shared memoization semantics for `Lazy`.

## Navigation

1. [x] [Step 1: Pointer Trait Foundation](./steps/01-pointer-traits/README.md) - Defining the base traits and brands.
2. [x] [Step 2: FnBrand Refactor](./steps/02-fn-brand-refactor/README.md) - Updating function brands to use the new pointer hierarchy.
3. [x] [Step 3: Lazy Refactor](./steps/03-lazy-refactor/README.md) - Implementing the new shared-memoization `Lazy` type.
4. [x] [Step 4: Integration & Polish](./steps/04-integration/README.md) - Cleanup, documentation, and final checks.
5. [x] [Step 5: Concurrency Testing](./steps/05-concurrency-testing/README.md) - Verifying thread safety with Loom.

[This document](../architecture.md) contains coding guidelines and conventions to be adhered to during implementation.

[This document](./extra-information.md) contains extra information about the project, such as analysis, challenges and solutions, justifications for the design decisions and known limitations.

## Summary

The plan introduces a unified pointer type class hierarchy:

```
Pointer                         (base: Deref + new)
└── RefCountedPointer           (adds: Clone via CloneableOf)
	└── SendRefCountedPointer   (adds: Send + Sync marker)
```

This abstraction enables:

1. **Unified Rc/Arc selection** across multiple library types via `RefCountedPointer`
2. **Shared memoization semantics** for `Lazy` (Haskell-like behavior)
3. **Reduced code duplication** by building multiple types on a single foundation
4. **Future extensibility** for Box, custom allocators, or alternative smart pointers via the `Pointer` base trait

**Note**: This is a breaking change. Backward compatibility is not a goal; the focus is on the best possible design.

## Background & Motivation

### Conversation Context

This plan originated from a code review of `fp-library/src/types/lazy.rs`, which revealed:

1. **The current `Lazy` implementation is correct** but uses value semantics:
   - Cloning a `Lazy` creates a deep copy of the `OnceCell`
   - Each clone maintains independent memoization state
   - Forcing one clone does not affect others

2. **This differs from Haskell's lazy evaluation**:
   - In Haskell, all references to a thunk share memoization
   - Once forced, all references see the cached result
   - This enables efficient graph-based computation sharing

3. **To achieve Haskell-like semantics**, the `OnceCell` must be wrapped in a shared smart pointer (`Rc` or `Arc`)

4. **The existing library already has similar patterns**:
   - `ClonableFn` abstracts over `RcFnBrand` vs `ArcFnBrand`
   - Users choose at call sites: `clonable_fn_new::<RcFnBrand, _, _>(...)`
   - This pattern can be generalized and unified

5. **`SendClonableFn` extends `ClonableFn`** with thread-safe semantics:
   - Uses a separate `SendOf` associated type
   - Only `ArcFnBrand` implements it (not `RcFnBrand`)
   - This pattern inspires the `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` hierarchy

### Current Architecture Gap

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CURRENT: Ad-hoc Rc/Arc Abstraction                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ClonableFn ─────extends───▶ SendClonableFn                                │
│       │                            │                                        │
│   ┌───┴───┐                    ┌───┘                                        │
│   │       │                    │                                            │
│ RcFnBrand ArcFnBrand ◀─────────┘  (only Arc implements SendClonableFn)      │
│                                                                             │
│                                                                             │
│   Lazy (current)                                                            │
│       │                                                                     │
│   Uses OnceBrand (not shared across clones)                                 │
│                                                                             │
│   Problem: Rc/Arc choice is duplicated; no shared foundation                │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     PROPOSED: Unified Pointer Hierarchy                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Pointer                      (base: Of<T> + new)                           │
│     │                                                                       │
│     ├── BoxBrand              (future: unique ownership)                    │
│     │                                                                       │
│     └── RefCountedPointer     (adds: CloneableOf<T> + cloneable_new)        │
│            │                                                                │
│            ├── RcBrand                                                      │
│            │                                                                │
│            └── SendRefCountedPointer  (marker for Send + Sync)              │
│                   │                                                         │
│                   └── ArcBrand                                              │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Library types use RefCountedPointer for shared semantics:             │  │
│  │                                                                       │  │
│  │  FnBrand<P: RefCountedPointer>                                        │  │
│  │    - Uses P::CloneableOf for clonable function wrappers               │  │
│  │    - Implements ClonableFn, SendClonableFn (when P: SendRefCounted)   │  │
│  │                                                                       │  │
│  │  Lazy<Config, A> where Config: LazyConfig                             │  │
│  │    - Config bundles PtrBrand, OnceBrand, FnBrand, ThunkOf             │  │
│  │    - Uses Config::PtrBrand::CloneableOf for shared memoization        │  │
│  │    - All clones share the same OnceCell                               │  │
│  │    - force returns Result for panic safety                            │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  Type Aliases (for convenience):                                            │
│    RcFnBrand  = FnBrand<RcBrand>                                            │
│    ArcFnBrand = FnBrand<ArcBrand>                                           │
│    RcLazy<'a, A>  = Lazy<'a, RcLazyConfig, A>                               │
│    ArcLazy<'a, A> = Lazy<'a, ArcLazyConfig, A>                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Design Goals

### Primary Goals

1. **Introduce `Pointer` trait** as a minimal base abstraction for all heap-allocated pointers
2. **Introduce `RefCountedPointer` trait** extending `Pointer` with `CloneableOf` for shared ownership (Rc/Arc)
3. **Introduce `SendRefCountedPointer` marker** for thread-safe reference counting (Arc only)
4. **Refactor `ClonableFn` to use `RefCountedPointer`** via `FnBrand<PtrBrand>` pattern
5. **Create `Lazy` type** with Haskell-like shared memoization semantics using `RefCountedPointer`
6. **Enable future extensibility** - Box support via `Pointer` without `RefCountedPointer`

### Non-Goals

1. **Backward compatibility** - this is a breaking change; best design takes priority
2. **Migration path** - not needed since we're not maintaining backward compat
3. **Implementing Box/UniquePointer now** - the `Pointer` base is established but Box impl deferred
4. **Automatic selection** of Rc vs Arc based on context (user explicitly chooses)

## Files to Create

| File                                      | Purpose                                                                            |
| ----------------------------------------- | ---------------------------------------------------------------------------------- |
| `fp-library/src/classes/pointer.rs`       | `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`, `UnsizedCoercible` traits |
| `fp-library/src/classes/try_semigroup.rs` | `TrySemigroup` trait for fallible combination                                      |
| `fp-library/src/classes/try_monoid.rs`    | `TryMonoid` trait extending `TrySemigroup`                                         |
| `fp-library/src/classes/send_defer.rs`    | `SendDefer` trait extending `Defer` with `Send + Sync` thunk bounds                |
| `fp-library/src/types/rc_ptr.rs`          | `Pointer` + `RefCountedPointer` + `UnsizedCoercible` impl for `RcBrand`            |
| `fp-library/src/types/arc_ptr.rs`         | All four traits impl for `ArcBrand`                                                |
| `fp-library/src/types/fn_brand.rs`        | `FnBrand<PtrBrand>` blanket implementations                                        |
| `fp-library/tests/loom_tests.rs`          | Loom-based concurrency tests for `ArcLazy`                                         |

## Files to Modify

| File                             | Changes                                                           |
| -------------------------------- | ----------------------------------------------------------------- |
| `fp-library/src/kinds.rs`        | Add missing `type Of<'a, A>` kind trait                           |
| `fp-library/src/brands.rs`       | Add `RcBrand`, `ArcBrand`, `BoxBrand`, `FnBrand<P>`, type aliases |
| `fp-library/src/classes.rs`      | Re-export `pointer` module                                        |
| `fp-library/src/types.rs`        | Re-export new modules, remove old                                 |
| `fp-library/src/types/lazy.rs`   | Complete rewrite with shared semantics using `RefCountedPointer`  |
| `fp-library/src/functions.rs`    | Re-export new free functions (`pointer_new`, `ref_counted_new`)   |
| `fp-library/src/classes/defer.rs`| Update doc examples for new `Lazy` API (added during impl)        |

## Files to Delete

| File                             | Reason                    |
| -------------------------------- | ------------------------- |
| `fp-library/src/types/rc_fn.rs`  | Replaced by `fn_brand.rs` |
| `fp-library/src/types/arc_fn.rs` | Replaced by `fn_brand.rs` |

## Notes

_Add implementation notes, decisions, and blockers here as work progresses._

### Decisions Made

| Date | Decision | Rationale |
| ---- | -------- | --------- |
| 2026-01-22 | `UnsizedCoercible` requires `'static` bound | `coerce_fn` is generic over `'a`, so the resulting pointer type must be valid for any `'a`, implying the brand must be `'static`. This is required for `Semigroupoid` implementation. |
| 2026-01-22 | `SendUnsizedCoercible` returns `SendOf` | `SendClonableFn` requires `SendOf` to be `Send + Sync`. `CloneableOf` does not guarantee this for generic pointers, so `SendUnsizedCoercible` must return `SendOf` which has the correct bounds. |
| 2026-01-22 | `LazyConfig` adds associated functions | Added `new_thunk`, `new_send_thunk`, `into_thunk` and bounds (`Deref`, `A: 'a`, `: 'static`) to `LazyConfig`/`SendLazyConfig` to support generic thunk construction and usage. |
| 2026-01-22 | `TrySemigroup` blanket impl for `Lazy` | Relied on blanket implementations for `Lazy` instead of explicit ones, as `Lazy` composition is infallible. `try_append` returns `Ok(Lazy)`, deferring panics to `force`. |
| 2026-01-22 | `ArcBrand` uses `std::sync::Mutex` | Used `std::sync::Mutex` instead of `parking_lot::Mutex` for `ThunkWrapper` to avoid adding a new dependency. |
| 2026-01-22 | `LazyBrand` struct bound omitted | Omitted `Config: LazyConfig` bound on `LazyBrand` struct definition to follow Rust best practices (avoiding unnecessary bounds on structs). Bound is enforced in `impl_kind!`. |
| 2026-01-22 | Added `thiserror` dependency | Added `thiserror` to support `#[derive(Error)]` for `LazyError`. |
| 2026-01-22 | `SendDefer` extends `Kind` | `SendDefer` extends `Kind` instead of `Defer` because `Defer::defer` requires generic `FnBrand` support, which is incompatible with `ArcLazy`'s `Send + Sync` thunk requirement. |
| 2026-01-22 | `RefCountedPointer` adds `try_unwrap` | Added `try_unwrap` to `RefCountedPointer` to support safe unwrapping and testing of reference counts. |
| 2026-01-22 | `Lazy` adds convenience methods | Added `force_or_panic`, `force_ref_or_panic`, `force_cloned`, `is_poisoned`, and `get_error` to `Lazy` for better ergonomics and safety. |
| 2026-01-22 | `ThunkWrapper` placed in `pointer.rs` | Moved `ThunkWrapper` trait from `lazy.rs` (as shown in Step 3 plan) to `pointer.rs` for better cohesion with other pointer traits. This keeps all pointer-related abstractions in one module. |
| 2026-01-22 | `SendLazyConfig` replaced by helper traits | Instead of `SendLazyConfig` extending `LazyConfig` with `SendThunkOf`, implemented separate `LazySemigroup`, `LazyMonoid`, and `LazyDefer` traits. This provides better separation of concerns and allows `ArcLazyConfig` to constrain `A: Send + Sync` only where needed. |
| 2026-01-22 | `defer.rs` doc examples updated | File not in original "Files to Modify" list. Required update because the `Lazy` API changed significantly: type from `Lazy<OnceBrand, FnBrand, A>` to `Lazy<'a, Config, A>`, constructor from `Lazy::new(clonable_fn_new(...))` to `RcLazy::new(RcLazyConfig::new_thunk(...))`, and `force` from returning `A` to `Result<&A, LazyError>`. |

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

- 
