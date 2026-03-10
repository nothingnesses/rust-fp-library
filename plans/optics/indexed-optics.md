# Plan: Indexed Optics for fp-library

## Context

Indexed optics augment standard optics by threading an **index** (e.g., a position in a `Vec`, a key in a `HashMap`) alongside each focus value. Currently the optics system has no indexed variants — this is identified as the largest missing piece in `docs/optics-analysis.md`. The PureScript `purescript-profunctor-lenses` library provides the reference design: a newtype `Indexed p i s t = Indexed (p (Tuple i s) t)` wraps any profunctor to carry index information, and indexed optic types are functions `Indexed p i a b -> p s t`.

## Implementation Phases

### Phase 1: `Indexed` Profunctor Wrapper

**New file:** `fp-library/src/types/optics/indexed.rs`

Create the core `Indexed` struct and `IndexedBrand`, following the pattern used by `Forget`/`ForgetBrand` (brand defined alongside the struct, not in `brands.rs`, since it depends on `Profunctor`).

```rust
pub struct Indexed<'a, PBrand, I, A, B> {
    pub inner: PBrand::Of<'a, (I, A), B>,  // via Apply! macro
}

pub struct IndexedBrand<PBrand, I>(PhantomData<(PBrand, I)>);
// impl_kind! for IndexedBrand with Of<'a, A, B> = Indexed<'a, PBrand, I, A, B>
```

Implement profunctor class instances for `IndexedBrand<PBrand, I>`:

| Instance | Bound on PBrand | Key transformation |
|----------|----------------|-------------------|
| `Profunctor` | `Profunctor` | `dimap(f, g, Indexed(p)) = Indexed(dimap(\|(i,a)\| (i, f(a)), g, p))` — index untouched |
| `Strong` | `Strong` | `first(Indexed(p)) = Indexed(dimap(\|(i,(a,c))\| ((i,a),c), id, P::first(p)))` — index stays with focused component |
| `Choice` | `Choice` | `left(Indexed(p)) = Indexed(dimap(\|(i,r)\| match r { Err(a) => Err((i,a)), Ok(c) => Ok(c) }, id, P::left(p)))` — index follows chosen branch |
| `Wander` | `Wander` | Wraps `TraversalFunc` with an `IWanderAdapter` that threads the index to each element; requires `I: Clone` |

**`Strong` implementation note:** `Strong::second<'a, A, B, C>` takes `P::Of<A, B>` → `P::Of<(C, A), (C, B)>`. For `IndexedBrand`, `second` must use `P::second::<(I, A), B, C>(pab.inner)` (not `P::second::<C, (I, A), B>`) to match the turbofish parameter order of the underlying profunctor. Then `dimap` rearranges: contravariant `(I, (C, A))` → `(C, (I, A))`, covariant `(C, B)` → `(C, B)` (identity).

**`Wander` implementation note:** The `IWanderAdapter` struct adapts a `TraversalFunc<'a, S, T, A, B>` into a `TraversalFunc<'a, (I, S), T, (I, A), B>`. Its `apply` method must exactly match the `TraversalFunc::apply` signature — use `Apply!` macro types (not shorthand `M::Of`), no extra lifetime parameters, and `crate::classes::applicative::Applicative` (not `crate::classes::monad::Applicative`):

```rust
impl TraversalFunc<'a, (I, S), T, (I, A), B> for IWanderAdapter<...> {
    fn apply<M: Applicative>(
        &self,
        f: Box<dyn Fn((I, A)) -> Apply!(<M as Kind!(...)>::Of<'a, B>) + 'a>,
        (i, s): (I, S),
    ) -> Apply!(<M as Kind!(...)>::Of<'a, T>) {
        let i_ref = i;
        self.traversal.apply::<M>(
            Box::new(move |a| f((i_ref.clone(), a))),
            s,
        )
    }
}
```

**Doc examples note:** All `#[document_examples]` must contain `assert` statements (enforced by the proc macro). The `Strong::first`/`second` examples must assert on tuples (e.g., `(26, 100)` / `(100, 26)`), not scalars. The `Wander` example should follow `FnBrand`'s pattern using a concrete `SingleTraversal` struct.

**Register module:** Add `mod indexed;` + `pub use` to `fp-library/src/types/optics.rs`.

### Phase 2: Indexed Optic Traits

**Modify:** `fp-library/src/classes/optics.rs`

Add indexed optic traits mirroring the existing hierarchy. An `IndexedOptic` takes `Indexed<P, I, A, B>` and returns `P::Of<S, T>` (the output is *not* indexed — the optic introduces the index at the focus and projects back to unindexed outer structure):

```rust
pub trait IndexedLensOptic<'a, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate<P: Strong>(&self, pab: Indexed<'a, P, I, A, B>)
        -> P::Of<'a, S, T>;
}

pub trait IndexedTraversalOptic<'a, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate<P: Wander>(&self, pab: Indexed<'a, P, I, A, B>)
        -> P::Of<'a, S, T>;
}

pub trait IndexedGetterOptic<'a, I: 'a, S: 'a, A: 'a> {
    fn evaluate<R: 'a + 'static, P: UnsizedCoercible + 'static>(
        &self, pab: Indexed<'a, ForgetBrand<P, R>, I, A, A>,
    ) -> ForgetBrand<P, R>::Of<'a, S, S>;
}

pub trait IndexedFoldOptic<'a, I: 'a, S: 'a, A: 'a> {
    fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
        &self, pab: Indexed<'a, ForgetBrand<P, R>, I, A, A>,
    ) -> ForgetBrand<P, R>::Of<'a, S, S>;
}

pub trait IndexedSetterOptic<'a, P: UnsizedCoercible, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate(&self, pab: Indexed<'a, FnBrand<P>, I, A, B>)
        -> FnBrand<P>::Of<'a, S, T>;
}
```

**New file:** `fp-library/src/classes/optics/indexed_traversal.rs`

Add `IndexedTraversalFunc` trait (the indexed version of `TraversalFunc`):

```rust
pub trait IndexedTraversalFunc<'a, I, S, T, A, B> {
    fn apply<M: Applicative>(
        &self,
        f: Box<dyn Fn(I, A) -> M::Of<'a, B> + 'a>,
        s: S,
    ) -> M::Of<'a, T>;
}
```

### Phase 3: Concrete Indexed Optic Structs

Follow the same pattern as `Lens`/`LensPrime` — each stores its data using `FnBrand<P>` and implements the indexed optic traits plus compatible non-indexed traits.

#### IndexedLens / IndexedLensPrime

**New file:** `fp-library/src/types/optics/indexed_lens.rs`

```rust
pub struct IndexedLens<'a, P, I, S, T, A, B> {
    // S -> ((I, A), B -> T)  —  indexed getter paired with setter
    pub(crate) to: FnBrand<P>::Of<'a, S, ((I, A), FnBrand<P>::Of<'a, B, T>)>,
}
```

- Constructor: `IndexedLens::new(to)` and `IndexedLens::from_iview_set(iview, set)` (where `iview: S -> (I, A)`)
- Methods: `iview(&self, s) -> (I, A)`, `set(&self, s, b) -> T`, `over(&self, s, f: impl Fn(I, A) -> B) -> T`
- `evaluate` uses `Q::dimap(to, |(b,f)| f(b), Q::first(pab.inner))` — same profunctor encoding as `Lens` but operating on the unwrapped `P::Of<(I,A), B>`
- Implement `IndexedLensOptic`, `IndexedTraversalOptic`, `IndexedFoldOptic`, `IndexedGetterOptic`, `IndexedSetterOptic`

#### IndexedTraversal / IndexedTraversalPrime

**New file:** `fp-library/src/types/optics/indexed_traversal.rs`

```rust
pub struct IndexedTraversal<'a, P, I, S, T, A, B, F>
where F: IndexedTraversalFunc<'a, I, S, T, A, B> + 'a {
    pub traversal: F,
    pub(crate) _phantom: PhantomData<(&'a (I, S, T, A, B), P)>,
}
```

- `evaluate` bridges to `Wander` by adapting `IndexedTraversalFunc` into `TraversalFunc<(I,S), T, (I,A), B>` via an `IWanderAdapter` struct
- Implement `IndexedTraversalOptic`, `IndexedFoldOptic`, `IndexedSetterOptic`

#### IndexedFold

**New file:** `fp-library/src/types/optics/indexed_fold.rs`

Wraps an indexed fold function. Follow the pattern of `Fold`/`FoldPrime`.

#### IndexedGetter / IndexedSetter

**New files:** `fp-library/src/types/optics/indexed_getter.rs`, `fp-library/src/types/optics/indexed_setter.rs`

- `IndexedGetter` stores `FnBrand<P>::Of<'a, S, (I, A)>`
- `IndexedSetter` stores `over_fn: FnBrand<P>::Of<'a, (S, Box<dyn Fn(I, A) -> B + 'a>), T>` — the modifier is passed via `Box<dyn Fn>` (pointer-brand-agnostic), matching how non-indexed `Setter` bridges between storage brand `P` and evaluation brand `Q`. The `evaluate` method wraps the incoming `pab.inner: FnBrand<Q>::Of<(I, A), B>` into a `Box<dyn Fn(I, A) -> B>`, passes it to `self.over_fn`, and wraps the result in a fresh `FnBrand<Q>` closure via `<FnBrand<Q> as Function>::new(...)`

### Phase 4: Bridge Functions

**Modify:** `fp-library/src/types/optics/functions.rs`

Add user-facing operations. Internal names follow the `indexed_` prefix pattern (matching the existing `optics_view`/`optics_over`/etc. style); re-exports in `functions.rs` add the `optics_` prefix:

| Internal name | Re-export name | Purpose |
|---------------|---------------|---------|
| `indexed_view(optic, s)` | `optics_indexed_view` | View focus with its index: `(I, A)`. Uses `Forget` with identity. |
| `indexed_over(optic, s, f)` | `optics_indexed_over` | Modify focus using index-aware function `f: (I, A) -> B`. |
| `indexed_set(optic, s, b)` | `optics_indexed_set` | Set focus (ignoring index). |
| `indexed_preview(optic, s)` | `optics_indexed_preview` | Preview with index: `Option<(I, A)>`. Uses `Forget` with `First` monoid. |
| `indexed_fold_map(optic, f, s)` | `optics_indexed_fold_map` | Fold with index: `(I, A) -> R` for any `Monoid R`. |
| `un_index(optic, pab)` | `optics_un_index` | Convert indexed optic to regular optic by ignoring the index. |
| `as_index(optic, pib)` | `optics_as_index` | Extract only the index, discarding the focus. |
| `reindexed(f, optic)` | `optics_reindexed` | Remap index type via `f: I -> J`. |

**`un_index` / `as_index` implementation note:** These return wrapper structs (`UnIndex`, `AsIndex`) that implement `Optic`. The `evaluate` method uses `P::dimap` to either discard the index (`|(_, a)| a`) or discard the focus (`|(i, _)| i`) before delegating to the inner indexed optic. These structs are internal to `functions.rs` and need helper adapter traits (`IndexedOpticAdapter`, `IndexedOpticAdapterDiscardsFocus`) to abstract over which indexed optic trait the inner optic implements. All `#[document_examples]` on these must contain assertions (proc macro requirement) — do not use placeholder comments.

**`optics_reindexed` lifetime fix:** The `Reindexed` struct's `evaluate_indexed` method uses `P::dimap` with a closure that captures `self.f`. Since `P::dimap` requires closures to be `+ 'a` but `&self` only lives for the method call duration `'1`, capturing `&self.f` fails with `'1 must outlive 'a`. Fix: require `F: Clone` on the `Reindexed` struct's `IndexedOpticAdapter` impl, then clone `self.f` into the closure so it's owned:

```rust
fn evaluate_indexed(&self, pab: Indexed<'a, P, J, A, B>) -> ... {
    let f = self.f.clone();  // owned F, satisfies 'a since F: Fn(I) -> J + 'a
    let inner = pab.inner;
    let dimapped = P::dimap(move |(i, a)| (f(i), a), |b| b, inner);
    self.optic.evaluate_indexed(Indexed { inner: dimapped })
}
```

Add `F: Clone + 'a` bound to both the impl block and the outer function's where clause.

**Re-export** in `fp-library/src/functions.rs`.

### Phase 5: Composition

**Modify:** `fp-library/src/types/optics/composed.rs`

Indexed optics compose with regular optics. When a regular `Optic` (outer) is composed with an `IndexedOptic` (inner), the result is an `IndexedOptic`. This works because `IndexedBrand<P, I>` is a valid profunctor, so the outer optic can be evaluated with it:

```rust
// In Composed impl:
impl IndexedLensOptic for Composed where O1: LensOptic, O2: IndexedLensOptic {
    fn evaluate<P: Strong>(&self, pab: Indexed<P, I, A, B>) -> P::Of<S, T> {
        let pmn = self.second.evaluate(pab);  // IndexedLensOptic -> P::Of<M, N>
        self.first.evaluate::<P>(pmn)  // LensOptic -> P::Of<S, T>  (turbofish required!)
    }
}
```

**Turbofish requirement:** In all indexed `Composed` impls, the call to `self.first.evaluate(pmn)` (where `O1` is a non-indexed optic like `LensOptic` or `TraversalOptic`) requires an explicit turbofish `::<P>` to help the compiler infer the profunctor type parameter. This is because `pmn` is `P::Of<M, N>` (an associated type projection), and the compiler can't reverse-map from the concrete type back to `P`. All five indexed `Composed` impls (`IndexedLensOptic`, `IndexedTraversalOptic`, `IndexedGetterOptic`, `IndexedFoldOptic`, `IndexedSetterOptic`) need this turbofish on the `self.first.evaluate` call.

Add `Composed` implementations for:
- `IndexedLensOptic` (O1: LensOptic, O2: IndexedLensOptic)
- `IndexedTraversalOptic` (O1: TraversalOptic, O2: IndexedTraversalOptic)
- `IndexedFoldOptic` (O1: FoldOptic, O2: IndexedFoldOptic)
- `IndexedGetterOptic` (O1: GetterOptic, O2: IndexedGetterOptic)
- `IndexedSetterOptic` (O1: SetterOptic, O2: IndexedSetterOptic)

### Phase 6: WithIndex Type Classes

**New files:**
- `fp-library/src/classes/functor_with_index.rs`
- `fp-library/src/classes/foldable_with_index.rs`
- `fp-library/src/classes/traversable_with_index.rs`

```rust
pub trait FunctorWithIndex<I>: Functor {
    fn map_with_index<'a, A: 'a, B: 'a>(f: impl Fn(I, A) -> B + 'a, fa: Self::Of<'a, A>) -> Self::Of<'a, B>;
}

pub trait FoldableWithIndex<I>: Foldable {
    fn fold_map_with_index<'a, A: 'a, R: Monoid>(f: impl Fn(I, A) -> R + 'a, fa: Self::Of<'a, A>) -> R;
}

pub trait TraversableWithIndex<I>: Traversable + FoldableWithIndex<I> + FunctorWithIndex<I> {
    fn traverse_with_index<'a, A: 'a, B: 'a, M: Applicative>(
        f: impl Fn(I, A) -> M::Of<'a, B> + 'a, ta: Self::Of<'a, A>,
    ) -> M::Of<'a, Self::Of<'a, B>>;
}
```

**Register modules** in `fp-library/src/classes.rs`.

**Implementations** (modify existing type files):
- `VecBrand: FunctorWithIndex<usize>` — uses `enumerate()` (`fp-library/src/types/vec.rs`)
- `VecBrand: FoldableWithIndex<usize>` — uses `enumerate()`
- `VecBrand: TraversableWithIndex<usize>` — uses `enumerate()`
- `OptionBrand: FunctorWithIndex<()>` — trivial unit index (`fp-library/src/types/option.rs`)
- `OptionBrand: FoldableWithIndex<()>`
- `OptionBrand: TraversableWithIndex<()>`

### Phase 7: Standard Indexed Optic Constructors

Add both associated functions on the concrete types and free-function versions (re-exported with `optics_` prefix):

| Associated function | Free function | Re-export | Description |
|---------------------|---------------|-----------|-------------|
| `IndexedTraversal::traversed()` | `indexed_traversed()` | `optics_indexed_traversed` | From `TraversableWithIndex<I>` |
| `IndexedFold::folded()` | `indexed_folded()` | `optics_indexed_folded` | From `FoldableWithIndex<I>` |
| `IndexedSetter::mapped()` | `indexed_mapped()` | `optics_indexed_mapped` | From `FunctorWithIndex<I>` |
| — | `positions(traversal)` | `optics_positions` | Converts any `Traversal` to `IndexedTraversal<usize>` |

Free functions live in `fp-library/src/types/optics/functions.rs`.

`positions` implementation: use `Cell<usize>` counter inside the traversal function. When each element is visited, assign the current counter value as its index and increment.

## File Summary

### New files (9)
1. `fp-library/src/types/optics/indexed.rs` — `Indexed`, `IndexedBrand`, profunctor instances
2. `fp-library/src/types/optics/indexed_lens.rs` — `IndexedLens`, `IndexedLensPrime`
3. `fp-library/src/types/optics/indexed_traversal.rs` — `IndexedTraversal`, `IndexedTraversalPrime`
4. `fp-library/src/types/optics/indexed_fold.rs` — `IndexedFold`, `IndexedFoldPrime`
5. `fp-library/src/types/optics/indexed_getter.rs` — `IndexedGetter`, `IndexedGetterPrime`
6. `fp-library/src/types/optics/indexed_setter.rs` — `IndexedSetter`, `IndexedSetterPrime`
7. `fp-library/src/classes/functor_with_index.rs` — `FunctorWithIndex` trait
8. `fp-library/src/classes/foldable_with_index.rs` — `FoldableWithIndex` trait
9. `fp-library/src/classes/optics/indexed_traversal.rs` — `IndexedTraversalFunc` trait

### Modified files (7)
1. `fp-library/src/types/optics.rs` — register new optic modules
2. `fp-library/src/classes/optics.rs` — indexed optic traits
3. `fp-library/src/classes.rs` — register WithIndex modules
4. `fp-library/src/types/optics/composed.rs` — indexed composition impls
5. `fp-library/src/types/optics/functions.rs` — bridge functions (`indexed_view`, `indexed_over`, etc.)
6. `fp-library/src/types/vec.rs` — `WithIndex` impls for `VecBrand`
7. `fp-library/src/functions.rs` — re-exports

## Build Order

Each step should compile and pass tests before moving on:

1. `Indexed` struct + `IndexedBrand` + `Profunctor` instance only
2. `Strong`, `Choice` instances for `IndexedBrand`
3. `IndexedTraversalFunc` trait + `Wander` instance for `IndexedBrand`
4. Indexed optic traits in `classes/optics.rs`
5. `IndexedLens` + `IndexedLensPrime` concrete structs
6. `IndexedGetter` + `IndexedGetterPrime`
7. `IndexedFold` + `IndexedFoldPrime`
8. `IndexedSetter` + `IndexedSetterPrime`
9. `IndexedTraversal` + `IndexedTraversalPrime`
10. Bridge functions: `indexed_view`, `indexed_over`, `indexed_set`, `indexed_preview`, `indexed_fold_map`, `un_index`, `as_index`, `reindexed`
11. `Composed` indexed impls
12. `FunctorWithIndex`, `FoldableWithIndex`, `TraversableWithIndex` traits
13. `VecBrand` and `OptionBrand` WithIndex impls
14. `IndexedTraversal::traversed()`, `IndexedFold::folded()`, `IndexedSetter::mapped()`, `positions()` constructors
15. Tests (unit + property-based for indexed optic laws)

## Key Design Decisions

- **`I: Clone` in Wander**: Required because traversals visit multiple foci and each needs the index. Indices (usize, keys) are always Clone.
- **Brand placement**: `IndexedBrand` lives alongside `Indexed` (not in `brands.rs`), following the pattern of `ForgetBrand`, `BazaarBrand`, etc.
- **No indexed Review/Grate**: These don't have meaningful indexed variants in standard optics literature.
- **Composition direction**: Regular (outer) + Indexed (inner) = Indexed. Indexed (outer) + Regular (inner) = the inner optic doesn't provide an index, so the outer's index is preserved; this just works via `Indexed p i` being a profunctor.

## Verification

1. `cargo check --workspace` after each build step
2. `cargo test --workspace` for all unit + doc tests
3. Test indexed lens: create `IndexedLens` for tuple field, verify `indexed_view` returns `(index, value)`, `indexed_over` modifies with index
4. Test indexed traversal over `Vec`: verify `indexed_traversed` yields `(usize, element)` pairs
5. Test composition: regular lens + indexed traversal composes correctly
6. Test `un_index`: indexed optic can be used as regular optic
7. `cargo clippy --workspace --all-features` for lint check
8. `cargo fmt --all -- --check` for formatting
