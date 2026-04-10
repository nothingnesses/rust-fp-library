# Dispatch Expansion Signature Survey

This document surveys 9 function pairs that have both Val and Ref versions with
closure arguments suitable for dispatch, but currently lack dispatch traits. For
each pair, it records the exact signatures, identifies the dispatch-driving
closure parameter, and proposes a dispatch trait design.

## Reference: Existing Dispatch Pattern

The dispatch system uses two marker types (`Val` and `Ref`) and a dispatch trait
with two blanket impls. The pattern is:

1. Define a trait `FooDispatch<'a, Brand, ..., FA, Marker>` with a `dispatch_foo`
   method.
2. Impl for `F where F: Fn(A) -> R` with `FA = Brand::Of<'a, A>` and
   `Marker = Val`, routing to `Brand::foo(self, fa)`.
3. Impl for `F where F: Fn(&A) -> R` with `FA = &'b Brand::Of<'a, A>` and
   `Marker = Ref`, routing to `Brand::ref_foo(self, fa)`.
4. Define a unified free function `foo<..., FA, Marker>(f: impl FooDispatch<...>,
fa: FA) -> ...` that calls `f.dispatch_foo(fa)`.

The turbofish count for the unified function equals the number of type parameters
that cannot be inferred, excluding `FA` and `Marker` (which are always inferred).

Existing examples:

- `map`: `map::<Brand, _, _, _, _>` -- 1 explicit (Brand), 4 inferred (A, B, FA, Marker).
- `filter_map`: `filter_map::<Brand, _, _, _, _>` -- 1 explicit, 4 inferred.
- `fold_right`: `fold_right::<FnBrand, Brand, _, _, _, _>` -- 2 explicit, 4 inferred.
- `traverse`: `traverse::<FnBrand, Brand, _, _, F, _, _>` -- 3 explicit, 4 inferred.

---

## 1. filter / ref_filter

### Val signature (filterable.rs)

```rust
pub fn filter<'a, Brand: Filterable, A: 'a + Clone>(
    func: impl Fn(A) -> bool + 'a,
    fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
```

### Ref signature (ref_filterable.rs)

```rust
pub fn ref_filter<'a, Brand: RefFilterable, A: 'a + Clone>(
    func: impl Fn(&A) -> bool + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(A) -> bool`.
- Ref type: `Fn(&A) -> bool`.

### Other type parameters beyond Brand

- `A` (element type, `Clone` bound in both variants).

### Proposed dispatch trait

```
FilterDispatch<'a, Brand, A, FA, Marker>
```

Unified free function:

```rust
pub fn filter<'a, Brand: Kind_..., A: 'a + Clone, FA, Marker>(
    func: impl FilterDispatch<'a, Brand, A, FA, Marker>,
    fa: FA,
) -> Apply!(<Brand as Kind!(...)>::Of<'a, A>)
```

### Turbofish count

- Current Val: `filter::<Brand, _>` -- 1 explicit (Brand), 1 inferred (A). Total: 2 positions.
- Current Ref: `ref_filter::<Brand, _>` -- 1 explicit (Brand), 1 inferred (A). Total: 2 positions.
- Dispatch: `filter::<Brand, _, _, _>` -- 1 explicit (Brand), 3 inferred (A, FA, Marker). Total: 4 positions.

---

## 2. partition / ref_partition

### Val signature (filterable.rs)

```rust
pub fn partition<'a, Brand: Filterable, A: 'a + Clone>(
    func: impl Fn(A) -> bool + 'a,
    fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> (
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
)
```

### Ref signature (ref_filterable.rs)

```rust
pub fn ref_partition<'a, Brand: RefFilterable, A: 'a + Clone>(
    func: impl Fn(&A) -> bool + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> (
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(A) -> bool`.
- Ref type: `Fn(&A) -> bool`.

### Other type parameters beyond Brand

- `A` (element type, `Clone` bound in both variants).

### Proposed dispatch trait

```
PartitionDispatch<'a, Brand, A, FA, Marker>
```

Unified free function:

```rust
pub fn partition<'a, Brand: Kind_..., A: 'a + Clone, FA, Marker>(
    func: impl PartitionDispatch<'a, Brand, A, FA, Marker>,
    fa: FA,
) -> (
    Apply!(<Brand as Kind!(...)>::Of<'a, A>),
    Apply!(<Brand as Kind!(...)>::Of<'a, A>),
)
```

### Turbofish count

- Current Val: `partition::<Brand, _>` -- 1 explicit, 1 inferred. Total: 2.
- Current Ref: `ref_partition::<Brand, _>` -- 1 explicit, 1 inferred. Total: 2.
- Dispatch: `partition::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.

Note: `partition` and `filter` have identical closure signatures (`Fn(A) -> bool`
vs `Fn(&A) -> bool`), so they share the same dispatch mechanism shape. The only
difference is the return type (tuple vs single).

---

## 3. partition_map / ref_partition_map

### Val signature (filterable.rs)

```rust
pub fn partition_map<'a, Brand: Filterable, A: 'a, E: 'a, O: 'a>(
    func: impl Fn(A) -> Result<O, E> + 'a,
    fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> (
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
)
```

### Ref signature (ref_filterable.rs)

```rust
pub fn ref_partition_map<'a, Brand: RefFilterable, A: 'a, E: 'a, O: 'a>(
    func: impl Fn(&A) -> Result<O, E> + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> (
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(A) -> Result<O, E>`.
- Ref type: `Fn(&A) -> Result<O, E>`.

### Other type parameters beyond Brand

- `A` (input element type).
- `E` (error/left type).
- `O` (ok/right type).

### Proposed dispatch trait

```
PartitionMapDispatch<'a, Brand, A, E, O, FA, Marker>
```

Unified free function:

```rust
pub fn partition_map<'a, Brand: Kind_..., A: 'a, E: 'a, O: 'a, FA, Marker>(
    func: impl PartitionMapDispatch<'a, Brand, A, E, O, FA, Marker>,
    fa: FA,
) -> (
    Apply!(<Brand as Kind!(...)>::Of<'a, E>),
    Apply!(<Brand as Kind!(...)>::Of<'a, O>),
)
```

### Turbofish count

- Current Val: `partition_map::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.
- Current Ref: `ref_partition_map::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.
- Dispatch: `partition_map::<Brand, _, _, _, _, _>` -- 1 explicit, 5 inferred. Total: 6.

---

## 4. wilt / ref_wilt

### Val signature (witherable.rs)

```rust
pub fn wilt<'a, F: Witherable, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
    func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
    ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
    'a,
    (
        Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
        Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
    ),
>)
where
    Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
    Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
```

### Ref signature (ref_witherable.rs)

```rust
pub fn ref_wilt<
    'a,
    Brand: RefWitherable,
    FnBrand,
    M: Applicative,
    A: 'a + Clone,
    E: 'a + Clone,
    O: 'a + Clone,
>(
    func: impl Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
    ta: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
    'a,
    (
        Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
        Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
    ),
>)
where
    FnBrand: LiftFn + 'a,
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
    Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(A) -> M::Of<'a, Result<O, E>>`.
- Ref type: `Fn(&A) -> M::Of<'a, Result<O, E>>`.

### Other type parameters beyond Brand

- `FnBrand` (cloneable function brand, only used by Ref path but must appear for uniformity, same pattern as `TraverseDispatch`).
- `M` (applicative context brand).
- `A` (input element type).
- `E` (error type).
- `O` (ok/success type).

### Proposed dispatch trait

```
WiltDispatch<'a, FnBrand, Brand, A, E, O, M, FTA, Marker>
```

Unified free function:

```rust
pub fn wilt<
    'a,
    FnBrand,
    Brand: Kind_...,
    A: 'a + Clone,
    E: 'a + Clone,
    O: 'a + Clone,
    M: Kind_...,
    FTA,
    Marker,
>(
    func: impl WiltDispatch<'a, FnBrand, Brand, A, E, O, M, FTA, Marker>,
    ta: FTA,
) -> Apply!(<M as Kind!(...)>::Of<'a, (Brand::Of<'a, E>, Brand::Of<'a, O>)>)
```

### Turbofish count

- Current Val: `wilt::<Brand, M, _, _, _>` -- 2 explicit (F/Brand, M), 3 inferred (A, E, O). Total: 5.
- Current Ref: `ref_wilt::<Brand, FnBrand, M, _, _, _>` -- 3 explicit, 3 inferred. Total: 6.
- Dispatch: `wilt::<FnBrand, Brand, _, _, _, M, _, _>` -- 3 explicit (FnBrand, Brand, M), 5 inferred (A, E, O, FTA, Marker). Total: 8.

Note: The Val path does not use `FnBrand` (same as `TraverseDispatch`), but the
parameter must appear for uniformity. This means the Val path gains one extra
turbofish position compared to the current `wilt`.

---

## 5. wither / ref_wither

### Val signature (witherable.rs)

```rust
pub fn wither<'a, F: Witherable, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
    func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
    ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
    'a,
    Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
>)
where
    Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
    Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
```

### Ref signature (ref_witherable.rs)

```rust
pub fn ref_wither<
    'a,
    Brand: RefWitherable,
    FnBrand,
    M: Applicative,
    A: 'a + Clone,
    B: 'a + Clone,
>(
    func: impl Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
    ta: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
    'a,
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
>)
where
    FnBrand: LiftFn + 'a,
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
    Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(A) -> M::Of<'a, Option<B>>`.
- Ref type: `Fn(&A) -> M::Of<'a, Option<B>>`.

### Other type parameters beyond Brand

- `FnBrand` (cloneable function brand, only used by Ref path).
- `M` (applicative context brand).
- `A` (input element type).
- `B` (output element type).

### Proposed dispatch trait

```
WitherDispatch<'a, FnBrand, Brand, A, B, M, FTA, Marker>
```

Unified free function:

```rust
pub fn wither<
    'a,
    FnBrand,
    Brand: Kind_...,
    A: 'a + Clone,
    B: 'a + Clone,
    M: Kind_...,
    FTA,
    Marker,
>(
    func: impl WitherDispatch<'a, FnBrand, Brand, A, B, M, FTA, Marker>,
    ta: FTA,
) -> Apply!(<M as Kind!(...)>::Of<'a, Brand::Of<'a, B>>)
```

### Turbofish count

- Current Val: `wither::<Brand, M, _, _>` -- 2 explicit, 2 inferred. Total: 4.
- Current Ref: `ref_wither::<Brand, FnBrand, M, _, _>` -- 3 explicit, 2 inferred. Total: 5.
- Dispatch: `wither::<FnBrand, Brand, _, _, M, _, _>` -- 3 explicit (FnBrand, Brand, M), 4 inferred (A, B, FTA, Marker). Total: 7.

---

## 6. filter_with_index / ref_filter_with_index

### Val signature (filterable_with_index.rs)

```rust
pub fn filter_with_index<'a, Brand: FilterableWithIndex, A: 'a + Clone>(
    func: impl Fn(Brand::Index, A) -> bool + 'a,
    fa: Brand::Of<'a, A>,
) -> Brand::Of<'a, A>
```

### Ref signature (ref_filterable_with_index.rs)

```rust
pub fn ref_filter_with_index<'a, Brand: RefFilterableWithIndex, A: 'a + Clone>(
    func: impl Fn(Brand::Index, &A) -> bool + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(Brand::Index, A) -> bool`.
- Ref type: `Fn(Brand::Index, &A) -> bool`.

The dispatch is driven by the second argument of the closure (`A` vs `&A`). The
first argument (`Brand::Index`) is always by-value since the index is a
`Clone`-bounded associated type.

### Other type parameters beyond Brand

- `A` (element type).

### Proposed dispatch trait

```
FilterWithIndexDispatch<'a, Brand, A, FA, Marker>
```

Unified free function:

```rust
pub fn filter_with_index<'a, Brand: Kind_... + WithIndex, A: 'a + Clone, FA, Marker>(
    func: impl FilterWithIndexDispatch<'a, Brand, A, FA, Marker>,
    fa: FA,
) -> Apply!(<Brand as Kind!(...)>::Of<'a, A>)
```

### Turbofish count

- Current Val: `filter_with_index::<Brand, _>` -- 1 explicit, 1 inferred. Total: 2.
- Current Ref: `ref_filter_with_index::<Brand, _>` -- 1 explicit, 1 inferred. Total: 2.
- Dispatch: `filter_with_index::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.

### Design note

The closure takes two arguments: `(Brand::Index, A)` vs `(Brand::Index, &A)`.
The dispatch mechanism distinguishes by the second argument's type. The Val impl
bounds the closure as `Fn(Brand::Index, A) -> bool`, the Ref impl bounds it as
`Fn(Brand::Index, &A) -> bool`. The compiler resolves the marker from the second
closure argument. This is the same pattern used in `FoldRightDispatch` and
`FoldLeftDispatch`, where the fold function has two arguments and only the
element argument drives dispatch.

---

## 7. filter_map_with_index / ref_filter_map_with_index

### Val signature (filterable_with_index.rs)

```rust
pub fn filter_map_with_index<'a, Brand: FilterableWithIndex, A: 'a, B: 'a>(
    func: impl Fn(Brand::Index, A) -> Option<B> + 'a,
    fa: Brand::Of<'a, A>,
) -> Brand::Of<'a, B>
```

### Ref signature (ref_filterable_with_index.rs)

```rust
pub fn ref_filter_map_with_index<'a, Brand: RefFilterableWithIndex, A: 'a, B: 'a>(
    func: impl Fn(Brand::Index, &A) -> Option<B> + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(Brand::Index, A) -> Option<B>`.
- Ref type: `Fn(Brand::Index, &A) -> Option<B>`.

### Other type parameters beyond Brand

- `A` (input element type).
- `B` (output element type).

### Proposed dispatch trait

```
FilterMapWithIndexDispatch<'a, Brand, A, B, FA, Marker>
```

Unified free function:

```rust
pub fn filter_map_with_index<'a, Brand: Kind_... + WithIndex, A: 'a, B: 'a, FA, Marker>(
    func: impl FilterMapWithIndexDispatch<'a, Brand, A, B, FA, Marker>,
    fa: FA,
) -> Apply!(<Brand as Kind!(...)>::Of<'a, B>)
```

### Turbofish count

- Current Val: `filter_map_with_index::<Brand, _, _>` -- 1 explicit, 2 inferred. Total: 3.
- Current Ref: `ref_filter_map_with_index::<Brand, _, _>` -- 1 explicit, 2 inferred. Total: 3.
- Dispatch: `filter_map_with_index::<Brand, _, _, _, _>` -- 1 explicit, 4 inferred. Total: 5.

---

## 8. partition_with_index / ref_partition_with_index

### Val signature (filterable_with_index.rs)

```rust
pub fn partition_with_index<'a, Brand: FilterableWithIndex, A: 'a + Clone>(
    func: impl Fn(Brand::Index, A) -> bool + 'a,
    fa: Brand::Of<'a, A>,
) -> (Brand::Of<'a, A>, Brand::Of<'a, A>)
```

### Ref signature (ref_filterable_with_index.rs)

```rust
pub fn ref_partition_with_index<'a, Brand: RefFilterableWithIndex, A: 'a + Clone>(
    func: impl Fn(Brand::Index, &A) -> bool + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> (
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(Brand::Index, A) -> bool`.
- Ref type: `Fn(Brand::Index, &A) -> bool`.

### Other type parameters beyond Brand

- `A` (element type).

### Proposed dispatch trait

```
PartitionWithIndexDispatch<'a, Brand, A, FA, Marker>
```

Unified free function:

```rust
pub fn partition_with_index<'a, Brand: Kind_... + WithIndex, A: 'a + Clone, FA, Marker>(
    func: impl PartitionWithIndexDispatch<'a, Brand, A, FA, Marker>,
    fa: FA,
) -> (
    Apply!(<Brand as Kind!(...)>::Of<'a, A>),
    Apply!(<Brand as Kind!(...)>::Of<'a, A>),
)
```

### Turbofish count

- Current Val: `partition_with_index::<Brand, _>` -- 1 explicit, 1 inferred. Total: 2.
- Current Ref: `ref_partition_with_index::<Brand, _>` -- 1 explicit, 1 inferred. Total: 2.
- Dispatch: `partition_with_index::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.

---

## 9. partition_map_with_index / ref_partition_map_with_index

### Val signature (filterable_with_index.rs)

```rust
pub fn partition_map_with_index<'a, Brand: FilterableWithIndex, A: 'a, E: 'a, O: 'a>(
    func: impl Fn(Brand::Index, A) -> Result<O, E> + 'a,
    fa: Brand::Of<'a, A>,
) -> (Brand::Of<'a, E>, Brand::Of<'a, O>)
```

### Ref signature (ref_filterable_with_index.rs)

```rust
pub fn ref_partition_map_with_index<'a, Brand: RefFilterableWithIndex, A: 'a, E: 'a, O: 'a>(
    func: impl Fn(Brand::Index, &A) -> Result<O, E> + 'a,
    fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> (
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
    Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
)
```

### Dispatch-driving closure

- Parameter: `func` (1st argument).
- Val type: `Fn(Brand::Index, A) -> Result<O, E>`.
- Ref type: `Fn(Brand::Index, &A) -> Result<O, E>`.

### Other type parameters beyond Brand

- `A` (input element type).
- `E` (error type).
- `O` (ok type).

### Proposed dispatch trait

```
PartitionMapWithIndexDispatch<'a, Brand, A, E, O, FA, Marker>
```

Unified free function:

```rust
pub fn partition_map_with_index<
    'a, Brand: Kind_... + WithIndex, A: 'a, E: 'a, O: 'a, FA, Marker
>(
    func: impl PartitionMapWithIndexDispatch<'a, Brand, A, E, O, FA, Marker>,
    fa: FA,
) -> (
    Apply!(<Brand as Kind!(...)>::Of<'a, E>),
    Apply!(<Brand as Kind!(...)>::Of<'a, O>),
)
```

### Turbofish count

- Current Val: `partition_map_with_index::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.
- Current Ref: `ref_partition_map_with_index::<Brand, _, _, _>` -- 1 explicit, 3 inferred. Total: 4.
- Dispatch: `partition_map_with_index::<Brand, _, _, _, _, _>` -- 1 explicit, 5 inferred. Total: 6.

---

## Summary Table

| #   | Function pair                                           | Val trait           | Ref trait              | Closure Val type            | Closure Ref type             | Extra type params   | Dispatch trait name           | Val turbofish | Ref turbofish | Unified turbofish |
| --- | ------------------------------------------------------- | ------------------- | ---------------------- | --------------------------- | ---------------------------- | ------------------- | ----------------------------- | ------------- | ------------- | ----------------- |
| 1   | filter / ref_filter                                     | Filterable          | RefFilterable          | `Fn(A) -> bool`             | `Fn(&A) -> bool`             | A                   | FilterDispatch                | 2             | 2             | 4                 |
| 2   | partition / ref_partition                               | Filterable          | RefFilterable          | `Fn(A) -> bool`             | `Fn(&A) -> bool`             | A                   | PartitionDispatch             | 2             | 2             | 4                 |
| 3   | partition_map / ref_partition_map                       | Filterable          | RefFilterable          | `Fn(A) -> Result<O,E>`      | `Fn(&A) -> Result<O,E>`      | A, E, O             | PartitionMapDispatch          | 4             | 4             | 6                 |
| 4   | wilt / ref_wilt                                         | Witherable          | RefWitherable          | `Fn(A) -> M<Result<O,E>>`   | `Fn(&A) -> M<Result<O,E>>`   | FnBrand, M, A, E, O | WiltDispatch                  | 5             | 6             | 8                 |
| 5   | wither / ref_wither                                     | Witherable          | RefWitherable          | `Fn(A) -> M<Option<B>>`     | `Fn(&A) -> M<Option<B>>`     | FnBrand, M, A, B    | WitherDispatch                | 4             | 5             | 7                 |
| 6   | filter_with_index / ref_filter_with_index               | FilterableWithIndex | RefFilterableWithIndex | `Fn(Idx, A) -> bool`        | `Fn(Idx, &A) -> bool`        | A                   | FilterWithIndexDispatch       | 2             | 2             | 4                 |
| 7   | filter_map_with_index / ref_filter_map_with_index       | FilterableWithIndex | RefFilterableWithIndex | `Fn(Idx, A) -> Option<B>`   | `Fn(Idx, &A) -> Option<B>`   | A, B                | FilterMapWithIndexDispatch    | 3             | 3             | 5                 |
| 8   | partition_with_index / ref_partition_with_index         | FilterableWithIndex | RefFilterableWithIndex | `Fn(Idx, A) -> bool`        | `Fn(Idx, &A) -> bool`        | A                   | PartitionWithIndexDispatch    | 2             | 2             | 4                 |
| 9   | partition_map_with_index / ref_partition_map_with_index | FilterableWithIndex | RefFilterableWithIndex | `Fn(Idx, A) -> Result<O,E>` | `Fn(Idx, &A) -> Result<O,E>` | A, E, O             | PartitionMapWithIndexDispatch | 4             | 4             | 6                 |

### Notes on the summary

- The "Unified turbofish" column counts total type parameter positions in the
  dispatch free function. The explicit positions are those the caller must
  annotate; the rest use `_`. In all 9 cases, `FA` and `Marker` are inferred,
  so the number of explicit positions equals the current Val turbofish count
  (or the current Ref count for pairs where the Ref path adds `FnBrand`).

- Pairs 1/2 and 6/8 share identical closure shapes (`Fn(A) -> bool` and
  `Fn(Idx, A) -> bool` respectively). They could theoretically share a dispatch
  trait since the closure signature is identical, with only the dispatch method's
  return type differing. However, separate traits are cleaner since each maps to
  a distinct type class method.

- Pairs 4 and 5 (wilt/wither) follow the same pattern as TraverseDispatch: the
  `FnBrand` parameter is unused in the Val impl but must appear for API
  uniformity with the Ref impl. The Val path gains one extra turbofish position
  compared to the current separate `wilt`/`wither` functions.

- The WithIndex variants (6-9) use `Brand::Index` (an associated type from the
  `WithIndex` trait) as the first closure argument. This is always passed
  by-value, so dispatch is driven entirely by the second argument (`A` vs `&A`).
  The Index type does not add a type parameter to the dispatch trait because it
  is determined by the Brand.

- All 9 dispatch traits use the same structural pattern: `Trait<'a, ..., FA, Marker>`
  with Val/Ref blanket impls. The only variation is how many type parameters sit
  between `'a` and `FA`.

### Grouping by complexity

**Simple (same shape as FunctorDispatch/FilterMapDispatch):**

- filter, partition, filter_with_index, partition_with_index -- no FnBrand, no
  extra brand params.

**Medium (similar to FilterMapDispatch with more output types):**

- partition_map, filter_map_with_index, partition_map_with_index -- adds E/O or B
  type params but otherwise follows the same pattern.

**Complex (similar to TraverseDispatch with FnBrand + effect brand):**

- wilt, wither -- requires FnBrand for the Ref path and an applicative M brand.
  These are the most complex dispatch traits in this batch.
