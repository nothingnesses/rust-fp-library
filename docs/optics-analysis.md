# Comprehensive Comparison: Rust `fp-library` Optics vs PureScript `purescript-profunctor-lenses`

This document provides a detailed, side-by-side comparison of the optics implementation in the Rust `fp-library` (under `fp-library/src/types/optics/`) against the PureScript reference implementation `purescript-profunctor-lenses` (under `src/Data/Lens/`). It evaluates correctness, completeness, and identifies flaws, inconsistencies, and missing features.

---

## 1. Fundamental Architecture

### PureScript Approach: Optics as Functions

In PureScript, an optic is a type alias for a function:

```purescript
type Optic p s t a b = p a b -> p s t
```

Specific optic families constrain `p` using rank-2 polymorphism:

```purescript
type Lens s t a b = forall p. Strong p => Optic p s t a b
type Prism s t a b = forall p. Choice p => Optic p s t a b
type Iso s t a b = forall p. Profunctor p => Optic p s t a b
type Traversal s t a b = forall p. Wander p => Optic p s t a b
type AffineTraversal s t a b = forall p. Strong p => Choice p => Optic p s t a b
type Grate s t a b = forall p. Closed p => Optic p s t a b
```

Composition is simply function composition (`<<<`), requiring no special machinery.

### Rust Approach: Optics as Trait Objects

Rust cannot express rank-2 types or universally quantified type aliases. The library uses a trait-based encoding:

```rust
pub trait Optic<'a, P: Profunctor, S, T, A, B> {
    fn evaluate(&self, pab: P::Of<'a, A, B>) -> P::Of<'a, S, T>;
}
```

Each optic family has a corresponding specialized trait (`IsoOptic`, `LensOptic`, `PrismOptic`, etc.) where the profunctor constraint is moved to the `evaluate` method's type parameter:

```rust
pub trait LensOptic<'a, S, T, A, B> {
    fn evaluate<P: Strong>(&self, pab: ...) -> ...;
}
```

Composition uses a `Composed<'a, S, T, M, N, A, B, O1, O2>` struct rather than function composition.

**Assessment:** This is a correct and well-motivated translation. Rust's monomorphization provides zero-cost abstraction. The dual-trait approach (`Optic<P>` for a fixed P, plus `LensOptic` for `forall P: Strong`) faithfully models the rank-2 polymorphism that PureScript gets for free.

---

## 2. Optic-by-Optic Comparison

### 2.1 Iso

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Iso s t a b = forall p. Profunctor p => p a b -> p s t` | `Iso<'a, P, S, T, A, B>` struct + `IsoOptic` trait | Correct |
| Construction | `iso :: (s -> a) -> (b -> t) -> Iso s t a b` via `dimap f g pab` | `Iso::new(from, to)` storing cloneable fns | Correct |
| Encoding | `iso f g pab = dimap f g pab` | `Q::dimap(from, to, pab)` | Correct |
| Concrete (A-) | `AnIso s t a b = Optic (Exchange a b) s t a b` | Uses `ExchangeBrand<A, B>` | Correct |
| Extraction | `withIso :: AnIso s t a b -> ((s -> a) -> (b -> t) -> r) -> r` | `optics_from` / `optics_to` (limited to monomorphic) | Partial |
| Reversal | `re :: Optic (Re p a b) s t a b -> Optic p b a t s` | `IsoPrime::reversed()` (monomorphic only) | Partial |
| Subtyping | Iso is a Lens, Prism, Traversal, Grate, Getter, Setter, Review, Fold | All implemented | Correct |

**Issues:**
- **Missing `withIso`**: PureScript's `withIso` extracts both morphisms from an `AnIso` in CPS style. Rust provides `optics_from`/`optics_to` but no unified CPS-style extractor.
- **Missing `re`**: PureScript has a general `re` combinator using the `Re` profunctor that works on any optic. Rust only has `IsoPrime::reversed()`, limited to monomorphic isos. The `Re` profunctor itself is not implemented.
- **Missing `cloneIso`**: PureScript has `cloneIso :: AnIso s t a b -> Iso s t a b`. No Rust equivalent.
- **Missing utility functions**: `au`, `auf`, `under`, `non`, `curried`, `uncurried`, `flipped`, `mapping`, `dimapping`, `coerced` are all absent.

### 2.2 Lens

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Lens s t a b = forall p. Strong p => p a b -> p s t` | `Lens<'a, P, S, T, A, B>` struct + `LensOptic` trait | Correct |
| Construction | `lens :: (s -> a) -> (s -> b -> t) -> Lens s t a b` | `Lens::new(to: S -> (A, B -> T))` | Correct |
| Encoding | `dimap (\s -> (get s, s)) (\(b, s) -> set s b) (first pab)` | Same: `dimap(to, |(b, f)| f(b), first(pab))` | Correct |
| Concrete (A-) | `ALens s t a b = Optic (Shop a b) s t a b` | Uses `ShopBrand<FnBrand, A, B>` | Correct |
| Extraction | `withLens :: ALens -> ((s -> a) -> (s -> b -> t) -> r) -> r` | `Lens` struct fields are accessible directly | Different but ok |
| Subtyping | Lens is a Traversal, Getter, Setter, Fold | All implemented | Correct |

**Issues:**
- **Setter signature difference**: PureScript's `lens` takes `(s -> a)` and `(s -> b -> t)` (curried setter). Rust's `Lens::new` takes a single function returning both the view and the setter closure `S -> (A, B -> T)`. This avoids `S: Clone`.
- **Legacy constructors**: Convenience constructors `Lens::from_view_set` are provided that match the old `(S -> A, (S, B) -> T)` signature but require `S: Clone`.
- **Missing `cloneLens`**: PureScript's `cloneLens :: ALens s t a b -> Lens s t a b`. Not present.
- **Missing `lensStore`**: PureScript's utility for extracting `Tuple a (b -> t)` from an `ALens`. Not present.
- **Missing indexed lens**: PureScript has `IndexedLens`, `ilens`, `ilens'`, `cloneIndexedLens`. Not present in Rust.

### 2.3 Prism

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Prism s t a b = forall p. Choice p => p a b -> p s t` | `Prism<'a, P, S, T, A, B>` struct + `PrismOptic` trait | Correct |
| Construction | `prism :: (b -> t) -> (s -> Either t a) -> Prism s t a b` | `Prism::new(preview: S -> Result<A, T>, review: B -> T)` | Correct |
| Encoding | `dimap fro (either id id) (right (rmap to pab))` | `dimap(preview, |r| match r { Ok(b) => review(b), Err(t) => t }, right(pab))` | Correct |
| Concrete (A-) | `APrism s t a b = Optic (Market a b) s t a b` | Uses `MarketBrand<FnBrand, A, B>` | Correct |
| `PrismPrime` | Uses `prism'` with `Maybe` | `PrismPrime::from_option(S -> Option<A>, A -> S)` | Correct |
| Subtyping | Prism is a Traversal, Fold, Setter, Review | All implemented | Correct |

**Issues:**
- **`S: Clone` requirement**: `PrismPrime` no longer requires `S: Clone` for its optic implementations. The internal encoding has been updated to use `Result<A, S>` instead of `Option<A>`. The legacy `new` constructor has been renamed to `from_option` (which still requires `S: Clone` to adapt `Option` to `Result`), and `new` now accepts a `S -> Result<A, S>` preview function without `Clone`.
- **Missing `only`, `nearly`**: Convenience prism constructors for equality-based matching. Not present.
- **Missing `is`, `isn't`**: Boolean predicates for prism matching. Not present.
- **Missing `matching`**: Extract the `Either t a` from a prism. Not present.
- **Missing `below`**: Lift a prism through a Traversable. Not present.
- **Missing `clonePrism`**, **`withPrism`**: Not present.

### 2.4 AffineTraversal

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `forall p. Strong p => Choice p => p a b -> p s t` | `AffineTraversal<'a, P, S, T, A, B>` struct | Correct |
| Constraint | `Strong + Choice` | `Strong + Choice` | Correct |
| Construction | `affineTraversal :: (s -> b -> t) -> (s -> Either t a) -> ...` | `AffineTraversal::new(to: S -> Result<(A, B -> T), T>)` | Correct |
| Encoding | `dimap to (\(Tuple b f) -> either identity b f) (second (right pab))` | `dimap(split, merge, right(first(pab)))` | **Different** |
| Concrete (A-) | `AnAffineTraversal = Optic (Stall a b) s t a b` | Uses `StallBrand<FnBrand, A, B>` | Correct |

**Issues:**
- **`S: Clone` requirement**: The `S: Clone` requirement has been removed from `AffineTraversal` and `AffineTraversalPrime`. The `new` constructor now accepts the internal encoding directly. Convenience constructors `from_preview_set` are provided for backward compatibility (requiring `S: Clone`).
- **Encoding difference**: PureScript uses `second (right pab)` while Rust uses `right(first(pab))`. Both are valid profunctor encodings of an affine traversal—they use `Strong` and `Choice` in different orders.
- **Missing `AffineTraversalOptic` trait**: Unlike other optics, there is no dedicated `AffineTraversalOptic` trait. The `AffineTraversal` struct implements `TraversalOptic`, `FoldOptic`, and `SetterOptic` but not a unique affine-specific trait. This means you cannot compose two affine traversals and guarantee the result is affine rather than a general traversal.
- **Missing `cloneAffineTraversal`**, **`withAffineTraversal`**: Not present.

### 2.5 Traversal

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Traversal s t a b = forall p. Wander p => p a b -> p s t` | `Traversal<'a, P, S, T, A, B, F>` struct + `TraversalOptic` trait | Correct |
| Construction | `wander :: (forall f. Applicative f => (a -> f b) -> s -> f t) -> p a b -> p s t` | Via `TraversalFunc` trait and `Wander::wander` | Correct |
| Concrete (A-) | `ATraversal = Optic (Bazaar (->) a b) s t a b` | No `Bazaar` equivalent | **Missing** |
| `traversed` | `traversed = wander traverse` | Not found as built-in | Partial |

**Issues:**
- **Extra type parameter `F`**: Rust's `Traversal` struct has an extra type parameter `F: TraversalFunc`, which is the traversal function itself. PureScript hides this in the closure. This adds complexity but is necessary for Rust's static dispatch.
- **Missing `Bazaar` profunctor**: PureScript's `Bazaar` is used for `ATraversal` (concrete traversal) and `cloneTraversal`. Not implemented in Rust.
- **Missing `cloneTraversal`**: No way to clone/reconstruct a traversal from its concrete representation.
- **Missing `traverseOf`**, **`sequenceOf`**, **`failover`**: Key traversal operation functions. Not present.
- **Missing `element`**, **`elementsOf`**: Index-based traversal focusing. Not present.
- **Missing `both`**: Bitraversable traversal. Not present.
- **Missing `traversed`**: The universal traversal for `Traversable` types. Not found as a built-in combinator.

### 2.6 Grate

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Grate s t a b = forall p. Closed p => p a b -> p s t` | `Grate<'a, P, S, T, A, B>` struct + `GrateOptic` trait | Correct |
| Encoding | `grate f pab = dimap (#) f (closed pab)` | `dimap(extract, reconstruct, closed(pab))` | Correct |
| Concrete (A-) | `AGrate = Optic (Grating a b) s t a b` | Uses `GratingBrand<FnBrand, A, B>` | Correct |

**Issues:**
- **`S: Clone` and `A: Clone` requirements**: The `S: Clone` and `A: Clone` requirements have been removed from the `GrateOptic` and `SetterOptic` implementations. The implementation now leverages `RefCountedPointer` to share the structure `S` within the internal closure without requiring the type itself to be `Clone`.
- **Missing `withGrate`**, **`cloneGrate`**: Not present.
- **Missing `cotraversed`**: The universal grate for `Distributive` functors. Not present.
- **Missing `zipWithOf`**, **`zipFWithOf`**, **`collectOf`**: Grate operation functions. Not present.
- **Missing `Zipping` profunctor**: PureScript's `Zipping` (for `zipWithOf`) is not implemented.

### 2.7 Getter

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Getter s t a b = forall r. Fold r s t a b` | `Getter<'a, P, S, T, A, B>` struct + `GetterOptic` trait | Correct |
| Fixed profunctor | `Forget r` | `ForgetBrand<P, R>` | Correct |
| Construction | `to :: (s -> a) -> Getter s t a b` | `Getter::new(view_fn)` | Correct |

**Issues:**
- **Getter type parameters**: PureScript's `Getter` is universally quantified over `r` (the Forget parameter). Rust's `GetterOptic` trait achieves this by having `evaluate<R: 'a + 'static, P: UnsizedCoercible + 'static>`. This is correct.
- **Missing `view` / `(^.)` operator**: PureScript provides `view :: AGetter s t a b -> s -> a`. Rust has `optics_view` but no operator-style usage.
- **Missing `takeBoth`**: Combine two getters. Not present.
- **Missing `use`**: Monadic getter in `MonadState`. Not present.
- **Missing `cloneGetter`**: Not present.
- **Missing indexed getters**: `iview`, `iuse`. Not present.

### 2.8 Setter

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Setter s t a b = Optic Function s t a b` | `Setter<'a, P, S, T, A, B>` struct + `SetterOptic` trait | Correct |
| Fixed profunctor | `Function` (`->`) | `FnBrand<P>` | Correct |

**Issues:**
- **Missing rich setter API**: PureScript provides `over`, `set`, `addOver`, `mulOver`, `subOver`, `divOver`, `disjOver`, `conjOver`, `appendOver`, `setJust`, `assign`, `modifying`, etc. Rust has `optics_set` and `optics_over` only.
- **Missing indexed setters**: `iover`. Not present.

### 2.9 Fold

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Fold r s t a b = Optic (Forget r) s t a b` | `Fold<'a, P, S, T, A, B>` struct + `FoldOptic` trait | Correct |
| Construction | Various (`folded`, `filtered`, `replicated`, `unfolded`) | `Fold::new(to_vec_fn)` | **Simplified** |

**Issues:**
- **Simplified construction**: PureScript's `Fold` is constructed through various combinators that work with the `Forget` monoid. Rust's `Fold` takes a `S -> Vec<A>` function, which is much simpler but loses the generality of the monoid-based approach.
- **Missing extensive fold API**: PureScript provides `preview`, `foldOf`, `foldMapOf`, `foldrOf`, `foldlOf`, `toListOf`, `firstOf`, `lastOf`, `maximumOf`, `minimumOf`, `allOf`, `anyOf`, `andOf`, `orOf`, `elemOf`, `notElemOf`, `sumOf`, `productOf`, `lengthOf`, `findOf`, `sequenceOf_`, `traverseOf_`, `has`, `hasn't`, `filtered`, `replicated`, `folded`, `unfolded`, `toArrayOf`. Rust has only `optics_preview`.
- **Missing indexed fold API**: `ifoldMapOf`, `ifoldrOf`, `ifoldlOf`, `iallOf`, `ianyOf`, `ifindOf`, `itoListOf`, `itraverseOf_`, `iforOf_`. Not present.

### 2.10 Review

| Aspect | PureScript | Rust | Match? |
|--------|-----------|------|--------|
| Type | `type Review s t a b = Optic Tagged s t a b` | `Review<'a, P, S, T, A, B>` struct + `ReviewOptic` trait | Correct |
| Fixed profunctor | `Tagged` | `TaggedBrand` | Correct |
| `review` function | `review :: Review s t a b -> b -> t` (via `under Tagged`) | `optics_review` | Correct |

**Issues:**
- No significant structural issues. The `Review` implementation is straightforward and correct.

---

## 3. Internal Profunctors

### 3.1 Exchange (Iso)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `Exchange (s -> a) (b -> t)` | `Exchange { get, set }` using `FnBrand` |
| Instances | `Profunctor` | `Profunctor` (via `ExchangeBrand`) |

**Assessment:** Correct. `Exchange` is now parameterized over a cloneable function brand (`ExchangeBrand<FnBrand, A, B>`), consistent with `Shop`, `Market`, `Stall`, and `Grating`. The helper functions `optics_from` and `optics_to` require an explicit `FnBrand` parameter (e.g., `optics_from::<RcFnBrand, _, _, _>`).

### 3.2 Shop (Lens)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `Shop (s -> a) (s -> b -> t)` | `Shop { get, set }` using `FnBrand` |
| Instances | `Profunctor`, `Strong` | `Profunctor`, `Strong` (via `ShopBrand`) |

**Assessment:** Correct.

### 3.3 Market (Prism)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `Market (b -> t) (s -> Either t a)` | `Market { preview, review }` using `FnBrand` |
| Instances | `Profunctor`, `Choice` (+ `Functor`) | `Profunctor`, `Choice` (via `MarketBrand`) |

**Assessment:** Correct.

### 3.4 Stall (AffineTraversal)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `Stall (s -> b -> t) (s -> Either t a)` | `Stall { get, set }` using `FnBrand` |
| Instances | `Profunctor`, `Strong`, `Choice` (+ `Functor`) | `Profunctor`, `Strong`, `Choice` (via `StallBrand`) |

**Assessment:** Correct.

### 3.5 Forget (Getter/Fold)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `newtype Forget r a b = Forget (a -> r)` | `Forget<'a, P, R, A, B>` wrapping `A -> R` |
| Instances | `Profunctor`, `Strong`, `Choice` (when `Monoid r`), `Wander` (when `Monoid r`), `Cochoice` | `Profunctor`, `Strong`, `Choice` (when `Monoid`), `Wander` (when `Monoid`) |

**Issues:**
- **Missing `Cochoice`**: PureScript's `Forget` implements `Cochoice`. Not present in Rust (the `Cochoice` class itself is not implemented).
- **Missing `Semigroup`/`Monoid` derivation**: PureScript derives `Semigroup` and `Monoid` for `Forget`. Rust does not.

### 3.6 Tagged (Review)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `newtype Tagged a b = Tagged b` | `Tagged<'a, A, B>(pub B)` tuple struct |
| Instances | `Profunctor`, `Choice`, `Closed`, `Costrong`, `Functor`, `Foldable`, `Traversable`, `Eq`, `Ord` | `Profunctor`, `Choice` (via `TaggedBrand`) |

**Issues:**
- **Missing `Closed`**: PureScript's `Tagged` implements `Closed`. Not present in Rust.
- **Missing `Costrong`**: Not present (trait not implemented).
- **Missing `Foldable`, `Traversable`**: Not applicable in the same way for Rust, but the trait instances are missing.

### 3.7 Grating (Grate)

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Data | `Grating (((s -> a) -> b) -> t)` | `Grating` wrapping the same function |
| Instances | `Profunctor`, `Closed` | `Profunctor`, `Closed` (via `GratingBrand`) |

**Assessment:** Correct.

### 3.8 Missing Internal Profunctors

| PureScript | Purpose | Present in Rust? |
|-----------|---------|-----------------|
| `Bazaar` | Concrete `ATraversal`, `cloneTraversal` | **No** |
| `Zipping` | `zipWithOf` for grates | **No** |
| `Re` | Reversing optics (`re` combinator) | **No** |
| `Indexed` | Indexed optics (`IndexedOptic`, etc.) | **No** |
| `Focusing` | `zoom` into `StateT` | **No** |

---

## 4. Optic Subtyping Hierarchy

PureScript establishes a lattice of optic subtyping through profunctor class inheritance:

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

The Rust implementation models this via manual trait implementations. Each concrete optic struct implements all the traits it should be a subtype of:

| Struct | Implements | Correct? |
|--------|-----------|----------|
| `Iso` | `IsoOptic`, `GrateOptic`, `LensOptic`, `PrismOptic`, `TraversalOptic`, `GetterOptic`, `FoldOptic`, `SetterOptic`, `ReviewOptic` | Yes |
| `Lens` | `LensOptic`, `TraversalOptic`, `GetterOptic`, `FoldOptic`, `SetterOptic` | Yes |
| `Prism` | `PrismOptic`, `TraversalOptic`, `FoldOptic`, `SetterOptic`, `ReviewOptic` | Yes |
| `AffineTraversal` | `TraversalOptic`, `FoldOptic`, `SetterOptic` | **Missing**: No `AffineTraversalOptic` |
| `Traversal` | `TraversalOptic`, `FoldOptic`, `SetterOptic` | Yes |
| `Grate` | `GrateOptic`, `SetterOptic` | Yes |
| `Getter` | `GetterOptic`, `FoldOptic` | Yes |
| `Fold` | `FoldOptic` | Yes |
| `Setter` | `SetterOptic` | Yes |
| `Review` | `ReviewOptic` | Yes |

**Issue:** `AffineTraversal` does not have its own `AffineTraversalOptic` trait. It should be positioned between Lens/Prism and Traversal in the hierarchy but is instead directly collapsed into `TraversalOptic`. This means composing a Lens with a Prism yields a `Traversal` rather than the more precise `AffineTraversal`.

---

## 5. Composition

### PureScript
Optics compose with `<<<` (function composition), which is seamless and preserves the tightest possible constraint:

```purescript
_foo <<< _bar  -- If both are Lenses, result is a Lens
```

### Rust
Optics compose with the `Composed` struct and `optics_compose` function. The `Composed` struct implements each `*Optic` trait when both sub-optics implement that trait:

```rust
impl<O1: LensOptic, O2: LensOptic> LensOptic for Composed<O1, O2> { ... }
impl<O1: PrismOptic, O2: PrismOptic> PrismOptic for Composed<O1, O2> { ... }
```

**Issues:**
- **Cross-family composition produces the wrong subtype**: Composing a `LensOptic` with a `PrismOptic` should produce an `AffineTraversal`, but there is no `AffineTraversalOptic` trait, and `Composed` doesn't have an impl that combines `LensOptic + PrismOptic -> AffineTraversalOptic`.
- **Verbose type annotations**: The `Composed` struct has many generic parameters (`S, T, M, N, A, B, O1, O2`), making composition verbose compared to PureScript's `<<<`.
- **Same-family only**: Each `Composed` impl requires both optics to implement the *same* trait. This means Lens-Lens composition is a Lens, but Lens-Prism composition only works as a `Traversal` (via the `Optic<P>` trait with `P: Strong + Choice`), not as a proper `AffineTraversal`.

---

## 6. Missing Features

### 6.1 Indexed Optics (Entirely Missing)

PureScript has comprehensive indexed optics support:

```purescript
type IndexedOptic p i s t a b = p (Tuple i a) b -> p s t
type IndexedTraversal i s t a b = forall p. Wander p => IndexedOptic p i s t a b
type IndexedFold r i s t a b = IndexedOptic (Forget r) i s t a b
type IndexedGetter i s t a b = IndexedFold (Tuple i a) i s t a b
type IndexedSetter i s t a b = IndexedOptic Function i s t a b
```

With the `Indexed` profunctor transformer and operations like `itraverseOf`, `ifoldMapOf`, `iover`, `iview`, `positions`, `iwander`, etc.

**None of this exists in the Rust implementation.** This is the single largest missing feature.

### 6.2 Re Profunctor (Missing)

PureScript's `Re` profunctor enables reversing optics:

```purescript
newtype Re p s t a b = Re (p b a -> p t s)
```

With instances that swap Strong/Costrong, Choice/Cochoice. This enables `re :: Optic (Re p a b) s t a b -> Optic p b a t s` which can reverse any suitable optic.

### 6.3 Zoom / Focusing (Missing)

PureScript provides `zoom` for using lenses within `StateT`:

```purescript
zoom :: Lens' s a -> StateT a m r -> StateT s m r
```

With the `Focusing` functor. Not present in Rust.

### 6.4 Common / Predefined Optics (Missing)

PureScript provides many predefined optics:
- `Data.Lens.Lens.Tuple`: `_1`, `_2`
- `Data.Lens.Lens.Unit`, `Data.Lens.Lens.Void`
- `Data.Lens.Prism.Maybe`: `_Just`, `_Nothing`
- `Data.Lens.Prism.Either`: `_Left`, `_Right`
- `Data.Lens.Record`: Row-polymorphic record lenses
- `Data.Lens.At`, `Data.Lens.Index`: Indexed container access

None of these exist in the Rust library.

---

## 7. Correctness Issues

### 7.1 Clone Requirements

Historically, several optic encodings required `Clone` on types where PureScript had no such constraint. This limitation has been largely addressed in recent updates:

| Optic | Rust `Clone` requirement | Status |
|-------|-------------------------|--------|
| `Lens` | `S: Clone` | **Removed** from `Lens::new` and optic impls. |
| `PrismPrime` | `S: Clone` | **Removed** from `PrismPrime::new` and optic impls. |
| `AffineTraversal` | `S: Clone` | **Removed** from `AffineTraversal::new` and optic impls. |
| `Grate` | `S: Clone + A: Clone` | **Removed** from optic impl |

The fundamental limitation of needing `s` twice (once for the getter, once for the setter) has been resolved by adopting internal encodings closer to PureScript's "prime" variants (e.g., `s -> (a, b -> t)` for Lens), which return the setter closure directly. The legacy constructors requiring `Clone` have been moved to `from_view_set` / `from_option` / `from_preview_set` for convenience, while the primary `new` constructors now accept the efficient non-cloning encoding.

### 7.2 `Prism` Encoding: `right` vs `left`

PureScript's `prism` uses `right`:

```purescript
prism to fro pab = dimap fro (either identity identity) (right (rmap to pab))
```

Rust's `Prism` also uses `right`:

```rust
Q::dimap(preview, |result| match result { Ok(b) => review(b), Err(t) => t }, Q::right(pab))
```

However, PureScript applies `rmap to` to `pab` before `right`, while Rust applies the review in the output dimap. Both are correct because:
- PureScript: `right(rmap to pab)` maps `b -> t` inside the profunctor, then `right` lifts to `Either`.
- Rust: `right(pab)` lifts raw `a -> b` to `Either`, then the output dimap maps `Ok(b) -> review(b)`.

These produce the same result. **No bug here.**

### 7.3 `AffineTraversal` Encoding Difference

PureScript uses `second (right pab)`:

```purescript
affineTraversal' to pab = dimap to (\(Tuple b f) -> either identity b f) (second (right pab))
```

Rust uses `right(first(pab))`:

```rust
Q::dimap(split, merge, Q::right(Q::first(pab)))
```

Both are valid. PureScript pairs the setter `(b -> t)` with `Either t a` and applies `second . right`. Rust pairs `(a, s)` inside `Right` and applies `right . first`. The composition order of `Strong` and `Choice` differs but both correctly implement an affine traversal. **No bug.**

---

## 8. API Completeness Summary

| Category | PureScript Functions | Rust Equivalents | Coverage |
|----------|---------------------|-----------------|----------|
| **Iso operations** | `iso`, `withIso`, `cloneIso`, `re`, `au`, `auf`, `under`, `non`, `curried`, `uncurried`, `flipped`, `mapping`, `dimapping`, `coerced` | `Iso::new`, `optics_from`, `optics_to`, `IsoPrime::reversed` | ~25% |
| **Lens operations** | `lens`, `lens'`, `withLens`, `cloneLens`, `lensStore`, `ilens`, `ilens'` | `Lens::new`, `LensPrime::new`, `optics_view`, `optics_set`, `optics_over` | ~40% |
| **Prism operations** | `prism`, `prism'`, `review`, `clonePrism`, `withPrism`, `matching`, `is`, `isn't`, `only`, `nearly`, `below` | `Prism::new`, `PrismPrime::new`, `optics_review`, `optics_preview` | ~30% |
| **Traversal operations** | `traversed`, `traverseOf`, `sequenceOf`, `failover`, `element`, `elementsOf`, `cloneTraversal`, `both`, `itraverseOf`, `iforOf` | `Traversal::new` | ~10% |
| **Grate operations** | `grate`, `withGrate`, `cloneGrate`, `cotraversed`, `zipWithOf`, `zipFWithOf`, `collectOf` | `Grate::new`, `GratePrime::new` | ~20% |
| **Getter operations** | `view`, `viewOn`, `to`, `takeBoth`, `use`, `cloneGetter`, `iview`, `iuse` | `optics_view` | ~15% |
| **Setter operations** | `over`, `set`, `iover`, 15+ operator variants, 8+ monadic variants | `optics_set`, `optics_over` | ~10% |
| **Fold operations** | `preview`, `foldOf`, `foldMapOf`, `foldrOf`, `foldlOf`, `toListOf`, `firstOf`, `lastOf`, `maximumOf`, `minimumOf`, `allOf`, `anyOf`, 20+ more | `optics_preview` | ~5% |
| **AffineTraversal operations** | `affineTraversal`, `affineTraversal'`, `withAffineTraversal`, `cloneAffineTraversal` | `AffineTraversal::new` | ~25% |
| **Indexed optics** | Full suite (Indexed, itraverseOf, ifoldMapOf, iover, iview, etc.) | None | 0% |
| **Common optics** | `_1`, `_2`, `_Just`, `_Nothing`, `_Left`, `_Right`, `traversed`, `folded`, etc. | None | 0% |

---

## 9. Summary of Findings

### What is Correct

1. **Core profunctor encoding** is faithfully translated from PureScript
2. **All primary optic families** are present (Iso, Lens, Prism, AffineTraversal, Traversal, Grate, Getter, Setter, Fold, Review)
3. **All concrete profunctors** are implemented (Exchange, Shop, Market, Stall, Forget, Tagged, Grating) and consistently parameterized over `FnBrand`
4. **Profunctor class hierarchy** correctly maps (Profunctor, Strong, Choice, Closed, Wander)
5. **Optic subtyping** is correctly modeled through manual trait implementations
6. **Composition** works correctly for same-family optics
7. **Helper functions** (`optics_view`, `optics_set`, `optics_over`, `optics_preview`, `optics_review`, `optics_from`, `optics_to`) are correct

### What Has Issues

1. **Missing `AffineTraversalOptic` trait** — breaks the subtyping hierarchy for cross-family composition
2. **Cross-family composition** (Lens + Prism -> AffineTraversal) is not supported at the optic trait level

### What is Missing

1. **Indexed optics** — the entire indexed optics system (Indexed profunctor, all indexed optic types and operations)
2. **Re profunctor** — for reversing optics
3. **Bazaar and Zipping profunctors** — for concrete traversals and grate zipping
4. **Focusing functor** — for StateT zoom
5. **`withX` / `cloneX` functions** — CPS extraction and reconstruction for all concrete optics
6. **Extensive operation APIs** — especially for Fold (20+ functions), Setter (15+ functions), Traversal, and Grate
7. **Common predefined optics** — `_1`, `_2`, `_Just`, `_Nothing`, `_Left`, `_Right`, `traversed`, `folded`, etc.
8. **Tagged missing `Closed` instance** — prevents certain optic combinations
9. **Cochoice / Costrong** — profunctor classes not implemented