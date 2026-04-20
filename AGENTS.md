# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository. It contains tool-agnostic project knowledge that applies regardless of which AI tool or harness is being used.

When adding new AI assistant instructions, put tool-agnostic guidance here (project conventions, architecture, code style, build commands, constraints). Put tool-specific guidance (MCP servers, IDE integration, tool-specific behavioural directives) in the relevant tool's own configuration file (e.g., `CLAUDE.md` for Claude Code, `.cursor/rules` for Cursor).

## Project Overview

`fp-library` is a functional programming library for Rust that implements Higher-Kinded Types (HKT) using the Brand pattern (lightweight higher-kinded polymorphism). The library provides comprehensive type classes (Functor, Monad, Traversable, etc.) and profunctor-based optics while maintaining zero-cost abstractions.

**Key Design Principle:** The library uses uncurried semantics with `impl Fn` for zero-cost abstractions. Functions like `map(f, fa)` use static dispatch and avoid heap allocation, unlike curried `map(f)(fa)` which requires boxing closures.

## Running Commands

All commands must be run via `just` recipes defined in the project's [justfile](justfile). The `justfile` loads the Nix development environment via direnv automatically. Run `just --list` to see all available recipes.

**Never run `cargo` directly.** Always use `just <recipe>` or `just cargo <subcommand>` for non-standard cargo commands.

## Development Commands

### Formatting & Linting

```bash
just fmt     # Format all files (Rust, Nix, Markdown, YAML, TOML)
just clippy  # Run clippy (--workspace --all-targets --all-features by default)
```

### Documentation

```bash
just doc                                    # Check docs (--workspace --all-features --no-deps by default)
just doc --workspace --all-features --open  # Build and open docs
```

### Testing

**Never run `cargo test` directly.** Use `just test` which caches output and only re-runs when source files change.

```bash
just test                          # Run all tests (--workspace --all-features by default, cached)
just test -p fp-library test_name  # Run a subset (no caching)
just test -p fp-library -- prop_   # Run property-based tests (by name filter)
just test --doc -p fp-library      # Run doc tests
```

Cache location: `.cache/test-output/` (gitignored). Uses content hashing (`git ls-files` + `md5sum`) so the cache is invalidated only when tracked file contents change, not when timestamps change (e.g., from formatting or git operations). Re-running `just test` with no content changes is instant and prints cached output. Use `just clean` to clear the cache and build artifacts.

### Building

```bash
just build  # Build (--workspace --all-targets --all-features by default)
just check  # Check without building (--workspace --all-targets --all-features by default)
```

### Benchmarking

```bash
just bench                                             # Run all benchmarks
just bench -p fp-library --bench benchmarks -- --list  # List benchmarks
just bench -p fp-library --bench benchmarks -- Vec     # Run specific benchmark
# Benchmark reports: target/criterion/report/index.html
```

### Verification

After making changes, verify in this order: **fmt, check, clippy, deny, doc, test**.

```bash
just verify  # Runs all six steps in order
```

Or individually:

```bash
just fmt
just check
just clippy
just deny
just doc
just test
```

## Architecture

For detailed design documentation, see [fp-library/docs/](fp-library/docs/):

- [fp-library/docs/hkt.md](fp-library/docs/hkt.md) - Higher-Kinded Types and the Brand pattern
- [fp-library/docs/brand-inference.md](fp-library/docs/brand-inference.md) - Brand inference, trait shapes, Marker invariant, and inference resolution
- [fp-library/docs/dispatch.md](fp-library/docs/dispatch.md) - Val/Ref dispatch system
- [fp-library/docs/zero-cost.md](fp-library/docs/zero-cost.md) - Zero-cost abstractions and uncurried semantics
- [fp-library/docs/lazy-evaluation.md](fp-library/docs/lazy-evaluation.md) - Lazy evaluation types, trade-offs, and decision guide
- [fp-library/docs/pointer-abstraction.md](fp-library/docs/pointer-abstraction.md) - Pointer traits, `FnBrand<P>`, and `LazyConfig`
- [fp-library/docs/coyoneda.md](fp-library/docs/coyoneda.md) - Free functor implementations and trade-offs
- [fp-library/docs/parallelism.md](fp-library/docs/parallelism.md) - Thread safety and parallel trait hierarchy
- [fp-library/docs/features.md](fp-library/docs/features.md) - Full feature list with type class hierarchy
- [fp-library/docs/architecture.md](fp-library/docs/architecture.md) - Module organization and documentation conventions
- [fp-library/docs/optics-analysis.md](fp-library/docs/optics-analysis.md) - Optics system design details

### Key Locations

- [fp-library/src/brands.rs](fp-library/src/brands.rs) - All brand types centralized here (leaf nodes in dependency graph)
- [fp-library/src/kinds.rs](fp-library/src/kinds.rs) - `Kind` trait definitions and type application machinery
- [fp-macros/src/hkt/](fp-macros/src/hkt/) - Procedural macros (`trait_kind!`, `impl_kind!`, `Apply!`)
- [fp-library/src/dispatch/](fp-library/src/dispatch/) - Val/Ref dispatch traits, inference wrappers, and explicit functions
- [fp-macros/src/analysis/dispatch.rs](fp-macros/src/analysis/dispatch.rs) - Dispatch trait analysis for HM signature generation
- [fp-library/src/types/optics/](fp-library/src/types/optics/) - Profunctor-encoded optics (Lens, Prism, Iso, Traversal, etc.)

### Module Dependency Ordering

Respect the dependency graph: brands -> classes -> types -> dispatch -> functions. Never create cycles. Dispatch modules (e.g., `dispatch/functor.rs`) contain dispatch traits, Val/Ref impls, inference wrapper functions, and explicit submodules. Free functions without dispatch (e.g., `compose`, `identity`) are defined in `classes/` and re-exported in `functions.rs`. Inference wrappers are re-exported from `crate::dispatch::*`.

### Optics

Optics use profunctor encoding. Internal profunctors: `Exchange` (isos), `Market` (prisms), `Forget` (getters/folds), `Shop` (lenses). Many optics currently hard-code `Rc`; per `docs/todo.md`, these should be refactored to use `FnBrand<P>`. See [fp-library/docs/optics-analysis.md](fp-library/docs/optics-analysis.md) for design details.

## Code Style & Documentation

### Formatting

The codebase uses custom rustfmt rules ([rustfmt.toml](rustfmt.toml)):

- Hard tabs (`\t`) for indentation
- Vertical layout for function parameters and imports
- Grouped imports by `StdExternalCrate`
- Single import per line (`imports_granularity = "One"`)

This codebase uses hard tab characters (`\t`) for indentation, not spaces. When editing files, preserve the existing tab indentation exactly. Do not convert tabs to spaces or vice versa.

**Always run `just fmt` before committing.** A pre-commit hook also runs treefmt automatically.

### No Emoji or Unicode

Never use emoji or unicode symbols in code, comments, or documentation. Use plain text and ASCII equivalents:

- Status markers: `Yes`, `No`, `Partial`, `N/A` (not checkmarks or crosses).
- Arrows: `->`, `<-`, `<->` (not unicode arrows).
- Math: `>=`, `<=`, `!=` (not unicode math symbols).
- Section dividers: no box-drawing characters.

### AST Pattern Matching in Macros

When working on proc macro code in [fp-macros/](fp-macros/), always use `syn` AST pattern matching instead of stringly-typed processing:

- Use `path.get_ident()`, `path.is_ident()`, or segment matching to compare types and trait names. Do not use `quote!(#ty).to_string()` to stringify AST nodes for comparison.
- Store `syn::Ident` or `syn::Type` in data structures instead of stringified representations. Avoid round-tripping through strings (stringify then re-parse with `syn::parse_str`).
- `ident.to_string()` is acceptable for map keys, error messages, and final output (e.g., `HmAst::Variable`), but not as a substitute for structural matching.

### Documentation Standards

Functions must include:

````rust
/// Short description.
///
/// Comprehensive explanation.
///
#[document_signature]
#[document_type_parameters(
	"Description of first type parameter.",
	"Description of second type parameter.",
)]
#[document_parameters(
	"Description of first parameter.",
	"Description of second parameter.",
)]
#[document_returns(
	"Description of returned value.",
)]
#[document_examples]
///
/// ```
/// // Code showing function usage and containing assertions about expected outputs using assertion macros.
/// ```
````

### Commit Message Style

When creating commits:

1. Use conventional commit prefixes (`feat`, `fix`, `docs`, `refactor`, `bench`, `test`, `chore`, etc.).
2. Use imperative mood ("Add feature" not "Added feature").
3. Keep first line under 70 characters.
4. Follow existing commit message patterns in `git log`.
5. Do not include `Co-Authored-By` or other attribution trailers.

### Self-Contained Test Documentation

All test files (POCs, integration tests, non-regression tests, UI tests) must have self-contained documentation. Do not reference external plan documents, plan phase numbers (e.g., "phase 1", "phase 2"), review finding IDs (e.g., "M4", "H1"), or file paths that may not exist in the future. This includes `#[ignore]` reason strings and section-header comments.

When writing test file headers, explain the background in concrete terms (the actual Rust code pattern, the trait shapes, the expected compiler behaviour). For `#[ignore]` reasons, describe the technical prerequisite (e.g., "bind inference wrapper not yet migrated to InferableBrand") rather than a plan milestone (e.g., "phase 2").

## Feature Flags

- `rayon` - Enables parallel folding (`ParFoldable`) and `VecBrand` parallel execution
- `serde` - Enables serialization/deserialization for pure data types
- `stacker` - Enables adaptive stack growth for deep Coyoneda map chains

## Common Patterns

### Implementing a New Type Class

1. Define the trait in [fp-library/src/classes/](fp-library/src/classes/)`new_class.rs`. Class files contain only trait definitions (and optionally a `Ref` variant, e.g., `RefFunctor`), not free functions.
2. Update [fp-library/src/classes.rs](fp-library/src/classes.rs) to export the module.
3. Create a dispatch module at [fp-library/src/dispatch/](fp-library/src/dispatch/)`new_class.rs` containing:
   - A dispatch trait (e.g., `NewClassDispatch<'a, Brand, A, B, FA, Marker>`).
   - A `Val` impl that routes to the owned trait method.
   - A `Ref` impl that routes to the `Ref*` variant trait method.
   - An inference wrapper function (the public API, e.g., `pub fn map(f, fa)`) that binds on `InferableBrand` with `Marker` projected, enabling brand inference from the container type.
   - An `explicit` submodule with the turbofish-taking variant (e.g., `pub mod explicit { pub fn map(...) }`).
4. Update [fp-library/src/dispatch.rs](fp-library/src/dispatch.rs) to export the new dispatch module.
5. Re-export the functions in [fp-library/src/functions.rs](fp-library/src/functions.rs). This file is the primary public API and draws from three sources: inference wrappers from `dispatch::` (top-level re-exports), class-level free functions from `classes::` (for operations where Brand cannot be inferred, e.g., `pure`, `empty`), and type-specific utilities from `types::`. Add the inference wrapper to the `dispatch::` re-export block and the explicit variant to `pub mod explicit`.
6. Add documentation following the template above.

### Adding a Brand for a New Type

1. Add a zero-sized brand struct to [fp-library/src/brands.rs](fp-library/src/brands.rs) (derive `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`).
2. Use `impl_kind!` in [fp-library/src/types/](fp-library/src/types/)`new_type.rs` to define the `Kind` implementation. For types with multiple type parameters (like `Result`), use separate `impl_kind!` blocks for each partial application and mark brands with `#[multi_brand]`.
3. Implement relevant type classes (both owned and `Ref` variants, e.g., `Functor` and `RefFunctor`) in the same types file.

### Working with Optics

When modifying optics code:

- Optics use profunctor encoding; understand `Profunctor`, `Strong`, `Choice` traits.
- Internal profunctors (Exchange, Market, Shop, Forget, etc.) are in [fp-library/src/types/optics/](fp-library/src/types/optics/). The profunctor types are parameterized over `FunctionBrand: LiftFn`, but many optic functions still hard-code `Rc`; per `docs/todo.md`, these should be refactored to use `FnBrand<P>`.
- See [fp-library/docs/optics-analysis.md](fp-library/docs/optics-analysis.md) for design details.

### Thread-Safe Operations

For parallel/concurrent code:

1. Use `ArcFnBrand` (i.e., `FnBrand<ArcBrand>`) instead of `RcFnBrand` (i.e., `FnBrand<RcBrand>`).
2. Use `SendCloneFn` trait instead of `CloneFn`. `SendCloneFn` has `Val` and `Ref` modes via a `ClosureMode` parameter; use `send_lift_fn_new()` or `send_ref_lift_fn_new()` accordingly.
3. Use `ParFoldable` trait for parallel folding (requires `rayon` feature; falls back to sequential without it).
4. Ensure all captured data is `Send + Sync`.

## Testing Strategy

The library uses multiple testing approaches:

1. **Unit Tests**: Inline `#[test]` modules in source files
2. **Property-Based Tests**: QuickCheck for testing type class laws
3. **Compile-Fail Tests**: `trybuild` tests in `tests/` directories for error messages
4. **Doc Tests**: Examples in documentation comments are tested
5. **Benchmarks**: Criterion benchmarks in `benches/` to verify zero-cost abstractions

When adding new functionality, include all relevant test types.

## Development Environment

This project uses Nix for reproducible development environments:

```bash
# Enter dev shell (if using direnv)
direnv allow

# Or manually
nix develop
```

The Nix shell provides the correct Rust toolchain and dependencies.

### Git Worktrees

After creating a git worktree, run `direnv allow` inside it before running any toolchain commands. Without this, `direnv export` fails silently and the system toolchain is used instead of the project's Nix toolchain. Never suppress direnv errors with `2>/dev/null`.

### Ad-hoc Packages

If a tool or package is not available in the development environment, use `nix shell` to access it without modifying the flake:

```bash
# Run a command with a package from nixpkgs
nix shell nixpkgs#jq -c jq '.version' package.json

# Interactive shell with a package
nix shell nixpkgs#hyperfine
```

## Important Constraints

1. **No Dynamic Dispatch in Core Operations**: Functions like `map`, `bind`, `fold` use `impl Fn` for zero-cost abstractions. Only use `dyn Fn` for functions-as-data (e.g., `Semiapplicative::apply`, `Lazy` thunks, `Endofunction`).

2. **Uncurried Semantics**: All functions use uncurried style `f(a, b, c)` not `f(a)(b)(c)`. This is fundamental to the zero-cost design.

3. **Brand Types are Marker Types**: Brands like `OptionBrand` are zero-sized and used only at type-level. They're never instantiated.

4. **Lifetime Constraints**: `Trampoline` requires `'static`, `Thunk` and `Lazy` support arbitrary lifetimes `'a`.

5. **Module Dependency Ordering**: Respect the dependency graph: brands -> classes -> types -> dispatch -> functions. Never create cycles.
