### Brand Dispatch Traits

The library uses two trait families to connect brands and concrete types.
Both describe the same underlying equation, `Brand::Of<A> = FA`, but
from different trait-selection angles. This doc explains how they fit
together.

#### The pair, at a glance

| Trait              | `Self` type   | Direction | Multiplicity per type | Primary use                         |
| ------------------ | ------------- | --------- | --------------------- | ----------------------------------- |
| `Kind_*`           | Brand         | Forward   | One-to-one            | Apply brand to type argument        |
| `InferableBrand_*` | Concrete type | Reverse   | One impl per brand    | Recover brand; carry Val/Ref Marker |

Each trait carries the same content-hash suffix derived from the Kind
signature (e.g. `Kind_cdc7cd43dac7585f`, `InferableBrand_cdc7cd43dac7585f`).
The hash correspondence signals that both traits concern the same Kind
shape. `trait_kind!` generates them together.

#### `Kind_*`: the forward mapping

`Kind_*` captures the higher-kinded-type encoding directly:

```rust,ignore
trait Kind_cdc7cd43dac7585f {
    type Of<'a, A: 'a>: 'a;
}

impl Kind_cdc7cd43dac7585f for OptionBrand {
    type Of<'a, A: 'a>: 'a = Option<A>;
}

impl<E> Kind_cdc7cd43dac7585f for ResultErrAppliedBrand<E> {
    type Of<'a, A: 'a>: 'a = Result<A, E>;
}
```

Read as a function: `Of: (Brand, A) -> ConcreteType`. Rust's trait
selection can answer "given `OptionBrand`, what is `Option<i32>`?" but
not the reverse. That asymmetry is what `InferableBrand_*` addresses.

#### `InferableBrand_*`: reverse mapping with Marker

`InferableBrand_*` inverts Kind. Brand and A are trait parameters (not associated
types), allowing multiple impls per concrete type keyed on different
Brand values. The trait carries an associated `type Marker` that
projects whether FA is owned (Val) or borrowed (Ref):

```rust,ignore
trait InferableBrand_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
    Brand: Kind_cdc7cd43dac7585f,
{
    type Marker;
}
```

Semantic content: "for this specific `(Brand, A)` pair, the equation
`Brand::Of<A> = FA` holds, and `Marker` records whether FA is an owned
or borrowed container."

##### Marker: Val/Ref dispatch routing

The `Marker` associated type is the key design element. It projects
from FA's reference-ness alone, before Brand and A are resolved:

- Direct impls for owned types set `type Marker = Val`.
- A single `&T` blanket sets `type Marker = Ref`.

When the inference wrapper projects
`<FA as InferableBrand<Brand, A>>::Marker`, the compiler commits
Marker from FA's ownership status immediately. This eliminates the
Val/Ref cross-competition that would otherwise block Ref + multi-brand
inference (where both Val and Ref dispatch impls appear as candidates
while Brand is still free).

**Marker-agreement invariant:** all InferableBrand impls for a given
Self type must agree on the same Marker value. Owned types always
produce Val; references always produce Ref. `impl_kind!` enforces
this by construction, since it is the sole generator of
InferableBrand impls.

##### Impl landscape

Every brand gets a direct InferableBrand impl. There is no blanket
from any other trait; all impls are generated individually by
`impl_kind!`.

Single-brand types have one impl:

```rust,ignore
impl<'a, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {
    type Marker = Val;
}
```

Multi-brand types have one impl per brand:

```rust,ignore
impl<'a, A: 'a, E: 'static> InferableBrand_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
    for Result<A, E>
{
    type Marker = Val;
}

impl<'a, T: 'static, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A>
    for Result<T, A>
{
    type Marker = Val;
}
```

The reference blanket (generated once globally per arity):

```rust,ignore
impl<'a, T: ?Sized, Brand, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, Brand, A> for &T
where
    T: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
    Brand: Kind_cdc7cd43dac7585f,
{
    type Marker = Ref;
}
```

Projection brands (e.g. `BifunctorFirstAppliedBrand<ResultBrand, A>`)
are skipped: `impl_kind!` does not generate InferableBrand impls when the
brand's `Of` target contains an `Apply!` macro invocation or a
qualified path with `::`.

##### How closure-directed inference resolves Brand

For `map(|x: i32| x + 1, Ok::<i32, String>(5))`:

1. `FA = Result<i32, String>` pinned by the argument.
2. `Marker` projected via InferableBrand: Result is owned, so Marker = Val.
3. With Marker committed, FunctorDispatch picks the Val impl. Its
   `Fn(A) -> B` bound pins `A = i32` from the closure.
4. With `A = i32`, only the `ResultErrAppliedBrand<String>` InferableBrand impl
   unifies with FA = `Result<i32, String>`. Brand commits.
5. Dispatch proceeds.

For `&Result<i32, String>` with `|x: &i32| *x + 1`:

1. `FA = &Result<i32, String>`.
2. The `&T` blanket projects Marker = Ref immediately.
3. FunctorDispatch Ref impl applies; `Fn(&A) -> B` pins A from `&i32`.
4. Inner InferableBrand impl on `Result<i32, String>` resolves to
   `ResultErrAppliedBrand<String>` with A = i32.
5. Dispatch proceeds through `RefFunctor::ref_map`.

##### Dual-bound inference for `apply`

`apply` has no direct closure, but the function payload inside `ff`
carries the type information. The inference wrapper introduces a
`WrappedFn` type parameter for the concrete wrapped function type
(e.g., `Rc<dyn Fn(i32) -> i32>`). Two InferableBrand bounds share
the Brand parameter:

- `FF: InferableBrand<Brand, WrappedFn>` keys on the wrapped
  function type inside `ff`.
- `FA: InferableBrand<Brand, A>` keys on the value type inside `fa`.

Rust's solver intersects the two bounds to commit a unique Brand.
A separate `InferableFnBrand<FnBrand, A, B, Marker>` bound on
`WrappedFn` resolves the FnBrand (e.g., `RcFnBrand`) from the
concrete wrapper type. This avoids a circular dependency that
would arise from bounding directly on
`<FnBrand as CloneFn>::Of<A, B>` (the solver would need FnBrand
to compute the associated type, but FnBrand is what it is trying
to infer).

#### The unified inference wrapper

`map` (and sibling closure-taking operations) binds on
`InferableBrand` with `Marker` projected:

```rust,ignore
pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
    f: impl FunctorDispatch<
        'a,
        Brand,
        A,
        B,
        FA,
        <FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
    >,
    fa: FA,
) -> Apply!(<Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>)
where
    Brand: Kind_cdc7cd43dac7585f,
    FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
```

`explicit::map` uses the same InferableBrand bounds but takes Brand as a
turbofish parameter, serving as the universal fallback for cases
inference cannot handle (e.g. `Result<T, T>` diagonal).

#### Coverage matrix

| Case                              | Behaviour                                   |
| --------------------------------- | ------------------------------------------- |
| Val + single-brand                | Inference (no change from before)           |
| Val + multi-brand                 | Inference via closure input                 |
| Ref + single-brand                | Inference (no change from before)           |
| Ref + multi-brand                 | Inference via closure input                 |
| Multi-brand + generic fixed param | Inference works                             |
| Multi-brand diagonal (`T=T`)      | Compile error; use `explicit::`             |
| Unannotated multi-brand           | Compile error; annotate or use `explicit::` |

#### Relationship to `#[multi_brand]`

The `#[multi_brand]` attribute on `impl_kind!` is a documentation
marker, not a codegen switch. Each `impl_kind!` invocation independently
emits at most one InferableBrand impl. Multiple InferableBrand impls
for a given concrete type come from multiple `impl_kind!` invocations
(one per brand). The attribute signals to human readers that this
brand shares its target type with other brands.

For single-brand types (no attribute), the macro generates one
InferableBrand impl. For multi-brand brands (attribute present), the
macro also generates one InferableBrand impl, but the reader knows
other brands targeting the same concrete type exist with their own
InferableBrand impls.

#### Practical guide: which trait for which purpose

| Task                                            | Use                |
| ----------------------------------------------- | ------------------ |
| Apply a brand to a type argument                | `Kind_*`           |
| Closure-directed dispatch (`map`, `bind`, ...)  | `InferableBrand_*` |
| Closureless dispatch (`join`, `alt`, ...)       | `InferableBrand_*` |
| Explicit dispatch via turbofish                 | `InferableBrand_*` |
| Handle multi-brand types in a closure-taking op | `InferableBrand_*` |

Operations outside the dispatch system (`pure`, `empty`) take Brand
as an explicit turbofish parameter and do not use InferableBrand.

#### Higher arities

The pattern extends mechanically. For any Kind arity `k`, the pair is:

- `Kind_k<Brand>` with `Of<A1, ..., Ak>`.
- `InferableBrand_k<Brand, A1, ..., Ak> for FA` with `type Marker`.

Each arity gets its own InferableBrand trait with the matching hash
suffix.
