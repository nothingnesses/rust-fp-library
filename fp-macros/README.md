# fp-macros

Procedural macros for the `fp-library` crate.

This crate provides a suite of macros designed to facilitate working with Higher-Kinded Types (HKT) in Rust. It automates the generation of Kind traits, simplifies their implementation for specific "brand" types, and provides a convenient syntax for type application.

## Macros

### `def_kind!`

Defines a new Kind trait based on a Higher-Kinded Type signature.

**Syntax:** `def_kind!((Lifetimes), (Types), (OutputBounds));`

- **Lifetimes**: Comma-separated list of lifetimes (e.g., `'a`).
- **Types**: Comma-separated list of types with optional bounds (e.g., `T: Display`).
- **OutputBounds**: `+`-separated list of bounds on the output type (e.g., `Display + Clone`).

**Example:**

```rust
use fp_macros::def_kind;
use std::fmt::Display;

// Defines a Kind trait for a signature with:
// - 1 lifetime ('a)
// - 1 type parameter (T) bounded by Display
// - Output type bounded by Debug
def_kind!(('a), (T: Display), (std::fmt::Debug));
```

### `impl_kind!`

Simplifies the implementation of a generated Kind trait for a specific brand type. It infers the correct Kind trait to implement based on the signature of the associated type `Of`.

**Syntax:**

```rust
impl_kind! {
    impl<GENERICS> for BrandType {
        type Of<PARAMS> = ConcreteType;
    }
}
```

**Example:**

```rust
use fp_macros::impl_kind;

struct OptionBrand;

impl_kind! {
    impl for OptionBrand {
        type Of<T> = Option<T>;
    }
}
```

### `Apply!`

Applies a brand to type arguments, projecting the brand to its concrete type. This macro is useful for using HKTs in function signatures, struct definitions, and type aliases.

**Modes:**

1.  **Unified Signature Mode** (Recommended): Uses a single `signature` parameter.
2.  **Explicit Kind Mode** (Advanced): Uses explicit `kind`, `lifetimes`, and `types` parameters.

**Example (Unified Signature):**

```rust
use fp_macros::Apply;

// Applies OptionBrand to type String.
type Concrete = Apply!(
    brand: OptionBrand,
    signature: (String)
);
// Concrete is Option<String>

// Applies a brand with lifetime and bounds
type ConcreteRef<'a> = Apply!(
    brand: RefBrand,
    signature: ('a, i32: 'a)
);
```

**Example (Explicit Kind):**

```rust
type Concrete = Apply!(
    brand: OptionBrand,
    kind: SomeKindTrait,
    lifetimes: (),
    types: (String)
);
```

### `Kind!`

Generates the name of a Kind trait based on its signature. This is primarily used internally by other macros but can be useful when you need to refer to the generated trait name directly (e.g., in bounds where macros aren't allowed).

**Example:**

```rust
// Generates the name for a Kind with 1 type parameter T
let name = Kind!((), (T), ());
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fp-macros = "0.1"
```

## License

BlueOak-1.0.0
