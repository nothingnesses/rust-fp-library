# Brand Inference: Open Questions Investigation 2

Focus: the `_explicit` rename, macro changes, and migration path.

## 1. Blast Radius of the `_explicit` Rename

### Issue

The plan renames `map` to `map_explicit`, `bind` to `bind_explicit`, etc.
Every turbofish call site (`map::<Brand, ...>`) must be updated. How large
is this migration?

### Research Findings

Turbofish call-site counts across the entire codebase (source, doc
comments, tests, benchmarks), counted by `grep` for `name::<`:

| Function          | Total occurrences | Files |
| ----------------- | ----------------- | ----- |
| `map::<`          | 566               | 89    |
| `pure::<`         | 311               | 37    |
| `bind::<`         | 191               | 34    |
| `fold_map::<`     | 148               | 37    |
| `traverse::<`     | 127               | 27    |
| `apply::<`        | 121               | 30    |
| `fold_right::<`   | 95                | 28    |
| `lift2::<`        | 73                | 28    |
| `fold_left::<`    | 67                | 27    |
| `filter_map::<`   | 59                | 15    |
| `bimap::<`        | 47                | 7     |
| `compact::<`      | 43                | 14    |
| `alt::<`          | 33                | 6     |
| `lift3-5::<`      | 16                | 3     |
| `join::<`         | 3                 | 2     |
| `bind_flipped::<` | 2                 | 2     |

Total across all functions targeted for rename: roughly 1,900 call sites.

`pure` is NOT renamed per the plan (311 occurrences stay as-is), so the
actual rename workload is approximately 1,600 call sites.

However, only the sites that use concrete brands (e.g., `OptionBrand`)
need to switch to the new inference-based `map(...)` form. Sites using
generic brand parameters (e.g., inside `impl Functor for SomeType`) must
switch to `map_explicit` because inference cannot resolve a generic brand.
Both paths require touching the call site.

### Breakdown by Location

The call sites are distributed across three categories with different
migration characteristics:

**Internal trait implementations (`fp-library/src/types/`):**

- `map::<`: 319 occurrences across 25 files.
- `bind::<`: 120 occurrences across 13 files.
- `pure::<`: 253 occurrences across 14 files (not renamed, but relevant).
- `traverse::<`: 69 occurrences across 10 files.
- These are generic code that uses brand type parameters, not concrete
  brands. All must become `map_explicit`, `bind_explicit`, etc.

**Internal class definitions (`fp-library/src/classes/`):**

- `map::<`: 179 occurrences across 37 files.
- `bind::<`: 47 occurrences across 9 files.
- Mix of doc examples (concrete brands, can use inference) and generic
  trait default method implementations (must use `_explicit`).

**Tests, benchmarks, and doc comments (concrete brands):**

- `map::<`: 31 occurrences across 11 files (tests + benchmarks).
- These use concrete brands and can switch to inference-based `map(...)`,
  dropping the turbofish entirely.

### Approaches

**A. Big-bang rename with sed/IDE refactor.** Rename all at once using
a mechanical find-and-replace. Fast but risky; hard to verify each site
individually.

**B. Incremental per-function rename (plan's current approach).** Rename
one function at a time (map first, then bind, etc.). Each step is
independently verifiable. Slower but safer.

**C. Deprecation period with both names available.** Keep the old `map`
as a deprecated alias for `map_explicit` during a transition period.
This avoids a hard break but creates naming confusion since the new
`map` (inference-based) occupies the same name.

### Recommendation

Approach B is correct but the plan underestimates the effort. The
roughly 500 `map::<` sites in `types/` alone are a substantial
mechanical churn. Consider scripting the rename: a simple
`sed 's/map::</map_explicit::</g'` handles the turbofish sites, but doc
comment examples that use concrete single-brand types should be manually
updated to use the clean inference syntax to serve as documentation of
the new API. The plan should explicitly note the expected volume and
recommend tooling.

---

## 2. Internal Library Code Cannot Use DefaultBrand

### Issue

The plan acknowledges (section "Generic Code Still Requires Explicit
Brands") that generic HKT code must use explicit brands. But it does not
quantify the internal impact or discuss whether the `_explicit` suffix
creates ergonomic friction for library maintainers writing new type class
implementations.

### Research Findings

Internal generic call sites that must use `_explicit` after the rename:

| Function     | `types/` (impl code) | `classes/` (impl code) | Total internal |
| ------------ | -------------------- | ---------------------- | -------------- |
| `map`        | ~207                 | ~70                    | ~277           |
| `bind`       | ~120                 | ~47                    | ~167           |
| `traverse`   | ~69                  | ~46                    | ~115           |
| `fold_map`   | ~100+                | ~50+                   | ~150+          |
| `fold_right` | ~60+                 | ~30+                   | ~90+           |
| `filter_map` | ~30+                 | ~20+                   | ~50+           |

The `types/` directory alone contains roughly 207 non-doc-comment
`map::<` calls. The `classes/` directory contains roughly 70 non-doc
`map::<` calls. These are all in generic contexts and must become
`map_explicit`.

This means the vast majority of `map::<` call sites in the codebase
(roughly 500 out of 566) become `map_explicit::<`, not the clean `map()`.
The inference-based `map()` is primarily used in leaf-level application
code: tests, benchmarks, and doc examples.

### Approaches

**A. Accept the `_explicit` suffix everywhere internally.** The internal
code is verbose, but it is library implementation detail, not user-facing.
Users benefit from the clean names.

**B. Use internal module-level `use` aliases.** Files in `types/` and
`classes/` could `use crate::functions::map_explicit as map;` to keep the
short name locally. This preserves readability but creates confusing
shadowing if someone adds the inference `map` to the same scope.

**C. Provide a `map_brand` or `map_with_brand` name instead of
`map_explicit`.** This is slightly shorter and more descriptive than
`_explicit`. However, this is a naming bikeshed and the plan already
chose `_explicit`.

### Recommendation

Approach A. The internal code already uses turbofish; adding `_explicit`
makes it slightly longer but the intent is clearer. Avoid approach B
because shadowing the inference-based `map` with a `use` alias is
confusing and error-prone. The `_explicit` suffix correctly communicates
that the brand is being specified manually.

---

## 3. m_do!/a_do! Codegen Changes

### Issue

The macros currently generate `bind::<#brand, _, _, _, _>(...)` and
`map::<#brand, _, _, _, _>(...)` with hard-coded turbofish. After the
rename, the macros must generate different code depending on whether the
user provided an explicit brand or is using inferred mode.

### Research Findings

**Current m_do! codegen** (`fp-macros/src/m_do/codegen.rs`):

- Bind statements generate:
  `bind::<#brand, _, _, _, _>(container, move |param| { body })`
- Sequence statements generate the same with `_` as the discard pattern.
- `pure(expr)` is rewritten to `pure::<#brand, _>(expr)` (or
  `ref_pure::<#brand, _>(&(expr))` in ref mode) by the `PureRewriter`
  AST visitor.

**Current a_do! codegen** (`fp-macros/src/a_do/codegen.rs`):

- 0 binds: `pure::<#brand, _>(final_expr)`.
- 1 bind: `map::<#brand, _, _, _, _>(|param| body, expr)`.
- N binds: `liftN::<#brand, underscores...>(|params| body, exprs...)`.

**Changes needed for inferred mode:**

For m_do! in inferred mode:

- Bind: `bind(container, move |param| { body })` (no turbofish, no brand).
- Sequence: same pattern.
- `pure(expr)`: CANNOT be rewritten (no brand to inject). Must emit an
  error or leave as-is. The plan says users must write concrete
  constructors.

For m_do! in explicit mode:

- Bind: `bind_explicit::<#brand, _, _, _, _>(container, ...)` (renamed).
- `pure(expr)`: `pure::<#brand, _>(expr)` (unchanged, `pure` is not
  renamed).

For a_do! in inferred mode:

- 0 binds: ERROR (cannot generate `pure` without a brand).
- 1 bind: `map(|param| body, expr)` (no turbofish).
- N binds: `liftN(|params| body, exprs...)` (no turbofish).

For a_do! in explicit mode:

- 0 binds: `pure::<#brand, _>(final_expr)` (unchanged).
- 1 bind: `map_explicit::<#brand, _, _, _, _>(|param| body, expr)`.
- N binds: `liftN_explicit::<#brand, ...>(|params| body, exprs...)`.

**Complexity assessment:**

The codegen changes are moderate. The core logic is a conditional on
whether a brand is present:

```
if brand.is_some() {
    // Explicit mode: generate *_explicit::<Brand, ...>
} else {
    // Inferred mode: generate bare function call
}
```

The `DoInput` struct must change `brand: Type` to `brand: Option<Type>`.
The parser must handle the new syntax where `{` appears immediately
(no brand token). The codegen for each statement type adds one branch.

The `PureRewriter` must be gated: in explicit mode it rewrites `pure`
calls; in inferred mode it must either leave `pure` calls alone (causing
a compile error downstream) or emit a macro-level error at the call site
with a helpful message.

The `a_do!` 0-bind case must emit a compile error in inferred mode. This
is a new error path.

The `liftN` functions would need `liftN_explicit` counterparts. The
plan's Tier 1 lists `lift2`-`lift5` for the rename, so this is covered.

### Approaches

**A. Conditional codegen (plan's implied approach).** Add the brand
`Option<Type>` check at each codegen point. Simple and direct.

**B. Two separate codegen paths.** Duplicate the codegen functions into
explicit and inferred variants. Cleaner separation but more code to
maintain.

**C. Emit a trait-based wrapper.** Instead of different function names,
emit a trait method call that dispatches at compile time. Over-engineered.

### Recommendation

Approach A. The conditional is straightforward and keeps the codegen
unified. The key implementation detail the plan should specify: in
inferred mode, the `PureRewriter` should emit a `compile_error!` with a
message like "pure() requires an explicit brand; use m_do!(Brand { ... })
or write the concrete constructor (e.g., Some(expr))" rather than
silently leaving the bare `pure` call, which would produce an unhelpful
"cannot find function `pure`" error if `pure` is not in scope, or the
wrong behavior if `pure` resolves to some other item.

---

## 4. `pure` in Inferred-Mode Macros

### Issue

The plan says `pure(expr)` cannot be rewritten in inferred mode, and
users must write concrete constructors. But what exactly does the macro
currently do with `pure`, and what edge cases arise?

### Research Findings

**Current behavior** (`PureRewriter` in `codegen.rs`):

The `PureRewriter` is an AST visitor that walks the expression tree.
When it finds a call to a bare `pure` (not `Foo::pure`, not
`pure::<T>(...)`, no turbofish), it rewrites it:

- Val mode: `pure(args)` -> `pure::<Brand, _>(args)`.
- Ref mode: `pure(args)` -> `ref_pure::<Brand, _>(&(args))`.

The `is_bare_pure_call` function checks:

1. The function is a path expression (not a method call).
2. No `qself` (not `<T>::pure`).
3. No leading `::` (not `::pure`).
4. Exactly one path segment.
5. The segment is named `pure`.
6. No turbofish arguments.

**Where `pure` appears in m_do! blocks:**

`pure` is typically the final expression: `m_do!(Brand { x <- expr; pure(x + 1) })`.
It can also appear in bind expressions: `x <- pure(42);`.

**Edge cases for inferred mode:**

1. **Final expression is `pure(expr)`.** This is the most common case.
   In inferred mode, this fails because `pure` needs a brand. The user
   must write `Some(expr)` or `vec![expr]` instead. This is a usability
   regression: `pure` is a brand-agnostic way to lift a value, while
   `Some(expr)` ties the code to a specific type.

2. **`pure` inside a bind expression.** `x <- pure(42);` becomes
   `bind(pure(42), ...)`. The outer `bind` can infer the brand, but the
   inner `pure(42)` still cannot. The user must write `x <- Some(42);`.

3. **`pure` nested inside other expressions.** `pure(x).map(f)` or
   `if cond { pure(a) } else { pure(b) }`. The rewriter currently
   handles these via recursive AST visiting. In inferred mode, none of
   these can be rewritten.

4. **0-bind a_do! in inferred mode.** `a_do!({ 42 })` would generate
   `pure(42)`. The plan correctly identifies this as unsupported.

5. **a_do! with 1+ binds and `pure` in the body.** `a_do!({ x <- Some(5); pure(x + 1) })` generates
   `map(|x| { pure(x + 1) }, Some(5))`. The `pure(x + 1)` inside the
   closure body is the return value. In the current explicit mode, the
   rewriter converts it to `pure::<Brand, _>(x + 1)`. In inferred mode,
   this bare `pure` would not be rewritten. But actually, the user
   does not need `pure` here at all: `a_do!({ x <- Some(5); x + 1 })`
   generates `map(|x| { x + 1 }, Some(5))`, which is correct. The
   `pure` is only needed when the final expression is the sole item
   (0-bind case). With 1+ binds, the final expression is wrapped in a
   closure by `map`/`liftN`, so `pure` is unnecessary.

   However, for m_do!, the final expression IS returned as the monadic
   value. `m_do!({ x <- Some(5); x + 1 })` generates
   `bind(Some(5), move |x| { x + 1 })`, where the closure returns
   `i32`, not `Option<i32>`. This is a type error. The user must write
   `m_do!({ x <- Some(5); Some(x + 1) })`. This is the expected
   behavior per the plan.

### Approaches

**A. Emit compile_error! for bare `pure` in inferred mode.** When the
rewriter encounters `pure(...)` in inferred mode, replace it with
`compile_error!("pure() requires an explicit brand ...")`. This gives
a clear error at the exact location.

**B. Leave `pure` unrewritten (plan's implied approach).** The bare
`pure(expr)` call remains in the output. If `pure` is imported, the
compiler says "cannot infer type for Brand parameter." If not imported,
"cannot find function `pure`." Neither message is very helpful.

**C. Emit `pure(expr)` but rely on the inference-based `pure` being
available.** This does not work because `pure` has no container
argument and cannot infer the brand, as confirmed by the POC.

### Recommendation

Approach A. The macro has enough context to emit a targeted error message.
The implementation is simple: in the `PureRewriter`, check if the brand
is `None`, and if so, replace the `pure(...)` call with a
`compile_error!` invocation. This is better than approach B because it
gives the user actionable guidance ("use explicit brand syntax or write
the concrete constructor").

---

## 5. Incremental Migration Strategy

### Issue

The plan says the rename happens per-function in tiers. During the
transition, some functions have inference and some do not. Is there a
risk of API inconsistency?

### Research Findings

The tier structure from the plan:

- Tier 1: `map`, `bind`, `bind_flipped`, `lift2`-`lift5`, `filter_map`.
- Tier 2: `alt`, `compact`, `separate`, `join`, `extract`, `extend`, etc.
- Tier 3: `fold_right`, `fold_left`, `fold_map`, `traverse`, `apply`, etc.
- Tier 4: `bimap`, bifunctor traversals.

During the transition between tiers, the API would look like:

```rust
// After Tier 1, before Tier 2:
let y = map(|x| x + 1, Some(5));           // Inference (new)
let z = alt::<OptionBrand, _, _>(y, None);  // Explicit (old name)

// After Tier 2:
let z = alt(y, None);                       // Inference (new)
```

**Risk 1: Naming inconsistency.** Between tiers, `map` means "brand
inferred" but `fold_right` means "brand explicit." A user might try
`fold_right(f, init, container)` (no turbofish) expecting inference,
get a confusing error, and not realize that `fold_right` has not been
migrated yet.

**Risk 2: Documentation lag.** Doc examples across the codebase use the
explicit style. If only Tier 1 functions are updated, doc examples for
Tier 2+ functions still show the old style. A user reading the `map`
docs sees the clean API, then reads the `fold_right` docs and sees
turbofish, creating confusion about whether turbofish is needed.

**Risk 3: Internal consistency during partial migration.** Within a
single `impl` block, some calls use `map_explicit` (already migrated)
and others use `fold_right` (not yet migrated, still has the old name).
When Tier 3 migrates, `fold_right` becomes `fold_right_explicit`. The
same `impl` block is touched twice.

### Approaches

**A. Accept the inconsistency (plan's current approach).** Each tier is
independently verifiable. Users of Tier 1 functions benefit immediately.
The inconsistency is temporary.

**B. Rename all functions to `_explicit` in one pass, then add inference
wrappers tier by tier.** This front-loads the rename pain (mechanical,
scriptable) and then each tier adds the new inference-based function
without renaming anything. After the big rename, all explicit-brand
functions consistently have the `_explicit` suffix, even if some do not
yet have an inference counterpart.

**C. Add inference wrappers with temporary `_infer` suffix, then do one
final name swap.** Each tier adds `map_infer`, `bind_infer`, etc. Only
at the end does the final rename happen (`map` -> `map_explicit`,
`map_infer` -> `map`). This avoids partial inconsistency but means
the clean names are not available until the very end.

### Recommendation

Approach B. Front-loading the rename eliminates the risk of touching the
same files multiple times across tiers. The rename from `map` to
`map_explicit` is mechanical and scriptable (roughly 1,600 sites). Once
done, adding inference-based `map` in Tier 1 is a pure addition with no
further renames needed. This also avoids the confusing state where `map`
and `fold_right` have different semantics (inferred vs explicit).

The key trade-off: approach B creates a brief period where ALL functions
have the `_explicit` suffix but no inference counterparts exist yet. This
is a breaking change to the entire API surface. However, since this is a
pre-1.0 library, this is acceptable. The plan should make this sequencing
explicit.

---

## 6. Re-Export Mechanism

### Issue

After the rename, `functions.rs` needs to export both `map` (inference)
and `map_explicit` (explicit). How does this interact with the
`generate_function_re_exports!` macro and the manual `pub use` block?

### Research Findings

`functions.rs` uses two re-export mechanisms:

1. **`generate_function_re_exports!` macro.** Scans `src/classes/*.rs`
   files, finds public functions, and generates `pub use` statements. It
   uses an alias map for name conflicts (e.g., `"category::identity"` ->
   `category_identity`). This macro handles functions defined directly in
   `classes/` modules (like `pure`, `alt`, `compact`, `fold_map` in their
   respective trait modules).

2. **Manual `pub use` block.** Explicitly re-exports dispatch functions
   (`map`, `bind`, `lift2`, `fold_right`, etc.) from
   `crate::classes::dispatch::*` because the dispatch sub-modules are not
   scanned by the macro.

**Current manual re-exports from dispatch:**

```rust
pub use crate::classes::dispatch::{
    bind, bind_flipped, compose_kleisli, compose_kleisli_flipped,
    filter_map, fold_left, fold_map, fold_right,
    lift2, lift3, lift4, lift5, map, traverse,
};
```

**After the rename:**

The dispatch module will contain both `map` (inference) and
`map_explicit` (explicit). The manual `pub use` block must export both.
This is straightforward: add `map_explicit` to the list.

For non-dispatch functions handled by the macro (e.g., `alt`, `compact`),
the macro scans for public functions in the module files. After the
rename, each module will have both `alt` (inference) and `alt_explicit`
(explicit). The macro will find both and re-export both. No conflict
arises because they have different names.

**Potential naming conflict:** The `generate_function_re_exports!` macro
currently handles conflicts via the alias map (e.g., `"filterable::filter_map"` -> `filterable_filter_map`, because `filter_map` also
exists in `dispatch`). After the rename:

- `dispatch::filter_map` becomes the inference version.
- `dispatch::filter_map_explicit` becomes the explicit version.
- `filterable::filter_map` (the trait method free function) may or may
  not be renamed. If it is also renamed to `filter_map_explicit`, the
  alias `filterable_filter_map` needs updating to
  `filterable_filter_map_explicit`. If it is not renamed (because it is
  not a dispatch function), the alias stays but now `filter_map` in
  `dispatch` has a different signature than `filter_map` in `filterable`.

This highlights a broader issue: the plan focuses on dispatch-based
functions but some functions exist in both dispatch and non-dispatch
forms. The non-dispatch versions (trait method wrappers like
`filterable::filter_map`) have a different signature (explicit `Brand`
parameter, no `FA` dispatch). These are currently aliased to avoid
conflicts. After the rename, the relationship between the dispatch and
non-dispatch versions must be clarified.

**The `traversable::traverse` alias:** Currently aliased to
`traversable_traverse` because `dispatch::traverse` takes the clean
name. After the rename, `dispatch::traverse` becomes the inference
version, `dispatch::traverse_explicit` is the explicit version, and
`traversable::traverse` (the non-dispatch version) keeps its alias. This
works but the alias `traversable_traverse` now points to the
explicit-style function, which might confuse users who expect all
`_explicit` functions to have that suffix.

### Approaches

**A. Update re-exports mechanically.** Add `_explicit` variants to the
manual `pub use` block and let the macro pick up the rest. Accept that
some non-dispatch functions keep their original names.

**B. Rename non-dispatch trait free functions too.** If `filterable::filter_map` is the non-dispatch version, also rename it
to `filterable::filter_map_explicit`. This maintains consistency but
increases the blast radius. These non-dispatch versions are internal
helpers rarely used directly by consumers.

**C. Remove non-dispatch free function re-exports entirely.** Since
dispatch versions supersede them, stop re-exporting the non-dispatch
versions. Users who need the non-dispatch version can import from the
trait module directly.

### Recommendation

Approach A for the initial migration, with a note to revisit approach C
later. The non-dispatch free functions (like `filterable::filter_map`,
`traversable::traverse`) predate the dispatch system and are largely
superseded by their dispatch counterparts. Renaming them adds churn
without clear benefit. The alias mechanism already prevents conflicts.
The plan should document that the non-dispatch versions keep their
original names and are not affected by the `_explicit` rename, and that
only the dispatch versions (which are the primary API) get the
inference/explicit split.
