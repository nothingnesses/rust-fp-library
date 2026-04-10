# Brand Inference: Open Questions Investigation 3

Focus: Proc-macro generation for `trait_kind!`, `impl_kind!`, and
`DefaultBrand!` macros.

## 1. Complexity of extending `trait_kind!` to generate DefaultBrand traits

**Issue:** The plan says `trait_kind!` should generate a
`DefaultBrand_{hash}` trait alongside each `Kind_{hash}` trait. How
complex is this change?

**What the research found:** `trait_kind_worker` in
`fp-macros/src/hkt/trait_kind.rs` is a short function (~25 lines of
codegen). It takes parsed `AssociatedTypes`, calls `generate_name()` to
get the `Kind_{hash}` identifier, iterates over associated types to emit
their declarations, and wraps everything in a `pub trait Kind_{hash} { ... }`
block. The function is self-contained and easy to extend.

The `DefaultBrand_{hash}` trait to generate is simpler than the Kind
trait itself. It has a single associated type `Brand` with a bound
`Brand: Kind_{hash}`. No generics, no associated type parameters.

**Approaches:**

A. Generate both traits in the same `trait_kind_worker` call. After
emitting `Kind_{hash}`, also emit `DefaultBrand_{hash}` using the same
hash. This requires changing `generate_name()` to either return both
names or accept a prefix parameter.

B. Add a separate `generate_default_brand_name()` function that takes
the same `AssociatedTypes` input, computes the same hash, and returns
`DefaultBrand_{hash}`. The `trait_kind_worker` calls both.

**Trade-offs:** Approach A is simpler and ensures the two traits are
always generated together. Approach B is more modular but risks
divergence if someone changes one call but not the other. The hash
computation is in `generate_name()`, which hardcodes the `Kind_` prefix
via `format_ident!("Kind_{:016x}", hash)`. A prefix parameter or a
separate function for the hash alone would be needed.

**Recommendation:** Extract the hash computation from `generate_name()`
into a `generate_hash(input) -> u64` function. Then `generate_name()`
calls `generate_hash()` and formats with `Kind_` prefix.
`trait_kind_worker` calls `generate_hash()` once and formats both
`Kind_{hash}` and `DefaultBrand_{hash}` from the same value. This is
a small, safe refactor: 5-10 lines changed in `canonicalizer.rs`, plus
~15 lines added to `trait_kind.rs`.

The `#[diagnostic::on_unimplemented]` attribute should be added to the
generated `DefaultBrand_{hash}` trait as specified in the plan. This is
straightforward since the attribute is just more `quote!` output.

**Complexity estimate:** Low. The `trait_kind!` extension is the easiest
part of the macro work.

## 2. How `impl_kind!` would extract type parameters for DefaultBrand impls

**Issue:** The plan says `impl_kind!` should generate
`impl<A> DefaultBrand_{hash} for ConcreteType<A> { type Brand = TheBrand; }`.
This requires extracting the type parameters from the `type Of<...> = ConcreteType<...>;`
definition and reconstructing them as generic parameters on the
DefaultBrand impl. How does the macro accomplish this, especially for
parameterized brands?

**What the research found:** The `ImplKindInput` struct already has all
the necessary data:

- `impl_generics`: the `impl<E>` parameters from the brand side.
- `brand`: the brand type (e.g., `ResultErrAppliedBrand<E>`).
- `definitions[0].target_type`: the concrete type (e.g., `Result<A, E>`).
- `definitions[0].signature.generics`: the associated type's generic
  parameters (e.g., `<'a, A: 'a>`).

To generate a `DefaultBrand` impl, the macro must:

1. Collect all generic parameters from both `impl_generics` (brand-level
   params like `E`, `Config`) and `definitions[0].signature.generics`
   (associated type params like `'a`, `A`).
2. Strip the bounds from the associated type generics (they constrain
   the Kind trait's associated type, not the DefaultBrand impl).
3. Use `definitions[0].target_type` as the `Self` type.
4. Use `brand` as the `Brand` associated type value.

**Parameterized brand example:** For
`impl<Config: LazyConfig> for LazyBrand<Config> { type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>; }`,
the macro should generate:

```
impl<'a, A: 'a, Config: LazyConfig> DefaultBrand_{hash} for Lazy<'a, A, Config> {
    type Brand = LazyBrand<Config>;
}
```

This means merging `impl_generics` (`<Config: LazyConfig>`) with the
associated type generics (`<'a, A: 'a>`). The ordering matters:
lifetimes first, then type params, then brand-level params. The current
`ImplKindInput` parser already separates these, so the merge is
mechanical.

**Complication: bound stripping vs. bound retention.** The associated
type generics have bounds like `A: 'a` that are needed for the Kind
trait's associated type but may or may not be needed on the DefaultBrand
impl. For `impl<'a, A: 'a> DefaultBrand for Thunk<'a, A>`, the `A: 'a`
bound IS needed because `Thunk<'a, A>` requires it. The macro should
preserve the bounds from the associated type generics, since they
reflect constraints that the concrete type itself imposes.

**Complication: where clauses.** `ImplKindInput` supports where clauses
on the impl block (e.g., `where E: std::fmt::Debug`). These should also
be forwarded to the `DefaultBrand` impl. The macro already parses them
into `impl_generics.where_clause`.

**Complication: multiple associated types.** Some `impl_kind!` blocks
have multiple associated types (e.g., `type Of<A>` and `type SendOf<B>`).
The macro only generates one `DefaultBrand` impl per `impl_kind!` block.
For multi-associated-type Kind traits, the DefaultBrand is keyed on the
full signature (all associated types contribute to the hash), not on any
single one. The concrete type used for the DefaultBrand impl should come
from the primary associated type. But which one is "primary"?

In practice, the current codebase has no `impl_kind!` invocation with
multiple associated types. All existing invocations define exactly one
associated type per block. This simplifies the problem: the macro always
uses `definitions[0]`. If a future multi-associated-type block appears,
the macro can require `#[no_default_brand]` for it, since the
DefaultBrand concept maps a single concrete type to a brand, and
multi-associated-type Kind traits describe brands with multiple type
constructors (one per associated type).

**Recommendation:** For the initial implementation, only support
DefaultBrand generation for `impl_kind!` blocks with exactly one
associated type definition. If the block has multiple definitions, skip
DefaultBrand generation and emit a note suggesting `#[no_default_brand]`
if one is desired. This sidesteps the "which type is primary" question
and covers all current use cases.

For the generic parameter merge, combine `impl_generics` and
`definitions[0].signature.generics`, preserving bounds from both. Use
`definitions[0].target_type` as-is for the `Self` type of the impl,
and `brand` as-is for the associated type value.

## 3. The `DefaultBrand!` macro and how it differs from `Kind!`

**Issue:** The plan proposes a `DefaultBrand!` macro analogous to `Kind!`
that resolves the trait name from the signature. How would it differ from
the `Kind!` macro?

**What the research found:** There is no `Kind!` macro as a standalone
invocation in the codebase. The `Kind!` syntax only appears inside the
`Apply!` macro as `Apply!(<Brand as Kind!( type Of...; )>::Of<...>)`.
The `Apply!` macro parses `Kind!( ... )` internally by looking for the
`Kind` identifier followed by `!` and parenthesized content, then
calling `generate_name()` on the parsed content.

There is a `#[kind(...)]` attribute macro in `kind_attr.rs` that adds a
`Kind_{hash}` supertrait bound to a trait. This also calls
`generate_name()`.

A `DefaultBrand!` macro would follow the same pattern as the `Kind!`
usage inside `Apply!`. It would:

1. Parse the associated type signature (same `AssociatedTypes` grammar).
2. Compute the hash (same `generate_hash()` function).
3. Emit `DefaultBrand_{hash}` instead of `Kind_{hash}`.

**Where would `DefaultBrand!` be used?** It would be used in trait
bounds and where clauses:

```rust
where FA: DefaultBrand!(type Of<'a, A: 'a>: 'a;)
```

This expands to:

```rust
where FA: DefaultBrand_cdc7cd43dac7585f
```

**Implementation complexity:** Very low. It is essentially the same as
`generate_name()` but with a `DefaultBrand_` prefix. If the hash
extraction is refactored as recommended in issue 1, the `DefaultBrand!`
macro is ~10 lines: parse input, compute hash, emit ident.

**Should it also be usable inside `Apply!`?** No. `Apply!` projects an
associated type from a Kind trait (`<Brand as Kind>::Of<...>`). The
`DefaultBrand` traits have a `Brand` associated type, not `Of`. There is
no need for an `Apply!`-style projection from DefaultBrand; callers
simply write `<FA as DefaultBrand_{hash}>::Brand` directly or use the
`DefaultBrand!` macro to resolve the trait name.

**Recommendation:** Implement `DefaultBrand!` as a minimal proc macro
that reuses the hash computation from `generate_name()`. Register it
alongside the existing macros in `fp-macros/src/lib.rs`. This is
straightforward.

## 4. Content hash stability between Kind and DefaultBrand

**Issue:** The `DefaultBrand_{hash}` trait names must use the exact same
hash as the corresponding `Kind_{hash}` traits. If the generation logic
is duplicated or the prefix changes the hash, the two will not match.

**What the research found:** The hash is computed in
`canonicalizer.rs::generate_name()`. It uses a fixed-seed `rapidhash`
(v3) with seed `0x1234567890abcdef`. The hash input is a canonical
string representation of the associated type signature, which includes:

- Associated type names (sorted for order-independence).
- Lifetime count, type parameter count.
- Canonicalized bounds on type parameters.
- Canonicalized output bounds.

The `Kind_` prefix is added AFTER the hash is computed:
`format_ident!("Kind_{:016x}", hash)`. The hash itself does not include
the prefix. This means the same hash can be used for both `Kind_` and
`DefaultBrand_` prefixes.

**Risk analysis:** There is no risk of hash mismatch as long as both
`trait_kind!` and `impl_kind!` call the same hash function with the same
canonical input. Currently, `impl_kind!` already calls `generate_name()`
to find the Kind trait name, so it has access to the same hash. The
proposed refactor (extract `generate_hash()`) makes this sharing
explicit.

If someone changed the canonical representation or hash seed in the
future, both Kind and DefaultBrand names would change together (since
they use the same code path). There is no scenario where one changes
and the other does not, as long as both derive from the same hash
function.

**One subtle risk:** If `trait_kind!` and `impl_kind!` are compiled in
different crate versions (e.g., the Kind trait is in an older dependency
and the DefaultBrand impl is in a newer dependency), and the hash
function changed between versions, the names would not match. This is
the same risk that already exists for Kind traits, and is mitigated by
the stability guarantee on the seed constant. The `DefaultBrand` traits
do not introduce a new risk here.

**Recommendation:** The current design is sound. Extract `generate_hash()`
as described in issue 1 to make the shared computation explicit. Add a
test that verifies `generate_hash()` produces the same value when called
from both `trait_kind!` and `impl_kind!` code paths (this test already
exists implicitly in the existing `impl_kind!` tests, since `impl_kind!`
must match the Kind trait name that `trait_kind!` generated). No further
action needed.

## 5. Multi-brand types and the `#[no_default_brand]` E0119 safety net

**Issue:** The plan claims that forgetting `#[no_default_brand]` on a
multi-brand type produces a "conflicting impl" compiler error (E0119).
Is this actually the case?

**What the research found:** Consider `Result<A, E>` which has two
arity-1 brands:

- `ResultErrAppliedBrand<E>`: `type Of<'a, A: 'a> = Result<A, E>;`
- `ResultOkAppliedBrand<A>`: `type Of<'a, A: 'a> = Result<T, A>;`

If both `impl_kind!` invocations generated DefaultBrand impls, the
result would be:

```rust
impl<'a, A: 'a, E: 'static> DefaultBrand_cdc7cd43dac7585f for Result<A, E> {
    type Brand = ResultErrAppliedBrand<E>;
}
impl<'a, T: 'static, A: 'a> DefaultBrand_cdc7cd43dac7585f for Result<T, A> {
    type Brand = ResultOkAppliedBrand<T>;
}
```

Both impls cover `Result<A, E>` for all `A` and `E`. The compiler would
report E0119 (conflicting implementations of trait
`DefaultBrand_cdc7cd43dac7585f` for type `Result<_, _>`). This IS the
expected behavior.

**But there is a subtlety with the `'static` bounds.** The first brand
has `E: 'static` and the second has `T: 'static`. If these bounds were
somehow non-overlapping, the impls would not conflict. But in practice,
most types satisfy `'static`, so the overlap is near-universal. The
compiler does not use `'static` bounds to distinguish impls; it
considers them overlapping if they could ever apply to the same type.
Specialization is unstable and not involved here. So E0119 is correctly
triggered.

**The bifunctor brands are separate.** `ResultBrand` at arity 2 uses a
different Kind hash (`Kind_266801a817966495` for `type Of<'a, A: 'a, B: 'a>: 'a;`)
and therefore a different DefaultBrand hash. Its DefaultBrand impl
(`impl DefaultBrand_266801a817966495 for Result<A, E>`) does not
conflict with the arity-1 impls because it is a different trait entirely.

**Additional multi-brand types to verify:**

- `(First, Second)` (Tuple2): `Tuple2FirstAppliedBrand<First>` and
  `Tuple2SecondAppliedBrand<Second>` both map to `(First, A)` and
  `(A, Second)` respectively. Both cover all tuples. E0119 confirmed.
- `Pair<First, Second>`: Same pattern as Tuple2. E0119 confirmed.
- `ControlFlow<B, C>`: `ControlFlowContinueAppliedBrand<C>` and
  `ControlFlowBreakAppliedBrand<B>` both map to `ControlFlow<B, C>`.
  E0119 confirmed.
- `TryThunk<'a, A, E>`: `TryThunkErrAppliedBrand<E>` and
  `TryThunkOkAppliedBrand<A>` both map to `TryThunk<'a, A, E>`.
  E0119 confirmed.

**BifunctorFirstAppliedBrand and BifunctorSecondAppliedBrand:** These
are generic over ANY bifunctor brand. Their target type is
`Apply!(<Brand as Kind!(...)>::Of<'a, A, B>)`, which expands to a
projection type, not a concrete type like `Result`. The DefaultBrand
impl would be for a projection type, which is not the same as `Result`.
However, after monomorphization, these project TO the same concrete
types as the direct brands. Whether this causes E0119 depends on
whether the compiler sees through the projection.

Actually, the BifunctorFirstAppliedBrand impl_kind! maps to
`<Brand as Kind_266801a817966495>::Of<'a, A, B>`, which is an
associated type projection. Rust does not allow implementing traits for
associated type projections (`impl Trait for <X as Y>::Assoc` is not
valid). So impl_kind! with these brands would NOT generate a valid
DefaultBrand impl. The macro would try to use the target_type as the
Self type, and the compiler would reject it.

This is actually safe: the macro should NOT generate DefaultBrand for
these brands regardless, because (a) the target type is a projection,
not a concrete type, and (b) these brands are inherently secondary
(the primary brand is the bifunctor brand itself). These invocations
would either need `#[no_default_brand]` or the macro should detect that
the target type is a projection and skip DefaultBrand generation.

**Recommendation:** In addition to the `#[no_default_brand]` attribute,
the macro should detect when the target type is an associated type
projection (contains `::`) and automatically skip DefaultBrand
generation for it, possibly with a warning. This covers the
BifunctorFirstAppliedBrand and BifunctorSecondAppliedBrand cases
without requiring manual annotation. For the ProfunctorFirst/
SecondAppliedBrand cases, the same logic applies.

## 6. Risk of `impl_kind!` generating DefaultBrand for reference types

**Issue:** The blanket `impl DefaultBrand for &T` is hand-written.
Could `impl_kind!` accidentally generate a DefaultBrand impl for a
reference type, conflicting with the blanket?

**What the research found:** The `impl_kind!` macro generates its
DefaultBrand impl using `definitions[0].target_type` as the Self type.
The target type comes from parsing `type Of<...> = ConcreteType<...>;`.
No existing `impl_kind!` invocation uses a reference type as the
concrete type (e.g., `type Of<A> = &SomeType<A>;`). This would be
unusual since HKT brands map to owned types, not references.

However, nothing in the macro prevents a user from writing:

```rust
impl_kind! {
    for SomeBrand {
        type Of<'a, A: 'a>: 'a = &'a SomeType<A>;
    }
}
```

If the macro generated a DefaultBrand impl for `&'a SomeType<A>`, it
would conflict with the blanket `impl<T: DefaultBrand> DefaultBrand for &T`
if `SomeType<A>` also implements DefaultBrand. This would be E0119.

If `SomeType<A>` does NOT implement DefaultBrand, there is no conflict,
but the impl is still problematic: it provides a brand for a specific
reference type, bypassing the blanket's delegation semantics.

**How likely is this?** Very unlikely in practice. All existing brands
map to owned types. Reference-type brands would break the HKT
abstraction (brands should be `'static` marker types, and the Kind
trait's `Of` should produce owned types that can be moved).

**Recommendation:** No special handling needed. The macro generates
DefaultBrand for whatever target type is written in the `impl_kind!`
invocation. If a user writes a reference type, the E0119 error (if
it occurs) or the type system's rejection of the brand (brands must be
`'static`) will catch the mistake. The blanket impl is not at risk from
the macro's normal usage.

## 7. Complete list of `impl_kind!` invocations requiring `#[no_default_brand]`

**Issue:** Which existing `impl_kind!` invocations map to multi-brand
types and therefore need the `#[no_default_brand]` opt-out?

**What the research found:** Every `impl_kind!` invocation was examined.
The following categorization applies:

### Invocations that SHOULD generate DefaultBrand (no opt-out needed)

These are the default case: one brand per concrete type at this arity.

| File                             | Brand                         | Target type                             |
| -------------------------------- | ----------------------------- | --------------------------------------- |
| `types/option.rs:22`             | `OptionBrand`                 | `Option<A>`                             |
| `types/vec.rs:25`                | `VecBrand`                    | `Vec<A>`                                |
| `types/identity.rs:44`           | `IdentityBrand`               | `Identity<A>`                           |
| `types/thunk.rs:412`             | `ThunkBrand`                  | `Thunk<'a, A>`                          |
| `types/send_thunk.rs:438`        | `SendThunkBrand`              | `SendThunk<'a, A>`                      |
| `types/cat_list.rs:225`          | `CatListBrand`                | `CatList<A>`                            |
| `types/lazy.rs:750`              | `LazyBrand<Config>`           | `Lazy<'a, A, Config>`                   |
| `types/try_lazy.rs:1492`         | `TryLazyBrand<E, Config>`     | `TryLazy<'a, A, E, Config>`             |
| `types/coyoneda.rs:572`          | `CoyonedaBrand<F>`            | `Coyoneda<'a, F, A>`                    |
| `types/rc_coyoneda.rs:693`       | `RcCoyonedaBrand<F>`          | `RcCoyoneda<'a, F, A>`                  |
| `types/arc_coyoneda.rs:698`      | `ArcCoyonedaBrand<F>`         | `ArcCoyoneda<'a, F, A>`                 |
| `types/coyoneda_explicit.rs:708` | `CoyonedaExplicitBrand<F, B>` | `BoxedCoyonedaExplicit<'a, F, B, A>`    |
| `types/const_val.rs:188`         | `ConstBrand<R>`               | `Const<'a, R, A>`                       |
| `types/tuple_1.rs:22`            | `Tuple1Brand`                 | `(A,)` (arity `type Of<A>`)             |
| `types/tuple_1.rs:28`            | `Tuple1Brand`                 | `(A,)` (arity `type Of<'a, A: 'a>: 'a`) |

Note: `Tuple1Brand` has two `impl_kind!` invocations at different
arities. Both generate the same brand for the same concrete type
`(A,)`, but at different DefaultBrand hash variants. Both are safe
to generate.

### Invocations that NEED `#[no_default_brand]` (multi-brand at arity 1)

These concrete types are reachable through multiple arity-1 brands.

| File                         | Brand                                | Target type          | Conflicting brand                    |
| ---------------------------- | ------------------------------------ | -------------------- | ------------------------------------ |
| `types/result.rs:347`        | `ResultErrAppliedBrand<E>`           | `Result<A, E>`       | `ResultOkAppliedBrand<T>`            |
| `types/result.rs:855`        | `ResultOkAppliedBrand<T>`            | `Result<T, A>`       | `ResultErrAppliedBrand<E>`           |
| `types/tuple_2.rs:290`       | `Tuple2FirstAppliedBrand<First>`     | `(First, A)`         | `Tuple2SecondAppliedBrand<Second>`   |
| `types/tuple_2.rs:1049`      | `Tuple2SecondAppliedBrand<Second>`   | `(A, Second)`        | `Tuple2FirstAppliedBrand<First>`     |
| `types/pair.rs:646`          | `PairFirstAppliedBrand<First>`       | `Pair<First, A>`     | `PairSecondAppliedBrand<Second>`     |
| `types/pair.rs:1431`         | `PairSecondAppliedBrand<Second>`     | `Pair<A, Second>`    | `PairFirstAppliedBrand<First>`       |
| `types/control_flow.rs:970`  | `ControlFlowContinueAppliedBrand<C>` | `ControlFlow<B, C>`  | `ControlFlowBreakAppliedBrand<B>`    |
| `types/control_flow.rs:1513` | `ControlFlowBreakAppliedBrand<B>`    | `ControlFlow<B, C>`  | `ControlFlowContinueAppliedBrand<C>` |
| `types/try_thunk.rs:964`     | `TryThunkErrAppliedBrand<E>`         | `TryThunk<'a, A, E>` | `TryThunkOkAppliedBrand<A>`          |
| `types/try_thunk.rs:1692`    | `TryThunkOkAppliedBrand<A>`          | `TryThunk<'a, A, E>` | `TryThunkErrAppliedBrand<E>`         |

### Invocations that SHOULD generate DefaultBrand at arity 2

These are the bifunctor brands, unambiguous at arity 2.

| File                        | Brand              | Target type                                                            |
| --------------------------- | ------------------ | ---------------------------------------------------------------------- |
| `types/result.rs:26`        | `ResultBrand`      | `Result<B, A>` (arity `type Of<A, B>`)                                 |
| `types/result.rs:38`        | `ResultBrand`      | `Result<B, A>` (arity `type Of<'a, A: 'a, B: 'a>: 'a`)                 |
| `types/tuple_2.rs:26`       | `Tuple2Brand`      | `(First, Second)` (arity `type Of<First, Second>`)                     |
| `types/tuple_2.rs:32`       | `Tuple2Brand`      | `(First, Second)` (arity `type Of<'a, First: 'a, Second: 'a>: 'a`)     |
| `types/pair.rs:51`          | `PairBrand`        | `Pair<First, Second>` (arity `type Of<First, Second>`)                 |
| `types/pair.rs:57`          | `PairBrand`        | `Pair<First, Second>` (arity `type Of<'a, First: 'a, Second: 'a>: 'a`) |
| `types/control_flow.rs:690` | `ControlFlowBrand` | `ControlFlow<B, C>` (arity `type Of<C, B>`)                            |
| `types/control_flow.rs:696` | `ControlFlowBrand` | `ControlFlow<B, C>` (arity `type Of<'a, C: 'a, B: 'a>: 'a`)            |
| `types/try_thunk.rs:1445`   | `TryThunkBrand`    | `TryThunk<'a, A, E>` (arity `type Of<'a, E: 'a, A: 'a>: 'a`)           |

### Invocations that SHOULD be auto-skipped (projection target types)

These brands use `Apply!()` in the target type, producing an associated
type projection. DefaultBrand impls for projection types are invalid
Rust. The macro should detect this and skip generation.

| File                        | Brand                                    | Target type (projection)                 |
| --------------------------- | ---------------------------------------- | ---------------------------------------- |
| `classes/bifunctor.rs:166`  | `BifunctorFirstAppliedBrand<Brand, A>`   | `Apply!(<Brand as Kind!(...)>::Of<...>)` |
| `classes/bifunctor.rs:207`  | `BifunctorSecondAppliedBrand<Brand, B>`  | `Apply!(<Brand as Kind!(...)>::Of<...>)` |
| `classes/profunctor.rs:402` | `ProfunctorFirstAppliedBrand<Brand, A>`  | `Apply!(<Brand as Kind!(...)>::Of<...>)` |
| `classes/profunctor.rs:442` | `ProfunctorSecondAppliedBrand<Brand, B>` | `Apply!(<Brand as Kind!(...)>::Of<...>)` |

### Invocations for profunctor/optics brands (DefaultBrand not useful)

These are internal profunctor types used in the optics system. They are
not user-facing container types and have no meaningful DefaultBrand
semantics (users do not write `map(f, some_shop_value)`). They should
use `#[no_default_brand]` or be auto-skipped.

| File                          | Brand                                  | Target type                                   |
| ----------------------------- | -------------------------------------- | --------------------------------------------- |
| `types/fn_brand.rs:36`        | `FnBrand<P>`                           | `P::CloneableOf<dyn Fn(A) -> B>` (projection) |
| `types/optics/exchange.rs:82` | `ExchangeBrand<FnBrand, A, B>`         | `Exchange<...>`                               |
| `types/optics/shop.rs:83`     | `ShopBrand<FnBrand, A, B>`             | `Shop<...>`                                   |
| `types/optics/market.rs:85`   | `MarketBrand<FnBrand, A, B>`           | `Market<...>`                                 |
| `types/optics/forget.rs:147`  | `ForgetBrand<PtrBrand, R>`             | `Forget<...>`                                 |
| `types/optics/stall.rs:84`    | `StallBrand<FnBrand, A, B>`            | `Stall<...>`                                  |
| `types/optics/bazaar.rs:63`   | `BazaarListBrand<FnBrand, A, B>`       | `BazaarList<...>`                             |
| `types/optics/bazaar.rs:487`  | `BazaarBrand<FnBrand, A, B>`           | `Bazaar<...>`                                 |
| `types/optics/grating.rs:93`  | `GratingBrand<FnBrand, A, B>`          | `Grating<...>`                                |
| `types/optics/tagged.rs:65`   | `TaggedBrand`                          | `Tagged<'a, A, B>`                            |
| `types/optics/zipping.rs:106` | `ZippingBrand<FnBrand>`                | `Zipping<...>`                                |
| `types/optics/indexed.rs:86`  | `IndexedBrand<P, I>`                   | `Indexed<...>`                                |
| `types/optics/reverse.rs:177` | `ReverseBrand<InnerP, PtrBrand, S, T>` | `Reverse<...>`                                |

### Special case: `String`

| File                 | Brand    | Target type |
| -------------------- | -------- | ----------- |
| `types/string.rs:19` | `String` | `String`    |

This is unusual: the brand IS the concrete type. It uses
`type Of<'a> = String` with no type parameters. Generating
`impl DefaultBrand for String { type Brand = String; }` is valid but
pointless, since `String` is used as a `Monoid`/`Semigroup` and not
as a container. It is harmless but unnecessary. The macro could
generate it, or `#[no_default_brand]` could be used.

**Recommendation:** Generate DefaultBrand for String. It is harmless,
and the consistent default-generation behavior is simpler than adding
special cases.

### Summary count

- Generate DefaultBrand: ~15 invocations (core types).
- Need `#[no_default_brand]`: 10 invocations (multi-brand arity-1).
- Auto-skip (projection types): 4 invocations.
- `#[no_default_brand]` recommended (optics/profunctor): 13 invocations.
- Special case (String): 1 invocation.

**Recommendation for handling optics/profunctor brands:** Rather than
requiring `#[no_default_brand]` on all 13 optics invocations, consider
making the default behavior configurable at the module or feature level.
Alternatively, since these brands are unlikely to cause E0119 (no two
optics brands map to the same concrete type), generating DefaultBrand
for them is harmless even if unused. Generating by default and letting
dead code analysis handle it is simpler than requiring 13 manual
annotations. However, note that `FnBrand<P>` uses a projection type as
its target and must be auto-skipped or annotated.

## 8. The `#[no_default_brand]` attribute: parsing and validation

**Issue:** How does the macro detect and handle `#[no_default_brand]`?

**What the research found:** The `ImplKindInput` parser already captures
outer attributes via `input.call(syn::Attribute::parse_outer)?` into the
`attributes` field. The existing codegen filters doc-specific attributes
using `attributes::filter_doc_attributes()`. A similar approach works
for `#[no_default_brand]`:

1. Check if any attribute in `input.attributes` has the path
   `no_default_brand`.
2. If present, skip DefaultBrand generation.
3. Filter it out of the attributes passed to the Kind impl (to avoid
   "unused attribute" warnings).

**Subtlety:** The `#[no_default_brand]` attribute is on the `impl_kind!`
invocation's outer attributes, which currently become doc comments on the
generated Kind impl. The macro must consume/remove this attribute before
generating the Kind impl, otherwise rustc would warn about an unknown
attribute.

**Recommendation:** Add a check in `impl_kind_worker` after parsing:

```rust
let no_default_brand = input.attributes.iter()
    .any(|attr| attr.path().is_ident("no_default_brand"));
```

Filter `no_default_brand` from the attributes before generating the Kind
impl. If `!no_default_brand`, generate the DefaultBrand impl alongside
the Kind impl. This is a small, mechanical change.

## 9. Duplicate DefaultBrand impls from same-brand, different-arity invocations

**Issue:** Several types have two `impl_kind!` invocations for the same
brand at different arities (e.g., `Tuple1Brand` with `type Of<A>` and
`type Of<'a, A: 'a>: 'a`). Each invocation would generate a DefaultBrand
impl for the same concrete type at different DefaultBrand trait variants.
But what about types like `ResultBrand` which has `type Of<A, B>` and
`type Of<'a, A: 'a, B: 'a>: 'a` at the same arity-2 level?

**What the research found:** `ResultBrand` has:

- `type Of<A, B> = Result<B, A>;` (hash for `type Of<A, B>`)
- `type Of<'a, A: 'a, B: 'a>: 'a = Result<B, A>;` (hash for `type Of<'a, A: 'a, B: 'a>: 'a`)

These are different Kind traits with different hashes, so they produce
different DefaultBrand traits. Each DefaultBrand impl is for a different
trait, so there is no conflict.

However, both would generate a DefaultBrand impl for `Result<B, A>`:

```rust
impl<A, B> DefaultBrand_5b1bcedfd80bdc16 for Result<B, A> {
    type Brand = ResultBrand;
}
impl<'a, A: 'a, B: 'a> DefaultBrand_266801a817966495 for Result<B, A> {
    type Brand = ResultBrand;
}
```

These are impls of DIFFERENT traits, so no E0119. Both are valid and
useful (the arity-2 DefaultBrand without lifetime and the arity-2
DefaultBrand with lifetime serve different calling contexts).

The same applies to `Tuple2Brand`, `PairBrand`, `ControlFlowBrand`, and
`Tuple1Brand`. No issues here.

**Recommendation:** No action needed. Different arities produce different
DefaultBrand traits, so multiple impls for the same brand are safe.

## 10. Target type parameter ordering in generated DefaultBrand impls

**Issue:** The target type in `impl_kind!` sometimes reorders parameters
relative to the associated type generics. For example, `ResultBrand`
has `type Of<'a, A: 'a, B: 'a>: 'a = Result<B, A>;` where `B` and `A`
are swapped. The macro must use `Result<B, A>` as-is for the Self type
of the DefaultBrand impl, not assume the parameters appear in the same
order as the generics.

**What the research found:** The macro already has `target_type` as a
parsed `syn::Type`. It does not need to reconstruct the type from the
generic parameters; it uses the type as-written. So `Result<B, A>` is
used directly in `impl DefaultBrand for Result<B, A>`. The generic
parameters on the impl block come from merging `impl_generics` and
`signature.generics`, which include `A` and `B` as declared in the
associated type.

This is correct: `impl<'a, A: 'a, B: 'a> DefaultBrand for Result<B, A>`
compiles fine because `A` and `B` are both in scope from the impl
generics.

**Potential issue:** The macro must ensure that all type parameters used
in `target_type` are declared in the impl's generic parameters. Since
the target type can only reference parameters from `impl_generics` and
`signature.generics`, and both are included in the merged impl
parameters, this is guaranteed by construction.

**Recommendation:** No action needed. The macro uses `target_type`
as-is, which is correct regardless of parameter ordering.
