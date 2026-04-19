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
consistency with the library's convention. Rename coercion traits
to `ToDynCloneFn`/`ToDynSendFn` and their methods to `new`/`ref_new`
for consistency with the `LiftFn`/`SendLiftFn` naming pattern.

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

| Link                                            | Necessary? | Reason                                                                                                            |
| :---------------------------------------------- | :--------- | :---------------------------------------------------------------------------------------------------------------- |
| `Pointer -> RefCountedPointer`                  | No         | No consumer uses `Pointer::Of` through `RefCountedPointer`.                                                       |
| `RefCountedPointer -> UnsizedCoercible`         | Yes        | `UnsizedCoercible` methods return `Self::CloneableOf<dyn Fn>`.                                                    |
| `RefCountedPointer -> SendRefCountedPointer`    | No         | `SendRefCountedPointer` has its own `SendOf`; no method references `CloneableOf`.                                 |
| `SendRefCountedPointer -> SendUnsizedCoercible` | Yes        | `SendUnsizedCoercible` methods return `Self::SendOf<dyn Fn + Send + Sync>`.                                       |
| `UnsizedCoercible -> SendUnsizedCoercible`      | No         | `SendUnsizedCoercible` methods only use `SendOf` (from `SendRefCountedPointer`), not `CloneableOf` or `coerce_fn` |

Additionally, the coercion trait names (`UnsizedCoercible`,
`SendUnsizedCoercible`) describe the Rust compiler mechanism (unsized
coercion) rather than what they do (wrap closures into `dyn Fn` trait
objects behind reference-counted pointers). Renaming to
`ToDynCloneFn` / `ToDynSendFn` makes the purpose clear and mirrors
the `CloneFn` / `SendCloneFn` naming.

The unnecessary links are all of the form "Send variant extends
non-Send variant." This is inconsistent with the `CloneFn`/`SendCloneFn`
hierarchy, which already uses the flat pattern (independent parallel
traits) for the same structural reason.

Additionally, `Pointer` currently has no production consumers but
serves as the extension point for non-clonable pointer brands. This
plan introduces `BoxBrand` and `ToDynFn` to validate that the flat
architecture accommodates pointer brands that only implement a subset
of the traits.

The associated types also use inconsistent naming:
`RefCountedPointer::CloneableOf`,
`SendRefCountedPointer::SendOf`. The library convention (used by
`Kind`, `CloneFn`, `SendCloneFn`) is to name the primary associated
type `Of`, with the trait name providing disambiguation.

After this plan:

```
Pointer                    (Of)              -- independent
  <- ToDynFn                                 -- methods use Pointer::Of
RefCountedPointer          (Of, TakeCellOf)  -- independent
  <- ToDynCloneFn                            -- methods use RefCountedPointer::Of
SendRefCountedPointer      (Of)              -- independent
  <- ToDynSendFn                             -- methods use SendRefCountedPointer::Of
```

`BoxBrand` implements `Pointer` + `ToDynFn` only. `RcBrand`
implements `Pointer` + `ToDynFn` + `RefCountedPointer` +
`ToDynCloneFn`. `ArcBrand` implements all six.

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
trait Pointer {                            // independent (no longer supertrait of RefCountedPointer)
    type Of<'a, T>;           // Deref
}

trait ToDynFn: Pointer + 'static {         // NEW: wraps closures into Box<dyn Fn>
    fn new(...) -> ...;      // impl Fn(A) -> B  ->  Pointer::Of<dyn Fn(A) -> B>
    fn ref_new(...) -> ...;  // impl Fn(&A) -> B ->  Pointer::Of<dyn Fn(&A) -> B>
    // methods return <Self as Pointer>::Of<dyn Fn>
}

trait RefCountedPointer {                  // independent (Pointer supertrait removed)
    type Of<'a, T>;           // Clone + Deref  (renamed from CloneableOf)
    type TakeCellOf<'a, T>;   // Clone
}

trait SendRefCountedPointer {              // independent (RefCountedPointer supertrait removed)
    type Of<'a, T: Send + Sync>;  // Clone + Send + Sync + Deref  (renamed from SendOf)
}

trait ToDynCloneFn: RefCountedPointer + 'static {  // renamed from UnsizedCoercible
    fn new(...) -> ...;      // renamed from coerce_fn
    fn ref_new(...) -> ...;  // renamed from coerce_ref_fn
    // methods return <Self as RefCountedPointer>::Of<dyn Fn>
    // (supertrait link kept: methods reference RefCountedPointer::Of)
}

trait ToDynSendFn: SendRefCountedPointer + 'static {  // renamed from SendUnsizedCoercible
    fn new(...) -> ...;      // renamed from coerce_send_fn
    fn ref_new(...) -> ...;  // renamed from coerce_send_ref_fn
    // methods return <Self as SendRefCountedPointer>::Of<dyn Fn + Send + Sync>
    // (supertrait link kept: methods reference SendRefCountedPointer::Of)
    // (ToDynCloneFn supertrait removed: methods don't use it)
}
```

### Changes

1. **Keep `Pointer` as independent trait.** Remove the `Pointer`
   supertrait from `RefCountedPointer`. `Pointer` becomes an
   independent trait. `RcBrand` and `ArcBrand` continue to implement
   it.

2. **Add `ToDynFn` trait.** New trait extending `Pointer + 'static`
   with methods `new` / `ref_new` that wrap closures into
   `<Self as Pointer>::Of<dyn Fn>`. Mirrors `ToDynCloneFn` and
   `ToDynSendFn` at the `Box` level. Implemented by `BoxBrand`,
   `RcBrand`, and `ArcBrand`.

3. **Add `BoxBrand`.** New brand struct implementing `Pointer`
   (`Of = Box<T>`) and `ToDynFn` (wraps `impl Fn` into
   `Box<dyn Fn>`). Does not implement `RefCountedPointer` (no
   `Clone`), `SendRefCountedPointer`, `ToDynCloneFn`, or
   `ToDynSendFn`. Validates that the flat architecture accommodates
   pointer brands with a subset of capabilities.

4. **Make `SendRefCountedPointer` independent.** Remove the
   `RefCountedPointer` supertrait. `SendRefCountedPointer` becomes a
   standalone trait with only its own `Of` and `send_new`, mirroring
   how `SendCloneFn` is independent of `CloneFn`.

5. **Rename `UnsizedCoercible` to `ToDynCloneFn`.** Rename methods
   `coerce_fn` to `new` and `coerce_ref_fn` to `ref_new`, matching
   the `LiftFn::new` / `RefLiftFn::ref_new` pattern. Keep the
   `RefCountedPointer` supertrait (methods return
   `RefCountedPointer::Of`).

6. **Rename `SendUnsizedCoercible` to `ToDynSendFn` and make it
   independent of `ToDynCloneFn`.** Remove the `UnsizedCoercible`
   (now `ToDynCloneFn`) supertrait. Keep the
   `SendRefCountedPointer` supertrait (methods return
   `SendRefCountedPointer::Of`). Rename methods `coerce_send_fn` to
   `new` and `coerce_send_ref_fn` to `ref_new`. Consumers that need
   both coercion capabilities bound on both:
   `P: ToDynCloneFn + ToDynSendFn`.

7. **Rename associated types to `Of`.** Rename
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

Current consumers of `SendUnsizedCoercible` (renamed to
`ToDynSendFn`):

| Consumer                                                     | Also needs `ToDynCloneFn`?                                                                        | Impact |
| :----------------------------------------------------------- | :------------------------------------------------------------------------------------------------ | :----- |
| `FnBrand<P: SendUnsizedCoercible>` (for `SendCloneFn` impls) | No. These impls only call `coerce_send_fn` / `coerce_send_ref_fn` (renamed to `new` / `ref_new`). | None.  |
| `FnBrand<P: UnsizedCoercible>` (for `CloneFn` impls)         | Separate impl block, already bounds on `UnsizedCoercible` (renamed to `ToDynCloneFn`).            | None.  |

## Decisions

### Decision A: Keep `Pointer` as independent trait, add `ToDynFn` and `BoxBrand`

**Adopted.** Keep `Pointer` but remove it as a supertrait of
`RefCountedPointer`. Add `ToDynFn` (extending `Pointer + 'static`)
for wrapping closures into `Box<dyn Fn>`. Add `BoxBrand` as a
concrete implementor of `Pointer` + `ToDynFn` only.

_Rationale:_ Introducing `BoxBrand` and `ToDynFn` alongside the
flattening validates that the architecture accommodates pointer
brands that implement only a subset of traits. Without a real
implementor, the design is speculative. With `BoxBrand`, the
`Pointer` / `ToDynFn` story is tested end-to-end. The cost is small
(one trait, one brand struct, a few impls and tests) and the result
is a concrete proof that the flat architecture works as intended.

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

### Decision C: `ToDynSendFn` independence from `ToDynCloneFn`

**Adopted.** Remove `ToDynCloneFn` (formerly `UnsizedCoercible`) as
a supertrait of `ToDynSendFn` (formerly `SendUnsizedCoercible`). Keep
`SendRefCountedPointer` as a supertrait (methods return
`SendRefCountedPointer::Of`).

_Rationale:_ `ToDynSendFn`'s methods (`new`, `ref_new`) only return
`SendRefCountedPointer::Of` types. They do not call `ToDynCloneFn`
methods or reference `RefCountedPointer::Of`. The supertrait was
convenience (letting `P: ToDynSendFn` imply `P: ToDynCloneFn`), not
a logical dependency. The sole consumer (`FnBrand<P>`) has separate
impl blocks for `P: ToDynCloneFn` (CloneFn impls) and
`P: ToDynSendFn` (SendCloneFn impls), so neither block needs the
other's bound.

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

### Decision E: Rename coercion traits and methods

**Adopted.** Rename `UnsizedCoercible` to `ToDynCloneFn` and
`SendUnsizedCoercible` to `ToDynSendFn`. Rename methods:
`coerce_fn` -> `new`, `coerce_ref_fn` -> `ref_new`,
`coerce_send_fn` -> `new`, `coerce_send_ref_fn` -> `ref_new`.

_Rationale:_ The old names describe the Rust compiler mechanism
(unsized coercion) rather than the purpose (wrapping closures into
`dyn Fn` trait objects). `ToDynCloneFn` / `ToDynSendFn` makes the
purpose clear: converting a concrete closure to a cloneable (or
send-safe) `dyn Fn` pointer. The `To` prefix conveys conversion.
The `Clone` / `Send` qualifier describes the wrapper's capabilities.
`DynFn` names the result.

The method names `new` / `ref_new` match the `LiftFn::new` /
`RefLiftFn::ref_new` and `SendLiftFn::new` /
`SendRefLiftFn::ref_new` pattern. Both `LiftFn` and `ToDynCloneFn`
do the same thing (wrap a closure), just returning different types
(`CloneFn::Of` vs `RefCountedPointer::Of<dyn Fn>`). Consistent
method names reflect this parallel.

### Decision F: Doc updates

Update doc comments on `SendRefCountedPointer` and `ToDynSendFn`
to describe them as independent parallel traits.

## Integration surface

### Will change

| Component                                             | Change                                                                                                                                                                                            |
| :---------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fp-library/src/classes/pointer.rs`                   | Keep. Remove supertrait link from `RefCountedPointer`.                                                                                                                                            |
| `fp-library/src/classes/to_dyn_fn.rs`                 | New file. `ToDynFn` trait extending `Pointer + 'static` with `new` / `ref_new` methods.                                                                                                           |
| `fp-library/src/classes/ref_counted_pointer.rs`       | Remove `Pointer` supertrait. Rename `CloneableOf` to `Of`.                                                                                                                                        |
| `fp-library/src/classes/send_ref_counted_pointer.rs`  | Remove `RefCountedPointer` supertrait. Rename `SendOf` to `Of`. Update doc comment.                                                                                                               |
| `fp-library/src/classes/unsized_coercible.rs`         | Rename to `to_dyn_clone_fn.rs`. Rename trait to `ToDynCloneFn`. Rename methods to `new` / `ref_new`. Rename free functions accordingly.                                                           |
| `fp-library/src/classes/send_unsized_coercible.rs`    | Rename to `to_dyn_send_fn.rs`. Rename trait to `ToDynSendFn`. Remove `ToDynCloneFn` supertrait (keep `SendRefCountedPointer + 'static`). Rename methods to `new` / `ref_new`. Update doc comment. |
| `fp-library/src/types/rc_ptr.rs`                      | Add `impl ToDynFn for RcBrand`.                                                                                                                                                                   |
| `fp-library/src/types/arc_ptr.rs`                     | Add `impl ToDynFn for ArcBrand`.                                                                                                                                                                  |
| `fp-library/src/types/box_ptr.rs`                     | New file. `BoxBrand` struct, `impl Pointer for BoxBrand`, `impl ToDynFn for BoxBrand`.                                                                                                            |
| `fp-library/src/brands.rs`                            | Add `BoxBrand` struct.                                                                                                                                                                            |
| `fp-library/src/classes.rs`                           | Add `to_dyn_fn` module export.                                                                                                                                                                    |
| All consumer sites referencing `CloneableOf`          | Rename to `Of` (qualified as `<P as RefCountedPointer>::Of`).                                                                                                                                     |
| All consumer sites referencing `SendOf`               | Rename to `Of` (qualified as `<P as SendRefCountedPointer>::Of`).                                                                                                                                 |
| All consumer sites referencing `UnsizedCoercible`     | Rename bound to `ToDynCloneFn`. Rename method calls: `coerce_fn` -> `new`, `coerce_ref_fn` -> `ref_new`.                                                                                          |
| All consumer sites referencing `SendUnsizedCoercible` | Rename bound to `ToDynSendFn`. Rename method calls: `coerce_send_fn` -> `new`, `coerce_send_ref_fn` -> `ref_new`.                                                                                 |
| `fp-library/docs/pointer-abstraction.md`              | Update hierarchy description.                                                                                                                                                                     |
| `fp-library/docs/limitations-and-workarounds.md`      | Update if it references the pointer hierarchy or old associated type names.                                                                                                                       |

### Unchanged

- **`Pointer`**: Kept as independent trait. `Of` name unchanged. `ToDynFn` extends it.
- **`ToDynCloneFn`** (renamed from `UnsizedCoercible`): Still extends `RefCountedPointer + 'static`. Methods return `<Self as RefCountedPointer>::Of` (renamed from `CloneableOf`).
- **`CloneFn` / `SendCloneFn`**: Already independent, already use `Of`. No changes.
- **`FnBrand<P>`**: Bounds renamed (`P: ToDynCloneFn` and `P: ToDynSendFn`). Internal references to `CloneableOf` and `SendOf` are renamed to `Of`.
- **Type class hierarchies**: Correct by construction.
- **Algebraic hierarchies**: Mathematical structure.
- **Optics system**: Bounds renamed to `ToDynCloneFn` (rename `CloneableOf` references to `Of`).
- **`LazyConfig`**: Bounds on `RefCountedPointer`, unaffected.
- **Dispatch system**: Unaffected.

## Implementation phasing

### Phase 0: Non-regression tests

1. Add a test file exercising the current pointer APIs:
   `RefCountedPointer::cloneable_new`, `try_unwrap`,
   `take_cell_new`, `take_cell_take`,
   `SendRefCountedPointer::send_new`,
   `UnsizedCoercible::coerce_fn`, `UnsizedCoercible::coerce_ref_fn`,
   `SendUnsizedCoercible::coerce_send_fn`,
   `SendUnsizedCoercible::coerce_send_ref_fn`.
   (These use the current names; they will be renamed in Phase 2.)
   Cover both `RcBrand` and `ArcBrand` through free functions and
   trait method syntax. Verify existing tests in `rc_ptr.rs` and
   `arc_ptr.rs` cover this adequately; add missing coverage.
2. Run `just verify`.

### Phase 1: Flatten supertrait links, add `ToDynFn` and `BoxBrand`

1. Remove `Pointer` supertrait from `RefCountedPointer`.
2. Remove `RefCountedPointer` supertrait from
   `SendRefCountedPointer`.
3. Remove `UnsizedCoercible` supertrait from `SendUnsizedCoercible`
   (keep `SendRefCountedPointer + 'static`).
4. Add `ToDynFn` trait (extends `Pointer + 'static`) with methods
   `new` / `ref_new`. Implement for `RcBrand`, `ArcBrand`.
5. Add `BoxBrand` struct to `brands.rs`. Implement `Pointer`
   (`Of = Box<T>`) and `ToDynFn` (wraps `impl Fn` into
   `Box<dyn Fn>`) for `BoxBrand`. Add test file `box_ptr.rs`.
6. Update doc comments on affected traits.
7. Check all consumers for any that relied on removed supertrait
   links. (Audit found none, but verify during implementation.)
8. Run `just verify`.

### Phase 2: Rename coercion traits and methods

1. Rename `UnsizedCoercible` to `ToDynCloneFn`. Rename file
   `unsized_coercible.rs` to `to_dyn_clone_fn.rs`.
2. Rename `SendUnsizedCoercible` to `ToDynSendFn`. Rename file
   `send_unsized_coercible.rs` to `to_dyn_send_fn.rs`.
3. Rename methods: `coerce_fn` -> `new`, `coerce_ref_fn` -> `ref_new`,
   `coerce_send_fn` -> `new`, `coerce_send_ref_fn` -> `ref_new`.
4. Rename free functions accordingly.
5. Update all consumer sites: trait bounds, method calls, imports.
6. Update `classes.rs` module exports.
7. Run `just verify`.

### Phase 3: Rename associated types to `Of`

1. Rename `RefCountedPointer::CloneableOf` to `Of`.
2. Rename `SendRefCountedPointer::SendOf` to `Of`.
3. Update all consumer sites: `CloneableOf` -> `Of`,
   `SendOf` -> `Of` (with qualified syntax where needed).
4. Update free function signatures and re-exports.
5. Run `just verify`.

### Phase 4: Documentation

Update all docs that reference the old hierarchy, `Pointer` trait,
old trait names (`UnsizedCoercible`, `SendUnsizedCoercible`),
old method names (`coerce_fn`, `coerce_ref_fn`, `coerce_send_fn`,
`coerce_send_ref_fn`), or old associated type names (`CloneableOf`,
`SendOf`):

1. `fp-library/docs/pointer-abstraction.md`: Rewrite hierarchy
   diagram and description. Describe `Pointer`,
   `RefCountedPointer`, and `SendRefCountedPointer` as independent
   traits. Add `ToDynFn`, `ToDynCloneFn`, `ToDynSendFn` descriptions.
   Add `BoxBrand`. Update associated type names to `Of`. Update
   `FnBrand<P>` description to reflect `SendCloneFn` independence.
2. `fp-library/docs/limitations-and-workarounds.md`: Update the
   "No Refinement of Associated Type Bounds in Subtraits" section
   to use the new `Of` names and reflect the flattened hierarchy.
   Update the "Foldable and CloneFn" section if it references the
   pointer hierarchy.
3. `fp-library/docs/zero-cost.md` (line 26): Update reference to
   "unified `Pointer` hierarchy" to describe the flat trait set.
4. `fp-library/docs/features.md` (line 211): Update mention of
   `Pointer`, `RefCountedPointer`, `SendRefCountedPointer` to
   reflect flattened hierarchy and new traits.
5. `fp-library/docs/std-coverage-checklist.md` (line 66): Update
   `Pointer` row, add `ToDynFn`/`BoxBrand` rows.
6. `CLAUDE.md` (line 115): Update pointer-abstraction.md description
   to reflect flattened hierarchy.
7. `fp-library/src/classes/pointer.rs` module-level doc: Update
   hierarchy overview to reflect flattened structure.
8. `fp-library/CHANGELOG.md`: No changes (historical record).
9. Run `just verify`.

## Follow-up opportunities

### Unify Coyoneda variants over pointer brand

`Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` are three separate types
that duplicate the same free-functor structure, differing only in the
pointer used to wrap inner layers and functions:

| Variant       | Inner layer pointer          | Function storage             | Cloneable | Send |
| :------------ | :--------------------------- | :--------------------------- | :-------- | :--- |
| `Coyoneda`    | `Box<dyn CoyonedaInner>`     | Inline (erased by outer Box) | No        | No   |
| `RcCoyoneda`  | `Rc<dyn RcCoyonedaLowerRef>` | `Rc<dyn Fn>`                 | Yes       | No   |
| `ArcCoyoneda` | `Arc<dyn ...>`               | `Arc<dyn Fn + Send + Sync>`  | Yes       | Yes  |

With the flat pointer/coercion traits introduced by this plan, these
could potentially be unified into a single `Coyoneda<P: Pointer>`
where `P` determines both the layer pointer and function storage:

- `Coyoneda<BoxBrand>`: uses `Box<dyn ...>` for layers, stores
  functions inline (erased by the outer `Box`). Single-owner,
  consumed by `lower`. Equivalent to current `Coyoneda`.
- `Coyoneda<RcBrand>`: uses `Rc<dyn ...>` for layers, wraps
  functions via `ToDynCloneFn`. Clonable, `lower_ref` borrows
  `&self`. Equivalent to current `RcCoyoneda`.
- `Coyoneda<ArcBrand>`: uses `Arc<dyn ...>` for layers, wraps
  functions via `ToDynSendFn`. Clonable + Send. Equivalent to
  current `ArcCoyoneda`.

The consuming `lower(self)` and borrowing `lower_ref(&self)` methods
could coexist on the same type, available based on which capabilities
`P` provides. This would eliminate code duplication across the three
variants.

This is out of scope for this plan but is a natural follow-up that
exercises the new trait hierarchy in a real use case.

## Success criteria

The plan is complete when:

- `Pointer` is an independent trait (no supertraits, not a supertrait
  of `RefCountedPointer`). `ToDynFn` extends it (logically
  necessary).
- `ToDynFn` trait exists with `new` / `ref_new` methods wrapping
  closures into `<Self as Pointer>::Of<dyn Fn>`.
- `BoxBrand` exists, implementing `Pointer` (`Of = Box<T>`) and
  `ToDynFn` only. Does not implement `RefCountedPointer` or any
  Send/Clone traits.
- `RefCountedPointer` is an independent trait (no supertraits).
  `ToDynCloneFn` extends it (logically necessary).
- `SendRefCountedPointer` is an independent trait (no supertraits).
  `ToDynSendFn` extends it (logically necessary).
- `ToDynSendFn` does not extend `ToDynCloneFn`.
- `UnsizedCoercible` is renamed to `ToDynCloneFn` with methods
  `new` / `ref_new`.
- `SendUnsizedCoercible` is renamed to `ToDynSendFn` with methods
  `new` / `ref_new`.
- `RefCountedPointer::Of` (renamed from `CloneableOf`) and
  `SendRefCountedPointer::Of` (renamed from `SendOf`) follow the
  library's `Of` naming convention.
- All existing tests, doctests, and benchmarks pass.
