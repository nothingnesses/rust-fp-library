# Brand Inference: Macro Interaction Analysis

## 1. Current Macro Codegen

### m_do! current output

`m_do!(ref OptionBrand { x <- Some(5); pure(x + 1) })` generates:

```rust
bind::<OptionBrand, _, _, _, _>(
    &(Some(5)),
    move |x: &_| {
        ref_pure::<OptionBrand, _>(&(x + 1))
    }
)
```

Key details:

- `bind` gets turbofish `<Brand, _, _, _, _>` (Brand, A, B, FA, Marker).
- Container argument is wrapped in `&(expr)` due to ref mode.
- Closure parameter gets `&_` type annotation for dispatch inference.
- `pure(x + 1)` is rewritten to `ref_pure::<Brand, _>(&(x + 1))`.

Without ref mode, `m_do!(OptionBrand { x <- Some(5); pure(x + 1) })` generates:

```rust
bind::<OptionBrand, _, _, _, _>(
    Some(5),
    move |x| {
        pure::<OptionBrand, _>(x + 1)
    }
)
```

### a_do! current output

`a_do!(ref OptionBrand { x <- Some(5); y <- Some(10); *x + *y })` generates:

```rust
lift2::<OptionBrand, _, _, _, _, _, _>(
    |x: &_, y: &_| { *x + *y },
    &(Some(5)),
    &(Some(10))
)
```

Key details:

- `lift2` gets turbofish `<Brand, _, _, _, _, _, _>` (Brand, A, B, C, FA, FB, Marker = 7 params, 6 underscores).
- For N binds, `liftN` turbofish has `2N + 2` underscores (N value types + 1 result type + N container types + 1 Marker).
- Container arguments are wrapped in `&(expr)` due to ref mode.
- Closure parameters get `&_` type annotations.

With 1 bind, `a_do!` uses `map` instead of `lift1`:

```rust
map::<OptionBrand, _, _, _, _>(
    |x: &_| { *x * 2 },
    &(Some(5))
)
```

With 0 binds, `a_do!` uses `pure` / `ref_pure` directly:

```rust
ref_pure::<OptionBrand, _>(&(42))
```

## 2. Inferred Brand Codegen

### What m_do! would generate

`m_do!({ x <- Some(5); pure(x + 1) })` would need to generate:

```rust
bind(
    Some(5),
    move |x| {
        /* pure(x + 1) -- what goes here? */
    }
)
```

The `bind` call without a turbofish relies on the `DefaultBrand` trait to
resolve the brand from the `FA` parameter type. For `bind(Some(5), ...)`,
the compiler sees `FA = Option<i32>`, resolves
`<Option<i32> as DefaultBrand>::Brand = OptionBrand`, and dispatches to the
correct `BindDispatch` impl.

**Does removing the turbofish work for `bind`?** Yes, in principle. The
inference-based `bind` function (from the plan) takes `FA` as a plain
generic parameter with a `DefaultBrand` bound. The compiler infers `FA`
from the argument, then resolves `Brand` via the associated type. The
`Marker` parameter is resolved by `BindDispatch` impl selection (Val vs
Ref), same as today. The type parameters `A` and `B` are inferred from the
closure signature and return type. No turbofish is needed.

**Does the compiler have enough information to infer the brand?** Yes,
for types that implement `DefaultBrand`. The concrete container argument
provides `FA`, which resolves `Brand`. The closure provides `A` (input)
and `B` (output). The `Marker` is resolved by dispatch. All type parameters
are determined.

### What a_do! would generate

`a_do!({ x <- Some(5); y <- Some(10); x + y })` would generate:

```rust
lift2(
    |x, y| { x + y },
    Some(5),
    Some(10)
)
```

The inference-based `lift2` would infer the brand from `FA` (the first
container argument). See Section 5 for complications with multiple
container arguments.

## 3. Ref Mode with Inference

`m_do!(ref { x <- Some(5); pure(x + 1) })` would generate:

```rust
bind(
    &(Some(5)),
    move |x: &_| {
        /* pure rewriting -- see Section 4 */
    }
)
```

**Does `&Some(5)` (which is `&Option<i32>`) correctly resolve `DefaultBrand`?**

This depends on whether `DefaultBrand` is implemented for `&Option<A>` or
only for `Option<A>`. The plan's `DefaultBrand` impls are written for owned
types: `impl<A> DefaultBrand for Option<A>`.

For ref mode, the `FA` argument type is `&Option<i32>`. There are two
approaches:

1. **Blanket impl for references.** Add `impl<T: DefaultBrand> DefaultBrand
for &T { type Brand = T::Brand; }`. This makes `&Option<i32>` resolve to
   `OptionBrand` automatically. The `defaultbrand-for-refs.md` analysis
   covers this approach.

2. **Inference function accepts `AsRef`-style bounds.** The inference-based
   `bind` could accept `FA: Borrow<Inner>` where `Inner: DefaultBrand`.
   This is more complex and less natural.

Approach (1) is strongly preferred. With a blanket `&T` impl, ref mode
works transparently: `bind(&Some(5), ...)` resolves `FA = &Option<i32>`,
then `<&Option<i32> as DefaultBrand>::Brand = OptionBrand`.

**For `a_do!(ref { ... })`,** the same applies: `map(|x: &_| ..., &Some(5))`
resolves via the `&T` blanket impl.

## 4. The `pure` Problem in Macros

`pure(expr)` is rewritten by the macro. With explicit brand, it becomes
`pure::<Brand, _>(expr)` (val mode) or `ref_pure::<Brand, _>(&(expr))`
(ref mode). Without a brand, there is no container argument to infer from.

### Option A: Keep `pure` with explicit brand in macro output

The macro could extract the brand from the first bind expression at
compile time and use it for `pure` rewriting. But the macro operates on
syntax, not types. It cannot determine what `DefaultBrand` impl applies
to `Some(5)`. The whole point of inference is to avoid naming the brand
in user code.

However, the macro could use a helper trait or function that derives
the brand from a "sibling" expression. For example, the macro could
generate:

```rust
bind(Some(5), move |x| {
    pure::<<Option<i32> as DefaultBrand>::Brand, _>(x + 1)
})
```

But the macro does not know the type of `Some(5)` at expansion time.
It only has syntax tokens. So it cannot write `Option<i32>` in the
turbofish. This approach is not feasible.

### Option B: Use return-type inference

Generate `pure(x + 1)` as a function that infers Brand from its return
position. The plan already notes this is unreliable: Rust only propagates
return-type constraints in limited cases. When `pure` appears as the last
expression in a chain of `bind` closures, the return type is constrained
by the `bind` function's return type, which is
`<Brand as Kind>::Of<'a, B>`. In theory, the compiler could work backward
from this constraint to infer `Brand`. In practice:

- The inference-based `bind` fixes `Brand` via `<FA as DefaultBrand>::Brand`.
  The return type of `bind`'s closure must match
  `<FA as DefaultBrand>::Brand as Kind::Of<'a, B>`. So `pure`'s return type
  IS fully constrained by the enclosing `bind`.
- A `pure_infer` function could be defined with a where clause that ties its
  output to some Brand, and let Rust unify from context. But the function
  signature `fn pure_infer<Brand, A>(a: A) -> Brand::Of<A>` gives the
  compiler no way to determine `Brand` from the argument alone. It must come
  from the return position.

Return-type inference for `pure` inside `bind` closures is actually
plausible because the closure's return type is fully constrained. The
chain is:

1. `bind(fa, |x| { ... pure_last(expr) })` where `bind` is the
   inference-based version.
2. `bind` constrains: closure return type = `<Brand as Kind>::Of<'a, B>`
   where `Brand = <FA as DefaultBrand>::Brand`.
3. `pure_last` returns `<SomeBrand as Kind>::Of<'a, B>`. The compiler
   unifies this with the constraint from (2).
4. If `pure_last` is defined as
   `fn pure_last<Brand: Pointed, A>(a: A) -> Brand::Of<A>`, the compiler
   can infer `Brand` from the unification.

**Verdict:** This approach works when `pure` is in return position of a
`bind` closure (which is the final expression of `m_do!`). It may also
work in intermediate bind positions because each `bind` closure constrains
its return type. However, it fails in standalone contexts like
`let x: Option<i32> = m_do!({ pure(5) })` where there is no enclosing
`bind` to provide the constraint (unless the user adds a type annotation).

**Risk:** Rust's return-type inference is not always reliable across
complex trait bounds and GATs. This needs POC validation. If it works,
it is the cleanest solution.

### Option C: Require users to write `Some(expr)` instead of `pure(expr)`

In inferred mode, the macro would not rewrite `pure(...)` at all. Users
would write the concrete constructor:

```rust
m_do!({
    x <- Some(5);
    Some(x + 1)   // instead of pure(x + 1)
})
```

This works because the concrete constructor (`Some`, `vec![]`, etc.) is
already type-determined. No brand inference is needed for the final
expression.

**Downsides:**

- Breaks the abstraction: the user's code is now tied to the concrete type.
- In Haskell, `pure` / `return` in do-notation is idiomatic. Losing it
  makes the macro feel less like a proper monadic do-notation.
- For types like `Thunk` or `Lazy`, the constructor is more verbose than
  `pure(expr)`.

**Verdict:** Workable as a fallback but not ideal. This should be the
documented workaround if Option B fails, not the primary path.

### Option D: Use a different mechanism

The macro could generate a `pure_from` function that takes a "witness"
container argument to determine the brand:

```rust
// Generated by macro:
bind(Some(5), move |x| {
    pure_from(&Some(5), x + 1)  // first arg is only used for type inference
})
```

But this requires evaluating the witness expression (or using a
`PhantomData`-based approach). A `PhantomData` variant:

```rust
fn pure_witness<FA: DefaultBrand, A>(
    _witness: std::marker::PhantomData<FA>,
    a: A,
) -> <<FA as DefaultBrand>::Brand as Kind>::Of<A>
```

The macro would need to thread a phantom witness through the expansion.
This is possible but ugly. The macro could capture the type of the first
bind expression and create `PhantomData::<_>` that unifies with it, but
again the macro doesn't know the type at expansion time.

A cleaner variant: the macro generates a closure-based encoding where
`pure` is replaced by the identity of the `bind` chain's expected
return type:

```rust
// pure(expr) in final position becomes just the expression,
// and the enclosing bind provides the wrapping.
```

But this changes the semantics: `pure` is not just the expression; it
wraps it in the functor. So this does not work.

**Verdict:** None of the Option D variants are satisfactory. Option B
(return-type inference) is the most promising if it can be validated.

### POC result: Option B FAILS

Tested in `brand_inference_feasibility.rs`. The call
`bind_infer(Some(5), |x: i32| pure_infer(x + 1))` fails with E0283:
"cannot infer type of the type parameter `Brand`". The compiler lists
19+ types implementing `Pointed` and cannot select one, even though the
enclosing `bind_infer` constrains the closure return type. Rust does not
propagate return-type constraints backward through GAT-projected generic
parameters.

### Recommended approach

**Use Option C (concrete constructors) as the documented approach.**
In inferred-mode macros, `pure(expr)` is not supported. Users write
the concrete constructor (`Some(expr)`, `vec![expr]`, `Lazy::new(|| expr)`,
etc.) or use the explicit-brand macro syntax where `pure(expr)` is
rewritten with a turbofish.

The macro should emit a clear compile error if `pure(...)` is used in
inferred mode. This can be done at macro expansion time: if the macro
detects a `pure(...)` call and no brand is specified, emit
`compile_error!("pure() requires an explicit brand; use the concrete
constructor or m_do!(Brand { ... }) syntax")`.

## 5. a_do! Complications

### Which container determines the brand?

`a_do!` uses `liftN` functions that take multiple container arguments.
With explicit brand, all containers are checked against the same brand
via the turbofish. With inference, the brand must be inferred from the
container arguments.

The inference-based `lift2` would look like:

```rust
fn lift2<FA, FB, A, B, C, Marker>(
    f: impl Lift2Dispatch<'a, <FA as DefaultBrand>::Brand, A, B, C, FA, FB, Marker>,
    fa: FA,
    fb: FB,
) -> ...
where
    FA: DefaultBrand,
    <FA as DefaultBrand>::Brand: Kind<Of<'a, A> = FA>,
    // FB must also be compatible with the same brand
    <FA as DefaultBrand>::Brand: Kind<Of<'a, B> = FB>,
```

The brand is determined by `FA` (the first container). The constraint
`<FA as DefaultBrand>::Brand: Kind<Of<'a, B> = FB>` ensures `FB` is
compatible with the same brand. If `FA = Option<i32>` and
`FB = Option<String>`, then `Brand = OptionBrand`, and the compiler
verifies `OptionBrand::Of<String> = Option<String>`. This works.

**What if the containers disagree?** If `FA = Option<i32>` and
`FB = Vec<String>`, the constraint `OptionBrand: Kind<Of<String> = Vec<String>>`
fails. The compiler produces an error. This is correct behavior:
applicative lifting requires all containers to share the same functor.

**Is first-argument priority a problem?** No. For well-typed programs,
all container arguments must agree on the brand. It does not matter which
one the compiler uses to determine it. Choosing the first is arbitrary
but consistent.

**What about `map` (1 bind)?** `map` takes only one container, so
inference is straightforward. The brand comes from that single container.

**What about 0 binds?** `a_do!` with 0 binds generates `pure::<Brand, _>(expr)`.
In inferred mode, this has the same `pure` problem as `m_do!`. The
macro could require a type annotation: `let x: Option<i32> = a_do!({ 42 })`.
Or it could simply not support 0-bind inferred mode (which is a rare case;
0-bind `a_do!` is just `pure`).

### Ref mode with multiple containers

In ref mode, `a_do!(ref { x <- Some(5); y <- Some(10); *x + *y })` would
generate:

```rust
lift2(
    |x: &_, y: &_| { *x + *y },
    &(Some(5)),
    &(Some(10))
)
```

With the `&T` blanket `DefaultBrand` impl, `FA = &Option<i32>` resolves
to `OptionBrand`. The same applies to `FB = &Option<i32>`. No issues
beyond those already covered in Section 3.

## 6. Backward Compatibility

### Can both syntaxes coexist?

`m_do!(Brand { ... })` and `m_do!({ ... })` must coexist. The macro
parser currently parses `Brand` as a `Type` token before the braced block.
The question is whether the parser can distinguish "no brand, just a
braced block" from "brand followed by braced block."

### Parser disambiguation

The current parser in `input.rs` does:

```rust
let ref_mode = input.peek(Token![ref]);
if ref_mode { input.parse::<Token![ref]>()?; }
let brand: Type = input.parse()?;
let content;
braced!(content in input);
```

If the user writes `m_do!({ ... })`, the parser tries to parse `{...}` as a
`Type`. Since `{ ... }` is not valid type syntax, `input.parse::<Type>()`
would fail.

To support both syntaxes, the parser needs to check whether the next token
is `{` (opening brace). If so, skip the brand parse. This is straightforward:

```rust
let ref_mode = input.peek(Token![ref]);
if ref_mode { input.parse::<Token![ref]>()?; }

let brand: Option<Type> = if input.peek(syn::token::Brace) {
    None  // inferred mode
} else {
    Some(input.parse()?)  // explicit brand
};

let content;
braced!(content in input);
```

The codegen then branches on `brand.is_some()`:

- `Some(brand)` -> generate turbofish calls as today.
- `None` -> generate inference-based calls (no turbofish).

This is a clean syntactic distinction. `{` is never the start of a valid
Rust type, so there is no ambiguity.

### Ref mode combinations

All four combinations are syntactically distinct:

| Syntax                     | Brand    | Ref | Example                            |
| -------------------------- | -------- | --- | ---------------------------------- |
| `m_do!(Brand { ... })`     | Explicit | No  | `m_do!(OptionBrand { ... })`       |
| `m_do!(ref Brand { ... })` | Explicit | Yes | `m_do!(ref OptionBrand { ... })`   |
| `m_do!({ ... })`           | Inferred | No  | `m_do!({ x <- Some(5); ... })`     |
| `m_do!(ref { ... })`       | Inferred | Yes | `m_do!(ref { x <- Some(5); ... })` |

For `m_do!(ref { ... })`, after consuming `ref`, the parser checks for
`{`. If present, it is inferred ref mode. Otherwise, parse a brand type.
This works because `ref` is a keyword, not a type name.

### a_do! same approach

`a_do!` shares the same `DoInput` parser, so the same disambiguation
applies. All four combinations work identically.

## Summary of Findings

| Aspect                        | Status  | Notes                                                                           |
| ----------------------------- | ------- | ------------------------------------------------------------------------------- |
| `bind` without turbofish      | Works   | `DefaultBrand` on `FA` determines Brand; A, B, Marker all inferred.             |
| `map` without turbofish       | Works   | Same mechanism as `bind`.                                                       |
| `liftN` without turbofish     | Works   | First container determines Brand; others verified by constraints.               |
| `pure` without brand          | FAILS   | E0283: return-type inference does not resolve Brand. Use concrete constructors. |
| `ref_pure` without brand      | FAILS   | Same as `pure`; return-type inference does not work.                            |
| Ref mode with inference       | Works   | Requires `&T` blanket `DefaultBrand` impl.                                      |
| Parser backward compatibility | Works   | `{` vs type token is unambiguous.                                               |
| 0-bind `a_do!` inferred       | Blocked | Same `pure` problem; may require type annotation.                               |

### Recommended implementation order for macros (step 10 in the plan)

1. Update `DoInput` parser to make brand optional (peek for `{`).

2. Update `m_do_worker` and `a_do_worker` to branch on `brand.is_some()`:
   explicit brand path unchanged, inferred path emits no turbofish for
   `bind`/`map`/`liftN`.

3. For `pure` in inferred mode: emit `compile_error!` if `pure(...)` is
   detected without a brand. This gives a clear, actionable error message
   instead of a confusing E0283. Users write the concrete constructor or
   switch to explicit-brand syntax.

4. For `ref_pure` in inferred ref mode: same `compile_error!` approach.

5. Add tests for all four syntax combinations (explicit val, explicit ref,
   inferred val, inferred ref). Inferred mode tests should NOT use `pure()`
   in the final expression.
