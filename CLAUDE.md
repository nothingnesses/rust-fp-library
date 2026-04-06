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
just fmt                                        # Format all files (Rust, Nix, Markdown, YAML, TOML)
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
just test --all-features                             # Run all tests with all features (cached)
just test                                            # Run all tests, default features (cached)
just test -p fp-library test_name                # Run a subset (no caching)
just test -p fp-library --test property          # Run property-based tests
just test --doc -p fp-library                    # Run doc tests
```

Cache location: `.claude/test-cache/` (gitignored). Uses content hashing (`git ls-files` + `md5sum`) so the cache is invalidated only when tracked file contents change, not when timestamps change (e.g., from formatting or git operations). Re-running `just test` with no content changes is instant and prints cached output. Use `just clean` to clear the cache and build artifacts.

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
just fmt
just clippy --workspace --all-features
just doc --workspace --all-features --no-deps
just test --all-features
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

For detailed design documentation, see the `fp-library/docs/` directory:

- `fp-library/docs/hkt.md` - Higher-Kinded Types and the Brand pattern
- `fp-library/docs/zero-cost.md` - Zero-cost abstractions and uncurried semantics
- `fp-library/docs/lazy-evaluation.md` - Lazy evaluation types, trade-offs, and decision guide
- `fp-library/docs/pointer-abstraction.md` - Pointer hierarchy, `FnBrand<P>`, and `LazyConfig`
- `fp-library/docs/coyoneda.md` - Free functor implementations and trade-offs
- `fp-library/docs/parallelism.md` - Thread safety and parallel trait hierarchy
- `fp-library/docs/features.md` - Full feature list with type class hierarchy
- `fp-library/docs/architecture.md` - Module organization and documentation conventions
- `fp-library/docs/optics-analysis.md` - Optics system design details

### Key Locations

- `fp-library/src/brands.rs` - All brand types centralized here (leaf nodes in dependency graph)
- `fp-library/src/kinds.rs` - `Kind` trait definitions and type application machinery
- `fp-macros/src/hkt/` - Procedural macros (`trait_kind!`, `impl_kind!`, `Apply!`)
- `fp-library/src/types/optics/` - Profunctor-encoded optics (Lens, Prism, Iso, Traversal, etc.)

### Module Dependency Ordering

Respect the dependency graph: brands -> classes -> types -> functions. Never create cycles. Free functions (e.g., `map`, `pure`) are defined in their trait's module (e.g., `classes/functor.rs`) and re-exported in `functions.rs`.

### Optics

Optics use profunctor encoding. Internal profunctors: `Exchange` (isos), `Market` (prisms), `Forget` (getters/folds), `Shop` (lenses). Many optics currently hard-code `Rc`; per `docs/todo.md`, these should be refactored to use `FnBrand<P>`. See `fp-library/docs/optics-analysis.md` for design details.

## Code Style & Documentation

### Formatting

The codebase uses custom rustfmt rules (`rustfmt.toml`):

- Hard tabs (`\t`) for indentation
- Vertical layout for function parameters and imports
- Grouped imports by `StdExternalCrate`
- Single import per line (`imports_granularity = "One"`)

**Always run `just fmt` before committing.** A pre-commit hook also runs treefmt automatically.

### No Emoji or Unicode

Never use emoji or unicode symbols in code, comments, or documentation. Use plain text and ASCII equivalents:

- Status markers: `Yes`, `No`, `Partial`, `N/A` (not checkmarks or crosses).
- Arrows: `->`, `<-`, `<->` (not unicode arrows).
- Math: `>=`, `<=`, `!=` (not unicode math symbols).
- Section dividers: `// -- Section name --` (not box-drawing characters).

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

1. Use imperative mood ("Add feature" not "Added feature")
2. Keep first line under 70 characters
3. Follow existing commit message patterns in `git log`

## Feature Flags

- `rayon` - Enables parallel folding (`ParFoldable`) and `VecBrand` parallel execution
- `serde` - Enables serialization/deserialization for pure data types
- `stacker` - Enables adaptive stack growth for deep Coyoneda map chains

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
- See `fp-library/docs/optics-analysis.md` for design details

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
