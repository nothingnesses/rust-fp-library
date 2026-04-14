# Brand Inference: Open Questions Investigation 1

Focus area: DefaultBrand trait design and type system interactions.

## 1. DefaultBrand for Parameterized Brands (Lazy, Coyoneda, etc.)

### Question

The plan lists `Lazy<'a, A, Config>` with brand `LazyBrand<Config>`. The
DefaultBrand impl would be:

```rust
impl<'a, A: 'a, Config: LazyConfig + 'a> DefaultBrand for Lazy<'a, A, Config> {
    type Brand = LazyBrand<Config>;
}
```

Does this work when `Config` is a type parameter? Can the compiler resolve
`DefaultBrand` for `Lazy<'a, i32, RcLazyConfig>`?

### Findings

**It works.** The POC test file (`fp-library/tests/brand_inference_feasibility.rs`)
already contains exactly this impl (line 34) and validates it with a passing
test (`ref_lazy_infer`, line 103):

```rust
impl<'a, A: 'a, Config: fp_library::classes::LazyConfig + 'a> DefaultBrand
    for Lazy<'a, A, Config>
{
    type Brand = LazyBrand<Config>;
}
```

The test `ref_lazy_infer` creates `RcLazy::pure(10)` (which is
`Lazy<'_, i32, RcLazyConfig>`) and calls `map_infer(|x: &i32| *x * 2, &lazy)`.
The compiler successfully resolves:

1. `&RcLazy<'_, i32>` -> blanket -> `RcLazy<'_, i32>` -> concrete impl -> `LazyBrand<RcLazyConfig>`.
2. `LazyBrand<RcLazyConfig>: Kind_cdc7cd43dac7585f` -> `Of<'a, i32> = Lazy<'a, i32, RcLazyConfig>`.
3. Dispatch selects the Ref impl.

The `Config` parameter flows through correctly because Rust's trait
resolution monomorphizes `Config = RcLazyConfig` before resolving the
associated type.

**One subtlety: the `Lazy` struct has a default type parameter**
(`Config: LazyConfig = RcLazyConfig`). When a user writes `Lazy<'a, i32>`
without specifying Config, Rust fills in `RcLazyConfig` and the DefaultBrand
impl resolves to `LazyBrand<RcLazyConfig>`. This is correct and expected.
The same pattern applies to all types with default type parameters.

**Trait bound propagation:** The `LazyBrand<Config>` struct requires
`Config: LazyConfig`. The DefaultBrand impl also requires
`Config: LazyConfig + 'a`. If a user has an unconstrained `Config` type
parameter, the `DefaultBrand` bound on `Lazy<'a, A, Config>` will fail
unless `Config: LazyConfig` is also in scope. This is correct behavior;
the type is not well-formed without the constraint.

### Recommendation

No action needed. This case is already validated by the POC.

## 2. DefaultBrand for Const (Phantom Type Parameter Stripping)

### Question

`Const<'a, R, A>` has brand `ConstBrand<R>`. The type has two type
parameters, but the brand only captures `R`, discarding `A`:

```rust
impl<'a, R: 'static, A: 'a> DefaultBrand for Const<'a, R, A> {
    type Brand = ConstBrand<R>;
}
```

Is this sound? Could stripping `A` cause inference ambiguity?

### Findings

**This is sound and does not cause ambiguity.** The key insight is that
`Const`'s `A` parameter is phantom: `Const<'a, R, A>` is defined as
`struct Const<'a, R, A>(pub R, pub PhantomData<&'a A>)`. The `A` parameter
does not affect the runtime representation. In the HKT system, `A` is the
type that `Kind::Of<'a, A>` varies over:

```rust
impl<R: 'static> Kind_cdc7cd43dac7585f for ConstBrand<R> {
    type Of<'a, A: 'a>: 'a = Const<'a, R, A>;
}
```

The `DefaultBrand` impl strips `A` because `A` is the "applied" type
parameter in the HKT sense. For any concrete `Const<'a, R, A>`, the brand
is always `ConstBrand<R>` regardless of `A`. There is no ambiguity because
there is exactly one brand that produces `Const<'a, R, A>` for a given `R`.

**The `R: 'static` bound** on `ConstBrand<R>` is important. The
`impl_kind!` invocation requires `R: 'static`. The DefaultBrand impl must
carry the same bound. If a user has `Const<'a, SomeNonStaticType, A>`, the
DefaultBrand impl will not apply, which is correct because
`ConstBrand<SomeNonStaticType>` would not implement Kind either.

**Potential confusion:** A user might expect `Const<'a, i32, String>` and
`Const<'a, i32, bool>` to have different brands, but they both resolve to
`ConstBrand<i32>`. This is correct, since the whole point of `Const` is to
ignore its second parameter. The `A` parameter exists only to satisfy the
Kind signature.

### Recommendation

No action needed. The phantom parameter stripping is a natural consequence
of the HKT encoding and does not cause issues.

## 3. DefaultBrand for CatList

### Question

What is the concrete type for `CatListBrand::Of<'a, A>`? Is it
`CatList<A>` directly, or something more complex?

### Findings

**`CatListBrand::Of<'a, A>` is directly `CatList<A>`.** The `impl_kind!`
invocation is:

```rust
impl_kind! {
    for CatListBrand {
        type Of<'a, A: 'a>: 'a = CatList<A>;
    }
}
```

`CatList<A>` is a plain struct with no lifetime parameter. The Kind
signature includes `'a` (required by the `Kind_cdc7cd43dac7585f` trait),
but `CatList<A>` does not use it. The `'a` lifetime is satisfied trivially
because `CatList<A>: 'a` holds whenever `A: 'a` (which the signature
already requires).

The DefaultBrand impl would be:

```rust
impl<A> DefaultBrand for CatList<A> {
    type Brand = CatListBrand;
}
```

This is straightforward and has no complications. `CatList` has exactly one
brand, the brand is not parameterized, and the concrete type maps directly.

**One consideration:** `CatList` is primarily an internal data structure
for the Free monad. It is public, but most users will not interact with it
directly. Implementing DefaultBrand is still correct and useful for
consistency, and it enables `map(f, cat_list)` without turbofish for users
who do work with CatList directly.

### Recommendation

No action needed. CatList is one of the simplest DefaultBrand cases.

## 4. Arity-2 DefaultBrand for Bifunctor Types (Result, Tuple2, etc.)

### Question

The plan proposes `DefaultBrand_266801a817966495` for arity-2 Kind:

```rust
impl<A, E> DefaultBrand_266801a817966495 for Result<A, E> {
    type Brand = ResultBrand;
}
```

Is this coherent given that `ResultBrand`'s arity-2 Kind impl swaps the
type parameters (`Of<'a, A, B> = Result<B, A>`)?

### Findings

**The impl is coherent, but the parameter ordering in the plan's example
is misleading.** The actual `Kind_266801a817966495` impl for `ResultBrand`
is:

```rust
impl Kind_266801a817966495 for ResultBrand {
    type Of<'a, A: 'a, B: 'a>: 'a = Result<B, A>;
}
```

Note: `Of<'a, A, B> = Result<B, A>`. The Kind's first parameter `A` is the
error type, and the second parameter `B` is the success type. This is by
design (functional programming convention: rightmost parameter is the
"primary" one).

For `DefaultBrand_266801a817966495`, the impl on `Result<A, E>` maps back
to `ResultBrand`. The trait just needs to recover the brand; it does not
need to know which type parameter is which. The compiler resolves:

```
<Result<i32, String> as DefaultBrand_266801a817966495>::Brand = ResultBrand
```

Then the `bimap` function uses `ResultBrand`'s `Kind_266801a817966495` impl
to project `Of<'a, A, B> = Result<B, A>`. The parameter mapping is handled
by the Kind impl, not by DefaultBrand. DefaultBrand only provides the
brand; it does not interpret the parameters.

**Coherence:** There is exactly one bifunctor brand for `Result<A, E>`,
namely `ResultBrand`. No other brand implements
`Kind_266801a817966495<Of<'a, _, _> = Result<_, _>>`, so the reverse
mapping is unambiguous.

**The plan's example `impl<A, E>` is correct.** The parameters `A` and `E`
in `impl<A, E> DefaultBrand_266801a817966495 for Result<A, E>` are the
type parameters of `Result` itself (success, error). The fact that
`ResultBrand`'s Kind swaps them is irrelevant to the DefaultBrand impl.

**Edge case: bimap parameter order.** When writing `bimap(f, g, Ok(5))`,
the `f` and `g` closures must correspond to the Kind's parameter ordering,
not Result's. Because `Of<'a, A, B> = Result<B, A>`, `f` maps the error
type (the Kind's first parameter) and `g` maps the success type (the
Kind's second parameter). This is already the established convention in
the library and is not changed by brand inference. However, the plan should
document this for users who might expect `bimap(success_fn, error_fn, ...)`.

### Recommendation

The plan's proposed impl is correct. Add a note to the plan clarifying that
`bimap`'s parameter order follows the Kind convention (error first, success
second for Result), not the Rust type's parameter order. This is an
existing API contract, not a brand-inference issue, but it becomes more
visible when users can write `bimap(f, g, Ok(5))` without a turbofish.

## 5. `#[diagnostic::on_unimplemented]` on Generated Trait Names

### Question

The plan proposes multiple DefaultBrand traits with content-hash names
(e.g., `DefaultBrand_cdc7cd43dac7585f`). Each needs
`#[diagnostic::on_unimplemented]`. Does this attribute work correctly
when the trait name is a generated identifier? Can the message reference
`{Self}` correctly?

### Findings

**The attribute works correctly with generated identifiers.**
`#[diagnostic::on_unimplemented]` is applied to the trait definition, not
to the trait name. The Rust compiler processes the attribute based on the
trait's identity (DefId), not its textual name. The `{Self}` placeholder
expands to the concrete type that failed to implement the trait, which is
independent of the trait's name.

The `trait_kind!` macro already generates traits with hash-suffixed names
and applies `#[allow(non_camel_case_types)]` to suppress naming warnings.
Adding `#[diagnostic::on_unimplemented]` to these generated traits follows
the same pattern.

**The trait name appearing in error messages is the concern, not the
attribute itself.** When the compiler reports "the trait
`DefaultBrand_cdc7cd43dac7585f` is not implemented for `Result<i32, String>`",
the hash-suffixed name is ugly and meaningless to users. The
`#[diagnostic::on_unimplemented]` attribute replaces the default message
with a custom one, which is exactly why it is needed:

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` has multiple brands and cannot use brand inference",
    note = "use `map_explicit::<YourBrand, _, _, _, _>(f, x)` to specify the brand"
)]
pub trait DefaultBrand_cdc7cd43dac7585f { ... }
```

With this attribute, the user sees:

```
error[E0277]: `Result<i32, String>` has multiple brands and cannot use brand inference
  note: use `map_explicit::<YourBrand, _, _, _, _>(f, x)` to specify the brand
```

The hash-suffixed trait name still appears in secondary diagnostics (e.g.,
"required by a bound in `map`"), but the primary error message is clear.

**Each arity needs a different message.** For arity-1 DefaultBrand, the
message should mention `map_explicit`. For arity-2 DefaultBrand, it should
mention `bimap_explicit`. The per-arity trait family naturally supports
this because each trait gets its own `#[diagnostic::on_unimplemented]`.

**The `note` field supports `{Self}`.** The `note` attribute also supports
placeholder expansion, so messages like
`note = "use map_explicit to map over {Self}"` work correctly.

### Recommendation

This works as designed. Two minor improvements:

1. Make the `note` message arity-specific (mention the correct `_explicit`
   function for each arity).
2. Consider adding `label = "this type"` to highlight the span of the
   problematic expression in the error output.

## 6. Blanket `impl DefaultBrand for &T` and Auto-Deref Edge Cases

### Question

When a user writes `map(f, &v)` where `v: Vec<i32>`, the compiler sees
`FA = &Vec<i32>` and the blanket resolves `Brand = VecBrand`. But Rust's
auto-deref might sometimes produce `&&Vec<i32>` or `&mut Vec<i32>`. Are
there edge cases where auto-deref interacts badly with the blanket impl?

### Findings

**Auto-deref does not apply in this context.** Rust's auto-deref (the
`Deref` trait chain) only activates in three situations:

1. Method calls (`x.method()`) - the receiver is auto-derefed.
2. Field access (`x.field`) - the receiver is auto-derefed.
3. The dereference operator (`*x`) - explicit, not automatic.

Function arguments are not auto-derefed. When a user writes `map(f, &v)`,
the compiler takes the expression `&v` at face value and infers
`FA = &Vec<i32>`. It does not auto-deref `&v` to `Vec<i32>` or auto-ref
it to `&&Vec<i32>`. The type of the expression is determined by the
expression itself, not by the function's parameter type, because `FA` is
a generic type parameter (not a concrete type that would trigger coercion).

**No implicit `&&T` creation.** Double references only arise when the user
explicitly writes `&&v` or when a variable already has type `&Vec<i32>`
and is passed by reference (`&borrowed_vec`). In both cases, the blanket
impl chains correctly (`&&Vec<i32>` -> `&Vec<i32>` -> `Vec<i32>` ->
`VecBrand`), and the downstream Kind constraint rejects the double
reference. The defaultbrand-for-refs analysis already covers this case.

**`&mut Vec<i32>` case.** If a user has `let mut v = vec![1, 2, 3]` and
writes `map(f, &mut v)`, the compiler infers `FA = &mut Vec<i32>`.
DefaultBrand is not implemented for `&mut T` (by design; see the
defaultbrand-for-refs analysis). The compiler reports that
`&mut Vec<i32>: DefaultBrand` is not satisfied. This is correct behavior.

The error message could be confusing because the user might not understand
why `&mut` does not work when `&` does. The `#[diagnostic::on_unimplemented]`
message should handle this gracefully. However, the current message
("`{Self}` has multiple brands and cannot use brand inference") is
misleading for `&mut Vec<i32>`, which does not have multiple brands; it
simply has no DefaultBrand impl at all.

**Deref coercion from `&mut T` to `&T`.** Rust does perform deref
coercion from `&mut T` to `&T` at coercion sites. However, generic
function parameters are not coercion sites; the type parameter `FA` is
inferred directly from the argument expression. So `map(f, &mut v)` will
NOT coerce `&mut Vec<i32>` to `&Vec<i32>`. The user would need to
explicitly reborrow: `map(f, &*v)` or `map(f, &v)`.

### Recommendation

1. No code changes needed for the blanket impl. Auto-deref does not
   interfere with function argument type inference.
2. Consider improving the `#[diagnostic::on_unimplemented]` message to
   handle the `&mut T` case. One approach: add a second note line
   suggesting `&v` instead of `&mut v` when the Self type is a mutable
   reference. However, `#[diagnostic::on_unimplemented]` does not support
   conditional notes based on Self's structure, so this may not be
   feasible. An alternative is to add a blanket
   `impl<T: DefaultBrand> DefaultBrand for &mut T` that resolves to the
   same brand, even though the dispatch system does not use it. This would
   make the error appear at the dispatch level ("no FunctorDispatch impl
   for &mut Vec<i32>") rather than at the DefaultBrand level, which is
   slightly more informative. Weigh this against the "confusing error"
   concern from the defaultbrand-for-refs analysis.

## 7. DefaultBrand and Type Aliases

### Question

If a user defines `type MyVec<A> = Vec<A>`, does `DefaultBrand` work for
`MyVec<i32>`?

### Findings

**Yes, it works transparently.** Type aliases in Rust are fully
transparent; they are expanded at the point of use before any trait
resolution occurs. `MyVec<i32>` is `Vec<i32>` as far as the type system
is concerned. The DefaultBrand impl `impl<A> DefaultBrand for Vec<A>`
applies directly.

This also applies to the library's own type aliases:

- `type RcLazy<'a, A> = Lazy<'a, A, RcLazyConfig>` -> DefaultBrand
  resolves to `LazyBrand<RcLazyConfig>`.
- `type ArcLazy<'a, A> = Lazy<'a, A, ArcLazyConfig>` -> DefaultBrand
  resolves to `LazyBrand<ArcLazyConfig>`.
- `type ArcLazyBrand = LazyBrand<ArcLazyConfig>` -> this is a brand alias,
  not a container alias, so DefaultBrand is not relevant here.

**No gotchas for simple aliases.** However, there is one edge case worth
noting for documentation purposes:

```rust
type MyResult<A> = Result<A, MyError>;
```

This alias fixes one type parameter. `MyResult<i32>` is `Result<i32, MyError>`.
Because `Result<A, E>` does NOT implement arity-1 DefaultBrand (multiple
brands), `MyResult<i32>` also does not implement it. The user must use
`map_explicit`. This might surprise users who think of `MyResult` as a
"single-parameter type" analogous to `Option`, but it is correct: the
underlying type is still `Result` with two type parameters, one of which
happens to be fixed. The ambiguity persists because
`ResultErrAppliedBrand<MyError>` and `ResultOkAppliedBrand<i32>` are both
valid brands for `Result<i32, MyError>`.

**Newtype wrappers are different from aliases.** If a user defines:

```rust
struct MyVec<A>(Vec<A>);
```

This is a distinct type and does NOT inherit Vec's DefaultBrand impl. The
user would need to define their own brand and Kind/DefaultBrand impls.
This is standard Rust behavior and not specific to brand inference.

### Recommendation

No code changes needed. Document the type alias transparency in the plan,
and specifically note that aliases over multi-brand types (like
`type MyResult<A> = Result<A, E>`) do not gain DefaultBrand just because
one parameter is fixed. Users who want inference for such aliases should
use a newtype wrapper with its own brand.

## Summary

| Question                      | Severity | Action needed                                           |
| ----------------------------- | -------- | ------------------------------------------------------- |
| Parameterized brands (Lazy)   | None     | Already validated by POC.                               |
| Const phantom parameter       | None     | Sound by construction.                                  |
| CatList                       | None     | Straightforward case.                                   |
| Arity-2 Result parameter swap | Low      | Document bimap parameter order for clarity.             |
| Diagnostic on generated names | Low      | Works; make notes arity-specific.                       |
| Auto-deref and &mut           | Low      | Consider &mut blanket impl or better error message.     |
| Type aliases                  | Low      | Document alias transparency and multi-brand limitation. |

No blocking issues found. All seven areas are either already validated or
have straightforward mitigations. The two most actionable items are:

1. Improving the `#[diagnostic::on_unimplemented]` message to handle
   `&mut T` gracefully (question 6).
2. Documenting the bimap parameter ordering convention more prominently
   now that brand inference makes it the default calling style (question 4).
