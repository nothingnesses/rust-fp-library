# Coyoneda Implementation Fix Plan

## Context

Five independent analyses identified issues in `Coyoneda` (coyoneda.rs) and
`CoyonedaExplicit` (coyoneda_explicit.rs). The most critical findings are:

- `CoyonedaExplicit` claims "zero-cost map fusion" but allocates a `Box<dyn Fn>` per `map`.
- `Coyoneda` performs no map fusion at all (k maps = k `F::map` calls).
- Missing `Semimonad` instance, `From` conversion, benchmarks, and documentation accuracy.

This plan addresses the issues in seven phases, ordered by risk (docs first, then code).
Phase 7 redesigns `CoyonedaExplicit` with a generic function type parameter to achieve
truly zero-cost fusion.

## Files

- `fp-library/src/types/coyoneda.rs` (Phases 1-3, 5)
- `fp-library/src/types/coyoneda_explicit.rs` (Phases 1, 3, 6, 7)
- `fp-library/benches/benchmarks.rs` (Phase 4)
- `fp-library/benches/benchmarks/coyoneda.rs` -- new file (Phase 4)

---

## Phase 1: Documentation Corrections

No code behavior changes. Fix false claims and add user guidance.

### 1a. Fix false "zero-cost" claims in coyoneda_explicit.rs

Each `map` call does `Box::new(compose(f, self.func))` (line 173), which is a heap
allocation. The following docs are inaccurate and must be corrected:

| Location                                              | Current claim                                                          | Correction                                                                                                                     |
| ----------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Line 1 (module doc)                                   | "zero-cost map fusion"                                                 | "single-pass map fusion"                                                                                                       |
| Lines 11-12 (module doc)                              | "No boxing, no dynamic dispatch, no heap allocation"                   | State that each `map` allocates one `Box<dyn Fn>` for the composed function; the win is a single `F::map` call at `lower` time |
| Line 21 (comparison table, "Heap allocation per map") | `0`                                                                    | `1 box`                                                                                                                        |
| Line 22 (comparison table, "Stack overflow risk")     | `No`                                                                   | `Yes (deep closures)`                                                                                                          |
| Line 30 (when-to-use)                                 | "zero-cost map fusion"                                                 | "single-pass map fusion"                                                                                                       |
| Line 74 (struct doc)                                  | "zero-cost map fusion"                                                 | "single-pass map fusion"                                                                                                       |
| Lines 84-85 (struct doc)                              | "eliminates all boxing, dynamic dispatch, and per-map heap allocation" | "reduces lowering to a single `F::map` call, though each `map` allocates one `Box<dyn Fn>` for the composed function"          |
| Lines 142-144 (map doc)                               | "No heap allocation occurs for the composition itself"                 | State clearly that a new `Box<dyn Fn>` is allocated wrapping the composed result                                               |

### 1b. Add cross-reference guidance in coyoneda.rs

In the module doc (around line 16), add a note directing users to `CoyonedaExplicit`
for single-pass fusion.

### 1c. Document fusion barrier behavior

- `apply` doc (around line 318-324): note it calls `lower()` on both arguments,
  materializing accumulated maps.
- `bind` doc (around line 368-373): same fusion-barrier note.
- `into_coyoneda` doc (around line 293-296): note that further `map` calls on the
  resulting `Coyoneda` do not fuse with the previously composed function.

### Verification

```
just verify
```

---

## Phase 2: Semimonad for CoyonedaBrand

Add `Semimonad` impl for `CoyonedaBrand<F>` using the lower-bind-relift pattern.

### Location

`coyoneda.rs`, inside `mod inner`, after the `Foldable` impl (after line 581), before
the closing `}` of `mod inner`.

### Import

Add `Semimonad` to the `use` block at lines 113-128.

### Implementation

Pattern follows `OptionBrand`'s `Semimonad` at `types/option.rs:207-245`:

```rust
#[document_type_parameters("The brand of the underlying type constructor.")]
impl<F: Functor + Semimonad + 'static> Semimonad for CoyonedaBrand<F> {
    // doc macros: #[document_signature], #[document_type_parameters],
    // #[document_parameters], #[document_returns], #[document_examples]
    // with example using Coyoneda::<OptionBrand, _>::lift + bind free fn
    fn bind<'a, A: 'a, B: 'a>(
        ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
        func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
        Coyoneda::lift(F::bind(ma.lower(), move |a| func(a).lower()))
    }
}
```

Requires `F: Functor` (for `lower`) and `F: Semimonad` (for `F::bind`).

### Tests (in `mod tests`)

- `bind_option_some`: bind on `Some` with a function returning `Some`.
- `bind_option_none`: bind on `None` returns `None`.
- `bind_vec`: flatmap-style expansion.
- `bind_after_map`: maps are applied before bind.

### Verification

```
just verify
```

---

## Phase 3: From Conversion and into_explicit

### 3a. From<CoyonedaExplicit> for Coyoneda

Place inside `coyoneda_explicit.rs` `mod inner` (around line 473, after the last
`impl` block) since the `func` and `fb` fields are private to that module. Delegate to
the existing `into_coyoneda` method.

```rust
impl<'a, F, B: 'a, A: 'a> From<CoyonedaExplicit<'a, F, B, A>> for Coyoneda<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
{
    // doc macros following thunk.rs From pattern (lines 340-362)
    fn from(explicit: CoyonedaExplicit<'a, F, B, A>) -> Self {
        explicit.into_coyoneda()
    }
}
```

Import `Coyoneda` in `coyoneda_explicit.rs`'s `mod inner` use block.

### 3b. From<Coyoneda> for CoyonedaExplicit

Place inside `coyoneda.rs` `mod inner` (after the Semimonad impl) since `lower` is
a method on `Coyoneda`. Requires `F: Functor` for the lowering step.

```rust
impl<'a, F, A: 'a> From<Coyoneda<'a, F, A>> for CoyonedaExplicit<'a, F, A, A>
where
    F: Kind_cdc7cd43dac7585f + Functor + 'a,
{
    // doc macros following thunk.rs From pattern (lines 340-362)
    //
    // Doc comment must include a warning:
    //   "This calls `lower()`, which applies all accumulated mapping layers
    //   via `F::map`. For eager containers like `Vec`, this allocates and
    //   traverses the full container. The cost is proportional to the number
    //   of chained maps and the container size."
    fn from(coyo: Coyoneda<'a, F, A>) -> Self {
        CoyonedaExplicit::lift(coyo.lower())
    }
}
```

Import `CoyonedaExplicit` in `coyoneda.rs`'s `mod inner` use block.

### Tests

- `from_explicit_preserves_semantics`: convert via `Coyoneda::from(explicit)`.
- `from_coyoneda_preserves_semantics`: convert via `CoyonedaExplicit::from(coyo)`.
- `from_coyoneda_then_map_lower`: `CoyonedaExplicit::from(coyo).map(f).lower()`.
- `from_coyoneda_roundtrip`: `CoyonedaExplicit -> .into() -> Coyoneda -> .into() -> CoyonedaExplicit` produces same result after lowering.

### Verification

```
just verify
```

---

## Phase 4: Criterion Benchmarks

### New file: `fp-library/benches/benchmarks/coyoneda.rs`

Benchmark three approaches at chain depths 1, 10, 100 on a `Vec<i32>` of size 1000:

1. **Direct**: chain k calls to `map::<VecBrand, _, _>(f, v)`.
2. **Coyoneda**: `lift -> k maps -> lower`.
3. **CoyonedaExplicit**: `lift -> k maps -> lower`.

Follow the pattern in `benchmarks/vec.rs`: use `criterion::Criterion`,
`benchmark_group`, `bench_with_input`, `iter_batched` with `BatchSize::SmallInput`.

### Modify: `fp-library/benches/benchmarks.rs`

Add `mod coyoneda;` and include `coyoneda::bench_coyoneda` in the `criterion_group!`
macro.

### Verification

```
just bench -p fp-library --bench benchmarks -- Coyoneda
just verify
```

---

## Phase 5: CoyonedaNewLayer Optimization

Eliminate 1 box allocation from `Coyoneda::new`.

### Current (`coyoneda.rs:306-316`)

```rust
Coyoneda(Box::new(CoyonedaMapLayer {
    inner: Box::new(CoyonedaBase { fa: fb }),  // unnecessary extra box
    func: Box::new(f),
}))
```

3 boxes: outer Coyoneda, inner CoyonedaBase, func.

### Change

Add `CoyonedaNewLayer` struct (after `CoyonedaMapLayer` impl, around line 250) that
stores `fb` and `func` directly, implementing `CoyonedaInner` with a `lower` that
calls `F::map(self.func, self.fb)`.

Update `Coyoneda::new` to use `CoyonedaNewLayer` instead of wrapping a `CoyonedaBase`
inside a `CoyonedaMapLayer`. Result: 2 boxes instead of 3.

Existing tests at lines 614-627 cover `new` correctness.

### Verification

```
just verify
```

---

## Phase 6: map_then_bind Combinator

Add to `CoyonedaExplicit` in `coyoneda_explicit.rs` (after `bind`, around line 402):

```rust
pub fn map_then_bind<C: 'a>(
    self,
    f: impl Fn(A) -> CoyonedaExplicit<'a, F, C, C> + 'a,
) -> CoyonedaExplicit<'a, F, C, C>
where
    F: Semimonad, {
    let func = self.func;
    CoyonedaExplicit::lift(F::bind(self.fb, move |b| f(func(b)).lower()))
}
```

Key advantages over `bind`:

- Only requires `F: Semimonad` (not `F: Functor + Semimonad`).
- Skips the intermediate `F::map` call that `bind` performs via `self.lower()`.
- Composes the accumulated function directly into the bind callback.

### Tests

- `map_then_bind_option`: basic correctness.
- `map_then_bind_vec`: flatmap with pre-composed maps.
- `map_then_bind_equivalent_to_bind`: verify same result as `bind` for all cases.

### Verification

```
just verify
```

---

## Phase 7: Generic Function Type Parameter for CoyonedaExplicit

Redesign `CoyonedaExplicit` so the composed function is a generic type parameter
instead of `Box<dyn Fn>`, achieving truly zero-cost map fusion (no heap allocation,
no dynamic dispatch, compiler can inline the entire chain).

See feasibility analyses in `plans/functor/analysis/feasibility-01.md` through
`feasibility-04.md` for detailed investigation.

### 7a. Change the struct definition

Replace:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a>
where F: Kind_cdc7cd43dac7585f + 'a {
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
```

With:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a>
where F: Kind_cdc7cd43dac7585f + 'a {
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: Func,
}
```

No default on `Func`. The type is always inferred from context.

### 7b. Type alias for the boxed variant

Provide a nameable alias for users who need to store the value in struct fields or
collections:

```rust
pub type BoxedCoyonedaExplicit<'a, F, B, A> =
    CoyonedaExplicit<'a, F, B, A, Box<dyn Fn(B) -> A + 'a>>;
```

### 7c. Update method signatures

- **`lift`**: returns `CoyonedaExplicit<'a, F, A, A, fn(A) -> A>`. The identity
  function is a function pointer (`fn(A) -> A`), which is `Copy + Send + Sync +
'static` and zero-sized.
- **`new`**: returns `CoyonedaExplicit<'a, F, B, A, impl Fn(B) -> A + 'a>`.
- **`map`**: returns `CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a>`.
  Calls `compose(f, self.func)` without boxing.
- **`lower`**: takes `self`, calls `F::map(self.func, self.fb)`. Works for any `Func`.
- **`hoist`**: forwards `Func` unchanged since it only transforms `fb`.
- **`fold_map`**: takes `self`, composes `func` with the fold function. Works for
  any `Func`.
- **`apply`/`bind`**: lower both sides and re-lift, so they return the `fn(C) -> C`
  identity form. Signatures become generic over the input `Func` types but the return
  is concrete.
- **`map_then_bind`** (Phase 6): same approach; return type uses identity.
- **`into_coyoneda`**: takes any `Func: Fn(B) -> A`, passes `self.func` to
  `Coyoneda::new` which accepts `impl Fn(B) -> A + 'a`.
- **`pure`/`pointed`**: returns the `fn(A) -> A` form (same as `lift`).

### 7d. Add `.boxed()` and `.boxed_send()` escape hatches

```rust
impl<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> CoyonedaExplicit<'a, F, B, A, Func>
where F: Kind_cdc7cd43dac7585f + 'a {
    pub fn boxed(self) -> BoxedCoyonedaExplicit<'a, F, B, A> {
        CoyonedaExplicit {
            fb: self.fb,
            func: Box::new(self.func),
        }
    }

    pub fn boxed_send(self) -> CoyonedaExplicit<'a, F, B, A, Box<dyn Fn(B) -> A + Send + 'a>>
    where Func: Send {
        CoyonedaExplicit {
            fb: self.fb,
            func: Box::new(self.func),
        }
    }
}
```

`.boxed()` is the escape hatch for storing in collections, struct fields, or loop
accumulation. `.boxed_send()` gives thread safety.

### 7e. Update From impls (Phase 3)

- `From<CoyonedaExplicit<'a, F, B, A, Func>> for Coyoneda<'a, F, A>`: add `Func`
  parameter to the impl generics. `into_coyoneda` passes `self.func` to
  `Coyoneda::new(self.func, self.fb)` which works for any `Func: Fn(B) -> A + 'a`.
- `From<Coyoneda<'a, F, A>> for CoyonedaExplicit<'a, F, A, A, fn(A) -> A>`:
  the return type becomes concrete with `fn(A) -> A` since it goes through `lift`.

### 7f. Update documentation

The "zero-cost" claims from Phase 1 corrections become accurate again for the
unboxed path. Update the module docs and comparison table to reflect the new design:

| Property                | `Coyoneda`         | `CoyonedaExplicit`          |
| ----------------------- | ------------------ | --------------------------- |
| Heap allocation per map | 2 boxes            | 0 (or 1 box if `.boxed()`)  |
| Dynamic dispatch        | Yes                | No (or Yes if `.boxed()`)   |
| Stack overflow risk     | Yes (deep nesting) | Unlikely (compiler inlines) |

Document that `.boxed()` reintroduces one box per map and dynamic dispatch but
enables storage, loops, and HKT integration via `BoxedCoyonedaExplicit`.

### 7g. Send/Sync

No separate `SendCoyonedaExplicit` needed. The compiler derives `Send` automatically
when `Func: Send` and `F::Of<'a, B>: Send`. Document this.

### 7h. HKT brand (optional, can be deferred)

`CoyonedaExplicitBrand<F, B>` is feasible with the boxed variant only:

```rust
impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B> {
        type Of<'a, A: 'a>: 'a = BoxedCoyonedaExplicit<'a, F, B, A>;
    }
}
```

The `Functor` impl delegates to `map` followed by `.boxed()`. Fusion is preserved
(one `F::map` at `lower` time). This is strictly better than `CoyonedaBrand<F>` for
fusion. Can be deferred to a follow-up.

### Tests

- All existing tests must be updated for the extra type parameter (most will infer it).
- Loop-based tests (e.g., `many_chained_maps`) must use `.boxed()` since each
  iteration produces a different `Func` type.
- Add `test_boxed_erases_type`: verify `.boxed()` produces a uniform type.
- Add `test_boxed_send`: verify `.boxed_send()` produces a `Send` type.
- Add `test_send_auto_derived`: verify unboxed `CoyonedaExplicit` is `Send` when
  the closure and container are `Send`.

### Verification

```
just verify
```

---

## Phase Dependencies and PR Strategy

Phases 1-3 form a natural first PR (docs + Semimonad + conversions).
Phase 4 (benchmarks) is an independent second PR.
Phases 5-6 are independent enhancements, each a separate PR.
Phase 7 is a larger redesign PR that supersedes Phase 1's doc fixes for
`CoyonedaExplicit` (the "zero-cost" claims become accurate for the unboxed path).

| Phase                | Depends on | Risk                                           |
| -------------------- | ---------- | ---------------------------------------------- |
| 1 (docs)             | None       | None (docs only)                               |
| 2 (Semimonad)        | None       | Low                                            |
| 3 (From conversions) | None       | Low                                            |
| 4 (benchmarks)       | None       | None (additive)                                |
| 5 (CoyonedaNewLayer) | None       | Low (internal refactor)                        |
| 6 (map_then_bind)    | None       | Low (additive method)                          |
| 7 (generic Func)     | 3, 6       | Medium (changes struct, all signatures, tests) |

Phase 7 depends on Phases 3 and 6 being done first so it can update their
signatures in one pass. Phase 4 benchmarks should run before and after Phase 7
to measure the impact.

After each phase: `just verify` (runs fmt, clippy, doc, test in order).
