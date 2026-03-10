# Plan: Convert Validation to Warnings & Add `impl Trait` Lint

## Context

`#[document_module]`'s validation pass (Pass 1.5) currently emits `compile_error!` for missing documentation attributes, which stops compilation. This should instead emit compile-time **warnings** so code continues to compile. Additionally, a new lint should detect named generic type parameters that could be `impl Trait` per the guidelines in `docs/impl-trait-vs-named-generics.md`.

Both the existing validation checks and the new lint should share the same warning emission system, using `proc-macro-warning` which emits warnings via `#[deprecated]` under the hood.

**Known limitation:** Warnings appear as deprecation notices:
```
warning: use of deprecated constant `..._fp_macros_warning_...`: <message>
```

## Overview

1. Add `proc-macro-warning` dependency
2. Create a shared `WarningEmitter` in `core/`
3. Refactor existing validation from `ErrorCollector`/`syn::Error` to `WarningEmitter`
4. Add `impl Trait` lint detection module in `analysis/`
5. Integrate the lint into the validation pass
6. Add `#[allow_named_generics]` suppression attribute

## Step 1: Add Dependency

**File:** [Cargo.toml](fp-macros/Cargo.toml)

Add `proc-macro-warning = "1"` under `[dependencies]`.

## Step 2: Create Shared Warning Emitter

**New file:** [warning_emitter.rs](fp-macros/src/core/warning_emitter.rs)

A shared warning collector analogous to `ErrorCollector` but producing `proc-macro-warning` tokens instead of `compile_error!`:

```rust
pub struct WarningEmitter {
		counter: usize,
		warnings: Vec<TokenStream>,
}

impl WarningEmitter {
		pub fn new() -> Self;
		pub fn warn(&mut self, span: Span, message: impl Into<String>);
		pub fn is_empty(&self) -> bool;
		pub fn into_tokens(self) -> Vec<TokenStream>;
}
```

`warn()` creates a `FormattedWarning::new_deprecated` with a unique name (`_fp_macros_warning_{counter}`) and appends its token stream. The single counter ensures unique names across all checks in one macro expansion.

Register in [core.rs](fp-macros/src/core.rs) or the core module.

## Step 3: Refactor Existing Validation to Use Warnings

**File:** [document_module.rs](fp-macros/src/documentation/document_module.rs)

### 3a. Change validation functions to use `WarningEmitter`

All validation functions currently take `warnings: &mut ErrorCollector` and call `warnings.push(syn::Error::new(span, msg))`. Change them to take `warnings: &mut WarningEmitter` and call `warnings.warn(span, msg)`:

- `validate_no_duplicate_doc_attrs` — e.g. "Method `map` has `#[document_signature]` applied 2 times"
- `validate_doc_attr_order` — e.g. "`#[document_parameters]` before `#[document_signature]`"
- `validate_method_documentation_core` — missing `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`
- `validate_container_documentation` — missing attrs on impl/trait blocks
- `validate_impl_documentation`, `validate_trait_documentation`, `validate_fn_documentation`

### 3b. Change `validate_documentation` return type

Currently returns `Vec<syn::Error>`. Change to accept `&mut WarningEmitter`:

```rust
fn validate_documentation(items: &[Item], emitter: &mut WarningEmitter)
fn validate_nested_modules(items: &[Item], emitter: &mut WarningEmitter)
```

### 3c. Update `document_module_worker` integration

```rust
let warning_tokens: Vec<TokenStream> = if validation_mode != ValidationMode::Off {
		let mut emitter = WarningEmitter::new();
		validate_documentation(&items, &mut emitter);
		validate_nested_modules(&items, &mut emitter);
		lint_impl_trait(&items, &mut emitter);       // new
		lint_impl_trait_nested(&items, &mut emitter); // new
		emitter.into_tokens()
} else {
		Vec::new()
};
```

## Step 4: Create `impl Trait` Lint Detection

**New file:** [impl_trait_lint.rs](fp-macros/src/analysis/impl_trait_lint.rs)

### Detection Algorithm

For each `GenericParam::Type` in a function signature:

1. **Has trait bounds?** Collect from inline bounds and where clause. Skip if only lifetime bounds or none.
2. **Appears exactly once in parameter types?** Walk each `FnArg` (skip receivers). Use `contains_type_param()` which recurses through `syn::Type` nodes and parses `Apply!` macros via `get_apply_macro_parameters`. Count distinct parameter positions containing the ident.
3. **Absent from return type?** Walk `sig.output` for the ident.
4. **Not cross-referenced?** Check if any *other* type parameter's bounds (inline or where clause) mention this ident. E.g., `where G: SomeTrait<F>` cross-references `F`.

If all conditions pass, it's a candidate.

### Examples

**Should warn:**
```rust
// F appears once in params, not in return type, no cross-refs
pub fn new<F>(f: F) -> Self where F: FnOnce() -> A + 'a
//  → "could use `impl FnOnce() -> A + 'a`"

pub fn map<B: 'static, F>(self, func: F) -> Trampoline<B>
where F: FnOnce(A) -> B + 'static
//  → "F could use `impl FnOnce(A) -> B + 'static`"

fn apply<R: Monoid, F: Fn(A) -> R + 'a>(&self, f: F, s: S) -> R
//  → "F could use `impl Fn(A) -> R + 'a`"
```

**Should NOT warn:**
```rust
// T in return type
fn identity<T>(x: T) -> T

// T in multiple positions
fn combine<T: Semigroup>(a: T, b: T) -> T

// F used as turbofish argument to inner named fn (false positive - requires suppression)
fn tail_rec_m<S, F>(f: F, initial: S) -> Self
where F: Fn(S) -> Trampoline<Step<S, A>> + Clone + 'static
// Inner fn: go::<A, S, F>(f, initial) — needs F nameable

// T has no trait bounds
fn wrap<T>(x: T) -> Box<T>
```

### Public API

```rust
pub struct ImplTraitCandidate {
		pub param_name: String,
		pub param_span: Span,
		pub bounds_display: String,
}

pub fn find_impl_trait_candidates(sig: &Signature) -> Vec<ImplTraitCandidate>;
```

### Helper: `contains_type_param`

```rust
fn contains_type_param(ty: &syn::Type, name: &str) -> bool
```

Recursive walk over `syn::Type`:
- `Type::Path` — check ident, recurse into generic args
- `Type::Macro` — parse `Apply!` via `get_apply_macro_parameters` from [patterns.rs](fp-macros/src/analysis/patterns.rs), recurse into each arg type
- `Type::Reference` — recurse into element
- `Type::Tuple` — recurse into each element
- `Type::ImplTrait` / `Type::TraitObject` — recurse into bounds
- `Type::BareFn` — recurse into inputs and output
- `Type::Array` / `Type::Slice` — recurse into element

Register in [analysis.rs](fp-macros/src/analysis.rs): `pub mod impl_trait_lint;`

## Step 5: Integrate Lint into Validation Pass

**File:** [document_module.rs](fp-macros/src/documentation/document_module.rs)

Add two functions:

```rust
fn lint_impl_trait(items: &[Item], emitter: &mut WarningEmitter)
fn lint_impl_trait_nested(items: &[Item], emitter: &mut WarningEmitter)
```

These walk all impl blocks, traits, and free functions. For each method/function:
1. Check for `#[allow_named_generics]` — skip if present
2. Call `find_impl_trait_candidates(&sig)`
3. For each candidate, call `emitter.warn(span, message)`

## Step 6: Add Suppression Attribute

**File:** [constants.rs](fp-macros/src/core/constants.rs)

Add in `attributes` module:
```rust
pub const ALLOW_NAMED_GENERICS: &str = "allow_named_generics";
```

Add to `DOCUMENT_SPECIFIC_ATTRS` array so it's stripped from output.

## Key Files

| File | Change |
|---|---|
| [Cargo.toml](fp-macros/Cargo.toml) | Add `proc-macro-warning` dependency |
| [warning_emitter.rs](fp-macros/src/core/warning_emitter.rs) | **New:** shared warning emission |
| [impl_trait_lint.rs](fp-macros/src/analysis/impl_trait_lint.rs) | **New:** detection logic + unit tests |
| [document_module.rs](fp-macros/src/documentation/document_module.rs) | Refactor validation to warnings, add lint integration |
| [analysis.rs](fp-macros/src/analysis.rs) | Register new module |
| [constants.rs](fp-macros/src/core/constants.rs) | Add `ALLOW_NAMED_GENERICS` |
| [patterns.rs](fp-macros/src/analysis/patterns.rs) | Reuse: `get_apply_macro_parameters` |

## Verification

1. `cargo check --workspace` — no new compile errors
2. `cargo test -p fp-macros` — new unit tests pass, existing tests pass
3. `cargo test --workspace --all-features` — no regressions
4. Existing validation messages now appear as warnings instead of errors
5. Temporarily revert one `impl Trait` function to named generic to verify lint warning appears
