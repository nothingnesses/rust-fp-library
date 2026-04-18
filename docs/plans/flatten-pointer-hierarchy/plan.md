# Plan: Flatten Pointer and Coercion Hierarchies

**Status:** DRAFT

## Current progress

None. Plan is in draft stage for review.

## Open questions, issues and blockers

None at this time.

## Deviations

None yet.

## Implementation protocol

After completing each step within a phase:

1. Run verification: `just fmt`, `just check`, `just clippy`,
   `just deny`, `just doc`, `just test` (or `just verify` which
   runs all six in order).
2. If verification passes, update the `Current progress`, `Open
questions, issues and blockers`, and `Deviations` sections at
   the top of this plan to reflect the current state.
3. Commit the step (including the plan updates).

---

Flatten the pointer and coercion trait hierarchies by removing
unnecessary supertrait links and dead traits, making the Send
variants independent of their non-Send counterparts. This matches
the pattern already used by `CloneFn`/`SendCloneFn`, which are
independent parallel traits.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when they
lead to a better end state. This plan prioritises design correctness
and internal coherence over preserving compatibility with the current
public surface.

## Motivation

The pointer and coercion trait hierarchies use a linear supertrait
chain:

```
Pointer
  <- RefCountedPointer
       <- SendRefCountedPointer
       <- UnsizedCoercible
            <- SendUnsizedCoercible (also extends SendRefCountedPointer)
```

This linear encoding implies that each level is a strict refinement of
the one below. In practice:

1. **The Send variants don't need to extend the non-Send variants.**
   `SendRefCountedPointer` extends `RefCountedPointer`, but its only
   consumer (`SendUnsizedCoercible`) already independently requires
   `UnsizedCoercible` (which itself requires `RefCountedPointer`).
   The supertrait link is redundant.

2. **Inconsistency with CloneFn/SendCloneFn.** The CloneFn hierarchy
   already uses the flat pattern: `CloneFn` and `SendCloneFn` are
   independent parallel traits because their `Of` types deref to
   different unsized types (`dyn Fn` vs `dyn Fn + Send + Sync`).
   `SendRefCountedPointer` has the same structural reason to be
   independent (its `SendOf` has different bounds than
   `RefCountedPointer::CloneableOf`), but is currently encoded as a
   supertrait instead.

After this plan, the pointer/coercion hierarchy will be:

```
Pointer                    (Of)
  <- RefCountedPointer     (CloneableOf, TakeCellOf)
       <- UnsizedCoercible
            <- SendUnsizedCoercible (also extends SendRefCountedPointer)
SendRefCountedPointer      (SendOf) -- independent
```

This matches the CloneFn pattern: the base trait and its Send variant
are independent, and consumers that need both list both as bounds.

### Hierarchy audit

The following hierarchies were audited and found to be correct:

**Type class hierarchies (keep as-is):** These mirror established
PureScript/Haskell algebraic relationships. Each supertrait link
reflects a genuine mathematical dependency.

- Functor -> Alt -> Plus; Applicative + Plus -> Alternative
- Lift + Functor -> Semiapplicative; Semiapplicative + Pointed -> Applicative
- Semimonad + Applicative -> Monad
- Extend + Extract -> Comonad
- RefFunctor/SendRefFunctor hierarchies (parallel to above)

**Algebraic hierarchies (keep as-is):** Mathematical structure.

- Semigroup -> Monoid
- Semiring -> Ring -> CommutativeRing -> EuclideanRing
- Ring -> DivisionRing; EuclideanRing + DivisionRing -> Field

**CloneFn/SendCloneFn (already flat):** Independent parallel traits.
No changes needed.

**LiftFn/RefLiftFn extending CloneFn (keep as-is):** These are
construction traits that need the base trait's associated type to
define their return type. The supertrait link is logically necessary.
Same for SendLiftFn/SendRefLiftFn extending SendCloneFn.

The only hierarchies that benefit from flattening are the pointer
and coercion chains.

## Design

### Before

```
trait Pointer {
    type Of<'a, T>;           // Deref
}

trait RefCountedPointer: Pointer {
    type CloneableOf<'a, T>;  // Clone + Deref
    type TakeCellOf<'a, T>;   // Clone
}

trait SendRefCountedPointer: RefCountedPointer {
    type SendOf<'a, T>;       // Clone + Send + Sync + Deref
}

trait UnsizedCoercible: RefCountedPointer + 'static {
    // methods return Self::CloneableOf<dyn Fn>
}

trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer + 'static {
    // methods return Self::SendOf<dyn Fn + Send + Sync>
}
```

### After

```
trait Pointer {
    type Of<'a, T>;           // Deref  (unchanged)
}

trait RefCountedPointer: Pointer {     // unchanged
    type CloneableOf<'a, T>;  // Clone + Deref
    type TakeCellOf<'a, T>;   // Clone
}

trait SendRefCountedPointer {           // independent, no supertrait
    type SendOf<'a, T: Send + Sync>;   // Clone + Send + Sync + Deref
}

trait UnsizedCoercible: RefCountedPointer + 'static {
    // methods return Self::CloneableOf<dyn Fn>  (unchanged)
}

trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer + 'static {
    // methods return <Self as SendRefCountedPointer>::SendOf<dyn Fn + Send + Sync>
    // (unchanged, but SendRefCountedPointer is now reached via direct bound,
    // not via RefCountedPointer supertrait chain)
}
```

### Changes

1. **Make `SendRefCountedPointer` independent.** Remove the
   `RefCountedPointer` supertrait. `SendRefCountedPointer` becomes a
   standalone trait with only `SendOf` and `send_new`, mirroring how
   `SendCloneFn` is independent of `CloneFn`.

2. **Update `SendUnsizedCoercible` supertraits.** Its current
   supertraits are `UnsizedCoercible + SendRefCountedPointer`. Since
   `UnsizedCoercible` already requires `RefCountedPointer`, and
   `SendRefCountedPointer` is now independent, the supertraits still
   provide access to both `CloneableOf` (via `UnsizedCoercible` ->
   `RefCountedPointer`) and `SendOf` (via `SendRefCountedPointer`
   directly). No change needed to `SendUnsizedCoercible`'s
   supertrait list; it already names both.

### Consumer impact

Consumers currently bounding on `SendRefCountedPointer` get
`RefCountedPointer` for free via the supertrait. After flattening,
they need to bound on both if they use both.

Current consumers of `SendRefCountedPointer`:

| Consumer                            | Also needs RefCountedPointer?              | Impact                             |
| ----------------------------------- | ------------------------------------------ | ---------------------------------- |
| `SendUnsizedCoercible` (supertrait) | Yes, already listed via `UnsizedCoercible` | None                               |
| `ArcBrand` impl                     | N/A (implementor, not consumer)            | Remove supertrait, no other change |

No consumer bounds on `SendRefCountedPointer` alone without also
bounding on `RefCountedPointer` through another path. The flattening
is transparent.

## Decisions

### Decision A: Keep `Pointer`

**Adopted.** Keep `Pointer` as the base trait for heap-allocated
pointers. `RefCountedPointer` continues to extend it.

_Rationale:_ `Pointer` provides the extension point for a future
`Box`-based pointer brand (`BoxBrand`) that supports heap allocation
and `Deref` but not `Clone`. Keeping it avoids having to reopen the
hierarchy later to insert it. The cost is minimal (one trait with one
associated type and one method, implemented by `RcBrand` and
`ArcBrand`).

### Decision B: `SendRefCountedPointer` independence

**Adopted.** Make `SendRefCountedPointer` independent of
`RefCountedPointer`, matching the `SendCloneFn`/`CloneFn` pattern.

_Rationale:_ `SendRefCountedPointer::SendOf` has different bounds
than `RefCountedPointer::CloneableOf` (`Send + Sync` on both the
content and the wrapper). This is the same structural reason that
`SendCloneFn` is independent of `CloneFn` (`dyn Fn + Send + Sync`
vs `dyn Fn`). Using the same pattern for both creates consistency.
The only consumer of `SendRefCountedPointer` (`SendUnsizedCoercible`)
already independently requires `RefCountedPointer` via
`UnsizedCoercible`, so the supertrait link is redundant.

### Decision C: `send_ref_counted_pointer.rs` doc update

Update the doc comment (already partially corrected in the previous
commit) to describe `SendRefCountedPointer` as an independent
parallel trait, no longer a supertrait of `RefCountedPointer`.

## Integration surface

### Will change

| Component                                            | Change                                                                                  |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `fp-library/src/classes/send_ref_counted_pointer.rs` | Remove `RefCountedPointer` supertrait from `SendRefCountedPointer`. Update doc comment. |
| `fp-library/docs/pointer-abstraction.md`             | Update hierarchy description.                                                           |
| `fp-library/docs/limitations-and-workarounds.md`     | Update if it references the pointer hierarchy.                                          |

### Unchanged

- **`Pointer`**: Kept as base trait. `RefCountedPointer` still extends it.
- **`RefCountedPointer`**: Still extends `Pointer`. No changes.
- **`UnsizedCoercible`**: Still extends `RefCountedPointer + 'static`.
- **`SendUnsizedCoercible`**: Still extends `UnsizedCoercible + SendRefCountedPointer + 'static`. Both supertraits are already listed explicitly.
- **`RcBrand` / `ArcBrand`**: Pointer and RefCountedPointer impls unchanged. Only `ArcBrand`'s `SendRefCountedPointer` impl drops its supertrait.
- **`CloneFn` / `SendCloneFn`**: Already independent. No changes.
- **`FnBrand<P>`**: Bounds on `P: UnsizedCoercible` and `P: SendUnsizedCoercible` are unchanged.
- **Type class hierarchies**: Correct by construction.
- **Algebraic hierarchies**: Mathematical structure.
- **Optics system**: Bounds on `UnsizedCoercible`, unaffected.
- **`LazyConfig`**: Bounds on `RefCountedPointer`, unaffected.
- **Dispatch system**: Unaffected.

## Implementation phasing

### Phase 0: Non-regression tests

1. Add a test file exercising the current pointer APIs:
   `RefCountedPointer::cloneable_new`, `try_unwrap`,
   `take_cell_new`, `take_cell_take`,
   `SendRefCountedPointer::send_new`,
   `UnsizedCoercible::coerce_fn`, `coerce_ref_fn`,
   `SendUnsizedCoercible::coerce_send_fn`, `coerce_send_ref_fn`.
   Cover both `RcBrand` and `ArcBrand` through free functions and
   trait method syntax. Verify existing tests in `rc_ptr.rs` and
   `arc_ptr.rs` cover this adequately; add missing coverage.
2. Run `just verify`.

### Phase 1: Flatten `SendRefCountedPointer`

1. Remove `RefCountedPointer` supertrait from
   `SendRefCountedPointer`.
2. Update `SendRefCountedPointer` doc comment to describe it as an
   independent parallel trait.
3. Check all consumers of `SendRefCountedPointer` for any that relied
   on the supertrait link to access `RefCountedPointer` methods.
   (Audit found none, but verify during implementation.)
4. Run `just verify`.

### Phase 2: Documentation

1. Update `fp-library/docs/pointer-abstraction.md` to describe the
   flattened hierarchy.
2. Update `fp-library/docs/limitations-and-workarounds.md` if it
   references the pointer hierarchy structure.
3. Update `fp-library/src/classes/pointer.rs` module-level doc to
   reflect that `SendRefCountedPointer` is now independent.
4. Run `just verify`.

## Success criteria

The plan is complete when:

- `Pointer` is kept as the base trait. `RefCountedPointer` extends it.
- `SendRefCountedPointer` has no supertraits (independent of
  `RefCountedPointer`), consistent with the `CloneFn`/`SendCloneFn`
  pattern.
- `UnsizedCoercible` extends `RefCountedPointer + 'static`
  (unchanged).
- `SendUnsizedCoercible` extends
  `UnsizedCoercible + SendRefCountedPointer + 'static` (unchanged).
- All existing tests, doctests, and benchmarks pass.
