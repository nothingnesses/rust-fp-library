# Optics Comparison: Rust `fp-library` vs PureScript `purescript-profunctor-lenses`

This document provides a detailed, side-by-side comparison of the optics implementation in the Rust `fp-library` (`fp-library/src/types/optics/`) against the PureScript reference implementation `purescript-profunctor-lenses` (`src/Data/Lens/`). It evaluates correctness, completeness, and identifies flaws, inconsistencies, and missing features.

---

## 1. Fundamental Architecture

### PureScript: Optics as Rank-2 Polymorphic Functions

In PureScript, an optic is a type alias for a universally quantified function:

```purescript
type Optic p s t a b = p a b -> p s t
type Lens s t a b = forall p. Strong p => Optic p s t a b
```

Composition is function composition (`<<<`). Concrete types (`ALens` using `Shop`, `APrism` using `Market`, etc.) exist to avoid impredicativity issues and improve type inference.

### Rust: Structs + Trait-Based Dispatch

Rust cannot express rank-2 types or universally quantified type aliases. The library uses:

1. **Concrete structs** (`Lens`, `Prism`, `Iso`, etc.) that hold the reified internal representation — equivalent to PureScript's `ALens`/`APrism`/`AnIso` types.
2. **Optic traits** defining profunctor evaluation:
   ```rust
   pub trait Optic<'a, P: Profunctor, S, T, A, B> {
       fn evaluate(&self, pab: P::Of<'a, A, B>) -> P::Of<'a, S, T>;
   }
   ```
3. **Specialized traits** (`LensOptic`, `PrismOptic`, etc.) that bind the profunctor constraint (`P: Strong`, `P: Choice`, etc.).
4. **`Composed` struct** for zero-cost static-dispatch composition, replacing function composition.

**Assessment:** This is a sound and idiomatic translation. It trades the elegance of `<<<` for static dispatch performance. The `Composed` struct is necessary but results in deeply nested types.

---

## 2. Optic-by-Optic Comparison

### 2.1 Iso (Isomorphism)

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Polymorphic type | `forall p. Profunctor p => Optic p s t a b` | `IsoOptic` trait bound | Correct |
| Concrete type | `AnIso = Optic (Exchange a b)` | `Iso<'a, P, S, T, A, B>` struct | Correct |
| Constructor | `iso :: (s -> a) -> (b -> t)` | `Iso::new(from, to)` | Correct |
| Encoding | `dimap from to pab` | `Q::dimap(from, to, pab)` | Correct |
| CPS extraction | `withIso :: AnIso -> ((s->a) -> (b->t) -> r) -> r` | — | **Missing** |
| Clone/reconstruct | `cloneIso :: AnIso -> Iso` | — | **Missing** |
| Reversal | `re :: Optic (Re p a b) s t a b -> Optic p b a t s` | `Iso::reversed()` (method) | **Partial** — `re` is generic across all optics; Rust only has an `Iso`-specific method. The generic `Re` profunctor is implemented but not exposed via a top-level `re` combinator. |
| Utility isos | `non`, `curried`, `uncurried`, `flipped`, `coerced` | — | **Missing** |
| Higher-order isos | `mapping`, `dimapping` | — | **Missing** |
| Iso combinators | `au`, `auf`, `under` | — | **Missing** |

### 2.2 Lens

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Polymorphic type | `forall p. Strong p => Optic p s t a b` | `LensOptic` trait bound | Correct |
| Concrete type | `ALens = Optic (Shop a b)` | `Lens<'a, P, S, T, A, B>` struct | Correct |
| Constructor | `lens :: (s -> a) -> (s -> b -> t)` | `Lens::new(S -> (A, B -> T))` | Correct |
| Alt. constructor | `lens' :: (s -> Tuple a (b -> t))` | `Lens::from_view_set(view, set)` (requires `S: Clone`) | Correct |
| Encoding | `dimap (\s -> (get s, s)) (\(b, s) -> set s b) (first pab)` | `dimap(to, \|(b, f)\| f(b), first(pab))` | Correct |
| CPS extraction | `withLens`, `lensStore` | — | **Missing** |
| Clone/reconstruct | `cloneLens :: ALens -> Lens` | — | **Missing** |
| Direct methods | — | `view()`, `set()`, `over()` on struct | Extra (good ergonomics) |

### 2.3 Prism

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Polymorphic type | `forall p. Choice p => Optic p s t a b` | `PrismOptic` trait bound | Correct |
| Concrete type | `APrism = Optic (Market a b)` | `Prism<'a, P, S, T, A, B>` struct | Correct |
| Constructor | `prism :: (b -> t) -> (s -> Either t a)` | `Prism::new(S -> Result<A, T>, B -> T)` | Correct |
| Simple constructor | `prism' :: (a -> s) -> (s -> Maybe a)` | — | **Missing** |
| CPS extraction | `withPrism` | — | **Missing** |
| Clone/reconstruct | `clonePrism :: APrism -> Prism` | — | **Missing** |
| Matching | `matching :: APrism -> s -> Either t a` | `Prism::preview()` returns `Result<A, T>` | Correct |
| Predicates | `is`, `isn't` | — | **Missing** |
| Special prisms | `only`, `nearly`, `below` | — | **Missing** |
| Direct methods | — | `preview()`, `review()` on struct | Extra (good ergonomics) |

### 2.4 AffineTraversal

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Polymorphic type | `Strong p => Choice p => Optic p s t a b` | `AffineTraversalOptic` trait bound | Correct |
| Concrete type | `AnAffineTraversal = Optic (Stall a b)` | `AffineTraversal<'a, P, S, T, A, B>` struct | Correct |
| Constructor | `affineTraversal :: (s->b->t) -> (s -> Either t a)` | `AffineTraversal::new(S -> Result<(A, B->T), T>)` | Correct |
| Alt. constructor | `affineTraversal' :: (s -> Tuple (b->t) (Either t a))` | `AffineTraversal::from_preview_set(preview, set)` | Correct |
| Encoding | `dimap ... (second (right pab))` | `dimap(split, merge, right(first(pab)))` | Correct — different but equivalent ordering of `Strong`/`Choice` |
| CPS extraction | `withAffineTraversal` | — | **Missing** |
| Clone/reconstruct | `cloneAffineTraversal` | — | **Missing** |
| Direct methods | — | `preview()`, `set()`, `over()` on struct | Extra (good ergonomics) |

### 2.5 Traversal

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Polymorphic type | `forall p. Wander p => Optic p s t a b` | `TraversalOptic` trait bound | Correct |
| Concrete type | `ATraversal = Optic (Bazaar (->) a b)` | `Traversal<'a, P, S, T, A, B, F>` struct | Correct |
| Constructor | `wander :: (forall f. Applicative f => (a -> f b) -> s -> f t) -> ...` | `Traversal::new(F: TraversalFunc)` | Correct — `TraversalFunc` trait is the Rust equivalent of the rank-2 traversal function |
| Universal traversal | `traversed :: Traversable t => Traversal (t a) (t b) a b` | `Traversal::traversed()` (via `TraversableWithIndex`) | Correct |
| Clone/reconstruct | `cloneTraversal :: ATraversal -> Traversal` | — | **Missing** |
| Effectful traversal | `traverseOf :: Optic (Star f) s t a b -> (a -> f b) -> s -> f t` | — | **Missing** |
| Sequencing | `sequenceOf :: Optic (Star f) s t (f a) a -> s -> f t` | — | **Missing** |
| Element access | `element :: Int -> Traversal' s a -> Optic' p s a` | — | **Missing** |
| Elements filter | `elementsOf :: IndexedTraversal' i s a -> (i -> Boolean) -> ...` | — | **Missing** |
| Bitraversal | `both :: Bitraversable r => Traversal (r a a) (r b b) a b` | — | **Missing** |
| Failover | `failover :: Alternative f => ...` | — | **Missing** |

### 2.6 Grate

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Polymorphic type | `forall p. Closed p => Optic p s t a b` | `GrateOptic` trait bound | Correct |
| Concrete type | `AGrate = Optic (Grating a b)` | `Grate<'a, P, S, T, A, B>` struct | Correct |
| Constructor | `grate :: (((s -> a) -> b) -> t) -> Grate` | `Grate::new(...)` | Correct |
| CPS extraction | `withGrate :: AGrate -> (((s->a)->b)->t) -> t` | — | **Missing** |
| Clone/reconstruct | `cloneGrate :: AGrate -> Grate` | — | **Missing** |
| Zipping | `zipWithOf :: Optic Zipping s t a b -> (a -> a -> b) -> s -> s -> t` | `zip_with_of(...)` free function | Correct |
| Cotraversal | `cotraversed :: Distributive f => Grate (f a) (f b) a b` | — | **Missing** |
| Zip with functor | `zipFWithOf :: Optic (Costar f) ... -> (f a -> b) -> f s -> t` | — | **Missing** |
| Collect | `collectOf :: Functor f => Optic (Costar f) ... -> (b -> s) -> f b -> t` | — | **Missing** |

### 2.7 Getter

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Type | `Getter s t a b = forall r. Fold r s t a b` | `GetterOptic` trait + `Getter` struct | Correct |
| Concrete type | `AGetter = Fold a s t a b` | `Getter<'a, P, S, T, A, B>` struct | Correct |
| Constructor | `to :: (s -> a) -> Getter` | `Getter::new(view_fn)` | Correct |
| View | `view :: AGetter -> s -> a` | `optics_view(optic, s)` free function | Correct |
| Infix view | `(^.) :: s -> AGetter -> a` | — | N/A (Rust has no custom operators) |
| Clone | `cloneGetter :: AGetter -> Getter` | — | **Missing** |
| Take both | `takeBoth :: AGetter -> AGetter -> Getter (Tuple a c)` | — | **Missing** |
| StateT | `use :: MonadState s m => Getter -> m a` | — | **Missing** |

### 2.8 Setter

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Type | `Setter = Optic Function s t a b` | `SetterOptic` trait + `Setter` struct | Correct |
| Over | `over :: Setter -> (a -> b) -> s -> t` | `optics_over(optic, s, f)` free function | Correct |
| Set | `set :: Setter -> b -> s -> t` | `optics_set(optic, s, a)` free function | Correct |
| Arithmetic | `addOver`, `subOver`, `mulOver`, `divOver` | — | **Missing** |
| Logical | `conjOver`, `disjOver`, `appendOver` | — | **Missing** |
| Set just | `setJust :: Setter s t a (Maybe b) -> b -> s -> t` | — | **Missing** |
| StateT | `assign`, `modifying`, `addModifying`, etc. | — | **Missing** |
| Operators | `%~`, `.~`, `+~`, `-~`, `*~`, `//~`, `\|\|~`, `&&~`, `<>~`, `?~` | — | N/A (Rust has no custom operators) |

### 2.9 Fold

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Type | `Fold r = Optic (Forget r) s t a b` | `FoldOptic` trait + `Fold` struct | Correct |
| Constructor | `folded :: Foldable g => Fold r (g a) b a t` | `Fold::new(F: FoldFunc)` + `IterableFoldFn` adapter | Correct |
| Unfolding | `unfolded :: (s -> Maybe (Tuple a s)) -> Fold r s t a b` | — | **Missing** |
| Replicated | `replicated :: Int -> Fold r a b a t` | — | **Missing** |
| Preview | `preview :: Fold (First a) s t a b -> s -> Maybe a` | `optics_preview(optic, s)` | Correct |
| FoldMap | `foldMapOf :: Fold r -> (a -> r) -> s -> r` | — | **Missing** |
| FoldOf | `foldOf :: Fold a -> s -> a` | — | **Missing** |
| To list | `toListOf :: Fold ... -> s -> List a` | — | **Missing** |
| To array | `toArrayOf :: Fold ... -> s -> Array a` | — | **Missing** |
| Left/right fold | `foldrOf`, `foldlOf` | — | **Missing** |
| First/last | `firstOf`, `lastOf` | — | **Missing** |
| Min/max | `maximumOf`, `minimumOf` | — | **Missing** |
| Quantifiers | `allOf`, `anyOf`, `andOf`, `orOf` | — | **Missing** |
| Membership | `elemOf`, `notElemOf` | — | **Missing** |
| Aggregation | `sumOf`, `productOf`, `lengthOf` | — | **Missing** |
| Search | `findOf` | — | **Missing** |
| Existence | `has`, `hasn't` | — | **Missing** |
| Filtering | `filtered :: Choice p => (a -> Boolean) -> Optic' p a a` | — | **Missing** |
| Effectful | `traverseOf_`, `sequenceOf_` | — | **Missing** |

### 2.10 Review

| Aspect | PureScript | Rust | Status |
|--------|-----------|------|--------|
| Type | `Review = Optic Tagged s t a b` | `ReviewOptic` trait + `Review` struct | Correct |
| Review | `review :: Review -> b -> t` | `optics_review(optic, a)` free function | Correct |

---

## 3. Internal Profunctors

| PureScript | Rust | Profunctor Instances | Status |
|------------|------|---------------------|--------|
| `Exchange a b` | `Exchange` / `ExchangeBrand` | `Profunctor` | **Complete** |
| `Shop a b` | `Shop` / `ShopBrand` | `Profunctor`, `Strong` | **Complete** |
| `Market a b` | `Market` / `MarketBrand` | `Profunctor`, `Choice` | **Complete** |
| `Stall a b` | `Stall` / `StallBrand` | `Profunctor`, `Strong`, `Choice` | **Complete** |
| `Forget r` | `Forget` / `ForgetBrand` | `Profunctor`, `Strong`, `Choice`, `Wander` | **Missing `Cochoice`** |
| `Tagged` | `Tagged` / `TaggedBrand` | `Profunctor`, `Choice`, `Cochoice`, `Costrong` | **Missing `Closed`** |
| `Bazaar p a b` | `Bazaar` / `BazaarBrand` | `Profunctor`, `Strong`, `Choice`, `Wander` | **Complete** |
| `Grating a b` | `Grating` / `GratingBrand` | `Profunctor`, `Closed` | **Complete** |
| `Zipping` | `Zipping` / `ZippingBrand` | `Profunctor`, `Closed` | **Complete** |
| `Re p s t` | `Re` / `ReBrand` | `Profunctor`, `Cochoice` (when `InnerP: Choice`), `Choice` (when `InnerP: Cochoice`), `Costrong` (when `InnerP: Strong`), `Strong` (when `InnerP: Costrong`) | **Complete** |
| `Indexed p i` | `Indexed` / `IndexedBrand` | `Profunctor`, `Strong`, `Choice`, `Wander` | **Complete** |
| `Focusing m s` | — | — | **Missing** — used for `zoom` in StateT |
| `Star f` | — | — | **Missing** — the library uses `FnBrand<P>` instead, but `Star` would enable `traverseOf` |

### Missing Profunctor Instance Details

**`Forget`: Missing `Cochoice`**
PureScript implements `Cochoice (Forget r)` when `Monoid r`. This enables `Forget` to work with the `Re` profunctor for reversed optics. Without this, operations like using `re` to turn a `Fold` into a `Review` are not possible.

**`Tagged`: Missing `Closed`**
PureScript implements `Closed Tagged` (`closed (Tagged b) = Tagged (const b)`). This would enable `Tagged` to work with `Grate` operations, allowing `review` through grate-like optics.

---

## 4. Profunctor Type Class Hierarchy

| PureScript | Rust | Status |
|------------|------|--------|
| `Profunctor` (`dimap`, `lcmap`, `rmap`) | `Profunctor` (`dimap`, `lmap`, `rmap`) | **Complete** |
| `Strong` (`first`, `second`) | `Strong` (`first`, `second`) | **Complete** |
| `Choice` (`left`, `right`) | `Choice` (`left`, `right`) | **Complete** |
| `Closed` (`closed`) | `Closed<FP>` (`closed`) | **Complete** — parameterized over function brand |
| `Wander` (`wander`) | `Wander` (`wander`) | **Complete** |
| `Costrong` (`unfirst`, `unsecond`) | `Costrong` (`unfirst`, `unsecond`) | **Complete** |
| `Cochoice` (`unleft`, `unright`) | `Cochoice` (`unleft`, `unright`) | **Complete** |

### Combinator Functions on Type Classes

| PureScript | Rust | Status |
|------------|------|--------|
| `(***)` (splitStrong) | — | **Missing** |
| `(&&&)` (fanout) | — | **Missing** |
| `(+++)` (splitChoice) | — | **Missing** |
| `(\|\|\|)` (fanin) | — | **Missing** |

---

## 5. Optic Subtyping Hierarchy

PureScript establishes an optic lattice through profunctor class inheritance. Rust models this via manual trait implementations on concrete structs and on `Composed`.

```
            Iso
          / | \  \
      Lens Prism Grate  Review
        \  |  /
   AffineTraversal
          |
      Traversal
       / | \
  Getter Fold Setter
```

Each concrete struct in Rust implements all super-traits. For example, `Iso` implements `IsoOptic`, `LensOptic`, `PrismOptic`, `AffineTraversalOptic`, `TraversalOptic`, `GrateOptic`, `GetterOptic`, `FoldOptic`, `SetterOptic`, and `ReviewOptic`. The `Composed` struct also implements all of these, enabling composition across the full hierarchy.

**Assessment:** Correct and complete. The subtyping lattice is faithfully reproduced through trait implementations.

---

## 6. Indexed Optics

### 6.1 Status Overview

| Component | PureScript | Rust | Status |
|-----------|-----------|------|--------|
| `Indexed` profunctor | `newtype Indexed p i s t = Indexed (p (Tuple i s) t)` | `Indexed<'a, P, I, A, B>` + `IndexedBrand` | **Complete** |
| `IndexedLens` | `forall p. Strong p => IndexedOptic p i s t a b` | `IndexedLens`, `IndexedLensPrime` structs | **Complete** |
| `IndexedTraversal` | `forall p. Wander p => IndexedOptic p i s t a b` | `IndexedTraversal`, `IndexedTraversalPrime` structs | **Complete** |
| `IndexedFold` | `IndexedFold r i s t a b = IndexedOptic (Forget r) i ...` | `IndexedFold`, `IndexedFoldPrime` structs | **Complete** |
| `IndexedGetter` | `IndexedGetter i s t a b = IndexedFold a i ...` | `IndexedGetter`, `IndexedGetterPrime` structs | **Complete** |
| `IndexedSetter` | `IndexedSetter i s t a b = IndexedOptic Function i ...` | `IndexedSetterOptic` trait | **Complete** |
| `IndexedOptic` type | `type IndexedOptic p i s t a b = Indexed p i a b -> p s t` | `IndexedOpticAdapter` trait | **Complete** |

### 6.2 Indexed Optic Functions

| PureScript | Rust | Status |
|------------|------|--------|
| `iview` | `optics_indexed_view` | **Complete** |
| `iover` | `optics_indexed_over` | **Complete** |
| Indexed set | `optics_indexed_set` | **Complete** |
| Indexed preview | `optics_indexed_preview` | **Complete** |
| `ifoldMapOf` | `optics_indexed_fold_map` | **Complete** |
| `unIndex` | `optics_un_index` | **Complete** |
| `asIndex` | `optics_as_index` | **Complete** |
| `reindexed` | `optics_reindexed` | **Complete** |
| `positions` | `positions` | **Complete** |
| `itraversed` | `IndexedTraversal::traversed()` (via `TraversableWithIndex`) | **Complete** |
| `iuse` | — | **Missing** — requires `MonadState` integration |
| `ifoldrOf`, `ifoldlOf` | — | **Missing** |
| `iallOf`, `ianyOf` | — | **Missing** |
| `ifindOf` | — | **Missing** |
| `itoListOf` | — | **Missing** |
| `itraverseOf`, `iforOf` | — | **Missing** |
| `itraverseOf_`, `iforOf_` | — | **Missing** |
| `iwander` | — (internal `IWanderAdapter` exists but not exposed) | **Missing** as public API |
| `imapped` | — | **Missing** — requires `FunctorWithIndex` |

### 6.3 Indexed Optic Assessment

The indexed optics infrastructure is **substantially complete**. The `Indexed` profunctor wrapper with all necessary instances (`Profunctor`, `Strong`, `Choice`, `Wander`) is correct. Concrete types (`IndexedLens`, `IndexedTraversal`, `IndexedFold`, `IndexedGetter`) are implemented with full trait hierarchies. The library provides both polymorphic (`IndexedLens<'a, P, I, S, T, A, B>`) and monomorphic (`IndexedLensPrime<'a, P, I, S, A>`) variants, mirroring PureScript's `'` convention.

The primary gaps are in the fold/traversal combinator functions (the indexed variants of the non-indexed fold functions that are also mostly missing).

---

## 7. Container Access Type Classes

| PureScript | Rust | Status |
|------------|------|--------|
| `Index m a b` (`ix :: a -> AffineTraversal' m b`) | — | **Missing** |
| `At m a b` (`at :: a -> Lens' m (Maybe b)`) | — | **Missing** |
| `sans :: At m a b => a -> m -> m` | — | **Missing** |

These type classes provide ergonomic container access:
- `ix 1` focuses on element at index 1 of a `Vec`
- `at "key"` focuses on a `Maybe` value in a `Map`

Without `Index` and `At`, users must manually construct affine traversals and lenses for every container access pattern.

---

## 8. Standard Combinators

### 8.1 Tuple Lenses

| PureScript | Rust | Status |
|------------|------|--------|
| `_1 :: Lens (Tuple a c) (Tuple b c) a b` | — | **Missing** |
| `_2 :: Lens (Tuple c a) (Tuple c b) a b` | — | **Missing** |

### 8.2 Option/Result Prisms

| PureScript | Rust | Status |
|------------|------|--------|
| `_Just :: Prism (Maybe a) (Maybe b) a b` | — | **Missing** |
| `_Nothing :: Prism (Maybe a) (Maybe b) Unit Unit` | — | **Missing** |
| `_Left :: Prism (Either a c) (Either b c) a b` | — | **Missing** |
| `_Right :: Prism (Either c a) (Either c b) a b` | — | **Missing** |

### 8.3 Special Lenses/Isos

| PureScript | Rust | Status |
|------------|------|--------|
| `united :: Lens' a Unit` | — | **Missing** |
| `devoid :: Lens' Void a` | — | **Missing** |
| `_Newtype :: Newtype t a => Iso t s a b` | — | **Missing** |

### 8.4 Record Lenses

| PureScript | Rust | Status |
|------------|------|--------|
| `prop :: Proxy l -> Lens (Record r1) (Record r2) a b` | — | N/A — Rust has no row polymorphism; `#[derive(Lens)]` would serve this role |

---

## 9. Composition

| Aspect | PureScript | Rust | Notes |
|--------|-----------|------|-------|
| Mechanism | Function composition (`<<<`, `>>>`) | `Composed<'a, S, T, M, N, A, B, O1, O2>` struct | Both correct |
| Zero-cost | Yes (functions inline) | Yes (static dispatch, monomorphization) | Equivalent |
| Type ergonomics | Composed types are invisible (just `Optic p s t a b`) | Types become deeply nested (`Composed<..., Composed<..., O>>`) | Rust is significantly worse |
| Constructor | `optic1 <<< optic2` | `optics_compose(optic1, optic2)` | Correct |
| Cross-family | Automatic — composition of `Lens` and `Prism` yields `AffineTraversal` | Automatic — `Composed` implements all traits that both operands satisfy | Correct |

---

## 10. Zoom / StateT Integration

| PureScript | Rust | Status |
|------------|------|--------|
| `zoom :: Optic' (Star (Focusing m r)) s a -> StateT a m r -> StateT s m r` | — | **Missing** |
| `Focusing` functor | — | **Missing** |

---

## 11. Partial/Unsafe Operations

| PureScript | Rust | Status |
|------------|------|--------|
| `unsafeView` / `(^?!)` | — | **Missing** — low priority, unsafe |
| `unsafeIndexedFold` / `(^@?!)` | — | **Missing** — low priority, unsafe |

---

## 12. Summary of Flaws and Inconsistencies

### 12.1 Correctness Issues

**No correctness bugs identified.** All profunctor encodings, trait hierarchies, and optic evaluations are faithful to the PureScript reference. The profunctor laws are preserved in all implementations.

### 12.2 Missing Profunctor Instances

1. **`Forget`: Missing `Cochoice`** — prevents using `Forget` with `Re` for reversed fold/getter operations.
2. **`Tagged`: Missing `Closed`** — prevents `Tagged` from working with `Grate` operations.
3. **`Star` profunctor not implemented** — prevents `traverseOf`/`sequenceOf` style effectful traversals. The library uses `FnBrand<P>` for function abstraction but lacks the Kleisli-arrow profunctor.

### 12.3 Architectural Gaps

1. **No top-level `re` combinator** — The `Re` profunctor and its instances (`Profunctor`, `Cochoice`, `Choice`, `Costrong`, `Strong`) are fully implemented, but there is no free function `re(optic) -> reversed_optic` that users can call. Only `Iso::reversed()` exists as a concrete method.

2. **No `clone*`/`with*` extraction functions** — PureScript provides `withLens`, `withPrism`, `withIso`, `withGrate`, `withAffineTraversal` for CPS-style extraction, and `cloneLens`, `clonePrism`, etc. for reconstructing polymorphic optics from concrete ones. None of these exist in Rust, though direct struct field access partially compensates.

3. **No `Index`/`At` type classes** — This is the single largest usability gap. Without these, every container access requires manually constructing an optic.

### 12.4 Type Parameter Ordering and Naming

#### 12.4.1 `P` Naming Ambiguity

The type parameter `P` is used for three semantically distinct roles:

| Context | `P` means | Bound | Example |
|---------|-----------|-------|---------|
| `Optic<'a, P, S, T, A, B>` | Profunctor brand | `P: Profunctor` | `Optic::evaluate` |
| `Lens<'a, P, S, T, A, B>` | Pointer brand (Rc vs Arc) | `P: UnsizedCoercible` | `Lens::new`, `Prism::new`, etc. |
| `optics_from<'a, P, O, S, A>` | Cloneable function brand | `P: CloneableFn` | `optics_from`, `optics_to` |

A user seeing `Lens<'a, P, ...>` alongside `Optic<'a, P, ...>` would reasonably assume the same kind of parameter. The bounds disambiguate at the definition site, but not at call sites where inference fills them in.

**Approaches:**

1. **Standardize on `Q` for pointer brands.** Profunctor stays `P`, pointer brand becomes `Q` everywhere, fn brand stays `FP`. Minimal diff. The `P`/`Q` distinction is sufficient when bounds are visible. This would change concrete optic structs (e.g. `Lens<'a, Q, S, T, A, B>`) and all free functions that currently use `P` for a pointer brand.

2. **Use multi-letter names.** Profunctor stays `P`, pointer brand becomes `Ptr`, fn brand stays `FP`. More verbose but unambiguous at every call site. Unusual for Rust type parameter names, but the codebase already uses `FP` as a multi-letter parameter.

3. **Rename the profunctor brand.** Since concrete optic structs (the most user-facing surface) use `P` for the pointer brand far more often than the `Optic` trait uses `P` for a profunctor, rename the profunctor parameter to `Prof`. This privileges the most common usage and only changes the base `Optic` trait and `optics_eval`.

#### 12.4.2 `P` vs `Q` Inconsistency in Free Functions

Free functions inconsistently use `P` and `Q` for the pointer brand:

| Function | Letter | Bound | Role |
|----------|--------|-------|------|
| `optics_view<'a, P, O, S, A>` | `P` | `UnsizedCoercible` | pointer brand |
| `optics_preview<'a, P, O, S, A>` | `P` | `UnsizedCoercible` | pointer brand |
| `optics_set<'a, Q, O, S, A>` | `Q` | `UnsizedCoercible` | pointer brand |
| `optics_over<'a, Q, O, S, A, F>` | `Q` | `UnsizedCoercible` | pointer brand |
| `optics_from<'a, P, O, S, A>` | `P` | `CloneableFn` | fn brand |
| `optics_eval<'a, P, O, S, T, A, B>` | `P` | `Profunctor` | profunctor brand |
| `optics_indexed_view<'a, P, O, I, S, A>` | `P` | `UnsizedCoercible` | pointer brand |
| `optics_indexed_over<'a, Q, O, I, S, A, F>` | `Q` | `UnsizedCoercible` | pointer brand |
| `optics_indexed_set<'a, Q, O, I, S, A>` | `Q` | `UnsizedCoercible` | pointer brand |

The split correlates with read-side (`P`) vs write-side (`Q`) functions, but this distinction has no semantic justification — both represent the same concept. This is compounded by `optics_eval` and `optics_from` reusing `P` for entirely different roles.

**Approaches:**

Whichever naming convention is adopted for the ambiguity issue (12.4.1) should be applied uniformly here. Any of the three approaches listed above would eliminate this inconsistency as a side effect.

#### 12.4.3 Trait-Level vs Method-Level Brand Placement

The Rust-specific brand parameters (pointer brand, fn brand) are placed at the trait level in some optic traits but at the method level in others:

| Trait | Brand position | Signature |
|-------|---------------|-----------|
| `IsoOptic<'a, S, T, A, B>` | method | `evaluate<P: Profunctor>(...)` |
| `LensOptic<'a, S, T, A, B>` | method | `evaluate<P: Strong>(...)` |
| `PrismOptic<'a, S, T, A, B>` | method | `evaluate<P: Choice>(...)` |
| `AffineTraversalOptic<'a, S, T, A, B>` | method | `evaluate<P: Strong + Choice>(...)` |
| `TraversalOptic<'a, S, T, A, B>` | method | `evaluate<P: Wander>(...)` |
| `GetterOptic<'a, S, A>` | method | `evaluate<R, P: UnsizedCoercible>(...)` |
| `FoldOptic<'a, S, A>` | method | `evaluate<R: Monoid, P: UnsizedCoercible>(...)` |
| `SetterOptic<'a, P: UnsizedCoercible, S, T, A, B>` | trait | `evaluate(...)` |
| `GrateOptic<'a, FP: CloneableFn, S, T, A, B>` | trait | `evaluate<Z: Profunctor>(...)` |
| `ReviewOptic<'a, S, T, A, B>` | neither | `evaluate(...)` (fixed to `TaggedBrand`) |

The first group (`IsoOptic` through `TraversalOptic`) places the profunctor at method level — these traits express "this optic works with *any* profunctor satisfying the constraint", and the method-level parameter captures that universality.

`GetterOptic` and `FoldOptic` also use method-level parameters, but for a *different* reason: they fix the profunctor to `ForgetBrand<P, R>` and expose `P` (pointer brand) and `R` (result/monoid type) as method parameters because different call sites may supply different `P` and `R` values for the same optic.

`SetterOptic` and `GrateOptic` place their brand at the trait level. For `SetterOptic`, this is because the setter is concretely tied to `FnBrand<P>` — the pointer brand `P` determines the implementation, so it's a fixed property of the optic instance rather than a call-site choice. `GrateOptic` similarly fixes the function brand.

The rationale is sound (trait-level for fixed brands, method-level for universal quantification), but the observable effect is inconsistent downstream bounds:

```rust
// Getter: no brand in trait position
fn optics_view<P, O, S, A>(optic: &O, s: S) -> A
where O: GetterOptic<'a, S, A> { ... }

// Setter: brand in trait position
fn optics_set<Q, O, S, A>(optic: &O, s: S, a: A) -> S
where O: SetterOptic<'a, Q, S, S, A, A> { ... }
```

A function constrained on *both* (e.g. a lens used as both getter and setter) must express the brand in two different structural positions. This also causes `GetterOptic` and `FoldOptic` to drop `T` and `B` from their trait parameters (since they only read), while `SetterOptic` keeps them — a further asymmetry, though one that is semantically justified.

### 12.5 Missing Combinator Categories

| Category | PureScript Count | Rust Count | Coverage |
|----------|-----------------|------------|----------|
| Iso construction/manipulation | 12 functions | 4 functions | 33% |
| Lens construction/manipulation | 5 functions | 2 constructors | 40% |
| Prism construction/manipulation | 8 functions | 1 constructor | 13% |
| Traversal construction/manipulation | 8 functions | 1 constructor + `traversed()` | 25% |
| Getter operations | 6 functions | 1 (`optics_view`) | 17% |
| Setter operations | 20+ functions/operators | 2 (`optics_set`, `optics_over`) | 10% |
| Fold operations | 25+ functions | 1 (`optics_preview`) | 4% |
| Grate operations | 5 functions | 1 (`zip_with_of`) | 20% |
| Standard combinators | 10+ (`_1`, `_Just`, etc.) | 0 | 0% |
| Indexed operations | 15+ functions | 8 functions | 53% |

### 12.6 Naming Convention for Internal Profunctor Brand Parameters

The internal profunctors consistently use the descriptive name `FnBrand` for their function brand parameter (`Exchange<'a, FnBrand: CloneableFn, A, B, S, T>`), while the `GrateOptic` trait uses `FP` and the `SetterOptic` trait uses plain `P`. These are three different names for the same concept across the codebase. Whichever convention is adopted for section 12.4.1 should also be applied here for consistency.

---

## 13. Recommendations

### Phase 1: Standard Combinators (High Impact / Low Complexity)

These are self-contained, require no new infrastructure, and provide immediate ergonomic value.

1. **Tuple lenses**: `_1`, `_2` (and `_3`, `_4`, etc. for larger tuples).
2. **Option/Result prisms**: `_Some`, `_None`, `_Ok`, `_Err`.
3. **Common isos**: `non` (converts `Option<A>` to `A` given a default), `curried`/`uncurried`.

Suggested location: `fp-library/src/types/optics/combinators.rs`.

### Phase 2: Fold Functions (High Impact / Medium Complexity)

The fold API is the most incomplete area. Implement these using `Forget` with appropriate monoids:

1. **Core**: `foldMapOf`, `foldOf`, `toListOf` (or `to_vec_of` for Rust idiom).
2. **Quantifiers**: `allOf`, `anyOf`, `has`, `hasn't`.
3. **Aggregation**: `sumOf`, `productOf`, `lengthOf`.
4. **Search**: `firstOf`, `lastOf`, `findOf`.
5. **Filtering**: `filtered` (an optic that only matches elements satisfying a predicate).
6. **Directional**: `foldrOf`, `foldlOf`.

### Phase 3: Missing Profunctor Instances (Medium Impact / Low Complexity)

1. **`Cochoice` for `Forget`**: Implement `unleft`/`unright` using monoid empty for the discarded branch.
2. **`Closed` for `Tagged`**: Implement `closed` as `Tagged(const b)`.

### Phase 4: `Index` / `At` Type Classes (High Impact / Medium Complexity)

1. Define `Index` trait with `ix` method returning `AffineTraversal'`.
2. Define `At` trait with `at` method returning `Lens' m (Option<b>)`.
3. Implement for: `Vec`, `HashMap`, `BTreeMap`, `Option`, `Result`.
4. Add `sans` helper (delete by key).

### Phase 5: Effectful Traversal Support (Medium Impact / High Complexity)

Implementing `Star` profunctor would unlock:
- `traverseOf` — effectful traversal
- `sequenceOf` — sequencing effects through a traversal
- `itraverseOf`, `iforOf` — indexed effectful traversal

This requires careful design around Rust's lack of higher-kinded types for the functor parameter.

### Phase 6: Generic `re` Combinator (Low Impact / Low Complexity)

Expose a top-level `re(optic)` function that wraps optic evaluation through the `Re` profunctor. This requires `Cochoice` for `Forget` (Phase 3) to be maximally useful.

### Phase 7: Derive Macros (High Impact / High Complexity)

`#[derive(Lens)]` and `#[derive(Prism)]` macros to auto-generate optics for struct fields and enum variants. This is the Rust equivalent of PureScript's `prop` record lens.

---

## 14. Conclusion

The Rust `fp-library` optics implementation is **architecturally sound and mathematically correct**. All profunctor encodings faithfully reproduce the PureScript reference. The core optic types (Iso, Lens, Prism, AffineTraversal, Traversal, Grate, Fold, Getter, Setter, Review) and their internal profunctors (Exchange, Shop, Market, Stall, Bazaar, Grating, Forget, Tagged, Zipping, Re) are complete and correct.

The indexed optics system is substantially implemented with the `Indexed` profunctor, `IndexedLens`, `IndexedTraversal`, `IndexedFold`, `IndexedGetter`, and their associated traits and free functions — a significant achievement given the complexity of encoding indexed profunctors in Rust's type system.

The primary maturity gaps are:
1. **Combinator functions** — particularly the Fold API (4% coverage), Setter API (10% coverage), and standard combinators (0% coverage).
2. **Container access** — `Index`/`At` type classes for ergonomic keyed/indexed access.
3. **Two missing profunctor instances** — `Cochoice` for `Forget` and `Closed` for `Tagged`.

The foundation is solid. The remaining work is predominantly additive (new functions and type class instances) rather than corrective.
