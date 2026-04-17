# Task: Fix HM signature generation for apply and eliminate stringly-typed processing

## Problem

The `apply` function in `dispatch/semiapplicative.rs` generates an
incorrect HM type signature:

```
forall Brand A B WrappedFn. Semiapplicative Brand => (Brand WrappedFn, Brand A) -> Brand B
```

The correct signature (matching PureScript's `Apply` class) is:

```
forall Brand A B. Semiapplicative Brand => (Brand (A -> B), Brand A) -> Brand B
```

The issue: `WrappedFn` is a type parameter representing the concrete
wrapped function type (e.g., `Rc<dyn Fn(A) -> B>`). The where clause
says `WrappedFn: InferableFnBrand<FnBrand, A, B, Mode>`, which means
`WrappedFn` is semantically `(A -> B)`. But the HM generator does not
understand `InferableFnBrand` bounds and renders `WrappedFn` as an
opaque type parameter.

## Scope

Two tasks, both in `fp-macros/src/documentation/`:

### 1. Teach the HM signature generator to resolve InferableFnBrand

When a type parameter is bounded by `InferableFnBrand<_, A, B, _>` in
the where clause, the generator should:

- Hide the type parameter from the `forall` clause (like Marker and
  FnBrand are hidden)
- Substitute the parameter with `(A -> B)` wherever it appears as a
  container element type

The entry point is `generation.rs`, specifically the `build_container_map`
and parameter rendering pipeline. The `InferableFnBrand` trait is defined
in `fp-library/src/dispatch/semiapplicative.rs`.

The semiapplicative signature snapshot test is currently disabled in
`signature_snapshot_tests.rs`. Re-enable it once the fix is in place,
with the expected snapshot:

```
apply: forall Brand A B. Semiapplicative Brand => (Brand (A -> B), Brand A) -> Brand B
```

### 2. Eliminate stringly-typed processing in the doc generation pipeline

Several functions in `generation.rs` use `quote!(#ty).to_string()`
to convert AST nodes to strings for matching. This is fragile and
should be replaced with direct AST pattern matching. Specifically:

- `is_dispatch_container_param`: converts `bounded_ty` to string for
  comparison. Should use `syn::Path::get_ident()` or segment matching.
- `build_container_map` / `extract_dispatch_type_args`: converts type
  args to strings. Should work with `syn::Type` directly.
- Any new code for InferableFnBrand resolution should use AST types
  from the start, not string conversion.

Look for existing AST utilities in the codebase that can be reused:

- `fp-macros/src/support/type_visitor.rs` - TypeVisitor for AST walking
- `fp-macros/src/support/` - general parsing and attribute utilities
- `fp-macros/src/analysis/traits.rs` - `classify_trait`, trait bound
  analysis
- `fp-macros/src/analysis/dispatch.rs` - dispatch trait analysis

Factor out common logic where patterns repeat. For example, scanning
where-clause bounds by trait name is done in both
`is_dispatch_container_param` and the new InferableFnBrand resolution;
these should share a utility.

## Key files

- `fp-macros/src/documentation/generation.rs` - main file to modify
- `fp-macros/src/documentation/signature_snapshot_tests.rs` - re-enable
  the semiapplicative test
- `fp-macros/src/documentation/document_signature.rs` - HM signature
  generation (may need changes for forall hiding)
- `fp-macros/src/core/constants.rs` - HIDDEN_TYPE_PARAMS (do NOT add
  WrappedFn here; the fix should be contextual, not global)
- `fp-library/src/dispatch/semiapplicative.rs` - the apply function
  whose signature needs fixing

## Constraints

- Use hard tabs for indentation
- Do not use emoji or unicode symbols
- Use AST types (syn::Type, syn::Path, syn::Ident) instead of string
  conversion wherever possible
- Reuse existing analysis utilities rather than duplicating logic
- Stage changes and tun `just verify` to confirm all tests pass after changes
