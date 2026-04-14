# Dispatch Expansion: Design Concerns

Analysis of design concerns for expanding the dispatch system to cover
9 additional function pairs beyond the 6 already implemented (map,
filter_map, bind, fold_right, fold_left, fold_map, traverse, lift2-5,
compose_kleisli).

## The 9 function pairs

For reference, these are the 9 Val/Ref pairs that need new dispatch
traits:

1. `filter` / `ref_filter`
2. `partition` / `ref_partition`
3. `partition_map` / `ref_partition_map`
4. `map_with_index` / `ref_map_with_index`
5. `filter_map_with_index` / `ref_filter_map_with_index`
6. `filter_with_index` / `ref_filter_with_index`
7. `partition_with_index` / `ref_partition_with_index`
8. `partition_map_with_index` / `ref_partition_map_with_index`
9. `wilt` / `ref_wilt` and `wither` / `ref_wither`

Pairs 4-8 involve the `WithIndex` hierarchy. Pair 9 involves effectful
filtering with multiple brand parameters.

Note: `fold_*_with_index` and `traverse_with_index` are also candidates
but are listed separately since they combine WithIndex concerns with the
foldable/traversable dispatch patterns already established.

---

## Concern 1: WithIndex dispatch and the Index type parameter

### Issue

The `*_with_index` functions have an extra `Index` type that comes from
`WithIndex::Index`, an associated type on the brand. How should dispatch
traits handle this? Does the Index type need to appear as an explicit
type parameter on the dispatch trait, or can it be derived from the
brand?

### Research findings

`WithIndex` is a supertrait shared across the entire hierarchy:

```
pub trait WithIndex {
    type Index: Clone;
}
```

Both `FunctorWithIndex` and `RefFunctorWithIndex` require `WithIndex` as
a supertrait, meaning they share the same `Index` associated type for a
given brand. For example:

- Val: `Fn(Self::Index, A) -> B` in `FunctorWithIndex::map_with_index`.
- Ref: `Fn(Self::Index, &A) -> B` in `RefFunctorWithIndex::ref_map_with_index`.

The `Index` type is always accessed as `Brand::Index` (or
`Self::Index`), never as a standalone type parameter. It is the same
type in both Val and Ref paths because both paths operate on the same
brand.

In the existing free functions, `Index` is not a standalone type
parameter. It is projected from the `Brand` bound:

```
pub fn map_with_index<'a, Brand: FunctorWithIndex, A: 'a, B: 'a>(
    f: impl Fn(Brand::Index, A) -> B + 'a,
    fa: Brand::Of<'a, A>,
) -> Brand::Of<'a, B>
```

### How Index interacts with Val/Ref closure distinction

The dispatch distinction is between:

- Val: `Fn(Brand::Index, A) -> B` (owned element).
- Ref: `Fn(Brand::Index, &A) -> B` (borrowed element).

The `Index` appears in the same position in both closures. The dispatch
marker is determined by whether the second argument is `A` or `&A`, not
by `Index`. Since `Index` is always passed by value (it implements
`Clone`), it does not interfere with the Val/Ref distinction.

### Flaws and limitations

None identified. The pattern is clean because `Index` is an associated
type on the brand, not a free type parameter.

### Approach

The dispatch trait should use `Brand::Index` as a projected type, not as
an explicit type parameter. The trait definition would look like:

```
pub trait MapWithIndexDispatch<'a, Brand: Kind_..., A: 'a, B: 'a, FA, Marker>
where
    Brand: WithIndex,
{
    fn dispatch(self, fa: FA) -> Brand::Of<'a, B>;
}
```

The Val impl bounds include `Brand: FunctorWithIndex` and the closure is
`Fn(Brand::Index, A) -> B`. The Ref impl bounds include
`Brand: RefFunctorWithIndex` and the closure is `Fn(Brand::Index, &A) -> B`.

The unified free function uses `Brand::Index` in the closure bound:

```
pub fn map_with_index<'a, Brand: Kind_... + WithIndex, A: 'a, B: 'a, FA, Marker>(
    f: impl MapWithIndexDispatch<'a, Brand, A, B, FA, Marker>,
    fa: FA,
) -> Brand::Of<'a, B>
```

### Recommendation

Pass Index through as a projection of Brand. No new type parameter
needed. This keeps the turbofish identical in shape to the non-indexed
dispatch traits (only `Brand` plus inferred parameters). The approach
is validated by the fact that existing non-dispatch free functions
already use `Brand::Index` in exactly this way.

---

## Concern 2: Wilt/wither dispatch with multiple Brand parameters

### Issue

`wilt` and `wither` have two brand parameters:

- `F` (the Witherable brand, i.e., the container being traversed).
- `M` (the Applicative brand, i.e., the effect context).

Additionally, `ref_wilt` and `ref_wither` have a `FnBrand` parameter
that the by-value versions do not. How should the dispatch trait handle
all of these?

### Research findings

Current free function signatures:

```
// By-value wilt
pub fn wilt<'a, F: Witherable, M: Applicative, A, E, O>(
    func: impl Fn(A) -> M::Of<'a, Result<O, E>>,
    ta: F::Of<'a, A>,
) -> M::Of<'a, (F::Of<'a, E>, F::Of<'a, O>)>

// By-ref wilt
pub fn ref_wilt<'a, Brand: RefWitherable, FnBrand, M: Applicative, A, E, O>(
    func: impl Fn(&A) -> M::Of<'a, Result<O, E>>,
    ta: &Brand::Of<'a, A>,
) -> M::Of<'a, (Brand::Of<'a, E>, Brand::Of<'a, O>)>
```

Key observations:

- The by-value `wilt` takes `F` and `M` (2 brands, no FnBrand).
- The by-ref `ref_wilt` takes `Brand`, `FnBrand`, and `M` (3 brands).
- The `FnBrand` in the Ref path comes from `RefTraversable`, which
  `RefWitherable` depends on. The Val path's `Traversable` does not need
  `FnBrand` because it uses `apply` internally (no `CloneFn` needed).

The existing `traverse` dispatch trait already handles a similar
situation:

```
pub trait TraverseDispatch<'a, FnBrand, Brand, A, B, F, FTA, Marker>
```

Here `FnBrand` is a type parameter on both the Val and Ref impls, but
the Val impl ignores it (the `FnBrand` bound is not used). The Ref impl
passes it through to `ref_traverse`. This "unused in Val, used in Ref"
pattern is already established.

### Turbofish counts

Current turbofish for wilt:

- By-value: `wilt::<OptionBrand, OptionBrand, _, _, _>(...)` = 5
  explicit type args (F, M, A, E, O).
- By-ref: `ref_wilt::<VecBrand, RcFnBrand, OptionBrand, _, _, _>(...)` =
  6 explicit type args (Brand, FnBrand, M, A, E, O).

Proposed unified dispatch:

```
pub fn wilt<'a, FnBrand, Brand, M, A, E, O, FA, Marker>(...)
```

That is 8 type params total. The caller specifies `FnBrand`, `Brand`,
`M`, and infers the rest:
`wilt::<RcFnBrand, VecBrand, _, _, _, OptionBrand, _, _>(...)`.

Wait, the order matters. Following the traverse pattern
(`FnBrand, Brand, A, B, F, FTA, Marker`), the wilt dispatch would be:

```
pub fn wilt<'a, FnBrand, Brand, M, A, E, O, FA, Marker>(...)
```

Turbofish: `wilt::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(...)`
= 8 slots, 3 explicit.

For wither:

```
pub fn wither<'a, FnBrand, Brand, M, A, B, FA, Marker>(...)
```

Turbofish: `wither::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(...)`
= 7 slots, 3 explicit.

### E0119 risk

No additional E0119 risk. The two-impl pattern distinguishes on:

- Val: `Fn(A) -> M::Of<'a, Result<O, E>>` with `FA = Brand::Of<'a, A>`.
- Ref: `Fn(&A) -> M::Of<'a, Result<O, E>>` with `FA = &Brand::Of<'a, A>`.

The `M` brand is the same in both impls and does not affect
distinguishability. The Val/Ref distinction is still driven by the
closure argument type (`A` vs `&A`) and the container type (owned vs
borrowed), exactly as in all other dispatch traits.

### Recommendation

Follow the traverse dispatch pattern exactly:

- Include `FnBrand` as a type parameter (unused in Val, passed through
  in Ref).
- Include `M` (the applicative brand) as an explicit type parameter.
- Include `FA` and `Marker` as inferred parameters.
- Val impl bounds: `Brand: Witherable`.
- Ref impl bounds: `Brand: RefWitherable`, `FnBrand: LiftFn`.

The turbofish is longer than the non-dispatch versions, but the pattern
is already established by traverse. Users specify 3 brands
(`FnBrand, Brand, M`) and infer everything else.

---

## Concern 3: Partition return types

### Issue

`partition` and `partition_map` return tuples `(FA, FA)` or `(FE, FO)`.
How does this interact with dispatch? The Val and Ref paths both produce
new owned containers as output, so the return type is the same
regardless of dispatch path.

### Research findings

By-value signatures:

```
fn partition_map<'a, A, E, O>(
    func: impl Fn(A) -> Result<O, E>,
    fa: Self::Of<'a, A>,
) -> (Self::Of<'a, E>, Self::Of<'a, O>)

fn partition<'a, A: Clone>(
    func: impl Fn(A) -> bool,
    fa: Self::Of<'a, A>,
) -> (Self::Of<'a, A>, Self::Of<'a, A>)
```

By-ref signatures:

```
fn ref_partition_map<'a, A, E, O>(
    func: impl Fn(&A) -> Result<O, E>,
    fa: &Self::Of<'a, A>,
) -> (Self::Of<'a, E>, Self::Of<'a, O>)

fn ref_partition<'a, A: Clone>(
    func: impl Fn(&A) -> bool,
    fa: &Self::Of<'a, A>,
) -> (Self::Of<'a, A>, Self::Of<'a, A>)
```

Key observation: the return types are identical between Val and Ref.
Both produce owned containers. The only differences are:

- The closure argument type (`A` vs `&A`).
- The container argument type (owned vs borrowed).

This is exactly the same pattern as `filter_map`, which also produces
an owned container regardless of dispatch path.

### Flaws and limitations

None. The tuple return type does not complicate dispatch. The dispatch
trait's return type is just `(Brand::Of<'a, E>, Brand::Of<'a, O>)` or
`(Brand::Of<'a, A>, Brand::Of<'a, A>)`, which is the same for both
impls.

### Recommendation

Handle partition/partition_map identically to filter_map dispatch. The
dispatch trait has the same shape; only the return type changes from
`Brand::Of<'a, B>` to a tuple. No special handling required.

---

## Concern 4: E0119 risk with the two-impl pattern

### Issue

The existing dispatch system uses two impls (Val + Ref) with an `FA`
parameter. A three-impl pattern causes E0119. Do all 9 new dispatch
traits maintain the Val/Ref distinguishability that prevents E0119?

### Research findings

The two-impl pattern avoids E0119 because:

- Val impl has `FA = Brand::Of<'a, A>` (a concrete owned type).
- Ref impl has `FA = &'b Brand::Of<'a, A>` (a reference to that type).
- The compiler can always distinguish these because a reference type is
  structurally different from a non-reference type.

The closure types provide a second axis of distinction:

- Val: `Fn(A) -> R` (or `Fn(Index, A) -> R`, `Fn(A, B) -> B`, etc.).
- Ref: `Fn(&A) -> R` (or `Fn(Index, &A) -> R`, `Fn(&A, B) -> B`, etc.).

For each of the 9 pairs, here are the distinguishing closure types:

| Pair                     | Val closure                    | Ref closure                     | Distinguishable? |
| ------------------------ | ------------------------------ | ------------------------------- | ---------------- |
| filter                   | `Fn(A) -> bool`                | `Fn(&A) -> bool`                | Yes              |
| partition                | `Fn(A) -> bool`                | `Fn(&A) -> bool`                | Yes              |
| partition_map            | `Fn(A) -> Result<O, E>`        | `Fn(&A) -> Result<O, E>`        | Yes              |
| map_with_index           | `Fn(Idx, A) -> B`              | `Fn(Idx, &A) -> B`              | Yes              |
| filter_map_with_index    | `Fn(Idx, A) -> Option<B>`      | `Fn(Idx, &A) -> Option<B>`      | Yes              |
| filter_with_index        | `Fn(Idx, A) -> bool`           | `Fn(Idx, &A) -> bool`           | Yes              |
| partition_with_index     | `Fn(Idx, A) -> bool`           | `Fn(Idx, &A) -> bool`           | Yes              |
| partition_map_with_index | `Fn(Idx, A) -> Result<O, E>`   | `Fn(Idx, &A) -> Result<O, E>`   | Yes              |
| wilt                     | `Fn(A) -> M::Of<Result<O, E>>` | `Fn(&A) -> M::Of<Result<O, E>>` | Yes              |
| wither                   | `Fn(A) -> M::Of<Option<B>>`    | `Fn(&A) -> M::Of<Option<B>>`    | Yes              |

In every case, the Val closure takes `A` (owned) where the Ref closure
takes `&A` (borrowed) in the element position. The `Index` parameter
(for WithIndex variants) does not affect distinguishability because it
appears in the same position in both closures.

The `FA` parameter provides a second, redundant axis of distinction:
`Brand::Of<'a, A>` (owned) vs `&Brand::Of<'a, A>` (borrowed). Even
if the compiler could not distinguish the closure types alone, the `FA`
type parameter would resolve the ambiguity.

### Recommendation

No E0119 risk for any of the 9 pairs. The two-impl pattern is safe.
This is a direct consequence of the structural distinction between
`A` and `&A` in the closure signature, combined with the owned-vs-
borrowed container type in `FA`. The same reasoning that validates the
existing 6 dispatch traits applies uniformly to all 9 new ones.

---

## Concern 5: Interaction with existing non-dispatch free functions

### Issue

When a dispatch function replaces two separate free functions, the old
functions need to either be removed, renamed, or kept as aliases. How
was this handled for existing dispatch traits?

### Research findings

Looking at `functions.rs`, the re-export mechanism works as follows:

1. The macro `generate_function_re_exports!` scans `src/classes/` and
   auto-generates `pub use` statements for all `pub fn` items found in
   those modules.

2. When a dispatch version exists in `src/classes/dispatch/`, the
   dispatch version is manually re-exported via explicit `pub use`
   statements in the second block of `functions.rs`.

3. For name conflicts, the macro accepts an alias map. For example:
   - `"filterable::filter_map": filterable_filter_map` -- the
     non-dispatch `filter_map` from `filterable.rs` is re-exported as
     `filterable_filter_map`.
   - `"traversable::traverse": traversable_traverse` -- the
     non-dispatch `traverse` is re-exported as `traversable_traverse`.

4. The dispatch version takes the canonical name (`filter_map`,
   `traverse`, `map`, etc.) and the non-dispatch version gets an
   aliased, less prominent name.

The non-dispatch free functions remain available (not deleted) for two
reasons:

- They are used in doc examples on the trait definitions themselves.
- They serve as the underlying implementation that the dispatch trait
  calls into.

### Approach for new dispatch traits

For each new dispatch function:

1. Add the dispatch module in `src/classes/dispatch/`.
2. Add the canonical name (`filter`, `partition`, etc.) to the manual
   `pub use` block in `functions.rs`.
3. Add the original non-dispatch function to the alias map in
   `generate_function_re_exports!` (e.g.,
   `"filterable::filter": filterable_filter`).
4. Add the `ref_*` function to the alias map as well (e.g.,
   `"ref_filterable::ref_filter": ref_filterable_ref_filter`), or
   simply remove it from the auto-generated exports since the dispatch
   function subsumes it.

### Flaws and limitations

The alias mechanism creates "orphan" re-exports (like
`filterable_filter_map`) that exist in the public API but are not the
preferred way to call the function. This is a minor API surface concern,
not a correctness issue. If desired, these aliases could be
`#[doc(hidden)]` or deprecated in a future pass.

### Recommendation

Follow the established pattern: keep non-dispatch free functions in
their original modules (they are needed for trait doc examples and as
the dispatch target), alias them in the re-export macro, and give the
dispatch version the canonical name.

---

## Concern 6: Turbofish count changes

### Methodology

For each function pair, the turbofish count is the total number of type
parameters on the free function. "Explicit" means the caller must write
a concrete type; "inferred" means the caller writes `_`.

The dispatch pattern always adds 2 inferred parameters (`FA` and
`Marker`) compared to the non-dispatch version, but removes the need
for separate `ref_*` functions.

### filter / ref_filter

Current Val (`filter`):
`filter::<'a, Brand, A>` = 2 explicit (Brand, A is inferred from
closure) -> turbofish `filter::<OptionBrand, _>` = 2 slots.

Current Ref (`ref_filter`):
`ref_filter::<'a, Brand, A>` = 2 explicit -> turbofish
`ref_filter::<VecBrand, _>` = 2 slots.

Proposed dispatch:
`filter::<'a, Brand, A, FA, Marker>` = 4 slots.
Turbofish: `filter::<OptionBrand, _, _, _>` = 4 slots (1 explicit).

Change: 2 -> 4 slots (+2 inferred).

### partition / ref_partition

Current Val: `partition::<'a, Brand, A>` = 2 slots.
Current Ref: `ref_partition::<'a, Brand, A>` = 2 slots.

Proposed dispatch: `partition::<'a, Brand, A, FA, Marker>` = 4 slots.
Turbofish: `partition::<OptionBrand, _, _, _>`.

Change: 2 -> 4 slots (+2 inferred).

### partition_map / ref_partition_map

Current Val: `partition_map::<'a, Brand, A, E, O>` = 4 slots.
Turbofish: `partition_map::<OptionBrand, _, _, _>`.

Current Ref: `ref_partition_map::<'a, Brand, A, E, O>` = 4 slots.
Turbofish: `ref_partition_map::<VecBrand, _, _, _>`.

Proposed dispatch: `partition_map::<'a, Brand, A, E, O, FA, Marker>` =
6 slots.
Turbofish: `partition_map::<OptionBrand, _, _, _, _, _>`.

Change: 4 -> 6 slots (+2 inferred).

### map_with_index / ref_map_with_index

Current Val: `map_with_index::<'a, Brand, A, B>` = 3 slots.
Turbofish: `map_with_index::<VecBrand, _, _>`.

Current Ref: `ref_map_with_index::<'a, Brand, A, B>` = 3 slots.
Turbofish: `ref_map_with_index::<LazyBrand<RcLazyConfig>, _, _>`.

Proposed dispatch:
`map_with_index::<'a, Brand, A, B, FA, Marker>` = 5 slots.
Turbofish: `map_with_index::<VecBrand, _, _, _, _>`.

Note: `Index` is not a type parameter; it is projected from `Brand`.

Change: 3 -> 5 slots (+2 inferred).

### filter_map_with_index / ref_filter_map_with_index

Current Val: `filter_map_with_index::<'a, Brand, A, B>` = 3 slots.
Turbofish: `filter_map_with_index::<VecBrand, _, _>`.

Current Ref: `ref_filter_map_with_index::<'a, Brand, A, B>` = 3 slots.

Proposed dispatch:
`filter_map_with_index::<'a, Brand, A, B, FA, Marker>` = 5 slots.
Turbofish: `filter_map_with_index::<VecBrand, _, _, _, _>`.

Change: 3 -> 5 slots (+2 inferred).

### filter_with_index / ref_filter_with_index

Current Val: `filter_with_index::<'a, Brand, A>` = 2 slots.
Turbofish: `filter_with_index::<VecBrand, _>`.

Current Ref: `ref_filter_with_index::<'a, Brand, A>` = 2 slots.

Proposed dispatch:
`filter_with_index::<'a, Brand, A, FA, Marker>` = 4 slots.
Turbofish: `filter_with_index::<VecBrand, _, _, _>`.

Change: 2 -> 4 slots (+2 inferred).

### partition_with_index / ref_partition_with_index

Current Val: `partition_with_index::<'a, Brand, A>` = 2 slots.
Current Ref: `ref_partition_with_index::<'a, Brand, A>` = 2 slots.

Proposed dispatch: `partition_with_index::<'a, Brand, A, FA, Marker>` =
4 slots.
Turbofish: `partition_with_index::<VecBrand, _, _, _>`.

Change: 2 -> 4 slots (+2 inferred).

### partition_map_with_index / ref_partition_map_with_index

Current Val: `partition_map_with_index::<'a, Brand, A, E, O>` = 4 slots.
Turbofish: `partition_map_with_index::<VecBrand, _, _, _>`.

Current Ref: `ref_partition_map_with_index::<'a, Brand, A, E, O>` =
4 slots.

Proposed dispatch:
`partition_map_with_index::<'a, Brand, A, E, O, FA, Marker>` = 6 slots.
Turbofish: `partition_map_with_index::<VecBrand, _, _, _, _, _>`.

Change: 4 -> 6 slots (+2 inferred).

### wilt / ref_wilt

Current Val: `wilt::<'a, F, M, A, E, O>` = 5 slots.
Turbofish: `wilt::<OptionBrand, OptionBrand, _, _, _>`.

Current Ref: `ref_wilt::<'a, Brand, FnBrand, M, A, E, O>` = 6 slots.
Turbofish: `ref_wilt::<VecBrand, RcFnBrand, OptionBrand, _, _, _>`.

Proposed dispatch:
`wilt::<'a, FnBrand, Brand, M, A, E, O, FA, Marker>` = 8 slots.
Turbofish: `wilt::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>`.

Change: Val 5 -> 8 (+3: FnBrand, FA, Marker); Ref 6 -> 8 (+2: FA,
Marker).

### wither / ref_wither

Current Val: `wither::<'a, F, M, A, B>` = 4 slots.
Turbofish: `wither::<OptionBrand, OptionBrand, _, _>`.

Current Ref: `ref_wither::<'a, Brand, FnBrand, M, A, B>` = 5 slots.
Turbofish: `ref_wither::<VecBrand, RcFnBrand, OptionBrand, _, _>`.

Proposed dispatch:
`wither::<'a, FnBrand, Brand, M, A, B, FA, Marker>` = 7 slots.
Turbofish: `wither::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>`.

Change: Val 4 -> 7 (+3: FnBrand, FA, Marker); Ref 5 -> 7 (+2: FA,
Marker).

### Summary table

| Function                 | Current Val | Current Ref | Dispatch | Delta (Val) |
| ------------------------ | ----------- | ----------- | -------- | ----------- |
| filter                   | 2           | 2           | 4        | +2          |
| partition                | 2           | 2           | 4        | +2          |
| partition_map            | 4           | 4           | 6        | +2          |
| map_with_index           | 3           | 3           | 5        | +2          |
| filter_map_with_index    | 3           | 3           | 5        | +2          |
| filter_with_index        | 2           | 2           | 4        | +2          |
| partition_with_index     | 2           | 2           | 4        | +2          |
| partition_map_with_index | 4           | 4           | 6        | +2          |
| wilt                     | 5           | 6           | 8        | +3          |
| wither                   | 4           | 5           | 7        | +3          |

The consistent +2 delta (FA + Marker) matches the existing dispatch
traits. Wilt and wither have +3 because the Val path gains `FnBrand`
(unused but required for uniformity with the Ref path), matching the
traverse dispatch pattern.

---

## Concern 7: fold_with_index and traverse_with_index dispatch

These are additional candidates beyond the 9 pairs listed above.

### fold\_\*\_with_index

The foldable-with-index family has three functions:

- `fold_map_with_index` / `ref_fold_map_with_index`
- `fold_right_with_index` / `ref_fold_right_with_index`
- `fold_left_with_index` / `ref_fold_left_with_index`

These follow the same pattern as the existing `fold_map`, `fold_right`,
`fold_left` dispatch traits, with the addition of `Index` as a projected
type from the brand. The closure types are:

| Function              | Val closure          | Ref closure           |
| --------------------- | -------------------- | --------------------- |
| fold_map_with_index   | `Fn(Idx, A) -> R`    | `Fn(Idx, &A) -> R`    |
| fold_right_with_index | `Fn(Idx, A, B) -> B` | `Fn(Idx, &A, B) -> B` |
| fold_left_with_index  | `Fn(Idx, B, A) -> B` | `Fn(B, Idx, &A) -> B` |

Note: `fold_left_with_index` has a different argument order between Val
and Ref. The Val version is `Fn(Self::Index, B, A) -> B` while the Ref
version is `Fn(B, Self::Index, &A) -> B`. This is a minor concern for
dispatch since the compiler distinguishes on the `A` vs `&A` position
regardless of where `Index` and `B` appear.

### traverse_with_index

Similar to traverse dispatch, with `Index` added:

- Val: `Fn(Brand::Index, A) -> M::Of<'a, B>`.
- Ref: `Fn(Brand::Index, &A) -> M::Of<'a, B>`.

No special concerns; follows the same pattern as traverse dispatch
plus WithIndex projection.

---

## Overall recommendations

1. **Index types:** Always project from `Brand::Index`. Never add
   `Index` as a standalone type parameter on dispatch traits.

2. **Multiple brands (wilt/wither):** Follow the traverse dispatch
   pattern. Include `FnBrand` as a type parameter (unused in Val,
   used in Ref). Include `M` as an explicit type parameter.

3. **Tuple returns (partition):** No special handling needed. The
   dispatch trait return type is a tuple; both impls return the same
   owned tuple type.

4. **E0119:** No risk for any of the 9 pairs. The two-impl pattern
   with `FA` (owned vs borrowed) and closure argument type (`A` vs
   `&A`) provides sufficient distinction in all cases.

5. **Re-exports:** Follow the established alias pattern in
   `functions.rs`. The dispatch version gets the canonical name; the
   non-dispatch versions get aliased names.

6. **Turbofish:** Every dispatch function gains exactly 2 inferred
   parameters (`FA`, `Marker`) compared to its Val counterpart, except
   wilt/wither which gain 3 (adding `FnBrand`). This is consistent
   with the existing dispatch traits and is the expected cost of
   unifying Val and Ref behind a single function.

7. **Implementation order:** Start with the simpler pairs (filter,
   partition, partition_map) that follow the filter_map dispatch pattern
   exactly. Then tackle the WithIndex variants, which add only the
   projected `Index` type. Finally, implement wilt/wither, which are
   the most complex due to multiple brand parameters.
