# Feasibility: Rust Ecosystem Precedents for Generic Function Type Parameters

## 1. Rust Futures / Future trait

Every `async fn` and combinator (`.map()`, `.then()`) returns a struct generic over
the inner type and closure (e.g., `Map<Fut, F>`), producing unnameable nested types.
The `futures` crate provides `FutureExt::boxed()` returning
`BoxFuture<'a, T>` (a `Pin<Box<dyn Future + Send + 'a>>`), plus `boxed_local()` for
non-Send. Users handle unnameable types via `impl Future` in return position,
`.boxed()` for struct fields and recursive async, and (eventually) TAIT.

## 2. Iterator adaptors in std

`std::iter::Map<I, F>`, `Filter<I, P>`, etc. are all generic over the closure type
since Rust 1.0. Users handle it with `impl Iterator` returns, `Box<dyn Iterator>` for
struct fields, `.collect()` for eager materialization, and `fn` pointers when closures
capture nothing. No `.boxed()` convenience exists in std but the `Box::new()` pattern
is trivial.

## 3. Tower middleware

`tower::Service` layers produce nested generic types like
`RateLimit<Timeout<MyService>>`. Tower provides `BoxService`, `BoxCloneService`, and
`BoxLayer` as escape hatches via `ServiceExt::boxed()`. Used in production by hyper,
tonic, axum.

## 4. FP/category-theory crates

No Rust FP crate has implemented this exact pattern for Coyoneda. `frunk` and `higher`
use associated types (no fusion, no unnameable types). However, `proptest` uses
precisely this pattern: strategies chain combinators producing unnameable types, with
`.boxed()` returning `BoxedStrategy<T>`.

## 5. Ergonomic lessons

Main pain points are struct field storage (requires `.boxed()`), generic parameter
propagation, and verbose error messages. Key mitigations:

- `impl Trait` in return position (stable 1.26)
- RPITIT (stable 1.75)
- `impl Trait` in let bindings (stable 1.83)
- TAIT (`type_alias_impl_trait`, tracking issue #63063) is still nightly-only as of
  early 2026 but would eliminate the struct field pain point when stabilized.

The pattern is viable without TAIT; the ecosystem proves this conclusively.
