# Macro Changes Analysis: ref Qualifier in m_do!/a_do!

## Overview

The `ref` qualifier adds by-reference dispatch mode to `m_do!` and `a_do!`. When
present, the macros generate closures that receive `&A` instead of `A`, routing
through `RefSemimonad::ref_bind` (m_do) and `RefLift::ref_lift2` (a_do) via
the dispatch system.

## 1. Parsing: ref Qualifier in DoInput

**File:** `fp-macros/src/m_do/input.rs`

The parser uses `input.peek(Token![ref])` to detect the `ref` keyword before
the brand type. This is clean and correct: `ref` is a Rust keyword token, so
it cannot collide with a brand type name. The `DoInput` struct stores a
`ref_mode: bool` field.

**Assessment:** Correct. The implementation is minimal and non-ambiguous.
Two parser tests (`parse_ref_mode`, `parse_non_ref_mode`) verify both paths.

## 2. pure(x) -> ref_pure(&x) Rewriting

**File:** `fp-macros/src/m_do/codegen.rs`

In ref mode, the `PureRewriter` AST visitor transforms:

- `pure(args)` -> `ref_pure::<Brand, _>(&(args))`

The `&(args)` wrapping uses parentheses around `args`, which handles both
single-argument and multi-argument cases correctly:

- `pure(x)` -> `ref_pure::<Brand, _>(&(x))` -- the parentheses are redundant
  but harmless.
- `pure(x + 1)` -> `ref_pure::<Brand, _>(&(x + 1))` -- parentheses ensure the
  reference applies to the full expression, not just the first token.

**Potential issue with multi-argument pure:** The `args` variable comes from
`call.args`, which is a `Punctuated<Expr, Token![,]>`. If a user writes
`pure(x, y)` (which would be invalid for `pure::<Brand, _>` anyway, since
`pure` takes one argument), `&(args)` would produce `&(x, y)`, creating a
reference to a tuple. This is actually a reasonable desugaring if someone
were to use a multi-arg pure, but the real `ref_pure` function signature
takes a single `&A`, so the compiler would catch any misuse. No issue here.

**Assessment:** Correct. The `&(args)` pattern is safe and handles expression
precedence correctly.

## 3. Untyped Bindings: |pattern: &\_| in Ref Mode

**File:** `fp-macros/src/m_do/codegen.rs` and `fp-macros/src/a_do/codegen.rs`

When ref mode is active and the user does not provide a type annotation, the
macro adds `: &_` to the closure parameter:

- `a <- expr;` generates `|a: &_| { ... }` instead of `|a| { ... }`

This is necessary because the dispatch system (`BindDispatch`) uses the closure's
argument type to determine whether to route to `Semimonad::bind` (takes `Fn(A)`)
or `RefSemimonad::ref_bind` (takes `Fn(&A)`). Without the `&_` annotation, the
compiler cannot infer whether to use `Val` or `Ref` dispatch.

**Does `&_` always work with type inference?** Yes, in practice. The `_` is
inferred from the monadic value's type parameter. For example, if the expression
is `Lazy::<i32, _>::new(|| 10)`, then `&_` resolves to `&i32`. The compiler can
always infer the concrete type from the container's `Of<A>` type.

**Edge case: pattern destructuring.** If the user writes a pattern like
`(a, b) <- expr;` in ref mode, the macro generates `|(a, b): &_|`. This
would mean the closure receives `&(A, B)` and tries to destructure it as a
tuple, which does not work because `&(A, B)` is a reference, not a tuple.
The user would need to write `(a, b): &(i32, i32)` explicitly. This is a
minor ergonomic limitation but is consistent with how Rust handles reference
patterns -- the user should use `&(a, b)` as the pattern instead.

**Assessment:** Correct for the common case (simple identifier patterns).
Pattern destructuring in ref mode requires explicit type annotation or a
reference pattern. This is acceptable but could be documented more explicitly.

## 4. Sequence Variant: |_: &_| in Ref Mode

**File:** `fp-macros/src/m_do/codegen.rs` and `fp-macros/src/a_do/codegen.rs`

For sequence statements (`expr;` without a binding), ref mode generates
`|_: &_|` instead of `|_|`. This is correct for the same reason as untyped
binds: the dispatch system needs the `&_` annotation to resolve to the `Ref`
marker.

**Assessment:** Correct. The wildcard pattern `_` works fine with `&_` type
annotation.

## 5. Multi-bind Limitation

**File:** `fp-library/src/types/lazy.rs` (test: `m_do_ref_lazy_multi_bind`)

The limitation is: in ref mode, inner closures cannot capture references from
outer binds because the closure is `move |a: &_| { ... }` and the reference
`&A` does not live long enough to be captured by the inner closure.

The workaround is to dereference/clone the value into a `let` binding:

```
m_do!(ref LazyBrand<RcLazyConfig> {
    a: &i32 <- lazy_a;
    let a_val = *a;          // Copy the value
    b: &i32 <- lazy_b;
    pure(a_val + *b)          // Use the copy
})
```

**Is this adequately documented?** The test file documents the pattern clearly
with a comment. The `lib.rs` macro documentation mentions ref mode but does not
explicitly call out this multi-bind limitation. The plan mentions it in step 27.
The documentation in `lib.rs` for `m_do!` should probably include a note about
this limitation and the workaround pattern.

**Is there a better solution?** The fundamental issue is that `bind` produces
nested closures: `bind(expr_a, |a: &_| bind(expr_b, |b: &_| ...))`. The outer
closure receives `a` as a reference whose lifetime is scoped to that closure
body. The inner closure is `move`, so it cannot capture `a` by reference.

Alternative approaches:

1. **Generate non-move closures.** This would allow capturing `a` by reference,
   but would prevent moving other values into the inner closure, which is
   typically required for correctness.
2. **Automatically insert clone/copy for captured bindings.** The macro could
   detect which bindings are used in later binds and auto-insert `let a_val = *a;`
   or `let a_val = a.clone();`. This would be a significant increase in macro
   complexity and might surprise users with implicit clones.
3. **Use `a_do!` instead.** Applicative do-notation does not have this problem
   because all binds are independent (no nesting). The combining closure receives
   all values simultaneously. This is the recommended approach when all binds
   are independent.

**Assessment:** The limitation is inherent to monadic do-notation with by-reference
semantics. The workaround is documented in the test but should also appear in the
macro's doc comment. Option 3 (use `a_do!`) should be highlighted as the preferred
alternative.

## 6. a_do! Codegen Changes

**File:** `fp-macros/src/a_do/codegen.rs`

### Zero-bind case

When there are no binds, ref mode generates `ref_pure::<Brand, _>(&(final_expr))`
instead of `pure::<Brand, _>(final_expr)`. This is correct.

### Single-bind case

Generates `map::<Brand, _, _, _>(|param| { ... }, expr)` with the parameter
annotated as `param: &_` in ref mode. The extra `_` in the turbofish
(4 type params instead of 3) accounts for the `Marker` dispatch parameter.
This is correct.

### Multi-bind case (liftN)

The underscore count calculation was changed from `(0 ..= n)` to `(0 ..= n + 1)`.
This produces `n + 2` underscores, which is correct:

- `lift2` has `Brand, A, B, C, Marker` = 5 params. For n=2: n+2=4 underscores +
  Brand = 5 total. Correct.
- `lift3` has `Brand, A, B, C, D, Marker` = 6 params. For n=3: n+2=5 underscores +
  Brand = 6 total. Correct.
- `lift5` has `Brand, A, B, C, D, E, F, Marker` = 8 params. For n=5: n+2=7
  underscores + Brand = 8 total. Correct.

**Assessment:** Correct. The underscore count change properly accounts for the
new `Marker` type parameter in all dispatched `liftN` functions.

## 7. document_module.rs Changes

**File:** `fp-macros/src/documentation/document_module.rs`

This change is not directly related to the ref qualifier but was made alongside
it. The change converts the `document_module` macro from fail-fast error handling
to error-collecting: instead of propagating documentation generation errors with
`?`, errors are collected into `doc_errors` and emitted alongside the module's
items.

**Why this matters:** When a documentation attribute macro fails (e.g., due to a
type resolution issue in the new ref hierarchy traits), the fail-fast behavior
would prevent the module's items (traits, impls) from being emitted. This causes
cascading "unresolved import" errors that obscure the real issue. By collecting
errors and still emitting items, the compiler shows the actual documentation error
plus the items remain available for downstream resolution.

**Assessment:** This is a good defensive change. It improves the developer
experience when working with the documentation macros, especially during the
addition of many new traits. It does not affect runtime behavior.

## 8. Edge Cases and Silent Failures

### Pure rewriting in non-call positions

The `is_bare_pure_call` function checks for a path expression `pure` without
turbofish arguments and without a qualifying path. It correctly skips:

- `Foo::pure(...)` -- qualified path
- `::pure(...)` -- leading colon
- `pure::<T>(...)` -- existing turbofish

However, if a user defines a local variable named `pure` and calls it as
`pure(x)`, the rewriter will incorrectly transform it to `ref_pure::<Brand, _>(&(x))`
(or `pure::<Brand, _>(x)` in val mode). This is an existing issue, not
introduced by the ref mode changes. The macro documentation says "bare
`pure(args)` calls are automatically rewritten", which implies this is the
intended behavior. Users who shadow `pure` can use `(pure)(x)` to prevent
rewriting.

### Ref mode with typed binds

When the user provides a type annotation in ref mode (`a: &i32 <- expr;`),
the macro uses the type as-is: `|a: &i32| { ... }`. This is correct because
the user is responsible for including the `&` in the type annotation. The
macro does not add an extra `&` on top of a user-provided type.

However, if the user forgets the `&` and writes `a: i32 <- expr;` in ref mode,
the closure will receive `i32` but the dispatch system will still try to use
`Ref` dispatch (because the bind expression is in a ref-mode block). Actually,
that is incorrect: the dispatch system infers `Val`/`Ref` from the closure's
argument type, not from the macro's `ref_mode` flag. So if the user writes
`a: i32` in ref mode, the closure has type `Fn(i32) -> ...`, which dispatches
to `Val`. But the bind expression might be a ref-mode expression that requires
`Fn(&A) -> ...`. This would result in a type error, which is the correct
behavior: the user made a mistake by not including `&` in their type annotation.

**Assessment:** The macro handles typed binds correctly by trusting the user's
type annotation. Incorrect annotations produce compile-time errors, not silent
failures.

### Ref mode with let bindings

Let bindings in ref mode are not affected: they remain plain `let` bindings
without any `&_` annotation. This is correct because let bindings are pure
Rust expressions, not monadic operations.

## Summary of Findings

| Area                             | Status                   | Notes                              |
| -------------------------------- | ------------------------ | ---------------------------------- |
| ref qualifier parsing            | Correct                  | Clean, non-ambiguous               |
| pure -> ref_pure rewriting       | Correct                  | &(args) handles precedence         |
| Untyped binds (&\_)              | Correct                  | Works for simple patterns          |
| Sequence variant (_: &_)         | Correct                  | Dispatch resolves correctly        |
| Multi-bind limitation            | Documented in tests      | Should be in macro docs too        |
| liftN underscore count           | Correct                  | n+2 matches type param count       |
| document_module error collection | Good improvement         | Prevents cascading errors          |
| Edge cases                       | No silent failures found | Type errors caught at compile time |

## Recommendations

1. **Document multi-bind limitation in macro docs.** The `m_do!` doc comment in
   `lib.rs` should mention that in ref mode, multi-bind blocks require `let`
   bindings to capture dereferenced values for use in later binds, and suggest
   `a_do!` as an alternative when binds are independent.

2. **Document pattern destructuring limitation.** Note that pattern destructuring
   in ref mode untyped binds (e.g., `(a, b) <- expr;`) may require explicit type
   annotations or reference patterns.

3. **Fix stale doc comment in ref_functor.rs.** The "Why `FnOnce`?" section at
   line 80 of `fp-library/src/classes/ref_functor.rs` still references `FnOnce`
   but the signature was changed to `Fn` in step 3 of the plan. This should be
   updated to "Why `Fn`?" with appropriate explanation.
