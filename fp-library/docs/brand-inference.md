### Brand Inference

The library's HKT encoding provides a **forward mapping** from brands to concrete types
(`OptionBrand` -> `Option<A>`), but the compiler has no way to go backwards: given
`Option<A>`, it cannot determine that `OptionBrand` is the right brand. This means
every free function call would require a turbofish annotation:

```rust
use fp_library::{brands::*, functions::explicit::*};

// Without brand inference, the brand must be specified explicitly
let y = map::<OptionBrand, _, _, _, _>(|x: i32| x + 1, Some(5));
```

Brand inference adds the **reverse mapping** (concrete type -> brand), letting the
compiler infer the brand automatically:

```rust
use fp_library::functions::*;

// Brand inferred from Option<i32>
let y = map(|x: i32| x + 1, Some(5));
assert_eq!(y, Some(6));
```

#### The trait pair

The library uses two trait families to connect brands and concrete types.
Both describe the same underlying equation, `Brand::Of<A> = FA`, but
from different trait-selection angles.

| Trait              | `Self` type   | Direction | Multiplicity per type | Primary use                         |
| ------------------ | ------------- | --------- | --------------------- | ----------------------------------- |
| `Kind_*`           | Brand         | Forward   | One-to-one            | Apply brand to type argument        |
| `InferableBrand_*` | Concrete type | Reverse   | One impl per brand    | Recover brand; carry Val/Ref Marker |

Each trait carries the same content-hash suffix derived from the Kind
signature (e.g. `Kind_cdc7cd43dac7585f`, `InferableBrand_cdc7cd43dac7585f`).
The hash correspondence signals that both traits concern the same Kind
shape. `trait_kind!` generates them together. For the naming convention
and common hash suffixes, see [Higher-Kinded Types](./hkt.md#kind-trait-naming).

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
projects whether FA is owned (Val) or borrowed (Ref). Each trait also
has a `#[diagnostic::on_unimplemented]` attribute for actionable error
messages when inference fails. `InferableBrand` impls are auto-generated
by `impl_kind!` for all brands (including multi-brand types).

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

#### How inference resolves Brand

When the caller writes `map(|x| x + 1, Some(5))`, the compiler infers
`FA = Option<i32>` from the argument, finds the matching `InferableBrand`
impl to resolve `Brand = OptionBrand` and `Marker = Val`, then selects
the correct dispatch impl. Borrowed containers work the same way via a
blanket `&T` impl that sets `Marker = Ref`.

##### Closure-directed inference (step by step)

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

#### Multi-brand types

Some types are reachable through multiple brands at a given arity.
`Result<A, E>` has both `ResultErrAppliedBrand<E>` and
`ResultOkAppliedBrand<A>` as arity-1 brands.

Closure-directed inference disambiguates which brand applies. When the
closure's input type is annotated, it pins the `A` type parameter, which
in turn selects a unique `InferableBrand` impl:

- `map(|x: i32| x + 1, Ok::<i32, String>(5))` pins `A = i32`, selecting
  `ResultErrAppliedBrand<String>` (maps over Ok values).
- `map(|e: String| e.len(), Err::<i32, String>("hi".into()))` pins
  `A = String`, selecting `ResultOkAppliedBrand<i32>` (maps over Err values).

For diagonal cases where both brands unify (e.g., `Result<T, T>`), the
closure cannot disambiguate and the compiler reports an ambiguity error.
Use [`explicit::map`](crate::functions::explicit::map) with a turbofish in
these cases.

At arity 2, `Result` has exactly one brand (`ResultBrand`), so
`bimap((f, g), Ok(5))` infers the brand without annotation. The ambiguity
is arity-specific, not type-specific.

#### The `#[multi_brand]` attribute

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

#### Known limitations

- `'static` bounds on multi-brand `InferableBrand` impls prevent non-static
  fixed parameters from using inference. For example, if `E` has a lifetime,
  `map(f, Ok::<i32, &str>(5))` works because `&str: 'static`, but a
  non-static reference type would require `explicit::map`.
- `&&T` (double reference) is not supported by `FunctorDispatch`'s `Ref`
  impl. Pass a single reference (`&T`) instead.
- Pre-bound closures (`let f = |x| x + 1; map(f, Ok(5))`) may lose deferred
  inference context for multi-brand types, because the closure's parameter
  type is committed before `map` can use it for brand resolution. Annotate
  the closure parameter type explicitly in these cases.
- `trait_kind!` and `impl_kind!` emit `::fp_library::dispatch::{Val, Ref}`
  in generated code. External crates must depend on `fp-library` for the
  macros to work correctly.

#### Relationship to Val/Ref dispatch

Brand inference determines _which type constructor_ to use; [Val/Ref dispatch](./dispatch.md)
determines _which trait method_ (by-value or by-reference) to call. The two
systems compose through the shared `FA` type parameter: brand inference resolves
`FA` to a brand, while dispatch resolves `FA`'s ownership to a `Val` or `Ref`
marker. Together they enable fully inferred calls like `map(|x: i32| x + 1, Some(5))`
with no turbofish and no explicit val/ref selection.

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
