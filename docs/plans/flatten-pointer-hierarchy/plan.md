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
supertrait links that are not logically necessary (i.e., where a
trait's method signatures do not reference the supertrait's associated
types). Make the Send variants independent of their non-Send
counterparts, matching the pattern already used by
`CloneFn`/`SendCloneFn`. Rename associated types to `Of` for
consistency with the library's convention.

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
the one below. Auditing each link reveals that only two are logically
necessary (trait methods reference the supertrait's associated types):

| Link                                            | Necessary? | Reason                                                                                                             |
| :---------------------------------------------- | :--------- | :----------------------------------------------------------------------------------------------------------------- |
| `Pointer -> RefCountedPointer`                  | No         | No consumer uses `Pointer::Of` through `RefCountedPointer`.                                                        |
| `RefCountedPointer -> UnsizedCoercible`         | Yes        | `UnsizedCoercible` methods return `Self::CloneableOf<dyn Fn>`.                                                     |
| `RefCountedPointer -> SendRefCountedPointer`    | No         | `SendRefCountedPointer` has its own `SendOf`; no method references `CloneableOf`.                                  |
| `SendRefCountedPointer -> SendUnsizedCoercible` | Yes        | `SendUnsizedCoercible` methods return `Self::SendOf<dyn Fn + Send + Sync>`.                                        |
| `UnsizedCoercible -> SendUnsizedCoercible`      | No         | `SendUnsizedCoercible` methods only use `SendOf` (from `SendRefCountedPointer`), not `CloneableOf` or `coerce_fn`. |

The unnecessary links are all of the form "Send variant extends
non-Send variant." This is inconsistent with the `CloneFn`/`SendCloneFn`
hierarchy, which already uses the flat pattern (independent parallel
traits) for the same structural reason.

Additionally, `Pointer` is dead code: no production consumer bounds
on it, projects through `Pointer::Of`, or calls `Pointer::new`. Its
`Of` and `new` are redundant with `RefCountedPointer::CloneableOf`
and `cloneable_new`. It exists only as `RefCountedPointer`'s
supertrait and in its own free function, impls, and tests.

The associated types also use inconsistent naming:
`RefCountedPointer::CloneableOf`,
`SendRefCountedPointer::SendOf`. The library convention (used by
`Kind`, `CloneFn`, `SendCloneFn`) is to name the primary associated
type `Of`, with the trait name providing disambiguation.

After this plan:

```
RefCountedPointer          (Of, TakeCellOf)  -- independent
  <- UnsizedCoercible                        -- methods use RefCountedPointer::Of
SendRefCountedPointer      (Of)            -- independent
  <- SendUnsizedCoercible                    -- methods use SendRefCountedPointer::Of
```

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
// Pointer trait removed entirely.

trait RefCountedPointer {                  // independent (Pointer supertrait removed)
    type Of<'a, T>;           // Clone + Deref  (renamed from CloneableOf)
    type TakeCellOf<'a, T>;   // Clone
}

trait SendRefCountedPointer {              // independent (RefCountedPointer supertrait removed)
    type Of<'a, T: Send + Sync>;  // Clone + Send + Sync + Deref  (renamed from SendOf)
}

trait UnsizedCoercible: RefCountedPointer + 'static {
    // methods return <Self as RefCountedPointer>::Of<dyn Fn>
    // (supertrait link kept: methods reference RefCountedPointer::Of)
}

trait SendUnsizedCoercible: SendRefCountedPointer + 'static {
    // methods return <Self as SendRefCountedPointer>::Of<dyn Fn + Send + Sync>
    // (supertrait link kept: methods reference SendRefCountedPointer::Of)
    // (UnsizedCoercible supertrait removed: methods don't use it)
}
```

### Changes

1. **Remove `Pointer` entirely.** No production consumer bounds on
   it, projects through `Pointer::Of`, or calls `Pointer::new`.
   Remove the trait, its impls for `RcBrand` and `ArcBrand`, its
   free function `pointer_new`, its re-export, and its tests. If a
   future `BoxBrand` needs a non-clonable pointer trait, an
   independent trait can be introduced at that time; the flat
   architecture makes this trivial.

2. **Make `SendRefCountedPointer` independent.** Remove the
   `RefCountedPointer` supertrait. `SendRefCountedPointer` becomes a
   standalone trait with only its own `Of` and `send_new`, mirroring
   how `SendCloneFn` is independent of `CloneFn`.

3. **Make `SendUnsizedCoercible` independent of `UnsizedCoercible`.**
   Remove the `UnsizedCoercible` supertrait. Keep the
   `SendRefCountedPointer` supertrait (methods return
   `SendRefCountedPointer::Of`). Consumers that need both coercion
   capabilities bound on both: `P: UnsizedCoercible + SendUnsizedCoercible`.

4. **Rename associated types to `Of`.** Rename
   `RefCountedPointer::CloneableOf` to `Of` and
   `SendRefCountedPointer::SendOf` to `Of`. Each trait uses `Of` as
   its primary associated type, disambiguated by the trait name (e.g.,
   `<P as RefCountedPointer>::Of` vs
   `<P as SendRefCountedPointer>::Of`). `RefCountedPointer::TakeCellOf`
   keeps its descriptive name since it is a secondary associated type
   on the same trait.

### Consumer impact

Consumers currently bounding on `SendRefCountedPointer` get
`RefCountedPointer` for free via the supertrait. After flattening,
they need to bound on both if they use both.

Current consumers of `SendRefCountedPointer`:

| Consumer                            | Also needs RefCountedPointer?                                                                                                                                                                                            | Impact                              |
| :---------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :---------------------------------- |
| `SendUnsizedCoercible` (supertrait) | No longer extends `UnsizedCoercible`, so does not need `RefCountedPointer` through this path. Consumers of `SendUnsizedCoercible` that also need `RefCountedPointer` access must add `UnsizedCoercible` to their bounds. | See Decision D.                     |
| `ArcBrand` impl                     | N/A (implementor, not consumer)                                                                                                                                                                                          | Remove supertrait, no other change. |

Current consumers of `SendUnsizedCoercible`:

| Consumer                                                     | Also needs UnsizedCoercible?                                       | Impact |
| :----------------------------------------------------------- | :----------------------------------------------------------------- | :----- |
| `FnBrand<P: SendUnsizedCoercible>` (for `SendCloneFn` impls) | No. These impls only call `coerce_send_fn` / `coerce_send_ref_fn`. | None.  |
| `FnBrand<P: UnsizedCoercible>` (for `CloneFn` impls)         | Separate impl block, already bounds on `UnsizedCoercible`.         | None.  |

## Decisions

### Decision A: Remove `Pointer`

**Adopted.** Remove `Pointer` trait entirely.

_Rationale:_ No production consumer bounds on `Pointer`, projects
through `Pointer::Of`, or calls `Pointer::new`. Both `RcBrand` and
`ArcBrand` set `Pointer::Of` to the same type as
`RefCountedPointer::CloneableOf`. It exists only as
`RefCountedPointer`'s supertrait, in its own free function, impls,
tests, and doc examples. If a future `BoxBrand` needs a non-clonable
pointer trait, an independent trait can be introduced at that time.
The flat architecture makes this trivial since there are no supertrait
chains to insert into.

### Decision B: `SendRefCountedPointer` independence

**Adopted.** Make `SendRefCountedPointer` independent of
`RefCountedPointer`, matching the `SendCloneFn`/`CloneFn` pattern.

_Rationale:_ `SendRefCountedPointer::Of` (renamed from `SendOf`) has
different bounds than `RefCountedPointer::Of` (renamed from
`CloneableOf`). This is the same structural reason that `SendCloneFn`
is independent of `CloneFn` (`dyn Fn + Send + Sync` vs `dyn Fn`).
No consumer of `SendRefCountedPointer` relies on the supertrait link
to access `RefCountedPointer` methods; `SendUnsizedCoercible` reaches
`RefCountedPointer` through its own `UnsizedCoercible` supertrait.

### Decision C: `SendUnsizedCoercible` independence from `UnsizedCoercible`

**Adopted.** Remove `UnsizedCoercible` as a supertrait of
`SendUnsizedCoercible`. Keep `SendRefCountedPointer` as a supertrait
(methods return `SendRefCountedPointer::Of`).

_Rationale:_ `SendUnsizedCoercible`'s methods (`coerce_send_fn`,
`coerce_send_ref_fn`) only return `SendRefCountedPointer::Of` types.
They do not call `UnsizedCoercible` methods or reference
`RefCountedPointer::Of`. The `UnsizedCoercible` supertrait was
convenience (letting `P: SendUnsizedCoercible` imply
`P: UnsizedCoercible`), not a logical dependency. The sole consumer
(`FnBrand<P>`) has separate impl blocks for `P: UnsizedCoercible`
(CloneFn impls) and `P: SendUnsizedCoercible` (SendCloneFn impls),
so neither block needs the other's bound.

### Decision D: Rename associated types to `Of`

**Adopted.** Rename `RefCountedPointer::CloneableOf` to `Of` and
`SendRefCountedPointer::SendOf` to `Of`.

_Rationale:_ The library convention (`Kind::Of`, `CloneFn::Of`,
`SendCloneFn::Of`, `Pointer::Of`) uses `Of` as the primary associated
type name, with the trait name providing disambiguation via qualified
syntax (`<P as RefCountedPointer>::Of`). The current names
(`CloneableOf`, `SendOf`) embed bound information that is already
expressed in the trait definition. Using `Of` consistently reduces
naming proliferation.

`RefCountedPointer::TakeCellOf` keeps its descriptive name because
it is a secondary associated type on the same trait and cannot also
be named `Of`.

### Decision E: Doc updates

Update doc comments on `SendRefCountedPointer` and
`SendUnsizedCoercible` to describe them as independent parallel
traits.

## Integration surface

### Will change

| Component                                            | Change                                                                                                                                                               |
| :--------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/classes/pointer.rs`                  | Remove entirely.                                                                                                                                                     |
| `fp-library/src/classes/ref_counted_pointer.rs`      | Remove `Pointer` supertrait. Rename `CloneableOf` to `Of`.                                                                                                           |
| `fp-library/src/classes/send_ref_counted_pointer.rs` | Remove `RefCountedPointer` supertrait. Rename `SendOf` to `Of`. Update doc comment.                                                                                  |
| `fp-library/src/classes/send_unsized_coercible.rs`   | Remove `UnsizedCoercible` supertrait (keep `SendRefCountedPointer + 'static`). Update return types to use `<Self as SendRefCountedPointer>::Of`. Update doc comment. |
| `fp-library/src/types/rc_ptr.rs`                     | Remove `impl Pointer for RcBrand`.                                                                                                                                   |
| `fp-library/src/types/arc_ptr.rs`                    | Remove `impl Pointer for ArcBrand`.                                                                                                                                  |
| `fp-library/src/classes.rs`                          | Remove `pointer` module export.                                                                                                                                      |
| `fp-library/src/functions.rs`                        | Remove `pointer_new` re-export.                                                                                                                                      |
| All consumer sites referencing `CloneableOf`         | Rename to `Of` (qualified as `<P as RefCountedPointer>::Of`).                                                                                                        |
| All consumer sites referencing `SendOf`              | Rename to `Of` (qualified as `<P as SendRefCountedPointer>::Of`).                                                                                                    |
| `fp-library/docs/pointer-abstraction.md`             | Update hierarchy description.                                                                                                                                        |
| `fp-library/docs/limitations-and-workarounds.md`     | Update if it references the pointer hierarchy or old associated type names.                                                                                          |

### Unchanged

- **`UnsizedCoercible`**: Still extends `RefCountedPointer + 'static`. Methods return `<Self as RefCountedPointer>::Of` (renamed from `CloneableOf`).
- **`CloneFn` / `SendCloneFn`**: Already independent, already use `Of`. No changes.
- **`FnBrand<P>`**: Bounds on `P: UnsizedCoercible` and `P: SendUnsizedCoercible` are unchanged. Internal references to `CloneableOf` and `SendOf` are renamed to `Of`.
- **Type class hierarchies**: Correct by construction.
- **Algebraic hierarchies**: Mathematical structure.
- **Optics system**: Bounds on `UnsizedCoercible`, unaffected (rename `CloneableOf` references to `Of`).
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

### Phase 1: Remove `Pointer` and flatten supertrait links

1. Remove `Pointer` supertrait from `RefCountedPointer`.
2. Remove `impl Pointer for RcBrand` and `impl Pointer for ArcBrand`.
3. Remove the `Pointer` trait definition, its module, its free
   function `pointer_new`, and its re-export in `functions.rs`.
4. Remove test code that exercises `Pointer` directly.
5. Update `classes.rs` module exports.
6. Remove `RefCountedPointer` supertrait from
   `SendRefCountedPointer`.
7. Remove `UnsizedCoercible` supertrait from `SendUnsizedCoercible`
   (keep `SendRefCountedPointer + 'static`).
8. Update doc comments on affected traits.
9. Check all consumers for any that relied on removed supertrait
   links. (Audit found none, but verify during implementation.)
10. Run `just verify`.

### Phase 2: Rename associated types to `Of`

1. Rename `RefCountedPointer::CloneableOf` to `Of`.
2. Rename `SendRefCountedPointer::SendOf` to `Of`.
3. Update all consumer sites: `CloneableOf` -> `Of`,
   `SendOf` -> `Of` (with qualified syntax where needed).
4. Update free function signatures and re-exports.
5. Run `just verify`.

### Phase 3: Documentation

Update all docs that reference the old hierarchy, `Pointer` trait,
or old associated type names (`CloneableOf`, `SendOf`):

1. `fp-library/docs/pointer-abstraction.md`: Rewrite hierarchy
   diagram and description. Remove `Pointer` from the chain. Describe
   `RefCountedPointer` and `SendRefCountedPointer` as independent
   traits. Update associated type names to `Of`. Update `FnBrand<P>`
   description to reflect `SendCloneFn` independence.
2. `fp-library/docs/limitations-and-workarounds.md`: Update the
   "No Refinement of Associated Type Bounds in Subtraits" section
   to use the new `Of` names and reflect the flattened hierarchy.
   Update the "Foldable and CloneFn" section if it references the
   pointer hierarchy.
3. `fp-library/docs/zero-cost.md` (line 26): Update reference to
   "unified `Pointer` hierarchy" to describe `RefCountedPointer`.
4. `fp-library/docs/features.md` (line 211): Update mention of
   `Pointer`, `RefCountedPointer`, `SendRefCountedPointer` to
   reflect `Pointer` removal and flattened hierarchy.
5. `fp-library/docs/std-coverage-checklist.md` (line 66): Remove
   `Pointer` row from the trait table.
6. `CLAUDE.md` (line 115): Update pointer-abstraction.md description
   to remove `Pointer` mention.
7. `fp-library/src/classes/pointer.rs` module-level doc: File is
   removed; move the hierarchy overview to
   `ref_counted_pointer.rs` module doc or the `classes.rs` module
   doc.
8. `fp-library/CHANGELOG.md`: No changes (historical record).
9. Run `just verify`.

## Success criteria

The plan is complete when:

- `Pointer` trait is removed.
- `RefCountedPointer` is an independent trait (no supertraits).
  `UnsizedCoercible` extends it (logically necessary).
- `SendRefCountedPointer` is an independent trait (no supertraits).
  `SendUnsizedCoercible` extends it (logically necessary).
- `SendUnsizedCoercible` does not extend `UnsizedCoercible`.
- `RefCountedPointer::Of` (renamed from `CloneableOf`) and
  `SendRefCountedPointer::Of` (renamed from `SendOf`) follow the
  library's `Of` naming convention.
- All existing tests, doctests, and benchmarks pass.
