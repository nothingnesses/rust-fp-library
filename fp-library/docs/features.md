## Features

### Higher-Kinded Types (HKT)

Implemented using lightweight higher-kinded polymorphism (type-level defunctionalization/brands).
Procedural macros (`trait_kind!`, `impl_kind!`, `Apply!`, `#[kind]`) simplify defining and
applying HKT encodings. `m_do!` provides monadic do-notation; `a_do!` provides applicative
do-notation. Both support a `ref` qualifier (`m_do!(ref Brand { ... })`) for by-reference
dispatch to `RefSemimonad`/`RefLift` traits.

### Type Class Hierarchy

The library provides a comprehensive set of type classes. Blanket implementations
automatically derive composite traits (`Applicative`, `Monad`, `Comonad`, `Alternative`,
`MonadPlus`) from their components.

```mermaid
graph TD
    Functor --> Alt --> Plus
    Functor --> Extend
    Extend --> Comonad
    Extract --> Comonad
    Functor --> Semiapplicative
    Lift --> Semiapplicative
    Lift --> ApplyFirst
    Lift --> ApplySecond
    Semiapplicative --> Applicative
    Pointed --> Applicative
    ApplyFirst --> Applicative
    ApplySecond --> Applicative
    Applicative --> Alternative
    Plus --> Alternative
    Applicative --> Monad
    Semimonad --> Monad
    Monad --> MonadPlus
    Alternative --> MonadPlus
    Monad --> MonadRec
    Foldable --> Traversable
    Functor --> Traversable
    Compactable --> Filterable
    Functor --> Filterable
    Filterable --> Witherable
    Traversable --> Witherable
```

```mermaid
graph TD
    Bifunctor --> Bitraversable
    Bifoldable --> Bitraversable
```

```mermaid
graph TD
    Profunctor --> Strong --> Wander
    Profunctor --> Choice --> Wander
    Profunctor --> Closed
    Profunctor --> Costrong
    Profunctor --> Cochoice
```

```mermaid
graph TD
    Semigroup --> Monoid
    Semigroupoid --> Category
```

**Indexed variants:** `FunctorWithIndex`, `FoldableWithIndex`, `TraversableWithIndex`,
`FilterableWithIndex` extend their base traits with a shared `WithIndex` associated index type.

**Parallel variants:** `ParFunctor`, `ParFoldable`, `ParCompactable`, `ParFilterable`,
`ParFunctorWithIndex`, `ParFoldableWithIndex`, `ParFilterableWithIndex` mirror the sequential
hierarchy with `Send + Sync` bounds. Enable the `rayon` feature for true parallel execution.

**By-reference hierarchy:** A full by-ref type class stack for memoized types and
by-reference iteration over collections:

- `RefFunctor`, `RefPointed`, `RefLift`, `RefSemiapplicative`, `RefSemimonad`,
  `RefApplicative`, `RefMonad`, `RefApplyFirst`, `RefApplySecond`
- `RefFoldable`, `RefTraversable`, `RefFilterable`, `RefWitherable`
- `RefFunctorWithIndex`, `RefFoldableWithIndex`, `RefFilterableWithIndex`,
  `RefTraversableWithIndex`

**Thread-safe by-reference:** `SendRefFunctor`, `SendRefPointed`, `SendRefLift`,
`SendRefSemiapplicative`, `SendRefSemimonad`, `SendRefApplicative`, `SendRefMonad`,
`SendRefFoldable`, `SendRefFoldableWithIndex`, `SendRefFunctorWithIndex`,
`SendRefApplyFirst`, `SendRefApplySecond`.

**Parallel by-reference:** `ParRefFunctor`, `ParRefFoldable`, `ParRefFilterable`,
`ParRefFunctorWithIndex`, `ParRefFoldableWithIndex`, `ParRefFilterableWithIndex`.

**Laziness and effects:** `Deferrable`, `SendDeferrable` for lazy construction.
`LazyConfig` for memoization strategy abstraction.

### Optics

Composable data accessors using profunctor encoding (port of PureScript's
`purescript-profunctor-lenses`): Iso, Lens, Prism, AffineTraversal, Traversal, Getter,
Setter, Fold, Review, Grate. Each has a monomorphic `Prime` variant. Indexed variants
available for Lens, Traversal, Getter, Fold, Setter. Zero-cost composition via `Composed`
and `optics_compose`.

### Data Types

**Standard library instances:** `Option`, `Result`, `Vec`, `String` implement relevant
type classes.

**Lazy evaluation and stack safety:**

| Type                                  | Purpose                                       |
| ------------------------------------- | --------------------------------------------- |
| `Thunk` / `SendThunk`                 | Lightweight deferred computation.             |
| `Trampoline`                          | Stack-safe recursion via the `Free` monad.    |
| `Lazy` (`RcLazy`, `ArcLazy`)          | Memoized (evaluate-at-most-once) computation. |
| `TryThunk` / `TrySendThunk`           | Fallible deferred computation.                |
| `TryTrampoline`                       | Fallible stack-safe recursion.                |
| `TryLazy` (`RcTryLazy`, `ArcTryLazy`) | Fallible memoized computation.                |

**Free functors:**

| Type               | Wrapper | Clone | Send        | Map fusion   |
| ------------------ | ------- | ----- | ----------- | ------------ |
| `Coyoneda`         | `Box`   | No    | No          | No (k calls) |
| `RcCoyoneda`       | `Rc`    | Yes   | No          | No (k calls) |
| `ArcCoyoneda`      | `Arc`   | Yes   | Yes         | No (k calls) |
| `CoyonedaExplicit` | None    | No    | Conditional | Yes (1 call) |

**Containers:** `Identity`, `Pair`, `CatList` (O(1) append/uncons catenable list).

**Function wrappers:** `Endofunction` (dynamically composed `a -> a`), `Endomorphism`
(monoidally composed `a -> a`).

### Numeric Algebra

`Semiring`, `Ring`, `CommutativeRing`, `EuclideanRing`, `DivisionRing`, `Field`,
`HeytingAlgebra`.

### Newtype Wrappers

`Additive`, `Multiplicative`, `Conjunctive`, `Disjunctive`, `First`, `Last`, `Dual`
for selecting `Semigroup`/`Monoid` instances.

### Helper Functions

`compose`, `constant`, `flip`, `identity`, `on`, `pipe`.
