# Architecture & Design Decisions

This document records architectural decisions and design patterns used in `fp-library`.

For the module layout and dependency graph, see [Project Structure](./project-structure.md).
For the type class hierarchy diagrams, see [Features](./features.md).

## 1. Module Organization

### 1.1. Brand Structs (Centralized)

**Decision:**

Brand structs (e.g., `OptionBrand`) are **centralized** in `src/brands.rs`.

**Reasoning:**

- **Leaf Nodes:** In the dependency graph, Brand structs are **leaf nodes**; they have no outgoing edges (dependencies) to other modules in the crate.
- **Graph Stability:** Centralizing these leaf nodes in `brands.rs` creates a stable foundation. Higher-level modules (like `types/*.rs`) can import from this common sink without creating back-edges or cycles.

### 1.2. Free Functions (Three Layers)

**Decision:**

Free functions exist in three layers, each adding a level of inference:

1. **`classes/`**: Brand-explicit functions with no dispatch. Defined in
   their trait's module (e.g., `classes/functor.rs` defines
   `map_explicit`). Callers specify the brand via turbofish.
2. **`dispatch/`**: Brand-explicit functions with Val/Ref dispatch.
   Defined in `dispatch/functor.rs` etc. The closure's argument type
   (owned vs borrowed) selects the by-value or by-reference trait.
3. **`functions/`**: Brand-inference wrappers. Defined in
   `functions/functor.rs` etc. The brand is inferred from the container
   type via `InferableBrand`. These are the primary user-facing API.

`functions.rs` acts as a **facade**, re-exporting inference wrappers as
the bare names (`map`, `bind`, etc.) and dispatch functions as `_explicit`
variants (`map_explicit`, `bind_explicit`, etc.).

**Reasoning:**

- **Primary API is inference-based:** Users write `map(f, Some(5))`
  with no turbofish. The `_explicit` variants are the escape hatch for
  multi-brand types like `Result`.
- **Downstream dependencies:** Each layer depends on the one below it
  (`functions/ -> dispatch/ -> classes/`), matching the module
  dependency graph without creating cycles.
- **Cycle prevention:** `functions.rs` also contains generic helpers
  (like `compose`) which are leaf nodes. Defining the downstream
  wrappers directly in `functions.rs` would create bidirectional
  dependencies if a trait module ever needed to import a helper.
- **Facade pattern:** `functions.rs` re-exports from all three layers
  to provide a unified API surface without coupling the underlying
  definition graph.

## 2. Type Class Hierarchy Design

For the hierarchy diagrams, see [Features](./features.md).

### 2.1. Composite Traits as Blanket Impls

**Decision:**

`Applicative`, `Monad`, `Alternative`, `MonadPlus`, and `Comonad` are
marker traits with blanket implementations over their component traits.
They define no methods of their own.

**Reasoning:**

- **Compositional:** A type becomes `Applicative` automatically by
  implementing `Pointed + Semiapplicative + ApplyFirst + ApplySecond`.
  No additional code is required.
- **Mirrors theory:** Matches the PureScript/Haskell pattern where
  composite classes are defined purely by superclass constraints
  (e.g., `class (Pointed f, Semiapplicative f) => Applicative f`).
- **Reduced boilerplate:** Implementors only write the fundamental
  operations; composite traits are derived for free.

### 2.2. Micro-Traits (Lift, ApplyFirst, ApplySecond)

**Decision:**

`Lift`, `ApplyFirst`, and `ApplySecond` are separate traits rather than
methods on `Semiapplicative`.

**Reasoning:**

- **Semantic granularity:** `Lift` provides the fundamental operation
  (`lift2`, `lift3`, etc.) for lifting uncurried functions into a
  functor. `ApplyFirst` and `ApplySecond` are sequencing combinators
  with default implementations via `Lift`.
- **Independent supertraits:** `Applicative` requires all four
  (`Pointed + Semiapplicative + ApplyFirst + ApplySecond`) as equal
  parents. This makes the hierarchy explicit rather than hiding
  `ApplyFirst`/`ApplySecond` inside `Semiapplicative`.
- **Overridable defaults:** Types can override `ApplyFirst`/`ApplySecond`
  independently for performance without touching `Lift`.

### 2.3. Pointed is Separate from Applicative

**Decision:**

`Pointed` (providing `pure`/`of`) is an independent trait, not part of
`Semiapplicative`.

**Reasoning:**

- **Follows PureScript's hierarchy:** `Pointed` is an independent
  superclass of `Applicative`, not bundled with `Apply`.
- **Avoids the "Why not Pointed?" problem:** In Haskell, `pure` is
  bundled into `Applicative`, meaning you cannot have `pure` without
  `<*>`. Separating them allows types to implement `Pointed` alone
  (wrapping values) without committing to `Semiapplicative`.
- **Blanket composition:** Keeping `Pointed` separate makes the
  `Applicative` blanket impl a clean intersection of four traits.

### 2.4. Compactable is Separate from Filterable

**Decision:**

`Compactable` (`compact`, `separate`) is a standalone trait.
`Filterable` extends `Compactable + Functor`.

**Reasoning:**

- **Staged capability:** `Compactable` provides the minimal operations
  (unwrap nested `Option`s, split `Result`s) without requiring `Functor`.
  `Filterable` adds predicate-based operations (`filter`, `partition`,
  `filter_map`, `partition_map`) that depend on mapping.
- **Default implementations flow downward:** `filter_map` defaults to
  `map` then `compact`; `filter` defaults to `filter_map` with a
  predicate. The staging makes these defaults natural.

### 2.5. Ref\* Hierarchy is Independent, Not a Subtrait

**Decision:**

`RefFunctor`, `SendRefFunctor`, etc. are independent traits, not
subtraits of each other or of the base hierarchy.

**Reasoning:**

- **Send/Sync incompatibility:** `ArcLazy::new` requires `Send` on the
  closure, which a generic `RefFunctor` cannot guarantee. As a result,
  `ArcLazy` implements only `SendRefFunctor` (not `RefFunctor`), and
  `RcLazy` implements only `RefFunctor` (not `SendRefFunctor`).
  Making `SendRefFunctor: RefFunctor` would prevent `ArcLazy` from
  implementing it without also implementing `RefFunctor`, which is
  unsound for `Rc`-based internals.
- **Separate from base traits:** By-reference operations return different
  types (e.g., `ref_map` returns `Brand::Of<B>` from `&Brand::Of<A>`,
  not from an owned value). This is a fundamentally different signature,
  not a refinement of `Functor::map`.

### 2.6. Par\* Hierarchy Mirrors the Sequential Hierarchy

**Decision:**

`ParFunctor`, `ParFoldable`, `ParCompactable`, `ParFilterable`, etc.
are separate traits that mirror the sequential hierarchy with
`Send + Sync` bounds throughout.

**Reasoning:**

- **Avoids over-constraining:** Adding `Send + Sync` to the base
  hierarchy would prevent non-thread-safe types (anything using `Rc`,
  `Cell`, etc.) from implementing `Functor`.
- **Feature-gated execution:** With the `rayon` feature, `par_*`
  functions use true parallel execution. Without it, they degrade to
  sequential equivalents. The trait hierarchy is identical either way;
  only the implementation changes.
- **Composable with Ref axis:** The `ParRef*` variants combine both
  the parallel and by-reference axes, giving four combinations total:
  base, `Ref*`, `Par*`, `ParRef*`.

## 3. Documentation & Examples

Documentation structure (sections, headings, parameter ordering) is enforced by the documentation macros in `fp-macros/src/documentation/` (`#[document_module]`, `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`). All modules should use `#[document_module]`. See `fp-macros/src/lib.rs` for usage.

This section covers content quality guidelines that the macros cannot enforce.

### 3.1. Type Signature Content

The type signature generated by `#[document_signature]` must accurately reflect the code. Contributors should verify:

- Signatures correctly indicate uncurried semantics (the library's standard).
- Brand type parameters are replaced with their corresponding concrete types for clarity. Write `Result e` not `ResultWithErrBrand e`.
- Quantifiers are accurate, correctly ordered (matching the code), and omit unused variables for clarity.

### 3.2. Example Content

Examples should demonstrate the library's intended usage patterns:

- Import items using grouped wildcards (`use fp_library::{brands::*, functions::*}`) instead of individually by name.
- For types with a single unambiguous brand (Option, Vec, Identity, etc.), use inference-based free functions without turbofish: `map(|x| x * 2, Some(5))`.
- For types with multiple brands (Result at arity 1, Tuple2, Pair), use `_explicit` variants with turbofish: `map_explicit::<ResultErrAppliedBrand<E>, _, _, _, _>(f, x)`.
- Prefer free functions over trait method calls (`OptionBrand::map(...)`).

**Reasoning:** The library is designed to be used via free functions with brand inference for the common single-brand case. The `_explicit` variants are the escape hatch for ambiguous types. Examples should demonstrate the inference-based API as the primary path.

## 4. Lint Policy

### 4.1. Restriction Lints

The workspace enables several clippy restriction lints as warnings (promoted to errors by `-D warnings` in CI):

- `clippy::unwrap_used`, `clippy::expect_used` - panicking unwrap/expect
- `clippy::indexing_slicing` - panicking index/slice
- `clippy::panic`, `clippy::todo`, `clippy::unimplemented`, `clippy::unreachable` - explicit panics

These lints are appropriate for production library code but overly strict for test and benchmark code. Test code suppresses them with `#[expect(...)]` rather than `#[allow(...)]`.

### 4.2. `#[expect]` vs `#[allow]`

**Decision:** Use `#[expect(...)]` everywhere the suppressed lint is known to fire. Reserve `#[allow(...)]` only where the lint does not currently fire but suppression is kept for correctness (e.g., `dead_code` on items consumed by macro expansion, `deprecated` on modules testing deprecation-based warnings).

**Reasoning:** `#[expect]` warns when the suppression becomes unnecessary (the lint no longer fires), preventing stale attributes from accumulating. `#[allow]` is silent when unused, so stale `#[allow]` attributes persist undetected.

### 4.3. Reasons on All Lint Attributes

**Decision:** Every `#[expect(...)]` and `#[allow(...)]` attribute must include a `reason = "..."` string.

**Reasoning:** Reasons make the intent self-documenting. Without them, a reader must infer why the lint was suppressed, which is error-prone for restriction lints where the suppression could be masking a real bug.

### 4.4. Suppression Scope

- **Inline test modules** (`#[cfg(test)] mod tests`): Place `#[expect(...)]` on the `mod tests` block, listing only the restriction lints that actually fire within that module.
- **Integration test files** (`tests/*.rs`): Use `#![expect(...)]` as an inner attribute at the file top.
- **Benchmark modules**: Use `#[expect(...)]` on the module declaration in `benchmarks.rs`. Benchmarks that intentionally use identity operations or `and_then` instead of `map` (for fair std-vs-fp comparison) suppress `identity_op`, `bind_instead_of_map`, etc.
- **Production code**: Suppress on the narrowest scope possible (individual statement or function), with a comment or reason explaining the safety invariant.
