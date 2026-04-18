# Plan: Marker Trait Refactor for Pointer and CloneFn Hierarchies

**Status:** DRAFT

## Current progress

None. Plan is in draft stage for review.

## Open questions, issues and blockers

1. Should `Pointer` be removed entirely, or retained as a capability
   trait for potential future `Box`-based pointer brands? Currently
   `Pointer::Of` and `Pointer::new` are unused outside their own free
   function and as `RefCountedPointer`'s supertrait. See Decision A.
2. Should `Arrow` be restructured to follow the same capability-trait
   pattern, or left as-is? Arrow has supertraits `Category + Strong`
   and its own `Of` associated type. It only applies to `FnBrand<P>`
   and is consumed by the optics system. See Decision F.

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

Refactor the pointer trait hierarchy (`Pointer`, `RefCountedPointer`,
`SendRefCountedPointer`, `UnsizedCoercible`, `SendUnsizedCoercible`)
and the CloneFn hierarchy (`CloneFn`, `SendCloneFn` and their Lift
subtraits) to use a capability-trait + marker-trait composition
pattern. Independent capability traits define associated types with
specific bounds. Marker traits compose capabilities via supertraits
and are blanket-implemented, so implementors only implement the
capability traits they support.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when they
lead to a better end state. This plan prioritises design correctness
and internal coherence over preserving compatibility with the current
public surface.

## Motivation

The current pointer and CloneFn hierarchies have two structural issues:

1. **Linear hierarchy encodes artificial ordering.** The current
   pointer hierarchy is a linear supertrait chain (`Pointer` <-
   `RefCountedPointer` <- `SendRefCountedPointer`), which implies
   that each level is a strict refinement of the one below. In
   reality, the capabilities (heap allocation, cloneability, thread
   safety, unsized coercion, take-cells) are logically independent.
   A flat composition of independent capability traits reflects the
   actual structure: these are orthogonal capabilities that happen
   to be co-implemented by the current brands, not a linear
   progression.

2. **Inconsistent composition patterns.** The pointer hierarchy uses
   the linear chain described above, while the CloneFn hierarchy uses
   independent parallel traits (`CloneFn` and `SendCloneFn` with no
   supertrait relationship). Both patterns exist for valid reasons,
   but the inconsistency makes the architecture harder to reason
   about. The capability-trait pattern unifies both hierarchies under
   the same organizational principle.

3. **Redundant associated types with identical implementations.** For
   `ArcBrand`, `Pointer::Of`, `RefCountedPointer::CloneableOf`, and
   `SendRefCountedPointer::SendOf` all resolve to `Arc<T>`. The
   separate types exist to express different bounds on the wrapper
   (`Deref` vs `Clone + Deref` vs `Clone + Send + Sync + Deref`), but
   the implementor writes the same `Arc<T>` three times.

4. **Dead abstractions.** `Pointer` as a standalone trait is never
   used as a bound anywhere in the codebase except by
   `RefCountedPointer`'s supertrait declaration and `Pointer`'s own
   free function.

The capability-trait pattern addresses all four: each capability is
an independent trait with its own associated type and bounds, marker
traits compose them into named bundles via blanket impls, the
hierarchy is flat rather than linear, and implementors only define the
capabilities they actually support.

## Design

### Capability traits

Each capability trait defines exactly one associated type named `Of`,
with specific bounds. The trait name disambiguates which `Of` is meant
(e.g., `<P as HasPointer>::Of` vs `<P as HasCloneablePointer>::Of`).
This follows the library's existing convention where `Kind`, `CloneFn`,
`SendCloneFn`, and `Pointer` all use `Of` as their associated type
name.

```rust
/// Wraps a value in a heap-allocated pointer.
pub trait HasPointer {
    type Of<'a, T: ?Sized + 'a>: Deref<Target = T> + 'a;

    fn ptr_new<'a, T: 'a>(value: T) -> Self::Of<'a, T>
    where
        Self::Of<'a, T>: Sized;
}

/// Wraps a value in a cloneable, reference-counted pointer.
pub trait HasCloneablePointer {
    type Of<'a, T: ?Sized + 'a>: Clone + Deref<Target = T> + 'a;

    fn clone_ptr_new<'a, T: 'a>(value: T) -> Self::Of<'a, T>
    where
        Self::Of<'a, T>: Sized;

    fn try_unwrap<'a, T: 'a>(
        ptr: Self::Of<'a, T>,
    ) -> Result<T, Self::Of<'a, T>>;
}

/// Wraps a value in a thread-safe, cloneable, reference-counted pointer.
pub trait HasSendPointer {
    type Of<'a, T: ?Sized + Send + Sync + 'a>:
        Clone + Send + Sync + Deref<Target = T> + 'a;

    fn send_ptr_new<'a, T: Send + Sync + 'a>(value: T) -> Self::Of<'a, T>
    where
        Self::Of<'a, T>: Sized;
}

/// Provides a cloneable one-shot take-cell.
pub trait HasTakeCell {
    type Of<'a, T: 'a>: Clone + 'a;

    fn take_cell_new<'a, T: 'a>(value: T) -> Self::Of<'a, T>;
    fn take_cell_take<'a, T: 'a>(cell: &Self::Of<'a, T>) -> Option<T>;
}

/// Coerces a sized closure to a dyn Fn trait object in a cloneable pointer.
pub trait HasUnsizedCoercion: HasCloneablePointer + 'static {
    fn coerce_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(A) -> B,
    ) -> <Self as HasCloneablePointer>::Of<'a, dyn 'a + Fn(A) -> B>;

    fn coerce_ref_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(&A) -> B,
    ) -> <Self as HasCloneablePointer>::Of<'a, dyn 'a + Fn(&A) -> B>;
}

/// Coerces a sized Send+Sync closure to a dyn Fn + Send + Sync trait
/// object in a thread-safe pointer.
pub trait HasSendUnsizedCoercion: HasUnsizedCoercion + HasSendPointer + 'static {
    fn coerce_send_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(A) -> B + Send + Sync,
    ) -> <Self as HasSendPointer>::Of<'a, dyn 'a + Fn(A) -> B + Send + Sync>;

    fn coerce_send_ref_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(&A) -> B + Send + Sync,
    ) -> <Self as HasSendPointer>::Of<'a, dyn 'a + Fn(&A) -> B + Send + Sync>;
}
```

### CloneFn capability traits

The CloneFn hierarchy uses the same pattern. `CloneFn` and
`SendCloneFn` remain independent (they must be, because their `Deref`
targets are different unsized types: `dyn Fn` vs
`dyn Fn + Send + Sync`). But each is structured as a capability trait.

```rust
/// Cloneable wrapper over a closure (by-value or by-reference).
pub trait HasCloneFn<Mode: ClosureMode = Val> {
    type PointerBrand: HasCloneablePointer;
    type Of<'a, A: 'a, B: 'a>: 'a + Clone
        + Deref<Target = Mode::Target<'a, A, B>>;
}

/// Construction for Val-mode CloneFn wrappers.
pub trait HasLiftFn: HasCloneFn<Val> {
    fn lift_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(A) -> B,
    ) -> <Self as HasCloneFn>::Of<'a, A, B>;
}

/// Construction for Ref-mode CloneFn wrappers.
pub trait HasRefLiftFn: HasCloneFn<Ref> {
    fn ref_lift_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(&A) -> B,
    ) -> <Self as HasCloneFn<Ref>>::Of<'a, A, B>;
}

/// Thread-safe cloneable wrapper over a closure.
pub trait HasSendCloneFn<Mode: ClosureMode = Val> {
    type Of<'a, A: 'a, B: 'a>: 'a + Clone + Send + Sync
        + Deref<Target = Mode::SendTarget<'a, A, B>>;
}

/// Construction for Val-mode SendCloneFn wrappers.
pub trait HasSendLiftFn: HasSendCloneFn<Val> {
    fn send_lift_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(A) -> B + Send + Sync,
    ) -> <Self as HasSendCloneFn>::Of<'a, A, B>;
}

/// Construction for Ref-mode SendCloneFn wrappers.
pub trait HasSendRefLiftFn: HasSendCloneFn<Ref> {
    fn send_ref_lift_fn<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(&A) -> B + Send + Sync,
    ) -> <Self as HasSendCloneFn<Ref>>::Of<'a, A, B>;
}
```

### Marker traits (blanket-implemented)

Marker traits compose capability traits and are automatically derived
via blanket impls. Implementors never implement these directly.

**Architectural constraint:** Marker traits in this design cannot have
their own associated types. A blanket impl cannot define an associated
type that unifies or combines `Of` types from multiple capability
supertraits, because the blanket has no way to know the concrete
relationship between them. For example, a marker trait cannot define
an `Of` that is guaranteed to be both `Clone` (from
`HasCloneablePointer::Of`) and `Send + Sync` (from
`HasSendPointer::Of`), because the trait system does not know these
are the same underlying type (even though they are for `ArcBrand`).

This does not affect consumer call sites: consumers bound on marker
traits (e.g., `P: SendRefCountedPointer`) and project through
whichever capability supertrait has the right `Of` for their needs
(e.g., `<P as HasSendPointer>::Of` for a thread-safe pointer). No
ad-hoc where clauses are needed.

The constraint only applies if a _new marker trait_ is introduced
that needs its own associated type with bounds drawn from multiple
capabilities. In that case, the marker trait must be manually
implemented per type rather than blanket-implemented, losing the
automatic derivation property. The alternative is to introduce a new
capability trait that provides the combined guarantee directly, with
manual implementations per brand.

For the current design, no marker trait needs a cross-capability
associated type, so this constraint does not apply. It is documented
here for visibility in case future requirements change.

```rust
/// A reference-counted pointer with cloning, take-cell, and unsized
/// coercion capabilities.
pub trait RefCountedPointer:
    HasPointer + HasCloneablePointer + HasTakeCell + HasUnsizedCoercion
{}
impl<T> RefCountedPointer for T
where T: HasPointer + HasCloneablePointer + HasTakeCell + HasUnsizedCoercion {}

/// A thread-safe reference-counted pointer with all capabilities.
pub trait SendRefCountedPointer:
    RefCountedPointer + HasSendPointer + HasSendUnsizedCoercion
{}
impl<T> SendRefCountedPointer for T
where T: RefCountedPointer + HasSendPointer + HasSendUnsizedCoercion {}

/// A function brand with by-value and by-reference closure wrapping.
pub trait CloneFnBrand:
    HasCloneFn<Val> + HasCloneFn<Ref> + HasLiftFn + HasRefLiftFn
{}
impl<T> CloneFnBrand for T
where T: HasCloneFn<Val> + HasCloneFn<Ref> + HasLiftFn + HasRefLiftFn {}

/// A thread-safe function brand with all closure wrapping capabilities.
pub trait SendCloneFnBrand:
    CloneFnBrand
    + HasSendCloneFn<Val>
    + HasSendCloneFn<Ref>
    + HasSendLiftFn
    + HasSendRefLiftFn
{}
impl<T> SendCloneFnBrand for T
where
    T: CloneFnBrand
        + HasSendCloneFn<Val>
        + HasSendCloneFn<Ref>
        + HasSendLiftFn
        + HasSendRefLiftFn,
{}
```

### Impl landscape

`RcBrand` implements: `HasPointer`, `HasCloneablePointer`,
`HasTakeCell`, `HasUnsizedCoercion`. It automatically satisfies
`RefCountedPointer`.

`ArcBrand` implements: `HasPointer`, `HasCloneablePointer`,
`HasTakeCell`, `HasUnsizedCoercion`, `HasSendPointer`,
`HasSendUnsizedCoercion`. It automatically satisfies both
`RefCountedPointer` and `SendRefCountedPointer`.

A future `BoxBrand` would implement only `HasPointer`. It would not
satisfy `RefCountedPointer` or any marker requiring cloneability.

`FnBrand<P: HasUnsizedCoercion>` implements: `HasCloneFn<Val>`,
`HasCloneFn<Ref>`, `HasLiftFn`, `HasRefLiftFn`. It automatically
satisfies `CloneFnBrand`.

`FnBrand<P: HasSendUnsizedCoercion>` additionally implements:
`HasSendCloneFn<Val>`, `HasSendCloneFn<Ref>`, `HasSendLiftFn`,
`HasSendRefLiftFn`. It automatically satisfies `SendCloneFnBrand`.

### What consumers bound on

Consumers choose the narrowest bound they need:

| Consumer needs                                                 | Bounds on                                            |
| -------------------------------------------------------------- | ---------------------------------------------------- |
| A heap-allocated pointer (no cloning)                          | `HasPointer`                                         |
| A cloneable pointer                                            | `HasCloneablePointer`                                |
| Cloneable pointer + unsized coercion                           | `HasUnsizedCoercion` (implies `HasCloneablePointer`) |
| Thread-safe pointer                                            | `HasSendPointer`                                     |
| Full ref-counted pointer (current `RefCountedPointer` surface) | `RefCountedPointer` (marker)                         |
| Thread-safe ref-counted pointer                                | `SendRefCountedPointer` (marker)                     |
| A by-value closure wrapper                                     | `HasCloneFn<Val>` or `HasLiftFn`                     |
| A thread-safe closure wrapper                                  | `HasSendCloneFn<Val>` or `HasSendLiftFn`             |
| Full CloneFn + SendCloneFn surface                             | `SendCloneFnBrand` (marker)                          |

### `HasPointer` and `Box`-based pointer brands

`HasPointer` is the non-clonable pointer capability. It appears in
the capability trait listing above and would be the sole capability
implemented by a `BoxBrand`:

```rust
impl HasPointer for BoxBrand {
    type Of<'a, T: ?Sized + 'a> = Box<T>;

    fn ptr_new<'a, T: 'a>(value: T) -> Box<T> {
        Box::new(value)
    }
}
```

`BoxBrand` would not implement `HasCloneablePointer` (since `Box` is
not `Clone`), so it would never satisfy `RefCountedPointer` or any
marker requiring cloneability. Code bounded on `HasPointer` alone
could accept both `Box`-based and `Rc`/`Arc`-based pointer brands,
while code needing cloneability would bound on `HasCloneablePointer`
or higher.

Decision A determines whether `HasPointer` is included now or
deferred. The current `Pointer` trait is dead code: no consumer
bounds on it, no consumer uses `Pointer::Of`. Including `HasPointer`
now provides a clean extension point for `BoxBrand` without
additional refactoring. Deferring it avoids paying for an abstraction
with no current consumer.

## Decisions

### Decision A: Disposition of `Pointer` / `HasPointer`

**A1** (recommended). Include `HasPointer` as a capability trait.
`RcBrand` and `ArcBrand` implement it (trivially, since their
`HasPointer::Ptr` is the same as `HasCloneablePointer::ClonePtr`).
`RefCountedPointer` marker includes `HasPointer` in its supertraits.

_Rationale:_ `HasPointer` is the natural extension point for a
`BoxBrand` (heap-allocated, non-clonable pointer). Including it now
means adding `BoxBrand` later requires only implementing `HasPointer`
for it, with no changes to the trait hierarchy. The cost is one
additional trivial impl per existing brand and one extra supertrait
on the `RefCountedPointer` marker. If `HasPointer` is omitted now
and `BoxBrand` is needed later, the hierarchy must be reopened to
insert it.

**A2** (alternative). Remove `Pointer` entirely. No `HasPointer`
capability trait is introduced.

_Rationale:_ `Pointer::Of` and `Pointer::new` are currently unused
outside their own free function and `RefCountedPointer`'s supertrait
declaration. No consumer bounds on `Pointer` alone. YAGNI applies.
The cost of deferral is that introducing `BoxBrand` later requires
adding `HasPointer` and updating `RefCountedPointer`'s supertraits
at that time.

### Decision B: Naming convention

Use `Has*` prefix for capability traits. Marker traits keep their
current names (`RefCountedPointer`, `SendRefCountedPointer`) or use
descriptive bundle names (`CloneFnBrand`, `SendCloneFnBrand`).

_Rationale:_ the `Has*` prefix clearly distinguishes capability traits
(which implementors write `impl` blocks for) from marker traits (which
are blanket-implemented and used as bounds). This follows the pattern
of capability traits in other Rust ecosystems (e.g., `HasLen`,
`HasColor` in various crates).

### Decision C: Separate `HasTakeCell` from `HasCloneablePointer`

**C1** (recommended). `HasTakeCell` is a separate capability trait.

_Rationale:_ take-cells are a distinct capability from cloneable
pointers. `RcBrand` uses `Rc<RefCell<Option<T>>>`, `ArcBrand` uses
`Arc<Mutex<Option<T>>>`. These are structurally different from the
main pointer types and involve interior mutability. A pointer brand
could theoretically provide cloneable pointers without take-cells.
Separating them follows the principle of not bundling unrelated
capabilities.

**C2** (alternative). Keep `HasTakeCell` merged into
`HasCloneablePointer` (matching the current `RefCountedPointer`).

_Rationale:_ every current implementor provides both. Separating
adds a trait that is always co-implemented. Simplicity over
theoretical flexibility.

### Decision D: `HasUnsizedCoercion` supertrait bound

`HasUnsizedCoercion` requires `HasCloneablePointer + 'static` as
supertraits, because its methods return
`Self::ClonePtr<'a, dyn Fn ...>`, which depends on `ClonePtr`.

Similarly, `HasSendUnsizedCoercion` requires
`HasUnsizedCoercion + HasSendPointer + 'static`, because its methods
return `Self::SendPtr<'a, dyn Fn ... + Send + Sync>`.

_Rationale:_ these are inherent dependencies, not design choices.
The coercion methods produce pointers, so they need access to the
pointer associated types.

### Decision E: Migration strategy

**E1** (recommended). Strangler-fig migration within the pointer and
CloneFn modules. Introduce capability traits, implement them for
existing brands, add blanket-implemented markers, then migrate
consumers from old trait bounds to new trait bounds one module at a
time. Remove old traits once all consumers are migrated.

_Rationale:_ allows incremental verification. The codebase compiles
at every step. Old and new traits coexist during migration.

### Decision F: Arrow trait disposition

**F1** (recommended). Leave `Arrow` unchanged for now. It has
supertraits `Category + Strong` and its own `Of` associated type.
It is only implemented by `FnBrand<P>` and consumed by the optics
system. Restructuring it to follow the capability-trait pattern
would require restructuring `Category`, `Semigroupoid`, `Strong`,
`Choice`, `Closed`, and `Wander`, which is a large change with
limited benefit.

_Rationale:_ Arrow and its related profunctor traits are a separate
subsystem (optics). Their structure does not suffer from the same
issues as the pointer/CloneFn hierarchies (no redundant associated
types, no dead abstractions, no inconsistent patterns). Scope
containment.

**F2** (alternative). Restructure Arrow and profunctor traits to
follow the capability-trait pattern in a follow-up plan.

### Decision G: `LazyConfig` adaptation

`LazyConfig` currently has `type PointerBrand: RefCountedPointer`.
Under the new design, `RefCountedPointer` becomes a blanket-
implemented marker trait. This bound continues to work as-is; no
change is needed.

_Rationale:_ marker traits are valid as bounds. The blanket impl
ensures any type implementing the required capabilities automatically
satisfies the marker. `LazyConfig` consumers do not need to change.

### Decision H: Re-export strategy

Capability traits and marker traits are both exported from
`fp-library/src/classes/`. The module structure mirrors the current
layout but with capability traits in their own files.

_Rationale:_ follows existing module conventions. Capability traits
are the implementation details; marker traits are the primary public
API for bounds.

## Integration surface

### Will change

| Component                                            | Change                                                                                                                                                                                              |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/classes/pointer.rs`                  | Convert `Pointer` to `HasPointer` capability trait (Decision A1).                                                                                                                                   |
| `fp-library/src/classes/ref_counted_pointer.rs`      | Split into `HasCloneablePointer` and `HasTakeCell` capability traits. Add blanket-implemented `RefCountedPointer` marker.                                                                           |
| `fp-library/src/classes/send_ref_counted_pointer.rs` | Convert to `HasSendPointer` capability trait. Add blanket-implemented `SendRefCountedPointer` marker.                                                                                               |
| `fp-library/src/classes/unsized_coercible.rs`        | Convert to `HasUnsizedCoercion` capability trait with `HasCloneablePointer` supertrait.                                                                                                             |
| `fp-library/src/classes/send_unsized_coercible.rs`   | Convert to `HasSendUnsizedCoercion` capability trait with `HasUnsizedCoercion + HasSendPointer` supertraits.                                                                                        |
| `fp-library/src/classes/clone_fn.rs`                 | Convert to `HasCloneFn`, `HasLiftFn`, `HasRefLiftFn` capability traits. Add blanket-implemented `CloneFnBrand` marker.                                                                              |
| `fp-library/src/classes/send_clone_fn.rs`            | Convert to `HasSendCloneFn`, `HasSendLiftFn`, `HasSendRefLiftFn` capability traits. Add blanket-implemented `SendCloneFnBrand` marker.                                                              |
| `fp-library/src/types/rc_ptr.rs`                     | Implement `HasPointer`, `HasCloneablePointer`, `HasTakeCell`, `HasUnsizedCoercion` for `RcBrand`.                                                                                                   |
| `fp-library/src/types/arc_ptr.rs`                    | Implement all capability traits for `ArcBrand`.                                                                                                                                                     |
| `fp-library/src/types/fn_brand.rs`                   | Implement `HasCloneFn`, `HasLiftFn`, `HasRefLiftFn` for `FnBrand<P: HasUnsizedCoercion>`. Implement `HasSendCloneFn`, `HasSendLiftFn`, `HasSendRefLiftFn` for `FnBrand<P: HasSendUnsizedCoercion>`. |
| `fp-library/src/classes.rs`                          | Update module exports and re-exports for new trait names.                                                                                                                                           |
| All consumer sites                                   | Migrate bounds from old trait names to new trait names (or marker traits).                                                                                                                          |
| `fp-library/docs/limitations-and-workarounds.md`     | Update "Parallel Traits" section to describe capability-trait architecture.                                                                                                                         |
| `fp-library/docs/pointer-abstraction.md`             | Update to describe new hierarchy.                                                                                                                                                                   |
| `fp-library/docs/parallelism.md`                     | Update `SendCloneFn` description.                                                                                                                                                                   |

### Unchanged

- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`, etc.).
  Optics bound on `UnsizedCoercible`; the marker trait
  `RefCountedPointer` (or direct `HasUnsizedCoercion` bound)
  continues to work.
- **Arrow and profunctor traits** (Decision F1).
- **Dispatch system** (`dispatch/*.rs`). Dispatch traits and
  inference wrappers are unaffected; they bound on `CloneFn` /
  `SendCloneFn` which become `HasCloneFn` / `HasSendCloneFn`.
- **HKT machinery** (`kinds.rs`, `brands.rs`, proc macros).
- **Type class traits** (`Functor`, `Monad`, `Foldable`, etc.).
- **Benchmarks**, **tests** (except for import path updates).
- **`LazyConfig`** (Decision G: marker trait bound continues to work).

## Implementation phasing

### Phase 0: Non-regression tests

1. Add a non-regression test file exercising the current pointer and
   CloneFn APIs: `Pointer::new`, `RefCountedPointer::cloneable_new`,
   `SendRefCountedPointer::send_new`, `try_unwrap`, `take_cell_new`,
   `take_cell_take`, `UnsizedCoercible::coerce_fn`,
   `SendUnsizedCoercible::coerce_send_fn`, `LiftFn::new`,
   `RefLiftFn::ref_new`, `SendLiftFn::new`, `SendRefLiftFn::ref_new`.
   All through both free functions and trait method syntax.
2. Run `just verify` to confirm baseline.

### Phase 1: Introduce capability traits (pointer hierarchy)

Introduce new capability traits alongside existing traits. Both
coexist; no consumers are migrated yet.

1. Convert `Pointer` to `HasPointer` capability trait in
   `fp-library/src/classes/pointer.rs` (or new file
   `has_pointer.rs`). Implement for `RcBrand` and `ArcBrand`.
2. Create `HasCloneablePointer` trait in
   `fp-library/src/classes/has_cloneable_pointer.rs`.
   Implement for `RcBrand` and `ArcBrand`.
3. Create `HasTakeCell` trait in
   `fp-library/src/classes/has_take_cell.rs`.
   Implement for `RcBrand` and `ArcBrand`.
4. Create `HasSendPointer` trait in
   `fp-library/src/classes/has_send_pointer.rs`.
   Implement for `ArcBrand`.
5. Create `HasUnsizedCoercion` trait (supertraits:
   `HasCloneablePointer + 'static`) in
   `fp-library/src/classes/has_unsized_coercion.rs`.
   Implement for `RcBrand` and `ArcBrand`.
6. Create `HasSendUnsizedCoercion` trait (supertraits:
   `HasUnsizedCoercion + HasSendPointer + 'static`) in
   `fp-library/src/classes/has_send_unsized_coercion.rs`.
   Implement for `ArcBrand`.
7. Add blanket-implemented `RefCountedPointer` and
   `SendRefCountedPointer` marker traits (new versions, under
   temporary names to avoid conflict with originals).
8. Run `just verify`.

### Phase 2: Introduce capability traits (CloneFn hierarchy)

1. Create `HasCloneFn<Mode>` trait in
   `fp-library/src/classes/has_clone_fn.rs`.
   Implement for `FnBrand<P: HasUnsizedCoercion>`.
2. Create `HasLiftFn` and `HasRefLiftFn` traits.
   Implement for `FnBrand<P: HasUnsizedCoercion>`.
3. Create `HasSendCloneFn<Mode>` trait in
   `fp-library/src/classes/has_send_clone_fn.rs`.
   Implement for `FnBrand<P: HasSendUnsizedCoercion>`.
4. Create `HasSendLiftFn` and `HasSendRefLiftFn` traits.
   Implement for `FnBrand<P: HasSendUnsizedCoercion>`.
5. Add blanket-implemented `CloneFnBrand` and `SendCloneFnBrand`
   marker traits (temporary names).
6. Run `just verify`.

### Phase 3: Migrate consumers

Migrate all consumer code from old trait bounds to new capability
trait or marker trait bounds. This is the largest phase; proceed
module by module.

1. Migrate `fn_brand.rs`: change `P: UnsizedCoercible` bounds to
   `P: HasUnsizedCoercion`, `P: SendUnsizedCoercible` to
   `P: HasSendUnsizedCoercion`.
2. Migrate `clone_fn.rs` consumers: change `CloneFn` bounds to
   `HasCloneFn` bounds throughout dispatch modules.
3. Migrate `send_clone_fn.rs` consumers: change `SendCloneFn` bounds
   to `HasSendCloneFn` bounds.
4. Migrate `ref_counted_pointer.rs` consumers: change
   `RefCountedPointer` bounds to `HasCloneablePointer` (or marker
   `RefCountedPointer` where multiple capabilities are needed).
5. Migrate optics system: change `UnsizedCoercible` bounds to
   `HasUnsizedCoercion`.
6. Migrate `LazyConfig` bound.
7. Migrate all remaining consumers.
8. Run `just verify` after each sub-step.

### Phase 4: Remove old traits

With all consumers migrated, remove the old trait definitions and
rename temporary marker traits to their final names.

1. Remove old `Pointer` trait and its impls (superseded by
   `HasPointer`, Decision A1).
2. Remove old `RefCountedPointer`, `SendRefCountedPointer`,
   `UnsizedCoercible`, `SendUnsizedCoercible` traits and their impls.
3. Remove old `CloneFn`, `LiftFn`, `RefLiftFn`, `SendCloneFn`,
   `SendLiftFn`, `SendRefLiftFn` traits and their impls.
4. Rename temporary marker traits to final names
   (`RefCountedPointer`, `SendRefCountedPointer`, `CloneFnBrand`,
   `SendCloneFnBrand`).
5. Remove old module files, update `classes.rs` exports.
6. Run `just verify`.

### Phase 5: Documentation and cleanup

1. Update `fp-library/docs/pointer-abstraction.md` to describe the
   capability-trait architecture.
2. Update `fp-library/docs/limitations-and-workarounds.md` to
   describe the new hierarchy.
3. Update `fp-library/docs/parallelism.md` for `HasSendCloneFn`.
4. Update doc comments on all new traits.
5. Update `CLAUDE.md` if any guidance changes.
6. Verify non-regression tests still pass.
7. Run `just verify`.

## Success criteria

The plan is complete when:

- `RcBrand` implements `HasPointer`, `HasCloneablePointer`,
  `HasTakeCell`, `HasUnsizedCoercion` and automatically satisfies
  `RefCountedPointer`.
- `ArcBrand` additionally implements `HasSendPointer`,
  `HasSendUnsizedCoercion` and automatically satisfies
  `SendRefCountedPointer`.
- `FnBrand<RcBrand>` automatically satisfies `CloneFnBrand`.
- `FnBrand<ArcBrand>` automatically satisfies `SendCloneFnBrand`.
- A future `BoxBrand` could implement only `HasPointer` and be
  accepted by code bounded on `HasPointer` without any hierarchy
  changes.
- The old `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`,
  `UnsizedCoercible`, `SendUnsizedCoercible`, `CloneFn`, `LiftFn`,
  `RefLiftFn`, `SendCloneFn`, `SendLiftFn`, and `SendRefLiftFn`
  traits are removed.
- All existing tests, doctests, and benchmarks pass.
- No consumer code uses the old trait names directly.
- The capability-trait + marker-trait pattern is consistently applied
  across both the pointer and CloneFn hierarchies.
