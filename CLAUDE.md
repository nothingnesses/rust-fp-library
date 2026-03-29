# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`fp-library` is a functional programming library for Rust that implements Higher-Kinded Types (HKT) using the Brand pattern (lightweight higher-kinded polymorphism). The library provides comprehensive type classes (Functor, Monad, Traversable, etc.) and profunctor-based optics while maintaining zero-cost abstractions.

**Key Design Principle:** The library uses uncurried semantics with `impl Fn` for zero-cost abstractions. Functions like `map(f, fa)` use static dispatch and avoid heap allocation, unlike curried `map(f)(fa)` which requires boxing closures.

## Running Commands

All commands must be run via `just` recipes defined in the project's `justfile`. The `justfile` loads the Nix development environment via direnv automatically. Run `just --list` to see all available recipes.

**Never run `cargo` directly.** Always use `just <recipe>` or `just cargo <subcommand>` for non-standard cargo commands.

## Development Commands

### Formatting & Linting

```bash
just fmt --all                              # Format code (rustfmt)
just fmt --all -- --check                   # Check formatting
just clippy --workspace --all-features      # Run clippy
```

### Documentation

```bash
just doc --workspace --all-features --no-deps   # Check docs (must produce zero warnings)
just doc --workspace --all-features --open       # Build and open docs
```

### Testing

**Never run `cargo test` directly.** Use `just test` which caches output and only re-runs when source files change.

```bash
just test                                        # Run all tests (cached)
just test-force                                  # Force re-run (ignores cache)
just test-subset -p fp-library test_name         # Run a subset (always runs, no cache update)
just test-subset -p fp-library --test property   # Run property-based tests
just test-subset --doc -p fp-library             # Run doc tests
```

Cache location: `.claude/test-cache/` (gitignored). Invalidated automatically when `.rs` files or `Cargo.toml` change.

### Building

```bash
just build --workspace                           # Build the workspace
just build -p fp-library --all-features          # Build with all features
just check --workspace                           # Check without building
```

### Benchmarking

```bash
just bench -p fp-library                                   # Run all benchmarks
just bench -p fp-library --bench benchmarks -- --list      # List benchmarks
just bench -p fp-library --bench benchmarks -- Vec         # Run specific benchmark
# Benchmark reports: target/criterion/report/index.html
```

### Verification

After making changes, verify in this order: **fmt, clippy, doc, test**.

```bash
just verify    # Runs all four steps in order
```

Or individually:

```bash
just fmt --all
just clippy --workspace --all-features
just doc --workspace --all-features --no-deps
just test
```

## Language Server & Code Intelligence

This repository has rust-analyzer configured via MCP (Model Context Protocol). Claude Code can use the LSP tool to access:

- **Type information** - Use `LSP` with `operation: "hover"` to get detailed type info, documentation, and trait implementations
- **Go to definition** - Navigate to where symbols are defined with `operation: "goToDefinition"`
- **Find references** - Find all uses of a symbol with `operation: "findReferences"`
- **Document symbols** - Get file structure with `operation: "documentSymbol"`
- **Workspace symbols** - Search across the codebase with `operation: "workspaceSymbol"`
- **Go to implementation** - Find trait implementations with `operation: "goToImplementation"`
- **Call hierarchy** - Analyze caller/callee relationships with `operation: "prepareCallHierarchy"`, `"incomingCalls"`, `"outgoingCalls"`

**When to use:** The LSP tool is especially valuable in this codebase due to:
1. Complex HKT machinery with Brand types and associated types
2. Heavy use of generic type parameters and trait bounds
3. Profunctor encodings in the optics system
4. Type-level programming that can be hard to trace manually

**Example:**
```
LSP with operation="hover", filePath="fp-library/src/types/optics/lens.rs", line=42, character=15
```

This provides rich type information that helps understand the library's complex type system without manually tracing through trait definitions.

## Architecture

### Higher-Kinded Types via the Brand Pattern

The library implements HKT using type-level defunctionalization. Each type constructor (e.g., `Option<T>`) has a corresponding `Brand` type (e.g., `OptionBrand`) that acts as a witness for the `Kind` trait mapping.

**Example:**
```rust
pub struct OptionBrand;
impl_kind! {
    for OptionBrand {
        type Of<'a, A: 'a>: 'a = Option<A>;
    }
}
```

**Key Locations:**
- `fp-library/src/brands.rs` - All brand types centralized here (leaf nodes in dependency graph)
- `fp-library/src/kinds.rs` - `Kind` trait definitions and type application machinery
- `fp-macros/src/hkt/` - Procedural macros (`trait_kind!`, `impl_kind!`, `Apply!`)

### Module Organization

The codebase follows a specific dependency structure to prevent cycles:

1. **Brands (`brands.rs`)**: Centralized leaf nodes with no dependencies
2. **Type Classes (`classes/*.rs`)**: Trait definitions that depend on brands
3. **Types (`types/*.rs`)**: Implementations that depend on both brands and classes
4. **Functions (`functions.rs`)**: Facade that re-exports free function wrappers

**Free functions** (e.g., `map`, `pure`) are defined in their trait's module (e.g., `classes/functor.rs`) and re-exported in `functions.rs`. This prevents circular dependencies.

### Pointer Abstraction Hierarchy

The library uses a unified pointer hierarchy to abstract over reference counting:

- `Pointer` - Base trait for heap-allocated pointers (requires `Deref`)
- `RefCountedPointer` - Adds `Clone` (implemented by `RcBrand`, `ArcBrand`)
- `SendRefCountedPointer` - Adds `Send + Sync` (implemented by `ArcBrand` only)

**Function Brands:**
- `FnBrand<P>` is parameterized over a pointer brand `P`
- `RcFnBrand` = `FnBrand<RcBrand>` (single-threaded, `!Send`)
- `ArcFnBrand` = `FnBrand<ArcBrand>` (thread-safe, `Send + Sync`)

**Thread Safety:** For parallel operations, use extension traits:
- `SendCloneableFn` - Thread-safe function wrappers
- `ParFoldable` - Parallel folding (enabled with `rayon` feature)

### Lazy Evaluation Types

The hierarchy consists of infallible computation types, fallible counterparts, and the `Free` monad infrastructure. Each type makes different trade-offs around stack safety, memoization, lifetimes, and thread safety.

| Type | Underlying | HKT | Stack Safe | Memoized | Lifetimes | Send |
|------|-----------|-----|-----------|----------|-----------|------|
| `Thunk<'a, A>` | `Box<dyn FnOnce() -> A + 'a>` | Yes (full) | Partial (`tail_rec_m` only) | No | `'a` | No |
| `SendThunk<'a, A>` | `Box<dyn FnOnce() -> A + Send + 'a>` | No | No | No | `'a` | Yes |
| `Trampoline<A>` | `Free<ThunkBrand, A>` | No | Yes | No | `'static` | No |
| `RcLazy<'a, A>` | `Rc<LazyCell<A, ...>>` | Partial (`RefFunctor`) | N/A | Yes | `'a` | No |
| `ArcLazy<'a, A>` | `Arc<LazyLock<A, ...>>` | Partial (`SendRefFunctor`) | N/A | Yes | `'a` | Yes |
| `TryThunk<'a, A, E>` | `Thunk<'a, Result<A, E>>` | Yes (full) | Partial (`tail_rec_m` only) | No | `'a` | No |
| `TrySendThunk<'a, A, E>` | `SendThunk<'a, Result<A, E>>` | No | No | No | `'a` | Yes |
| `TryTrampoline<A, E>` | `Trampoline<Result<A, E>>` | No | Yes | No | `'static` | No |
| `RcTryLazy<'a, A, E>` | `Rc<LazyCell<Result<A, E>, ...>>` | Partial (`RefFunctor`, `Foldable`) | N/A | Yes | `'a` | No |
| `ArcTryLazy<'a, A, E>` | `Arc<LazyLock<Result<A, E>, ...>>` | Partial (`SendRefFunctor`, `Foldable`) | N/A | Yes | `'a` | Yes |
| `Free<F, A>` | CatList-based "Reflection without Remorse" | No | Yes | No | `'static` | No |

Supporting traits:

| Trait | Purpose | Implementors in hierarchy |
|-------|---------|--------------------------|
| `Deferrable<'a>` | Lazy construction from thunk | `Thunk`, `SendThunk`, `Trampoline`, `RcLazy`, `ArcLazy`, `RcTryLazy`, `ArcTryLazy`, `TryThunk`, `TrySendThunk`, `Free<ThunkBrand, A>` |
| `SendDeferrable<'a>` | Thread-safe lazy construction (extends `Deferrable`) | `SendThunk`, `TrySendThunk`, `ArcLazy`, `ArcTryLazy` |
| `RefFunctor` | Mapping with `&A` input | `LazyBrand<RcLazyConfig>`, `TryLazyBrand<E, RcLazyConfig>` |
| `SendRefFunctor` | Thread-safe mapping with `&A` input | `LazyBrand<ArcLazyConfig>`, `TryLazyBrand<E, ArcLazyConfig>` |
| `LazyConfig` | Infallible memoization strategy (pointer + cell choice) | `RcLazyConfig`, `ArcLazyConfig` |
| `TryLazyConfig` | Fallible memoization strategy (extends `LazyConfig`) | `RcLazyConfig`, `ArcLazyConfig` |

**Pattern:** Use `Trampoline` for stack-safe recursion, wrap in `Lazy` for memoization, use `Thunk` for lightweight views. Use `SendThunk` when the deferred computation must cross thread boundaries without eager evaluation.

### Optics System

Optics are implemented using profunctor encoding for type-safe composition:

**Key Files:**
- `fp-library/src/types/optics/base.rs` - Core optic type definitions
- `fp-library/src/types/optics/lens.rs` - Lens (fully polymorphic S→T, A→B)
- `fp-library/src/types/optics/prism.rs` - Prism (sum type variants)
- `fp-library/src/types/optics/iso.rs` - Isomorphism
- `fp-library/src/types/optics/traversal.rs` - Traversal (multiple foci)

**Internal Profunctors:**
- `Exchange` - For isomorphisms (forward/backward functions)
- `Market` - For prisms (matching/construction)
- `Forget` - For getters/folds
- `Shop` - For lenses (get/set pairs)

**Current State:** Many optics are parameterized with `Rc` hard-coded. Per `docs/todo.md`, these should be refactored to use `FnBrand<P>` for flexible pointer choice.

## Code Style & Documentation

### Formatting

The codebase uses custom rustfmt rules (`rustfmt.toml`):
- Hard tabs (`\t`) for indentation
- Vertical layout for function parameters and imports
- Grouped imports by `StdExternalCrate`
- Single import per line (`imports_granularity = "One"`)

**Always run `cargo fmt` before committing.**

### Documentation Standards

Functions must include:
```rust
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
```

### Commit Message Style

When creating commits:
1. Use imperative mood ("Add feature" not "Added feature")
2. Keep first line under 70 characters
3. Follow existing commit message patterns in `git log`

## Feature Flags

- `rayon` - Enables parallel folding (`ParFoldable`) and `VecBrand` parallel execution
- `serde` - Enables serialization/deserialization for pure data types

## Common Patterns

### Implementing a New Type Class

1. Define trait in `fp-library/src/classes/new_class.rs`
2. Add free function wrapper in same file
3. Update `fp-library/src/classes.rs` to export the module
4. Add documentation following the template above

### Adding a Brand for a New Type

1. Add brand struct to `fp-library/src/brands.rs`
2. Use `impl_kind!` macro to define the `Kind` implementation
3. Implement relevant type classes in `fp-library/src/types/`

### Working with Optics

When modifying optics code:
- Optics use profunctor encoding - understand `Profunctor`, `Strong`, `Choice` traits
- Internal profunctors (Exchange, Market, etc.) are in `types/optics/`
- Many optics currently hard-code `Rc` usage - refactor to use `FnBrand<P>` for parameterization
- See `docs/optics-analysis.md` for design details

### Thread-Safe Operations

For parallel/concurrent code:
1. Use `ArcFnBrand` instead of `RcFnBrand`
2. Use `SendCloneableFn` trait instead of `CloneableFn`
3. Use `ParFoldable` trait for parallel folding (requires `rayon` feature)
4. Ensure all captured data is `Send + Sync`

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

5. **Module Dependency Ordering**: Respect the dependency graph: brands → classes → types → functions. Never create cycles.
