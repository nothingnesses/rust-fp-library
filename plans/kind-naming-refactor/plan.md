# Kind Trait Naming Scheme Refactoring Plan

## 1. Executive Summary

This document outlines a comprehensive plan to improve the HKT (Higher-Kinded Type) simulation in `fp-library` by:

1. **Enhancing the naming scheme** with hash-based canonicalization for robustness
2. **Providing full macro abstraction** so users never interact with cryptic trait names
3. **Future consideration**: Semantic aliases for common patterns

The goal is to maintain correctness while dramatically improving developer experience.

---

## 2. Current State Analysis

### 2.1 Architecture Overview

The library uses [Yallop & White's "Lightweight higher-kinded polymorphism"](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf) approach:

```
┌─────────────────┐     implements      ┌─────────────────┐
│   Brand Type    │ ──────────────────► │   Kind Trait    │
│  (OptionBrand)  │                     │ (Kind_L1_T1_*)  │
└─────────────────┘                     └────────┬────────┘
                                                 │
                                                 │ defines GAT
                                                 ▼
                                        ┌─────────────────┐
                                        │  type Of<...>   │
                                        │  = Option<A>    │
                                        └─────────────────┘
```

### 2.2 Current Naming Convention

Traits are named using De Bruijn index notation:

```
Kind_L{n}_T{m}[_B{bounds}][_O{output}]
```

Where:

- `L{n}`: Number of lifetime parameters
- `T{m}`: Number of type parameters
- `_B{bounds}`: Constraints on type parameters (positional)
- `_O{output}`: Constraints on the `Of` associated type

**Examples:**
| Name | Meaning | GAT Signature |
|------|---------|---------------|
| `Kind_L0_T1` | No lifetimes, 1 type | `type Of<A>` |
| `Kind_L0_T2` | No lifetimes, 2 types | `type Of<A, B>` |
| `Kind_L1_T1_B0l0_Ol0` | 1 lifetime, 1 type, both bounded | `type Of<'a, A: 'a>: 'a` |

### 2.3 Current Implementation

#### `def_kind!` Macro ([`fp-macros/src/lib.rs:136`](../../../fp-macros/src/lib.rs))

```rust
def_kind!(
	('a), // lifetimes
	(A: 'a), // types with bounds
	('a) // output bounds
);
// Generates: pub trait Kind_L1_T1_B0l0_Ol0 { type Of<'a, A: 'a>: 'a; }
```

#### `Kind!` Macro ([`fp-macros/src/lib.rs:129`](../../../fp-macros/src/lib.rs))

```rust
Kind!(('a), (A: 'a), ('a)) // Expands to: Kind_L1_T1_B0l0_Ol0
```

#### `Apply!` Macro ([`fp-library/src/macros.rs:133`](../../../fp-library/src/macros.rs))

```rust
Apply!(OptionBrand, Kind_L1_T1_B0l0_Ol0, ('a), (A))
// Expands to: <OptionBrand as Kind_L1_T1_B0l0_Ol0>::Of<'a, A>
```

### 2.4 Identified Problems

| ID     | Problem                         | Severity | Example                                         |
| ------ | ------------------------------- | -------- | ----------------------------------------------- |
| **P1** | Poor human readability          | High     | `Kind_L1_T2_B0l0_B1tCopy` is cryptic            |
| **P2** | Simplified trait bound encoding | Medium   | `std::fmt::Debug` becomes `tDebug`, losing path |
| **P3** | No complex constraint support   | Medium   | HRTBs, associated types ignored                 |
| **P4** | Manual trait enumeration        | Medium   | Each kind requires explicit `def_kind!`         |
| **P5** | Direct trait name exposure      | High     | Users must write `Kind_L1_T1_*` in impl blocks  |

---

## 3. Proposed Solutions

### 3.1 Solution A: Hash-Based Naming (Robustness)

**Objective:** Create collision-free, deterministic names that can encode any signature.

#### 3.1.1 Enhanced Canonicalization

Replace the current simplified bound encoding with a comprehensive representation:

**Current ([`fp-macros/src/lib.rs:74`](../../../fp-macros/src/lib.rs)):**

```rust
fn canonicalize_bound(&self, bound: &TypeParamBound) -> String {
	match bound {
		TypeParamBound::Trait(tr) => {
			// Only takes last segment - loses path info
			let last_segment = tr.path.segments.last().unwrap();
			format!("t{}", last_segment.ident)
		}
		// ...
	}
}
```

**Proposed:**

```rust
fn canonicalize_bound(&self, bound: &TypeParamBound) -> String {
	match bound {
		TypeParamBound::Trait(tr) => {
			// Full path with generic arguments
			let path = tr.path.segments.iter()
				.map(|seg| {
					let ident = seg.ident.to_string();
					match &seg.arguments {
						PathArguments::None => ident,
						PathArguments::AngleBracketed(args) => {
							let args_str = args.args.iter()
								.map(|a| self.canonicalize_generic_arg(a))
								.collect::<Vec<_>>()
								.join(",");
							format!("{}<{}>", ident, args_str)
						}
						PathArguments::Parenthesized(args) => {
							// Fn trait bounds: Fn(A) -> B
							let inputs = args.inputs.iter()
								.map(|t| self.canonicalize_type(t))
								.collect::<Vec<_>>()
								.join(",");
							let output = match &args.output {
								ReturnType::Default => "()".to_string(),
								ReturnType::Type(_, ty) => self.canonicalize_type(ty),
							};
							format!("{}({})->{}", ident, inputs, output)
						}
					}
				})
				.collect::<Vec<_>>()
				.join("::");
			format!("t{}", path)
		}
		TypeParamBound::Lifetime(lt) => {
			let idx = self.lifetime_map.get(&lt.ident.to_string())
				.expect("Unknown lifetime");
			format!("l{}", idx)
		}
		_ => panic!("Unsupported bound type"),
	}
}
```

#### 3.1.2 Hash-Based Name Generation

For complex signatures, use content-addressable hashing:

```rust
fn generate_name(input: &KindInput, use_hash: bool) -> Ident {
	let canon = Canonicalizer::new(&input.lifetimes, &input.types);

	// Build canonical representation
	let l_count = input.lifetimes.len();
	let t_count = input.types.len();

	let mut canonical_parts = vec![
		format!("L{}", l_count),
		format!("T{}", t_count),
	];

	// Type bounds
	for (i, ty) in input.types.iter().enumerate() {
		if !ty.bounds.is_empty() {
			let bounds_str = canon.canonicalize_bounds(&ty.bounds);
			canonical_parts.push(format!("B{}{}", i, bounds_str));
		}
	}

	// Output bounds
	if !input.output_bounds.is_empty() {
		let bounds_str = canon.canonicalize_bounds(&input.output_bounds);
		canonical_parts.push(format!("O{}", bounds_str));
	}

	let canonical_repr = canonical_parts.join("_");

	// Check for special characters that are invalid in identifiers
	let has_special_chars = canonical_repr.chars().any(|c| !c.is_alphanumeric() && c != '_');

	// Always use hash for consistency and to avoid length issues
	let hash = rapidhash::rapidhash(canonical_repr.as_bytes());
	format_ident!("Kind_{:016x}", hash)
}
```

#### 3.1.3 No Backward Compatibility

The new naming scheme will fully replace the old one. No type aliases for old names will be generated. This is a breaking change that requires migration of all existing code.

### 3.2 Solution B: Full Macro Abstraction (Usability)

**Objective:** Users never write or see raw Kind trait names.

#### 3.2.1 New `impl_kind!` Macro

```rust
// Instead of:
impl Kind_L1_T1_B0l0_Ol0 for OptionBrand {
	type Of<'a, A: 'a> = Option<A>;
}

// Users write:
impl_kind! {
	for OptionBrand {
		type Of<'a, A: 'a>: 'a = Option<A>;
	}
}
```

**Macro Implementation:**

```rust
#[proc_macro]
pub fn impl_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ImplKindInput);

	// Extract GAT signature from the type definition
	let signature = extract_gat_signature(&input.of_type);

	// Generate or lookup the corresponding Kind trait
	let kind_trait = generate_name(&signature);

	// Generate documentation comments
	let doc_comment = format!("Generated Kind trait for signature: {}", signature);

	// Generate the impl block
	let brand = &input.brand;
	let of_generics = &input.of_type.generics;
	let of_bounds = &input.of_type.bounds;
	let of_ty = &input.of_type.ty;

	quote! {
		impl #kind_trait for #brand {
			type Of<#of_generics> #of_bounds = #of_ty;
		}
	}.into()
}
```

#### 3.2.2 Enhanced `Apply!` Macro

Allow signature-based lookup using fully named parameters:

```rust
// New syntax with named parameters:
Apply!(
	brand: OptionBrand,
	signature: ('a, A: 'a) -> 'a,
	lifetimes: ('a),
	types: (A)
)
```

#### 3.2.3 Type Class Trait Integration

Modify type class traits to hide Kind references:

```rust
// Current Functor trait usage in types/option.rs:
impl Functor for OptionBrand {
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
	) -> Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B))
	where
		F: Fn(A) -> B + 'a,
	{ ... }
}

// Could become (with macro magic):
impl Functor for OptionBrand {
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Self::Of<'a, A>,  // Directly use GAT
	) -> Self::Of<'a, B>
	where
		F: Fn(A) -> B + 'a,
	{ ... }
}
```

This requires the `Functor` trait to be generic over the Kind, or use a standard Kind for all Functors.

---

## 4. Future Consideration: Semantic Aliases (Solution C)

### 4.1 Overview

For very common patterns, provide human-readable type aliases:

```rust
/// Kind for simple functors: `F<A>` where `A: 'a` and `F<A>: 'a`
pub type KindFunctor = Kind_L1_T1_B0l0_Ol0;

/// Kind for bifunctors: `F<A, B>` with no lifetime constraints
pub type KindBifunctor = Kind_L0_T2;

/// Kind for pure type constructors: `F<A>` with no constraints
pub type KindPure = Kind_L0_T1;

/// Kind for reference-like types: `&'a T` patterns
pub type KindRef = Kind_L1_T1_B0l0_Ol0;
```

### 4.2 Benefits

- **Immediate readability**: `impl KindFunctor for MyBrand` is self-documenting
- **Low implementation cost**: Just type aliases
- **No breaking changes**: Additive only

### 4.3 Limitations

- **Not comprehensive**: Only covers common patterns
- **Semantic ambiguity**: `KindFunctor` and `KindRef` might have the same signature
- **Still requires systematic names**: Edge cases fall back to `Kind_*`

### 4.4 Implementation Requirements

1. Identify the most common Kind patterns used in the library
2. Create semantic names that don't conflict
3. Document which patterns use which names
4. Keep systematic names as the authoritative source

**Recommended semantic aliases:**

| Alias           | Underlying Kind       | Use Case                              |
| --------------- | --------------------- | ------------------------------------- |
| `KindFunctor`   | `Kind_L1_T1_B0l0_Ol0` | `Option<A>`, `Vec<A>`, `Box<A>`       |
| `KindBifunctor` | `Kind_L0_T2`          | `Result<A, E>`, `Either<A, B>`        |
| `KindPure`      | `Kind_L0_T1`          | Simple `F<A>` without lifetime bounds |
| `KindTriple`    | `Kind_L0_T3`          | Three-type constructors               |
| `KindLifetime`  | `Kind_L1_T0`          | Pure lifetime carriers                |

---

## 5. Implementation Approach

### 5.1 Phase 1: Foundation (Hash-Based Naming)

**Goal:** Improve the naming scheme's robustness.

1. **Enhance canonicalization** in `fp-macros/src/lib.rs`

   - Full path preservation for trait bounds
   - Generic argument handling
   - Fn trait support

2. **Implement Hash-Based Naming**

   - Always use hash for all signatures
   - Deterministic hashing function (e.g., rapidhash)

3. **Remove backward compatibility**
   - Replace old naming scheme entirely
   - Update documentation to reflect breaking changes

### 5.2 Phase 2: Abstraction (Macro Layer)

**Goal:** Hide Kind trait names from users entirely.

1. **Create `impl_kind!` macro**

   - Parse GAT definitions
   - Generate corresponding Kind trait impl
   - Generate documentation comments on the trait including the input signature

2. **Enhance documentation**

   - Usage examples
   - Migration guide from direct trait names

3. **Update library code**
   - Migrate existing impl blocks to use `impl_kind!`
   - Keep both syntaxes working during transition

### 5.3 Future Considerations (Semantic Aliases)

_Note: This phase is not part of the initial implementation._

**Goal:** Provide human-friendly names for common patterns.

1. **Audit Kind usage**
2. **Create aliases**
3. **Update examples**

---

## 6. Technical Specifications

### 6.1 Dependencies

Add to `fp-macros/Cargo.toml`:

```toml
[dependencies]
proc-macro2 = "1"
quote = "1"
rapidhash = "4.2"  # For deterministic hashing
syn = { version = "2", features = ["full", "parsing", "extra-traits"] }
```

### 6.2 Module Structure

```
fp-macros/src/
├── lib.rs			  # Macro entry points
├── parse.rs			# Input parsing
├── canonicalize.rs	 # Canonicalization logic
├── generate.rs		 # Name generation (with hashing)
└── impl_kind.rs		# impl_kind! macro implementation
```

### 6.3 Error Handling

The macros should provide clear error messages:

```rust
// Good error messages
error: Unknown lifetime `'b` in type bound
  --> src/types/example.rs:15:23
   |
15 | impl_kind!(for MyBrand { type Of<'a, A: 'b> = ... })
   |									  ^^ help: did you mean `'a`?
```

### 6.4 Testing Strategy

1. **Unit tests** for canonicalization logic
2. **Macro expansion tests** using `trybuild`
3. **Integration tests** ensuring existing code still compiles
4. **Property tests** for deterministic naming

---

## 7. Migration Guide

### 7.1 For Library Maintainers

```rust
// Before (current)
impl Kind_L1_T1_B0l0_Ol0 for OptionBrand {
	type Of<'a, A: 'a> = Option<A>;
}

// After (Phase 2)
impl_kind! {
	for OptionBrand {
		type Of<'a, A: 'a>: 'a = Option<A>;
	}
}
```

### 7.2 For Library Users

```rust
// Before
use fp_library::hkt::Kind_L1_T1_B0l0_Ol0;
Apply!(MyBrand, Kind_L1_T1_B0l0_Ol0, ('a), (A))

// After (Phase 2)
use fp_library::impl_kind;
impl_kind! { for MyBrand { type Of<'a, A: 'a>: 'a = MyType<A>; } }
Apply!(
	brand: MyBrand,
	signature: ('a, A: 'a) -> 'a,
	lifetimes: ('a),
	types: (A)
)
```

---

## 8. Risk Assessment

| Risk                   | Likelihood | Impact | Mitigation                            |
| ---------------------- | ---------- | ------ | ------------------------------------- |
| Hash collisions        | Very Low   | High   | Use 128-bit hash; test extensively    |
| Breaking existing code | Medium     | High   | Maintain type aliases; phased rollout |
| Macro complexity       | Medium     | Medium | Comprehensive testing; good errors    |
| Performance impact     | Low        | Low    | Hashing is compile-time only          |
| User confusion         | Medium     | Medium | Clear documentation; examples         |

---

## 9. Success Criteria

1. **Clean migration path** for existing code (breaking changes accepted)
2. **All Kind traits** can be defined via `impl_kind!`
3. **Hash-based names** are deterministic across compilations
4. **Error messages** clearly indicate issues in macro input
5. **Documentation** covers all usage patterns
6. **Tests** achieve >90% coverage of new code

---

## 10. References

- [Lightweight Higher-Kinded Polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf) - Yallop & White
- [De Bruijn Index](https://en.wikipedia.org/wiki/De_Bruijn_index) - Wikipedia
- [Existing Kind Macro Plan](../kind-trait-macro/plan.md) - Previous planning document
- [fp-macros Implementation](../../fp-macros/src/lib.rs) - Current macro code
