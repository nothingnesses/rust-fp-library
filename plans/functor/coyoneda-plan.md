# Coyoneda Improvement Plan

Date: 2026-03-31

This document defines the remaining work on the Coyoneda free functor, derived from
the analysis in `plans/functor/analysis/00-summary.md`. Items are grouped by status
and ordered by priority within each group.

---

## Already Resolved

These issues from the analysis are closed by the `CoyonedaExplicit` implementation
(`fp-library/src/types/coyoneda_explicit.rs`, committed 2026-03-31).

| Issue                            | Resolution                                                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| No map fusion                    | `CoyonedaExplicit::map` composes via `compose`; `lower` calls `F::map` once.                                       |
| Stack overflow from deep nesting | `CoyonedaExplicit` has no layered recursion; `lower` is a single call.                                             |
| Foldable lower-then-fold cost    | `CoyonedaExplicit::fold_map` composes the fold function with the accumulated function and folds `F B` in one pass. |
| Foldable requires `F: Functor`   | `CoyonedaExplicit::fold_map` requires only `F: Foldable`, matching PureScript.                                     |
| Hoist requires `F: Functor`      | `CoyonedaExplicit::hoist` applies the natural transformation to `F B` directly.                                    |
| Identity allocation in `lift`    | Already resolved in `Coyoneda` via `CoyonedaBase`; `CoyonedaExplicit::lift` stores no function box.                |
| Fn vs FnOnce                     | Confirmed correct: `Fn` is required for multi-element containers. No action.                                       |

The limitations that remain in `Coyoneda` (no map fusion, stack risk, `F: Functor` on
`Foldable` and `hoist`) are now documented as inherent to the trait-object encoding.
`CoyonedaExplicit` is the recommended path when those properties matter.

---

## Remaining Work

### 1. Add property-based and compile-fail tests to `Coyoneda` (Medium)

**File:** `fp-library/src/types/coyoneda.rs`

The functor law tests use single hardcoded inputs. The project testing strategy
(CLAUDE.md) requires property-based tests for type class laws.

**Tasks:**

- Add QuickCheck property tests for the functor laws:
  - Identity: `map(identity, lift(fa)).lower() == fa` for arbitrary `fa`.
  - Composition: `map(compose(f, g), lift(fa)).lower() == map(f, map(g, lift(fa))).lower()`
    for arbitrary `f`, `g`, `fa`.
- Add a QuickCheck property test for the `Foldable` consistency law:
  `fold_map(f, coyo)` equals `fold_map(f, coyo.lower())` on the lowered value.
- Add a compile-fail test (`tests/compile_fail/`) verifying that calling `lower` on
  a `Coyoneda<F, A>` where `F: !Functor` produces a clear error message.
- Add a test exercising `Coyoneda` with borrowed data (lifetime `'a` shorter than
  `'static`) to verify the lifetime parameterization compiles correctly.

**Note:** `CoyonedaExplicit` should receive the same property-based tests.

---

### 2. Reduce allocation in `Coyoneda::new` (Low)

**File:** `fp-library/src/types/coyoneda.rs`

`Coyoneda::new(f, fb)` currently creates 3 heap allocations by wrapping a
`CoyonedaBase` inside a `CoyonedaMapLayer`. A unified struct holding both `fb` and
`func` directly would need only 2. All five agents identified this. The design
document calls this struct `CoyonedaImpl`.

**Tasks:**

- Add a third `CoyonedaInner` implementor, `CoyonedaImpl<'a, F, B, A>`, that stores
  `fb: F::Of<'a, B>` and `func: Box<dyn Fn(B) -> A + 'a>` directly.
- Implement `CoyonedaInner::lower` for `CoyonedaImpl`: call `F::map(self.func, self.fb)`.
- Update `Coyoneda::new` to construct a `CoyonedaImpl` instead of a `CoyonedaMapLayer`
  wrapping a `CoyonedaBase`. This reduces `new` from 3 allocations to 2.
- `Coyoneda::lift` continues to use `CoyonedaBase` (1 allocation, no function box).

---

### 3. Implement `Clone` and `Send` support for `Coyoneda` (Medium)

**Files:** `fp-library/src/types/coyoneda.rs`, `fp-library/src/brands.rs`

`Box<dyn CoyonedaInner>` is not `Clone`, `Send`, or `Sync`. This blocks `Traversable`
and thread-safe use. `Semiapplicative` and `Semimonad` can be implemented now via
lower-delegate-lift (see item 4); both arguments are consumed by value so `Clone` is
not required. The recommended approach from four of five
agents is to parameterize over the pointer brand, following the `FnBrand<P>` and
`LazyBrand<Config>` patterns already established in the library.

**Design sketch:**

Introduce a `CoyonedaConfig` trait (analogous to `LazyConfig`) that associates a
pointer type and a function wrapper brand:

```
trait CoyonedaConfig {
    type Ptr<T: ?Sized>: Deref<Target = T>;
    type FnBrand: CloneableFn;
}

struct RcCoyonedaConfig;   // Ptr = Rc, FnBrand = RcFnBrand
struct ArcCoyonedaConfig;  // Ptr = Arc, FnBrand = ArcFnBrand
```

`SharedCoyoneda<'a, F, A, Config: CoyonedaConfig>` then wraps
`Config::Ptr<dyn CoyonedaInner<'a, F, A>>` and stores functions as
`Config::FnBrand::Of<'a, B, A>` (cloneable via `Rc`/`Arc`).

Type aliases `RcCoyoneda` and `ArcCoyoneda` cover the common cases.

**Tasks:**

- Define `CoyonedaConfig` trait and `RcCoyonedaConfig`/`ArcCoyonedaConfig` impls.
- Implement `SharedCoyoneda<'a, F, A, Config>` with `Clone` (always), `Send + Sync`
  (when `Config = ArcCoyonedaConfig`).
- Add brands `SharedCoyonedaBrand<F, Config>` with `impl_kind!`.
- Implement `Functor`, `Pointed`, and `Foldable` for `SharedCoyonedaBrand<F, Config>`.
- Implement `Traversable` for `SharedCoyonedaBrand<F, Config>` where `F: Traversable`.
- Implement `Semiapplicative` for `SharedCoyonedaBrand<F, Config>` where
  `F: Semiapplicative`.
- Add type aliases and re-exports to `types.rs`.

**Dependency:** None; can proceed independently of item 3.

---

### 4. Add missing type class instances to `Coyoneda` (Low-Medium)

**File:** `fp-library/src/types/coyoneda.rs`

Several instances are implementable now via lower-then-delegate, requiring only
`F: Functor` for lowering plus the corresponding constraint on `F`.

**Tasks (in priority order):**

- **`Debug`:** Implement `fmt::Debug` for `Coyoneda<'a, F, A>` with an opaque output
  (`Coyoneda { .. }`) that does not require lowering. This requires no bounds beyond
  `F: Kind`. An opaque impl is sufficient for development use and avoids forcing
  evaluation.
- **`Semiapplicative`:** Lower both sides, apply `F`'s instance, re-lift. Requires
  `F: Functor + Semiapplicative`. Implement for `CoyonedaBrand<F>`.
- **`Semimonad`:** Same pattern. Requires `F: Functor + Semimonad`.

**Deferred to item 4 (requires `Clone`):**

- `Traversable` - requires `SharedCoyoneda`.
- `Eq`, `Ord` - `PartialEq` takes `&self`, but `lower` consumes `self`; non-destructive
  lowering requires the shared variant.

**Note:** `CoyonedaExplicit` should receive `Debug` (opaque) in the same pass.
`Semiapplicative` and `Semimonad` are implemented as standalone methods (`apply` and
`bind`) rather than type class instances, because `CoyonedaExplicit` has no brand and
adding one would require fixing `B` in the brand, making a general `Pointed` impl
impossible. Both methods lower both sides, delegate to `F`, and re-lift the result
(resetting the fusion pipeline to the identity function). This is already done as of
the commit that added these methods. `Traversable` remains available immediately since
`CoyonedaExplicit` is already `Clone` when `F::Of<'a, B>: Clone` and the function is
cloneable (currently requires `FnBrand`-wrapped functions or a structural clone).

---

## Accepted Limitations (no action)

These issues were identified in the analysis but are accepted as inherent to the
trait-object encoding of `Coyoneda`. They are documented in the module-level
documentation.

| Issue                                          | Reason accepted                                                                 |
| ---------------------------------------------- | ------------------------------------------------------------------------------- |
| `Foldable` requires `F: Functor` in `Coyoneda` | dyn-compatibility prevents `fold_map_inner<M>`; `CoyonedaExplicit` solves this. |
| `hoist` requires `F: Functor` in `Coyoneda`    | dyn-compatibility prevents `hoist_inner<G>`; `CoyonedaExplicit` solves this.    |
| No `unCoyoneda`                                | Rank-2 types are not available in Rust.                                         |

---

## Work Order

| Priority | Item                                                   | Effort | Dependency |
| -------- | ------------------------------------------------------ | ------ | ---------- |
| 1        | Property-based and compile-fail tests                  | Medium | None       |
| 2        | Reduce allocation in `Coyoneda::new`                   | Small  | None       |
| 3        | `Debug` instance for `Coyoneda` and `CoyonedaExplicit` | Small  | None       |
| 4        | `Semiapplicative` and `Semimonad` via lowering         | Medium | None       |
| 5        | `SharedCoyoneda` (Clone/Send variant)                  | Large  | None       |
| 6        | `Traversable`, `Eq`, `Ord`                             | Medium | Item 5     |
