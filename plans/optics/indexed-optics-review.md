# Indexed Optics Implementation Review

## Overview

This document analyses the indexed optics implementation (diff from `5757637b` to `8630bc4`) against the plan in `plans/optics/indexed-optics.md` and the PureScript reference in `purescript-profunctor-lenses/src/Data/Lens/Indexed.purs`.

**Build status:** `cargo check --workspace` passes. `cargo test --doc` fails (doc example issues, not analysed here).

---

## 1. Correctly Implemented Components

### 1.1 `Indexed` Struct & `IndexedBrand` (Phase 1) — Correct

The core `Indexed<'a, P, I, A, B>` struct and `IndexedBrand<P, I>` are correctly implemented in `types/optics/indexed.rs`. The profunctor instances faithfully follow both the plan and the PureScript semantics:

| Instance | Status | Notes |
|----------|--------|-------|
| `Profunctor` | Correct | `dimap(f, g, Indexed(p)) = Indexed(dimap(|(i,a)| (i, f(a)), g, p))` — index untouched |
| `Strong::first` | Correct | `|(i, (a, c))| ((i, a), c)` rearrangement matches plan |
| `Strong::second` | Correct | `|(i, (c, a))| (c, (i, a))` rearrangement matches plan |
| `Choice::left` | Correct | Index follows the `Err` branch (matching `Left` in PS convention) |
| `Choice::right` | Correct | Index follows the `Ok` branch |
| `Wander` | Correct | `IWanderAdapter` threads index to each element; `I: Clone` required |

### 1.2 Indexed Optic Traits (Phase 2) — Correct

All five indexed optic traits in `classes/optics.rs` match the plan:

- `IndexedLensOptic<'a, I, S, T, A, B>` — requires `P: Strong`
- `IndexedTraversalOptic<'a, I, S, T, A, B>` — requires `P: Wander`
- `IndexedGetterOptic<'a, I, S, A>` — uses `ForgetBrand<P, R>`
- `IndexedFoldOptic<'a, I, S, A>` — uses `ForgetBrand<P, R>` with `R: Monoid`
- `IndexedSetterOptic<'a, P, I, S, T, A, B>` — uses `FnBrand<P>`

The helper traits `IndexedOpticAdapter` and `IndexedOpticAdapterDiscardsFocus` are also correctly defined.

### 1.3 `IndexedTraversalFunc` Trait (Phase 2) — Correct

Defined in `classes/optics/indexed_traversal.rs`. Matches the plan signature.

### 1.4 Concrete Indexed Optic Structs (Phase 3) — Correct

All six structs are implemented and follow the plan's patterns:

- **`IndexedLens` / `IndexedLensPrime`** (`indexed_lens.rs`): Stores `S -> ((I, A), B -> T)` via `FnBrand<P>`. Implements all five indexed optic traits plus both adapter traits. The `evaluate` method correctly uses `Q::dimap(to, |(b,f)| f(b), Q::first(pab.inner))`.

- **`IndexedGetter` / `IndexedGetterPrime`** (`indexed_getter.rs`): Stores `S -> (I, A)` via `FnBrand<P>`. Implements `IndexedGetterOptic` and `IndexedFoldOptic`. `IndexedGetterPrime` is a type alias (correct, since getters are inherently monomorphic in the focus).

- **`IndexedFold` / `IndexedFoldPrime`** (`indexed_fold.rs`): Wraps `IndexedFoldFunc`. Correctly implements `IndexedFoldOptic` by cloning `pab.inner.0` and delegating to the fold function.

- **`IndexedSetter` / `IndexedSetterPrime`** (`indexed_setter.rs`): Wraps `IndexedSetterFunc`. The `evaluate` implementation correctly wraps `pab.inner` into `Box<dyn Fn(I, A) -> B>` and passes to the setter function.

- **`IndexedTraversal` / `IndexedTraversalPrime`** (`indexed_traversal.rs`): Wraps `IndexedTraversalFunc`. The `evaluate` method correctly bridges to `Wander` via a local `IWanderAdapter` struct that converts `IndexedTraversalFunc<I, S, T, A, B>` into `TraversalFunc<S, T, (I, A), B>`.

### 1.5 Composition (Phase 5) — Correct

All five `Composed` indexed impls in `composed.rs` are correct:

- `IndexedLensOptic` (O1: `LensOptic`, O2: `IndexedLensOptic`)
- `IndexedTraversalOptic` (O1: `TraversalOptic`, O2: `IndexedTraversalOptic`)
- `IndexedGetterOptic` (O1: `GetterOptic`, O2: `IndexedGetterOptic`)
- `IndexedFoldOptic` (O1: `FoldOptic`, O2: `IndexedFoldOptic`)
- `IndexedSetterOptic` (O1: `SetterOptic`, O2: `IndexedSetterOptic`)

Turbofish `::<P>` is correctly used on `self.first.evaluate::<P>(pmn)` for `IndexedLensOptic` and `IndexedTraversalOptic`, and `::<R, P>` for getter/fold. The setter variant does not need turbofish (concrete `FnBrand<Q>`).

### 1.6 WithIndex Type Classes (Phase 6) — Correct

- `FunctorWithIndex<I>` in `classes/functor_with_index.rs`
- `FoldableWithIndex<I>` in `classes/foldable_with_index.rs`
- `TraversableWithIndex<I>` in `classes/traversable_with_index.rs`

Trait signatures match the plan. Supertraits (`Functor`, `Foldable`, `Traversable`) are correct.

### 1.7 `VecBrand` / `OptionBrand` WithIndex Impls (Phase 6) — Correct

- `VecBrand: FunctorWithIndex<usize>` — uses `enumerate()`
- `VecBrand: FoldableWithIndex<usize>` — uses `enumerate()` with monoid fold
- `VecBrand: TraversableWithIndex<usize>` — uses `enumerate()` with `M::lift2`
- `OptionBrand: FunctorWithIndex<()>` — trivial unit index
- `OptionBrand: FoldableWithIndex<()>` — trivial
- `OptionBrand: TraversableWithIndex<()>` — trivial

### 1.8 Bridge Functions (Phase 4) — Mostly Correct

The following bridge functions in `types/optics/functions.rs` are correctly implemented:

| Function | Status | Notes |
|----------|--------|-------|
| `optics_indexed_view` | Correct | Uses `Forget` with identity |
| `optics_indexed_over` | Correct | Creates `FnBrand<Q>` from indexed function |
| `optics_indexed_set` | Correct | Delegates to `optics_indexed_over` with `|_, _| a.clone()` |
| `optics_indexed_preview` | Correct | Uses `Forget` with local `First` monoid |
| `optics_indexed_fold_map` | Correct | Uses `Forget` with monoid `R` |
| `optics_un_index` | Correct | `dimap(|(_, a)| a, |b| b, pab)` matches PS `dimap snd identity` |
| `optics_as_index` | Correct | `dimap(|(i, _)| i, |b| b, pib)` matches PS `dimap fst identity` |
| `optics_reindexed` | Correct | See §2.1 for detailed analysis |

### 1.9 `reindexed` — Correct (Contrary to Initial Concern)

The `Reindexed::evaluate_indexed` method:
```rust
let dimapped = P::dimap(move |(i, a)| (f(i), a), |b| b, inner);
```
where `f: I -> J`, `inner: P::Of<(J, A), B>`.

The contravariant mapping takes `(I, A)` and produces `(J, A)` via `(i, a) -> (f(i), a)`. Applied to `P::Of<(J, A), B>`, this yields `P::Of<(I, A), B>` — exactly matching PureScript's `lcmap (first ij)` where `ij: i -> j`. The inner optic receives `Indexed<P, I, A, B>` as expected. The `F: Clone` fix for lifetime issues is correctly applied per the plan.

### 1.10 Standard Constructors (Phase 7) — Partially Complete

| Constructor | Status |
|-------------|--------|
| `IndexedTraversal::traversed()` | Correct — delegates to `TraversableWithIndex` |
| `IndexedTraversalPrime::traversed()` | Correct |
| `IndexedFold::folded()` | Correct — delegates to `FoldableWithIndex` |
| `IndexedFoldPrime::folded()` | Correct |
| `IndexedSetter::mapped()` | Correct — delegates to `FunctorWithIndex` |
| `IndexedSetterPrime::mapped()` | Correct |
| `positions()` | **Semantically different from PureScript** — see §2.2 |

---

## 2. Issues, Flaws, and Inconsistencies

### 2.1 `positions` Does Not Match PureScript Semantics — Semantic Flaw

**Severity: High**

**PureScript:**
```purescript
positions :: Traversal s t a b -> IndexedTraversal Int s t a b
positions tr = iwander \f s ->
  flip evalState 0 $ unwrap $ flip unwrap s $ tr $ Star \a ->
    Compose $ (f <$> get <*> pure a) <* modify (_ + 1)
```

PureScript's `positions` takes an **existing `Traversal`** and decorates it with integer position indices using a `State` monad counter. The original element type `a` is preserved as the focus, and the integer position is added as the index.

**Rust implementation:**
```rust
pub fn positions<'a, P, I, Brand, A>() -> IndexedTraversal<..., I, Brand::Of<A>, Brand::Of<I>, I, I, Positions<Brand, A>>
```

The Rust `positions`:
1. Takes **no `Traversal` argument** — it constructs one from `TraversableWithIndex<I>`
2. The **focus type becomes `I`** (the index), not the original element type `A`
3. The **target type becomes `Brand::Of<I>`** (container of indices), not `Brand::Of<B>`
4. The index type is **generic `I`**, not fixed to `Int`/`usize`

In the implementation body:
```rust
Brand::traverse_with_index::<A, I, M>(Box::new(move |i: I, _a: A| f(i.clone(), i)), s)
```

The actual element `_a` is **discarded**. Both arguments to `f` receive the index `i`. This means `positions` produces an `IndexedTraversal` whose foci are the indices themselves, not the original elements.

**Impact:** Users expecting PureScript-style `positions` (decorate any traversal with position counting) will find the Rust version does something entirely different. It is more like a "project to indices" operation.

### 2.2 Missing `iwander` Function

**Severity: Medium**

The plan (Phase 7) lists `positions` as depending on `iwander`-like bridging. PureScript defines:

```purescript
iwander :: (forall f. Applicative f => (i -> a -> f b) -> s -> f t) -> IndexedTraversal i s t a b
iwander itr = wander (\f s -> itr (curry f) s) <<< unwrap
```

`iwander` is the indexed analogue of `wander` — it converts a rank-2 indexed traversal function into a proper `IndexedTraversal`. The Rust implementation has no equivalent. Instead, `IndexedTraversal::new` takes an `IndexedTraversalFunc` trait object, which serves a similar role but requires defining a struct implementing the trait rather than passing a closure.

This is not necessarily a flaw (it follows the existing non-indexed pattern where `Traversal::new` takes a `TraversalFunc`), but it means there's no convenient way to construct an `IndexedTraversal` from a closure.

### 2.3 Missing `functions.rs` Re-exports

**Severity: Medium**

The plan (Phase 4) explicitly specifies:

> **Re-export** in `fp-library/src/functions.rs`.

The `functions.rs` file contains **no indexed optic re-exports**. The following should be re-exported:

- `optics_indexed_view`
- `optics_indexed_over`
- `optics_indexed_set`
- `optics_indexed_preview`
- `optics_indexed_fold_map`
- `optics_un_index`
- `optics_as_index`
- `optics_reindexed`
- `positions`

Currently users must import from `fp_library::types::optics::*` instead of the convenience `fp_library::functions::*` facade.

### 2.4 `optics_un_index` / `optics_as_index` Are Monomorphic in Profunctor

**Severity: Medium**

**PureScript:**
```purescript
unIndex :: forall p i s t a b. Profunctor p => IndexedOptic p i s t a b -> Optic p s t a b
```
The return type is **polymorphic in `p`** — the same returned optic can be used with any profunctor.

**Rust:**
```rust
pub fn optics_un_index<'a, P, O, I, S, T, A, B>(optic: &'a O) -> impl Optic<'a, P, S, T, A, B> + 'a
```
The return type is `impl Optic<'a, P, ...>` where `P` is **fixed at the call site**. The returned optic can only be used with that specific profunctor.

This is a Rust type system limitation (no rank-2 types / `forall p.` in return position), not a bug. However, it means:
- You cannot `optics_view` and `optics_over` the same `un_index`ed optic — each would require a different `P`.
- Each use site must call `optics_un_index` independently with the appropriate `P` inferred from context.

### 2.5 `TraversableWithIndex::traverse_with_index` Requires `A: Clone + B: Clone`

**Severity: Low**

```rust
fn traverse_with_index<'a, A: 'a + Clone, B: 'a + Clone, M: Applicative>(
    f: impl Fn(I, A) -> M::Of<'a, B> + 'a,
    ta: Self::Of<'a, A>,
) -> M::Of<'a, Self::Of<'a, B>>;
```

PureScript's `traverseWithIndex` has no `Clone` constraint on `a` or `b`. This extra bound propagates to `IndexedTraversal::traversed()` and `positions()`, limiting their use with non-`Clone` types. The non-indexed `Traversable::traverse` in this codebase also requires `Clone`, so this is consistent within the project, but it's a divergence from the PureScript reference.

### 2.6 `IndexedTraversalFunc::apply` Has Unused `'b` Lifetime Parameter

**Severity: Low (cosmetic)**

```rust
pub trait IndexedTraversalFunc<'a, I, S, T, A, B> {
    fn apply<'b, M: Applicative>(
        &self,
        f: Box<dyn Fn(I, A) -> ... + 'a>,
        s: S,
    ) -> ...;
}
```

The `'b` lifetime on `apply` is declared but never referenced in any parameter or return type. It should be removed.

### 2.7 `IndexedFoldFunc::apply` Has Unused `Q` Type Parameter

**Severity: Low (cosmetic)**

```rust
pub trait IndexedFoldFunc<'a, I, S, A> {
    fn apply<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
        &self,
        f: Box<dyn Fn(I, A) -> R + 'a>,
        s: S,
    ) -> R;
}
```

The `Q: UnsizedCoercible + 'static` parameter is declared but never used in the function signature or body. It appears to be copied from the optic trait signature but serves no purpose in the fold function itself.

### 2.8 `IndexedSetterFunc::apply` Uses `Box<dyn Fn>` for the Modifier

**Severity: Low (design note)**

```rust
pub trait IndexedSetterFunc<'a, I, S, T, A, B> {
    fn apply(&self, f: Box<dyn Fn(I, A) -> B + 'a>, s: S) -> T;
}
```

This uses `Box<dyn Fn>` for the modifier function, which involves heap allocation. The non-indexed `Setter` in this codebase follows a similar pattern for bridging between brands, so this is consistent. However, it's worth noting that the `over` method on `IndexedSetter` also boxes the closure:

```rust
pub fn over(&self, s: S, f: impl Fn(I, A) -> B + 'a) -> T {
    self.setter_fn.apply(Box::new(f), s)
}
```

This means every `over` call allocates, even when the setter could theoretically use static dispatch.

### 2.9 No `IndexedOpticAdapter` / `IndexedOpticAdapterDiscardsFocus` Impls for Non-Lens Types

**Severity: Medium**

The `IndexedOpticAdapter` and `IndexedOpticAdapterDiscardsFocus` traits (used by `optics_un_index`, `optics_as_index`, and `optics_reindexed`) are only implemented for `IndexedLens` and `IndexedLensPrime`. They are **not** implemented for:

- `IndexedTraversal` / `IndexedTraversalPrime`
- `IndexedFold` / `IndexedFoldPrime`
- `IndexedGetter` / `IndexedGetterPrime` (for `IndexedOpticAdapterDiscardsFocus`)
- `IndexedSetter` / `IndexedSetterPrime`

This means `optics_un_index`, `optics_as_index`, and `optics_reindexed` can only be used with `IndexedLens`/`IndexedLensPrime`, not with any other indexed optic type. Users cannot un-index an `IndexedTraversal` or reindex an `IndexedFold`.

The PureScript versions (`unIndex`, `asIndex`, `reindexed`) work with any `IndexedOptic` because the optic is a plain function `Indexed p i a b -> p s t` — no trait dispatch needed. The Rust encoding requires explicit trait implementations on each concrete type.

### 2.10 `Composed` Indexed + Indexed Not Supported

**Severity: Low**

The plan specifies composition of Regular (outer) + Indexed (inner) = Indexed. This is implemented. However, composition of Indexed (outer) + Indexed (inner) is not addressed in the plan or implementation. In PureScript, this would require choosing which index to keep (typically the inner one, with the outer's index discarded or combined). This is a design gap, not necessarily a flaw.

---

## 3. Comparison with PureScript Reference

### 3.1 Type Encoding

| Concept | PureScript | Rust |
|---------|-----------|------|
| `Indexed` | `newtype Indexed p i a b = Indexed (p (Tuple i a) b)` | `struct Indexed<'a, P, I, A, B> { inner: P::Of<'a, (I, A), B> }` |
| `IndexedOptic` | `type IndexedOptic p i s t a b = Indexed p i a b -> p s t` | Separate traits per optic kind (`IndexedLensOptic`, etc.) |
| `IndexedTraversal` | `type IndexedTraversal i s t a b = forall p. Wander p => IndexedOptic p i s t a b` | `struct IndexedTraversal<..., F: IndexedTraversalFunc>` |

The Rust encoding reifies each indexed optic as a **concrete struct** rather than a **polymorphic function**. This is consistent with the non-indexed optics in the codebase and is a reasonable design choice given Rust's type system. The trade-off is that adapter traits (`IndexedOpticAdapter`) are needed where PureScript can simply compose functions.

### 3.2 Function Correspondence

| PureScript | Rust | Match? |
|-----------|------|--------|
| `unIndex` | `optics_un_index` | Semantics correct, but monomorphic in P and only works on IndexedLens |
| `asIndex` | `optics_as_index` | Semantics correct, same limitations as above |
| `reindexed` | `optics_reindexed` | Correct, same limitations |
| `iwander` | (missing) | No equivalent |
| `ifolded` | `IndexedFold::folded()` | Correct |
| `imapped` | `IndexedSetter::mapped()` | Correct |
| `itraversed` | `IndexedTraversal::traversed()` | Correct |
| `positions` | `positions()` | **Different semantics** — see §2.1 |

---

## 4. Summary

### What's correct (majority of the implementation):
- Core `Indexed` profunctor wrapper with all four class instances
- All five indexed optic traits
- All six concrete indexed optic structs with full trait implementations
- All five `Composed` indexed implementations with correct turbofish usage
- All three `WithIndex` type classes with `VecBrand`/`OptionBrand` impls
- Six of eight bridge functions (`indexed_view`, `indexed_over`, `indexed_set`, `indexed_preview`, `indexed_fold_map`, `reindexed`)
- Standard constructors (`traversed`, `folded`, `mapped`)

### What needs attention:

| # | Issue | Severity | Category |
|---|-------|----------|----------|
| 1 | `positions` has different semantics from PureScript | High | Semantic flaw |
| 2 | `optics_un_index`/`as_index`/`reindexed` only work on `IndexedLens` | Medium | Missing impls |
| 3 | Missing `functions.rs` re-exports | Medium | Incomplete |
| 4 | `optics_un_index`/`optics_as_index` monomorphic in P | Medium | Design limitation |
| 5 | Missing `iwander` equivalent | Medium | Missing feature |
| 6 | `TraversableWithIndex` requires `Clone` bounds | Low | Design choice |
| 7 | Unused `'b` lifetime on `IndexedTraversalFunc::apply` | Low | Cosmetic |
| 8 | Unused `Q` parameter on `IndexedFoldFunc::apply` | Low | Cosmetic |
| 9 | Doc test failures | Medium | Documentation |
| 10 | Indexed + Indexed composition not addressed | Low | Design gap |
