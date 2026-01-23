# fp-macros

[![crates.io](https://img.shields.io/crates/v/fp-macros.svg)](https://crates.io/crates/fp-macros)
[![docs.rs](https://docs.rs/fp-macros/badge.svg)](https://docs.rs/fp-macros)
[![GitHub License](https://img.shields.io/github/license/nothingnesses/rust-fp-library?color=blue)](https://github.com/nothingnesses/rust-fp-library/blob/main/LICENSE)

Procedural macros for the [`fp-library`](https://github.com/nothingnesses/rust-fp-library) crate.

This crate provides a suite of macros designed to facilitate working with Higher-Kinded Types (HKT) in Rust. It automates the generation of `Kind` traits, simplifies their implementation for specific `Brand` types, and provides a convenient syntax for type application.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fp-macros = "0.3"
```

> **Note:** If you are using [`fp-library`](https://crates.io/crates/fp-library), these macros are already re-exported at the crate root. You only need to add this dependency if you are using the macros independently.

## Macros

### `def_kind!`

Defines a new `Kind` trait based on a Higher-Kinded Type signature.

This macro generates a trait definition for a Higher-Kinded Type signature. It takes a list of associated type definitions, similar to a trait definition.

**Syntax:**

```rust
def_kind!(
    type AssocName<Params>: Bounds;
    // ... more associated types
);
```

**Examples:**

```rust
use fp_macros::def_kind;

// Simple definition
def_kind!(type Of<T>;);

// Definition with bounds and lifetimes
def_kind!(type Of<'a, T: Display>: Debug;);

// Multiple associated types
def_kind!(
    type Of<T>;
    type SendOf<T>: Send;
);
```

### `impl_kind!`

Simplifies the implementation of a generated `Kind` trait for a specific brand type. It infers the correct `Kind` trait to implement based on the signature of the associated types provided in the block.

The signature (names, parameters, and bounds) of the associated types must match the definition used in `def_kind!` or `Kind!` to ensure the correct trait is implemented.

**Syntax:**

```rust
impl_kind! {
    // Optional impl generics
    impl<Generics> for BrandType
    // Optional where clause
    where Bounds
    {
        type AssocName<Params> = ConcreteType;
        // ... more associated types
    }
}
```

**Examples:**

```rust
use fp_macros::impl_kind;

struct OptionBrand;

// Simple implementation
impl_kind! {
    for OptionBrand {
        type Of<A> = Option<A>;
    }
}

struct ResultBrand<E>(std::marker::PhantomData<E>);

// Implementation with generics
impl_kind! {
    impl<E> for ResultBrand<E> {
        type Of<A> = Result<A, E>;
    }
}

// Implementation with where clause and multiple types
impl_kind! {
    impl<E> for MyBrand<E> where E: Clone {
        type Of<A> = MyType<A, E>;
        type SendOf<A> = MySendType<A, E>;
    }
}
```

### `Apply!`

Applies a `Brand` to type arguments.

This macro projects a `Brand` type to its concrete type using the appropriate `Kind` trait. It uses a syntax that mimics a fully qualified path, where the `Kind` trait is specified by its signature.

**Syntax:**

```rust
Apply!(<Brand as Kind!( KindSignature )>::AssocType<Args>)
```

*   `Brand`: The brand type (e.g., `OptionBrand`).
*   `KindSignature`: A list of associated type definitions defining the `Kind` trait schema.
*   `AssocType`: The associated type to project (e.g., `Of`).
*   `Args`: The concrete arguments to apply.

**Examples:**

```rust
use fp_macros::Apply;

// Applies MyBrand to lifetime 'static and type String.
type Concrete = Apply!(<MyBrand as Kind!( type Of<'a, T>; )>::Of<'static, String>);

// Applies MyBrand to a generic type T with bounds.
type Concrete = Apply!(<MyBrand as Kind!( type Of<T: Clone>; )>::Of<T>);

// Complex example with lifetimes, types, and output bounds.
type Concrete = Apply!(<MyBrand as Kind!( type Of<'a, T: Clone + Debug>: Display; )>::Of<'a, T>);

// Use a custom associated type for projection.
type Concrete = Apply!(<MyBrand as Kind!( type Of<T>; type SendOf<T>; )>::SendOf<T>);
```

### `Kind!`

Generates the name of a `Kind` trait based on its signature.

This macro takes a list of associated type definitions, similar to a trait definition. It is primarily used internally by other macros (like `Apply!`) but can be useful when you need to refer to the generated trait name directly (e.g., in bounds where macros aren't allowed).

**Syntax:**

```rust
Kind!(
    type AssocName<Params>: Bounds;
    // ...
)
```

**Examples:**

```rust
use fp_macros::Kind;

// Simple signature
let name = Kind!(type Of<T>;);

// Signature with bounds and lifetimes
let name = Kind!(type Of<'a, T: Display>: Debug;);
```

**Limitations:**

Due to Rust syntax restrictions, this macro cannot be used directly in positions where a concrete path is expected by the parser, such as:
*   Supertrait bounds: `trait MyTrait: Kind!(...) {}` (Invalid)
*   Type aliases: `type MyKind = Kind!(...);` (Invalid)
*   Trait aliases: `trait MyKind = Kind!(...);` (Invalid)

In these cases, you must use the generated name directly (e.g., `Kind_...`).

The generated trait name is a deterministic hash of the signature (e.g., `Kind_a1b2c3d4e5f67890`). To find the exact name for use in restricted positions, you can inspect the expanded code (using `cargo expand`) or check the compiler error output when attempting to use the macro in a valid position first.

## License

This project is licensed under the [Blue Oak Model License 1.0.0](../LICENSE).
