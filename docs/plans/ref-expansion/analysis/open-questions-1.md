# Ref Expansion: Open Questions Investigation 1

Focus: trait design and signatures.

---

## 1. Supertrait consistency

### Issue

The plan proposes `RefAlt: RefFunctor`, mirroring `Alt: Functor`. It also
proposes `RefCompactable` as a standalone trait with no supertraits (mirroring
`Compactable`, which has no supertraits). But Filterable requires
`Compactable + Functor`, and the existing `RefFilterable` requires
`RefFunctor + Compactable` (by-value Compactable, not RefCompactable). After
adding RefCompactable, should RefFilterable's supertrait change to include
RefCompactable instead of (or in addition to) Compactable?

### Research findings

**Alt -> RefAlt supertrait chain:**

- `Alt: Functor` (by-value). The plan's `RefAlt: RefFunctor` mirrors this
  exactly. No issue here.

**Compactable has no supertraits.** `Compactable` inherits only from the Kind
constraint. The plan's `RefCompactable` also has no supertraits beyond Kind.
This is consistent.

**Filterable -> RefFilterable supertrait chain:**

- `Filterable: Compactable + Functor` (by-value, line 132 of filterable.rs).
- `RefFilterable: RefFunctor + Compactable` (by-ref, line 39 of
  ref_filterable.rs).

RefFilterable currently uses by-value `Compactable` (not `RefCompactable`)
as a supertrait. This is deliberate: RefFilterable's default implementations
work by calling `ref_map` to produce an owned `F<Option<B>>` or
`F<Result<O, E>>`, then passing that owned container to the by-value
`compact` or `separate`. The container flowing into `compact`/`separate` is
already owned (it was just constructed by `ref_map`), so there is no need for
by-reference compact/separate.

For example, `ref_partition_map` defaults to:

```
Self::separate::<E, O>(Self::ref_map::<A, Result<O, E>>(func, fa))
```

The `ref_map` returns an owned container, and `separate` consumes it. No
RefCompactable is needed.

### Approaches

**Approach A: Keep RefFilterable's supertrait as `RefFunctor + Compactable`.**

The default implementations work correctly as-is. Adding RefCompactable as a
supertrait would be an unnecessary burden, forcing implementors to provide
RefCompactable even when their RefFilterable implementations never use it.

**Approach B: Change RefFilterable to require `RefFunctor + RefCompactable`.**

This would mirror the by-value hierarchy more closely, but it would break the
existing defaults (which delegate to by-value compact/separate) and force all
RefFilterable implementors to also implement RefCompactable. Since
RefCompactable is a new trait with fewer implementors, this narrows the set of
types that can be RefFilterable.

**Approach C: Require `RefFunctor + Compactable + RefCompactable`.**

Overly restrictive. No benefit over Approach A since the defaults use by-value
compact/separate.

### Recommendation

Approach A: keep RefFilterable's supertrait as `RefFunctor + Compactable`.
The existing design is correct and intentional. RefCompactable is a standalone
trait for direct by-reference compact/separate operations, not a prerequisite
for RefFilterable. This creates a minor asymmetry with the by-value hierarchy
(where Filterable requires Compactable), but the asymmetry is justified by the
different data flow in the defaults.

---

## 2. RefBitraversable's FnBrand parameter

### Issue

The plan includes `FnBrand` in `RefBitraversable::ref_bi_traverse`, following
the `RefTraversable` pattern. Is this correct? Does `RefBifoldable` also need
`FnBrand`? The plan shows `FnBrand` on `RefBifoldable` methods as well. What
is the justification?

### Research findings

**By-value trait FnBrand usage:**

- `Traversable::traverse`: NO FnBrand on the trait method (line 100 of
  traversable.rs). The free function `traverse` takes FnBrand as a type
  parameter but does not pass it through to the trait method.
- `Bitraversable::bi_traverse`: NO FnBrand on the trait method (line 117 of
  bitraversable.rs). No FnBrand anywhere in bitraversable.rs.
- `Foldable::fold_right`: YES, FnBrand on the trait method (line 102 of
  foldable.rs), used for the Endofunction-based default implementation.
- `Bifoldable::bi_fold_right`: YES, FnBrand on the trait method (line 131 of
  bifoldable.rs), used for the Endofunction-based default implementation.

**Ref trait FnBrand usage:**

- `RefTraversable::ref_traverse`: YES, FnBrand on the trait method (line 67
  of ref_traversable.rs). However, concrete implementations (e.g., VecBrand at
  line 2331 of vec.rs) do not use FnBrand in their bodies. It appears to be
  present for consistency with the free function signature and to allow future
  default implementations.
- `RefFoldable::ref_fold_map`: YES, FnBrand on the trait method (line 77 of
  ref_foldable.rs), used for the Endofunction-based defaults.

**Why FnBrand is on RefTraversable but not by-value Traversable:**

The by-value `Traversable::traverse` does not need FnBrand because its default
implementation (`sequence(map(func, ta))`) does not use Endofunction. The
by-value free function `traverse` takes FnBrand as a type parameter but never
passes it to the trait method; it exists only for API consistency with other
free functions (fold_right, etc.) that do need it.

RefTraversable has FnBrand on the trait method itself, likely to match the
pattern of RefFoldable (which is its supertrait) and to allow future default
implementations that might use Endofunction internally.

**Implication for RefBifoldable:**

The plan correctly includes FnBrand on `ref_bi_fold_right`,
`ref_bi_fold_left`, and `ref_bi_fold_map`. This mirrors the by-value
`Bifoldable`, which has FnBrand on all three methods for its
Endofunction-based defaults. RefBifoldable will need FnBrand for the same
reason: its default implementations will use Endofunction to derive methods
from each other.

**Implication for RefBitraversable:**

The plan includes FnBrand on `ref_bi_traverse` and `ref_bi_sequence`. This
follows the RefTraversable pattern. However, the by-value Bitraversable does
NOT have FnBrand on `bi_traverse`. The question is whether RefBitraversable
actually needs it.

If RefBitraversable has no default implementations that use Endofunction (and
the by-value Bitraversable does not), then FnBrand is not strictly necessary
on the trait method. It could be limited to the free function signature for
API consistency.

### Approaches

**Approach A: Include FnBrand on RefBitraversable (as planned).**

Follows the RefTraversable precedent. Consistent API surface between all Ref
traversal traits. Minor cost: implementors must carry the unused type
parameter.

**Approach B: Omit FnBrand from RefBitraversable trait method; include only on
the free function.**

More precise: FnBrand is only present where actually needed. Follows the
by-value Bitraversable precedent (which has no FnBrand). Breaks consistency
with RefTraversable.

### Recommendation

Approach A: include FnBrand on `RefBitraversable::ref_bi_traverse` and
`ref_bi_sequence`. The RefTraversable precedent is established, and
consistency within the Ref trait family is more important than mirroring the
by-value trait exactly. The FnBrand parameter is already present on
RefTraversable without being used in concrete implementations; the same
pattern is acceptable for RefBitraversable.

---

## 3. TryThunk and RefBifunctor

### Issue

The plan says TryThunk implements Bifunctor but not Bitraversable. Can
TryThunk implement RefBifunctor? TryThunk wraps a `Box<dyn FnOnce() -> A>`
(via `Thunk`). Calling `ref_bimap` on `&TryThunk` would need to create a new
TryThunk that applies f/g after evaluation, but the inner closure is FnOnce
and cannot be called from a reference.

### Research findings

`TryThunk<'a, A, E>` wraps `Thunk<'a, Result<A, E>>` (line 104 of
try_thunk.rs). `Thunk<'a, A>` wraps `Box<dyn FnOnce() -> A + 'a>` (line 89
of thunk.rs).

The by-value `Bifunctor` impl for TryThunkBrand (line 1457 of try_thunk.rs)
works by calling `p.0.map(move |result| ...)`, which consumes the inner
Thunk via `Thunk::map`. The `map` function takes `self` (consuming the Thunk)
to compose the new closure with the old FnOnce.

For `ref_bimap` on `&TryThunk`, we would need to:

1. Borrow the `&TryThunk<A, E>`.
2. Create a new `TryThunk<B, D>` that, when evaluated, evaluates the
   original and applies f/g.

The problem: the inner `Box<dyn FnOnce()>` cannot be cloned or called from a
shared reference. `FnOnce` can only be called once, and there is no `Clone`
impl for `Box<dyn FnOnce()>`.

There is no `Rc` wrapping here. The plan's description ("clone the inner Rc")
is incorrect. TryThunk uses `Box<dyn FnOnce>`, not `Rc<dyn Fn>`. This is
fundamentally different from `Lazy` (which uses `Rc<OnceCell>` and can be
shared).

**Can TryThunk implement RefBifunctor?** No, not without changing TryThunk's
internals. You cannot create a new closure that captures `&Thunk` and later
calls it, because:

- `Thunk::evaluate` takes `self` by value (line 415 of try_thunk.rs).
- The inner `FnOnce` cannot be cloned.
- There is no memoized result to borrow.

This is the same fundamental limitation that prevents Thunk from implementing
RefFunctor: Thunk is a non-memoized, consume-on-evaluate type.

### Approaches

**Approach A: Exclude TryThunk from RefBifunctor.**

The plan already lists 5 implementors for RefBifunctor. TryThunk would drop
to 4. This matches the pattern where Thunk does not implement RefFunctor.

**Approach B: Add a Clone-based RefBifunctor for TryThunk.**

TryThunk cannot implement Clone (FnOnce cannot be cloned). This is not
viable.

**Approach C: Change TryThunk to use `Rc<dyn Fn>` instead of
`Box<dyn FnOnce>`.**

This would fundamentally change TryThunk's semantics and performance
characteristics. Not appropriate.

### Recommendation

Approach A: exclude TryThunk from RefBifunctor. The plan's statement that
TryThunk implements RefBifunctor is incorrect. The implementor list should be
reduced from 5 to 4: Result, Pair, Tuple2, ControlFlow. Similarly, TryThunk
should be excluded from RefBifoldable (the by-value Bifoldable impl evaluates
the thunk, which consumes it; `ref_bi_fold_right` on `&TryThunk` is not
feasible for the same reason).

This means TryThunk should be excluded from ALL three Ref bi-traits, not just
RefBitraversable.

**Corrected implementor counts:**

| Trait            | Plan says | Corrected |
| ---------------- | --------- | --------- |
| RefBifunctor     | 5         | 4         |
| RefBifoldable    | 5         | 4         |
| RefBitraversable | 4         | 4         |

---

## 4. ControlFlow and RefBifunctor

### Issue

Can ControlFlow implement RefBifunctor? Verify by examining the by-value
Bifunctor impl.

### Research findings

The by-value `Bifunctor` impl for `ControlFlowBrand` (line 702 of
control_flow.rs) delegates to `ControlFlowBrand::bimap(p, f, g)`. The Kind
mapping is `Of<C, B> = ControlFlow<B, C>` (line 692), meaning the first type
parameter is Continue and the second is Break.

For `ref_bimap` on `&ControlFlow<B, C>`:

```
match p {
    ControlFlow::Continue(c) => ControlFlow::Continue(f(c)),
    ControlFlow::Break(b) => ControlFlow::Break(g(b)),
}
```

Both closures produce owned output from references. No Clone needed on the
input types. This is straightforward because ControlFlow is a simple enum
(like Result) with no lazy evaluation or interior mutability.

### Recommendation

No issue found. ControlFlow can implement RefBifunctor cleanly, following the
exact same pattern as Result.

---

## 5. RefCompactable and Filterable interaction

### Issue

RefFilterable currently has `RefFunctor + Compactable` as supertraits (NOT
RefCompactable). After adding RefCompactable, should RefFilterable's
supertrait change? What are the implications?

### Research findings

This question is the same as question 1, analyzed from a different angle. The
key insight is in how RefFilterable's default implementations work.

The default `ref_filter_map` is:

```
Self::compact(Self::ref_map(func, fa))
```

This calls `ref_map` (from RefFunctor) to produce an owned `F<Option<B>>`,
then passes it to by-value `compact` (from Compactable). The intermediate
container is owned, so by-value compact is appropriate.

If RefFilterable were changed to require RefCompactable, the default could
instead be:

```
Self::ref_compact(&Self::ref_map(func, fa))
```

But this is strictly worse: it borrows the intermediate container (which is a
temporary) and then clones elements out of it via RefCompactable, when the
by-value compact could have simply moved them.

### Recommendation

Do not change RefFilterable's supertraits. The by-value Compactable
supertrait is correct for RefFilterable's defaults. RefCompactable serves a
different purpose (compacting a container you already have by reference and
want to keep), not as a building block for RefFilterable.

---

## 6. Missing Ref supertraits for proposed traits

### Issue

Do any of the proposed Ref traits need supertraits that are not mentioned in
the plan?

### Research findings

**RefBifunctor:**

- By-value Bifunctor has no supertrait beyond Kind.
- Plan proposes RefBifunctor with no supertrait beyond Kind.
- No issue.

**RefBifoldable:**

- By-value Bifoldable has no supertrait beyond Kind.
- Plan proposes RefBifoldable with no supertrait beyond Kind.
- No issue.

**RefBitraversable:**

- By-value Bitraversable requires `Bifunctor + Bifoldable`.
- Plan proposes `RefBitraversable: RefBifunctor + RefBifoldable`.
- Question: should RefBitraversable also require by-value
  `Bifunctor + Bifoldable`?

The by-value `Traversable` requires `Functor + Foldable`. The existing
`RefTraversable` requires `RefFunctor + RefFoldable` (line 35 of
ref_traversable.rs). It does NOT require by-value `Traversable` or by-value
`Functor + Foldable`.

Following this precedent, `RefBitraversable: RefBifunctor + RefBifoldable` is
correct and should NOT additionally require by-value `Bifunctor + Bifoldable`.

However, note a subtlety: all 4 implementors of RefBitraversable (Result,
Pair, Tuple2, ControlFlow) already implement by-value Bifunctor and
Bifoldable. So in practice, the by-value supertrait would not exclude any
implementors. But adding it as a supertrait would be inconsistent with the
RefTraversable precedent and is unnecessary.

**RefCompactable:**

- By-value Compactable has no supertrait beyond Kind.
- Plan proposes RefCompactable with no supertrait beyond Kind.
- No issue.

**RefAlt:**

- By-value Alt requires `Functor`.
- Plan proposes `RefAlt: RefFunctor`.
- Question: should RefAlt also require by-value `Functor`?

Again, following precedent: RefFilterable requires `RefFunctor + Compactable`
but does NOT require by-value `Functor` or by-value `Filterable`. The pattern
is that Ref traits reference other Ref traits as supertraits, not their
by-value counterparts, unless the by-value trait provides functionality used
in defaults (as Compactable does for RefFilterable).

`RefAlt: RefFunctor` is sufficient and correct.

### Recommendation

No missing supertraits found. The proposed supertrait chains are consistent
with established patterns:

- Ref traits use Ref supertraits, not by-value supertraits.
- By-value supertraits are included only when the by-value trait's methods are
  used in default implementations (e.g., Compactable for RefFilterable).

---

## Summary

| #   | Issue                                                | Finding                                                                                                      | Action needed                                                                           |
| --- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| 1   | RefFilterable supertrait after adding RefCompactable | No change needed. RefFilterable's defaults use by-value compact on owned intermediates.                      | None.                                                                                   |
| 2   | FnBrand on RefBitraversable                          | Not strictly needed, but follow RefTraversable precedent for consistency.                                    | Keep FnBrand as planned.                                                                |
| 3   | TryThunk and RefBifunctor                            | TryThunk CANNOT implement RefBifunctor (or RefBifoldable). Thunk's FnOnce cannot be called from a reference. | Correct plan: exclude TryThunk from all three Ref bi-traits, not just RefBitraversable. |
| 4   | ControlFlow and RefBifunctor                         | Straightforward enum; no issues.                                                                             | None.                                                                                   |
| 5   | RefCompactable and RefFilterable interaction         | Same as item 1. Do not change RefFilterable.                                                                 | None.                                                                                   |
| 6   | Missing Ref supertraits                              | None found. Proposed chains are consistent with existing patterns.                                           | None.                                                                                   |

The most significant finding is item 3: the plan incorrectly lists TryThunk as
an implementor of RefBifunctor and RefBifoldable. TryThunk wraps a
`Box<dyn FnOnce>` (via Thunk), which cannot be evaluated from a shared
reference. The plan should be corrected to exclude TryThunk from all Ref
bi-traits, reducing the implementor count from 5 to 4 for RefBifunctor and
RefBifoldable.
