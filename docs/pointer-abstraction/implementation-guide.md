# Pointer Abstraction Implementation Guide

This guide outlines the implementation plan for introducing a unified pointer type class hierarchy to the library. This is a breaking change designed to allow library types to be parameterized over the choice of smart pointer (such as Rc/Arc) and to enable shared memoization semantics for `Lazy`.

## Navigation

1. [Step 1: Pointer Trait Foundation](./steps/01-pointer-traits/README.md) - Defining the base traits and brands.
2. [Step 2: FnBrand Refactor](./steps/02-fn-brand-refactor/README.md) - Updating function brands to use the new pointer hierarchy.
3. [Step 3: Lazy Refactor](./steps/03-lazy-refactor/README.md) - Implementing the new shared-memoization `Lazy` type.
4. [Step 4: Integration & Polish](./steps/04-integration/README.md) - Cleanup, documentation, and final checks.
5. [Step 5: Concurrency Testing](./steps/05-concurrency-testing/README.md) - Verifying thread safety with Loom.

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

| File                           | Changes                                                           |
| ------------------------------ | ----------------------------------------------------------------- |
| `fp-library/src/kinds.rs`      | Add missing `type Of<'a, A>` kind trait                           |
| `fp-library/src/brands.rs`     | Add `RcBrand`, `ArcBrand`, `BoxBrand`, `FnBrand<P>`, type aliases |
| `fp-library/src/classes.rs`    | Re-export `pointer` module                                        |
| `fp-library/src/types.rs`      | Re-export new modules, remove old                                 |
| `fp-library/src/types/lazy.rs` | Complete rewrite with shared semantics using `RefCountedPointer`  |
| `fp-library/src/functions.rs`  | Re-export new free functions (`pointer_new`, `ref_counted_new`)   |

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

### Blockers

| Issue | Status | Resolution |
| ----- | ------ | ---------- |

### Open Questions

- 
