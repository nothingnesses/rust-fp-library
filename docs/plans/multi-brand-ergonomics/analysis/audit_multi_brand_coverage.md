# Audit: Functions That Could Benefit from Multi-Brand Inference

This audit identifies free functions outside the dispatch modules that require
Brand or FnBrand turbofish and assesses whether they could use InferableBrand
or InferableFnBrand inference instead.

## Methodology

Searched all `.rs` files in `fp-library/src/` for free functions (not trait
methods) with Brand or FnBrand type parameters bounded by Kind traits or
CloneFn. Excluded functions where Brand appears only in the return type
(inference from return type is not supported by Rust), and functions already
covered by dispatch modules.

Call site counts include doctests, unit tests, benchmarks, and production code
within the fp-library crate.

---

## Worth Migrating

These functions take a container argument from which Brand can be inferred,
have meaningful call-site counts, and have a clear inference path.

### 1. `extend` (HIGH priority)

- **File:** `fp-library/src/classes/extend.rs:312`
- **Signature:** `pub fn extend<'a, Brand: Extend, A: 'a + Clone, B: 'a>(f, wa) -> ...`
- **Call sites:** 32 (across extend.rs, identity.rs, thunk.rs, cat_list.rs, vec.rs)
- **Inference source:** `wa` container argument
- **Notes:** No dispatch module exists for extend. Would need a new
  `dispatch/extend.rs` with Val/Ref variants. The `wa` argument directly
  carries Brand, so InferableBrand inference is straightforward.

### 2. `extract` (MEDIUM priority)

- **File:** `fp-library/src/classes/extract.rs:120`
- **Signature:** `pub fn extract<'a, F, A>(fa: Apply!(...)) -> A where F: Extract`
- **Call sites:** 20 (across extract.rs, identity.rs, thunk.rs)
- **Inference source:** `fa` container argument
- **Notes:** No dispatch module exists. `extract` takes ownership of `fa`, so
  only a Val variant is needed (no Ref dispatch). The InferableBrand bound on
  `fa` would resolve `F` automatically.

### 3. `par_fold_map` (MEDIUM priority)

- **File:** `fp-library/src/classes/par_foldable.rs:134`
- **Signature:** `pub fn par_fold_map<'a, Brand: ParFoldable, A: 'a + Send, M: Monoid + Send + 'a>(f, fa) -> M`
- **Call sites:** 17 (across par_foldable.rs, vec.rs, cat_list.rs, thread_safety.rs, benchmarks)
- **Inference source:** `fa` container argument
- **Notes:** Parallel functions always take owned values (Send requirement), so
  only Val dispatch is needed. No existing dispatch module. Could be a simpler
  wrapper (no Ref variant) that adds an InferableBrand bound.

### 4. `par_map` (MEDIUM priority)

- **File:** `fp-library/src/classes/par_functor.rs:138`
- **Signature:** `pub fn par_map<'a, Brand: ParFunctor, A: 'a + Send, B: 'a + Send>(f, fa) -> ...`
- **Call sites:** 14 (across par_functor.rs, vec.rs, cat_list.rs, thread_safety.rs, benchmarks)
- **Inference source:** `fa` container argument
- **Notes:** Same as par_fold_map: owned-only, Val-only dispatch. Straightforward.

### 5. `bi_sequence` (LOW priority)

- **File:** `fp-library/src/classes/bitraversable.rs:266`
- **Signature:** `pub fn bi_sequence<'a, Brand, Applicative_, A, B>(ta) -> ...`
- **Call sites:** 12 (across bitraversable.rs, ref_bitraversable.rs, pair.rs, result.rs)
- **Inference source:** `ta` bifunctor container argument
- **Notes:** A bitraversable dispatch module already exists but may not cover
  `bi_sequence`. The bifunctor container carries Brand, so InferableBrand
  inference would work if the dispatch module is extended.

### 6. `duplicate` (LOW priority)

- **File:** `fp-library/src/classes/extend.rs:349`
- **Signature:** `pub fn duplicate<'a, Brand: Extend, A: 'a + Clone>(wa) -> ...`
- **Call sites:** 3 (extend.rs, vec.rs, cat_list.rs)
- **Inference source:** `wa` container argument
- **Notes:** Natural companion to `extend`. Would go in the same dispatch
  module. Low call count but worth including if extend is migrated.

### 7. `par_compact` (LOW priority)

- **File:** `fp-library/src/classes/par_compactable.rs:163`
- **Call sites:** 8
- **Inference source:** `fa` container argument

### 8. `par_filter_map` (LOW priority)

- **File:** `fp-library/src/classes/par_filterable.rs:188`
- **Call sites:** 8
- **Inference source:** `fa` container argument

### 9. `par_separate` (LOW priority)

- **File:** `fp-library/src/classes/par_compactable.rs:202`
- **Call sites:** 5
- **Inference source:** `fa` container argument

### 10. `par_filter` (LOW priority)

- **File:** `fp-library/src/classes/par_filterable.rs:221`
- **Call sites:** 4
- **Inference source:** `fa` container argument

---

## Not Worth Migrating

### No container input (Brand in return type only)

These functions construct containers from nothing or from non-branded inputs.
Brand cannot be inferred because no branded container is passed in.

| Function                              | File                                  | Reason                                          |
| ------------------------------------- | ------------------------------------- | ----------------------------------------------- |
| `pure`                                | classes/applicative.rs                | Constructs container from value.                |
| `empty`                               | classes/plus.rs                       | Constructs empty container.                     |
| `guard`                               | classes/plus.rs                       | Constructs from bool.                           |
| `ref_pure`, `send_ref_pure`           | classes/                              | Same as pure.                                   |
| `category_identity`, `arrow`          | classes/category.rs, classes/arrow.rs | Construct profunctors.                          |
| `tail_rec_m`                          | classes/monad_rec.rs                  | Closures produce containers but none passed in. |
| `forever`, `while_some`, `until_some` | classes/monad_rec.rs                  | Same.                                           |
| `repeat_m`, `while_m`, `until_m`      | classes/monad.rs                      | Same.                                           |

### FnBrand construction functions

These take FnBrand as a type parameter to control which wrapper (Rc vs Arc) to
use when wrapping a closure. No wrapped function container is passed in, so
InferableFnBrand cannot help.

| Function               | File                     |
| ---------------------- | ------------------------ |
| `lift_fn_new`          | classes/clone_fn.rs      |
| `ref_lift_fn_new`      | classes/clone_fn.rs      |
| `send_lift_fn_new`     | classes/send_clone_fn.rs |
| `send_ref_lift_fn_new` | classes/send_clone_fn.rs |

### Already superseded by dispatch modules

These class-level free functions have corresponding dispatch wrappers that use
InferableBrand. The class versions serve as explicit fallbacks by design.

All functions in: functor, semimonad (bind/join), semiapplicative (apply),
foldable, traversable, filterable, compactable, alt, bifunctor, bifoldable,
bitraversable, contravariant, witherable, and their \_with_index variants.

### Low usage / internal only

| Function                                                             | File                   | Call sites | Reason                                                      |
| -------------------------------------------------------------------- | ---------------------- | ---------- | ----------------------------------------------------------- |
| `when`, `unless`                                                     | classes/applicative.rs | 4 each     | Low usage, mostly doctests.                                 |
| `if_m`, `when_m`, `unless_m`                                         | classes/monad.rs       | 3-4 each   | Low usage, all doctests.                                    |
| `ref_if_m`, `ref_unless_m`                                           | classes/monad.rs       | 1 each     | Minimal usage.                                              |
| `extend_flipped`                                                     | classes/extend.rs      | 1          | Single call site.                                           |
| `compose_co_kleisli`, `compose_co_kleisli_flipped`                   | classes/extend.rs      | 1 each     | Single call site.                                           |
| `compose_kleisli`, `compose_kleisli_flipped`                         | classes/monad.rs       | Various    | Brand in closure return type only, not in input containers. |
| `send_ref_map`, `send_ref_bind`                                      | classes/               | 10, 8      | Internal/doctest usage only.                                |
| `coerce_ref_fn`, `coerce_fn`, `coerce_send_ref_fn`, `coerce_send_fn` | classes/               | Internal   | Pointer type internals.                                     |

### FnBrand in foldable/traversable dispatch

The foldable/traversable inference wrappers (e.g., `fold_left`, `traverse`)
infer Brand via InferableBrand but still require FnBrand turbofish
(e.g., `fold_left::<RcFnBrand, _, _, _, _>(...)`). FnBrand controls which
cloneable wrapper is used internally. The caller does not provide a wrapped
function container, so InferableFnBrand cannot help. This is a construction
parameter, not an inference target.

---

## Needs Investigation

### Profunctor family

| Function                                     | File                             | Call sites |
| -------------------------------------------- | -------------------------------- | ---------- |
| `semigroupoid_compose`                       | classes/semigroupoid.rs:142      | 14         |
| `dimap`                                      | classes/profunctor.rs:274        | 10         |
| `lmap`, `rmap`                               | classes/profunctor.rs:317,359    | Low        |
| `first`, `second`, `split_strong`, `fan_out` | classes/profunctor/strong.rs     | Low        |
| `left`, `right`, `split_choice`, `fan_in`    | classes/profunctor/choice.rs     | Low        |
| `unfirst`, `unsecond`                        | classes/profunctor/costrong.rs   | Low        |
| `unleft`, `unright`                          | classes/profunctor/cochoice.rs   | Low        |
| `wander`                                     | classes/profunctor/wander.rs:302 | Low        |

**Open question:** These functions take profunctor values (e.g.,
`Rc<dyn Fn(A) -> B>`) and the Brand is the profunctor brand (`RcFnBrand`,
`ArcFnBrand`). Inference would require InferableBrand impls for
`Rc<dyn Fn(A) -> B>` mapping back to `RcFnBrand`. This is architecturally
different from functor/monad inference and could create coherence issues with
the existing InferableFnBrand trait in the semiapplicative dispatch module.

The `semigroupoid_compose` function has 14 call sites and would benefit most,
but the profunctor inference design needs to be worked out separately.

### Parallel \_with_index variants

| Function                    | File                                 | Call sites |
| --------------------------- | ------------------------------------ | ---------- |
| `par_fold_map_with_index`   | classes/par_foldable_with_index.rs   | Low        |
| `par_map_with_index`        | classes/par_functor_with_index.rs    | Low        |
| `par_filter_map_with_index` | classes/par_filterable_with_index.rs | Low        |

These could be migrated if the parent par*\* functions get dispatch modules, but
individually have low impact. Worth including in any batch migration of par*\*
functions.

---

## Summary

| Category            | Count | Total call sites |
| ------------------- | ----- | ---------------- |
| Worth migrating     | 10    | ~131             |
| Not worth migrating | ~30   | N/A              |
| Needs investigation | ~13   | ~30+             |

The highest-impact migrations are `extend` (32 calls), `extract` (20 calls),
`par_fold_map` (17 calls), and `par_map` (14 calls). These four functions alone
account for 83 turbofish call sites that could become inference-based.

The parallel function family (`par_*`) is a natural batch: all are owned-only
(no Ref dispatch needed) and follow the same pattern. They could use a simpler
wrapper than full dispatch modules since they only need Val mode.
