# Comprehensive Analysis: Rust `fp-library` Optics vs PureScript `purescript-profunctor-lenses`

This document provides a detailed, side-by-side comparison of the optics implementation in the Rust `fp-library` (under `fp-library/src/types/optics/`) against the PureScript reference implementation `purescript-profunctor-lenses` (under `src/Data/Lens/`). It evaluates correctness, completeness, and identifies flaws, inconsistencies, and missing features.

---

## 1. Fundamental Architecture

### PureScript Approach: Optics as Functions

In PureScript, an optic is a type alias for a universally quantified function using rank-2 polymorphism:

```purescript
type Optic p s t a b = p a b -> p s t
type Lens s t a b = forall p. Strong p => Optic p s t a b
```

Composition is simply function composition (`<<<`), requiring no special machinery. "Pre-composed" concrete types (like `ALens` using `Shop`) exist primarily to help with type inference and impredicativity.

### Rust Approach: Structs Implementing Traits

Rust cannot express rank-2 types or universally quantified type aliases in the same way. The library uses a trait-based encoding with concrete structs:

1.  **Concrete Structs**: `Lens`, `Prism`, `Iso`, etc., are concrete structs that hold the "reified" internal representation of the optic (equivalent to PureScript's `Shop`, `Market`, `Exchange`, etc.).
    *   *Note*: This means `Lens` in Rust is effectively `ALens` in PureScript terminology.
2.  **Optic Trait**: The behavior is defined via traits:
    ```rust
    pub trait Optic<'a, P: Profunctor, S, T, A, B> {
        fn evaluate(&self, pab: P::Of<'a, A, B>) -> P::Of<'a, S, T>;
    }
    ```
3.  **Specialized Traits**: Each family has a trait (e.g., `LensOptic`) that defines the `evaluate` method with the specific profunctor bound (e.g., `P: Strong`).

Composition uses a `Composed` struct to enable static dispatch and zero-cost abstraction, rather than function composition.

**Assessment:** This is a robust and idiomatic translation to Rust. It trades the elegance of function composition for the performance and type-safety of static dispatch, which is appropriate for Rust.

---

## 2. Optic-by-Optic Comparison

### 2.1 Iso

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Iso s t a b = forall p. Profunctor p => p a b -> p s t` | `Iso<'a, P, S, T, A, B>` struct + `IsoOptic` trait | Correct |
| Construction | `iso :: (s -> a) -> (b -> t) -> Iso s t a b` | `Iso::new(from, to)` | Correct |
| Encoding | `dimap f g pab` | `Q::dimap(from, to, pab)` | Correct |
| Concrete (A-) | `AnIso s t a b = Optic (Exchange a b) s t a b` | Uses `ExchangeBrand` | Correct |
| Extraction | `withIso`, `cloneIso` | `optics_from`, `optics_to` | Partial |
| Reversal | `re` (generic combinator) | `Iso::reversed()` (concrete method) | Partial |

**Issues:**
- **Missing `withIso`**: PureScript's `withIso` extracts both morphisms from an `AnIso` in CPS style. Rust provides `optics_from`/`optics_to` helper functions but no unified CPS-style extractor.
- **Missing `re`**: PureScript has a general `re` combinator using the `Re` profunctor that works on any optic (turning a Getter into a Review, etc.). Rust only has `Iso::reversed()`, which returns a concrete `Iso`. The `Re` profunctor itself is not implemented.
- **Missing utility functions**: `au`, `auf`, `under`, `non`, `curried`, `uncurried`, `flipped`, `mapping`, `dimapping`, `coerced`.

### 2.2 Lens

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Lens s t a b = forall p. Strong p => p a b -> p s t` | `Lens<'a, P, S, T, A, B>` struct + `LensOptic` trait | Correct |
| Construction | `lens :: (s -> a) -> (s -> b -> t) -> Lens` | `Lens::new(to: S -> (A, B -> T))` | Correct |
| Encoding | `dimap (\s -> (get s, s)) (\(b, s) -> set s b) (first pab)` | `dimap(to, |(b, f)| f(b), first(pab))` | Correct |
| Concrete (A-) | `ALens s t a b = Optic (Shop a b) s t a b` | Uses `ShopBrand` | Correct |
| Extraction | `withLens`, `cloneLens`, `lensStore` | Struct fields accessible | Different but ok |

**Issues:**
- **Setter signature**: Rust's `Lens::new` takes a single function `S -> (A, B -> T)` returning both the view and a setter closure. This avoids `S: Clone` which was required in earlier versions. The legacy `Lens::from_view_set` constructor, taking `(S -> A, (S, B) -> T)`, is still present for convenience but requires `S: Clone`.
- **Missing `cloneLens`**: PureScript's `cloneLens :: ALens s t a b -> Lens s t a b`. Not present.
- **Missing `lensStore`**: Utility for extracting `Tuple a (b -> t)` from an `ALens`. Not present.
- **Missing Indexed Lenses**: `IndexedLens`, `ilens`, `ilens'`, `cloneIndexedLens` are completely missing.

### 2.3 Prism

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Prism s t a b = forall p. Choice p => p a b -> p s t` | `Prism<'a, P, S, T, A, B>` struct + `PrismOptic` trait | Correct |
| Construction | `prism :: (b -> t) -> (s -> Either t a) -> Prism` | `Prism::new(preview: S -> Result<A, T>, review: B -> T)` | Correct |
| Encoding | `dimap fro (either id id) (right (rmap to pab))` | `dimap(preview, |r| match r { Ok(b) => review(b), Err(t) => t }, right(pab))` | Correct |
| Concrete (A-) | `APrism s t a b = Optic (Market a b) s t a b` | Uses `MarketBrand` | Correct |

**Issues:**
- **Clone Requirement**: `S: Clone` requirement has been successfully removed by using `Result<A, T>` in the internal encoding.
- **Encoding note**: PureScript's `prism` applies `rmap to` to `pab` before passing it to `right` (`right (rmap to pab)`), while Rust applies the review in the output `dimap` instead (`dimap(preview, |r| match r { Ok(b) => review(b), Err(t) => t }, right(pab))`). Both produce the same result and are correct.
- **Missing Helpers**: `only`, `nearly` (equality matching), `is`, `isn't` (predicates), `matching` (extract `Either`), `below`.
- **Missing `clonePrism`, `withPrism`**: Not present.

### 2.4 AffineTraversal

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type AffineTraversal ... = Strong p => Choice p => ...` | `AffineTraversal<'a, P, S, T, A, B>` struct + `AffineTraversalOptic` trait | Correct |
| Construction | `affineTraversal :: (s -> b -> t) -> (s -> Either t a)` | `AffineTraversal::new(to: S -> Result<(A, B -> T), T>)` | Correct |
| Encoding | `dimap ... (second (right pab))` | `dimap(split, merge, right(first(pab)))` | Valid alternative |
| Concrete (A-) | `AnAffineTraversal` (uses `Stall`) | Uses `StallBrand` | Correct |

**Issues:**
- **Encoding difference**: PureScript uses `second (right pab)` while Rust uses `right(first(pab))`. Both are valid profunctor encodings of an affine traversal — they apply `Strong` and `Choice` in different orders but are equivalent. This is not a bug.
- **Missing `cloneAffineTraversal`, `withAffineTraversal`**: Not present.

### 2.5 Traversal

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Traversal ... = Wander p => ...` | `Traversal<'a, P, S, T, A, B, F>` struct + `TraversalOptic` trait | Correct |
| Construction | `wander` | `Traversal::new(F: TraversalFunc)` | Correct |
| Concrete (A-) | `ATraversal = Optic (Bazaar ...)` | No `Bazaar` equivalent | **Missing** |

**Issues:**
- **Missing `Bazaar` profunctor**: PureScript's `Bazaar` is used for `ATraversal` (concrete traversal) and `cloneTraversal`. Not implemented in Rust.
- **Missing `cloneTraversal`**: No way to clone/reconstruct a traversal from its concrete representation.
- **Missing Operations**: `traverseOf`, `sequenceOf`, `failover`, `element`, `elementsOf`, `both`.
- **Missing `traversed`**: The universal traversal for `Traversable` types. Not found as a built-in combinator.

### 2.6 Grate

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Grate ... = Closed p => ...` | `Grate<'a, P, S, T, A, B>` struct + `GrateOptic` trait | Correct |
| Encoding | `dimap (#) f (closed pab)` | `dimap(extract, reconstruct, closed(pab))` | Correct |
| Concrete (A-) | `AGrate` (uses `Grating`) | Uses `GratingBrand` | Correct |

**Issues:**
- **Missing Operations**: `zipWithOf`, `zipFWithOf`, `collectOf`, `cotraversed`.
- **Missing `Zipping` profunctor**: Not implemented.

### 2.7 Getter

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Getter ... = Fold r ...` | `Getter` struct + `GetterOptic` trait | Correct |
| Profunctor | `Forget r` | `ForgetBrand` | Correct |

**Issues:**
- **Missing `view` / `(^.)` operator**: PureScript provides `view`. Rust has `optics_view` helper.
- **Missing Indexed Getters**: `iview`, `iuse`.

### 2.8 Setter

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Setter ... = Optic Function ...` | `Setter` struct + `SetterOptic` trait | Correct |
| Profunctor | `Function` | `FnBrand` | Correct |

**Issues:**
- **Missing Rich API**: PureScript provides `addOver`, `mulOver`, `setJust`, `assign`, `modifying`, etc. Rust has `optics_set` and `optics_over` only.
- **Missing Indexed Setters**: `iover`.

### 2.9 Fold

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Fold r ... = Optic (Forget r) ...` | `Fold` struct + `FoldOptic` trait | Correct |
| Construction | `folded`, `unfolded`, etc. | `Fold::new(F: FoldFunc)` | Correct |
| Allocation | Zero intermediate allocation | Zero intermediate allocation via `FoldFunc::apply` | Correct |

**Notes:**
- **`FoldFunc` trait**: The `Fold` struct stores a generic `F: FoldFunc<'a, S, A>`, where `FoldFunc::apply` folds directly into any `Monoid` without collecting an intermediate `Vec`. `IterableFoldFn<F>` is provided as a convenience adapter for any `Fn(S) -> impl IntoIterator<Item = A>`.
- **Missing Operations**: `foldOf`, `foldMapOf`, `foldrOf`, `foldlOf`, `toListOf`, `firstOf`, `lastOf`, `maximumOf`, `minimumOf`, `allOf`, `anyOf`, `sumOf`, `lengthOf`, `findOf`, `has`, `filtered`. Rust only has `optics_preview`.
- **Missing Indexed Folds**: `ifoldMapOf`, `ifindOf`, etc.

### 2.10 Review

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Review ... = Optic Tagged ...` | `Review` struct + `ReviewOptic` trait | Correct |
| Profunctor | `Tagged` | `TaggedBrand` | Correct |

---

## 3. Internal Profunctors

| PureScript | Rust | Status |
|------------|------|--------|
| `Exchange` | `Exchange` (via `ExchangeBrand`) | **Correct**. Parameterized over `FnBrand`. |
| `Shop` | `Shop` (via `ShopBrand`) | **Correct**. |
| `Market` | `Market` (via `MarketBrand`) | **Correct**. |
| `Stall` | `Stall` (via `StallBrand`) | **Correct**. |
| `Forget` | `Forget` (via `ForgetBrand`) | **Correct**. Missing `Cochoice` instance. |
| `Tagged` | `Tagged` (via `TaggedBrand`) | **Correct**. Missing `Closed`, `Costrong` instances. |
| `Grating` | `Grating` (via `GratingBrand`) | **Correct**. |
| `Bazaar` | - | **Missing**. Used for concrete `ATraversal`. |
| `Zipping` | - | **Missing**. Used for grates. |
| `Re` | - | **Missing**. Used for reversing optics. |
| `Indexed` | - | **Missing**. Used for indexed optics. |
| `Focusing` | - | **Missing**. Used for `Zoom`. |

---

## 4. Optic Subtyping Hierarchy

PureScript establishes a lattice of optic subtyping through profunctor class inheritance. The Rust implementation models this via manual trait implementations on the concrete structs.

```
         Iso
        / | \
    Lens Prism Grate
      \  |  /
  AffineTraversal
        |
    Traversal
      / | \
  Getter Fold Setter   Review
```

*   **Correct**: `Iso`, `Lens`, `Prism`, `AffineTraversal`, `Traversal`, `Grate`, `Getter`, `Fold`, `Setter`, `Review` structs all implement the correct super-traits. The specialized optic traits (`IsoOptic`, `LensOptic`, `PrismOptic`, `AffineTraversalOptic`, `TraversalOptic`, `GrateOptic`, `GetterOptic`, `FoldOptic`, `SetterOptic`, `ReviewOptic`) are all defined and implemented, with `AffineTraversalOptic` now completing the chain from `Iso` down through `AffineTraversal`. `Composed` implements all specialized traits, enabling static-dispatch composition across the full hierarchy.

---

## 5. Major Missing Features

### 5.1 Indexed Optics (Critical)
The entire hierarchy of **Indexed Optics** is missing.
*   **Missing Types**: `IndexedLens`, `IndexedTraversal`, `IndexedFold`, `IndexedGetter`, `IndexedSetter`.
*   **Missing Profunctor**: `Indexed p i a b`.
*   **Impact**: It is impossible to traverse structures while retaining access to keys or indices (e.g., iterating over a Map with keys, or a Vector with indices).

### 5.2 Re Profunctor
PureScript's `Re` profunctor allows reversing optics (turning an Iso around, or a Getter into a Review). This is completely missing in Rust.

### 5.3 Bazaar & internal machinery
`Bazaar` (for concrete traversals), `Zipping` (for grates), `Costrong`, `Cochoice` are all missing. This limits the ability to implement advanced combinators that rely on reifying optics into data structures.

---

## 6. Summary of Flaws & Inconsistencies

1.  **Missing Standard Combinators**: The library provides the *types* but very few of the standard *combinators* (`_1`, `_2`, `_Just`, `_Left`, `traversed`) that make optics ergonomic to use.
2.  **Composition Verbosity**: While necessary, the `Composed` struct makes type signatures for composed optics extremely verbose and complex compared to `.` or `<<<`.

## 7. Conclusion

`fp-library` provides a solid, type-safe foundation for profunctor optics in Rust. The core encoding of `Iso`, `Lens`, `Prism`, `AffineTraversal`, `Traversal`, and `Fold` is high-fidelity and correct, with a complete specialized-trait hierarchy and full `Composed` support across all families. `Fold` in particular folds directly into any `Monoid` via `FoldFunc::apply` with no intermediate allocation, matching the semantic intent of PureScript's profunctor-based `Fold`. However, the library is significantly less mature than `purescript-profunctor-lenses` in terms of:
1.  **Completeness**: Completely missing Indexed optics.
2.  **Ecosystem**: Missing standard combinators and convenience functions.

---

## 8. Recommended Next Steps

Based on the analysis, the following roadmap is recommended to bring `fp-library` to parity with `purescript-profunctor-lenses`, prioritized by impact and complexity.

### Phase 1: Standard Combinators (High Impact / Low Complexity)
The library currently lacks the standard combinators that make optics ergonomic to use. Implementing these will provide immediate value.

*   **Tuple Combinators**: `_1`, `_2`, etc. for accessing tuple elements.
*   **Result/Option Combinators**: `_Ok`, `_Err`, `_Some`, `_None` (often called `_Just`, `_Left`, `_Right` in other ecosystems).
*   **Collection Combinators**: `traversed` (or `traverse`) for iterating over standard collections like `Vec`, `Option`, `Result`.
*   **Implementation Location**: These should be added to `fp-library/src/types/optics/combinators.rs` (new module) or `helpers.rs`.

### Phase 2: Internal Machinery (Foundational)
Implementing these missing internal profunctors is a prerequisite for advanced features like optic reversal and reification.

1.  **`Re` Profunctor**: Required for reversing optics (e.g., `iso.re()`, `getter.re()` to get a `Review`).
2.  **`Bazaar` Profunctor**: Required for the concrete representation of `Traversal` (enables `cloneTraversal`).
3.  **`Zipping` Profunctor**: Required for `Grate`.
4.  **`Costrong` & `Cochoice`**: Required for fully implementing `Re` and other dual concepts.

### Phase 3: Helper Functions (Ergonomics)
Add helpers to allow extracting internal functions from concrete optics, bridging the gap between the profunctor encoding and concrete data structures.

*   **Extraction**: Implement `withLens`, `withPrism`, `withIso` style helpers (partial support exists via `optics_from`/`to` for Iso).
*   **Cloning**: Implement `cloneLens`, `clonePrism`, `cloneTraversal` to allow reconstructing optics from their profunctor encoding.

### Phase 4: Indexed Optics (Critical / High Complexity)
This is the largest missing piece. It requires defining a new hierarchy of traits and profunctors.

1.  **Infrastructure**: Define `Indexed` profunctor trait.
2.  **Traits**: Define `IndexedOptic`, `IndexedLens`, `IndexedTraversal`, `IndexedFold`, `IndexedGetter`, `IndexedSetter`.
3.  **Implementations**: Create concrete structs and implementations for these traits.
4.  **Combinators**: Implement `itraversed`, `iover`, `ifoldMap`, etc.

---

## 9. Phase 2 Component Analysis

This section analyzes the implementation complexity of the missing components identified in Phase 2 to determine the optimal implementation order.

### 9.1 Zipping Profunctor (Low Complexity)
*   **Purpose**: Enables `Grate` optics.
*   **PureScript Definition**: `newtype Zipping a b = Zipping (a -> a -> b)`
*   **Rust Implementation**: `struct Zipping<F>(F)` where `F: Fn(A, A) -> B`.
*   **Dependencies**: Requires the `Closed` trait, which is already implemented.
*   **Analysis**: This is the easiest component to implement. It has self-contained logic and relies only on existing traits. It provides a quick win by enabling `Grate` optics.

### 9.2 Costrong / Cochoice Traits (Low/Medium Complexity)
*   **Purpose**: Required for `Re` to be fully functional (enabling `Re` to be `Strong` and `Choice`).
*   **PureScript Definition**: Type classes with `unfirst`, `unleft`, etc.
*   **Rust Implementation**: Traits mirroring `Strong` and `Choice` but with "un-" methods.
*   **Dependencies**: None.
*   **Analysis**: These are foundational traits. While the traits themselves are simple definitions, implementing them for existing profunctors might require careful consideration of ownership and closures.

### 9.3 Re Profunctor (Medium Complexity)
*   **Purpose**: Enables reversing optics (e.g., `iso.re()`).
*   **PureScript Definition**: `newtype Re p s t a b = Re (p b a -> p t s)`
*   **Rust Implementation**: `struct Re<P, S, T>(Box<dyn Fn(P::Of<B, A>) -> P::Of<T, S>>)` (conceptual).
*   **Dependencies**: Strongly benefits from `Costrong` and `Cochoice`. Without them, `Re` is only a `Profunctor` (useful only for `Iso`). To reverse `Lens` and `Prism`, it needs `Strong` (via `Costrong`) and `Choice` (via `Cochoice`).
*   **Analysis**: The implementation involves wrapping functions that operate on higher-kinded types (`P::Of`). This is a known pattern but more verbose than `Zipping`.

### 9.4 Bazaar Profunctor (High Complexity)
*   **Purpose**: Concrete representation of `Traversal`, enabling `cloneTraversal`.
*   **PureScript Definition**: `newtype Bazaar p a b s t = Bazaar (forall f. Applicative f => p a (f b) -> s -> f t)`
*   **Rust Implementation**: Requires encoding rank-2 polymorphism (`forall f`) and `Applicative` constraints on higher-kinded types.
*   **Dependencies**: `Wander`.
*   **Analysis**: This is the most complex component due to Rust's type system limitations regarding higher-ranked trait bounds on higher-kinded types. It requires a sophisticated encoding strategy.

### Conclusion & Recommendation

The **`Zipping` profunctor** is the easiest component to implement. It is isolated, simple, and immediately unlocks `Grate` functionality.

**Recommended Implementation Order:**
1.  `Zipping` (Enables `Grate`)
2.  `Costrong` / `Cochoice` (Foundation for `Re`)
3.  `Re` (Enables optic reversal)
4.  `Bazaar` (Enables concrete traversals - hardest)
