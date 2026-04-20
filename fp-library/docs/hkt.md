### Higher-Kinded Types (HKT)

Since Rust doesn't support HKTs directly (i.e., it's not possible to use `Option` in `impl Functor for Option`, instead of `Option<T>`), this library uses **Lightweight Higher-Kinded Polymorphism** (also known as the "Brand" pattern or type-level defunctionalization).

#### How it works

In languages with native HKT support, you can write something like
`impl Functor for Option`, where `Option` refers to the unapplied type
constructor (a function from types to types: given `A`, produce
`Option<A>`). Rust has no way to refer to `Option` without its type
parameter, so this is not expressible directly.

The Brand pattern works around this by splitting the concept into two
parts:

1. A zero-sized **brand type** (e.g., `OptionBrand`) that stands in for
   the unapplied type constructor. Since it is a concrete type with no
   parameters, it can appear as `Self` in trait impls.
2. A **`Kind` trait** with an associated type `Of` that acts as
   type-level function application. `<OptionBrand as Kind>::Of<A>`
   evaluates to `Option<A>`, recovering the concrete type.

This lets you write `impl Functor for OptionBrand` and have `Functor`'s
methods operate on `Self::Of<A>` (i.e., `Option<A>`). A function generic
over any `Functor` can accept any brand `F: Functor` and work with
`F::Of<A>` without knowing the concrete type, which is exactly the
abstraction that HKTs provide.

The library provides two macros to simplify defining brands:
`trait_kind!` defines the `Kind` trait for a given signature, and
`impl_kind!` implements it for a specific brand.

#### `trait_kind!`

`trait_kind!` defines a new `Kind` trait (and a matching `InferableBrand`
trait for brand inference) from an associated type signature:

```rust,ignore
use fp_macros::trait_kind;

trait_kind! {
	/// The applied type.
	type Of<'a, A: 'a>: 'a;
}
```

This generates two traits sharing a deterministic content hash:

1. `Kind_{hash}`: the forward mapping trait with the specified associated
   type. Brands implement this to map from brand + type arguments to
   a concrete type.
2. `InferableBrand_{hash}`: the reverse mapping trait. Concrete types
   implement this (via `impl_kind!`) to enable the compiler to infer
   the brand from a container value. It carries an associated
   `type Marker` (`Val` or `Ref`) used by the dispatch system.

The input is one or more associated type definitions. Lifetimes, type
bounds, and output bounds are all supported:

```rust,ignore
// Simple: no lifetime, no bounds
trait_kind!(type Of<A>;);

// With lifetime and bounds
trait_kind!(type Of<'a, A: 'a>: 'a;);

// Bifunctor: two type parameters
trait_kind!(type Of<'a, A: 'a, B: 'a>: 'a;);
```

The library's `Kind` traits are defined in the `kinds` module using
`trait_kind!`. Most types use the `type Of<'a, A: 'a>: 'a` signature.

#### `impl_kind!`

`impl_kind!` implements a `Kind` trait for a brand, mapping the
associated type to a concrete type. It also generates the corresponding
`InferableBrand` impl for brand inference:

```rust,ignore
use fp_library::{
	impl_kind,
	kinds::*,
};

pub struct OptionBrand;

impl_kind! {
	for OptionBrand {
		type Of<'a, A: 'a>: 'a = Option<A>;
	}
}
```

The signature in `impl_kind!` must match a signature previously defined
by `trait_kind!` so that the correct `Kind` and `InferableBrand` traits
are resolved.

#### Kind trait naming

Each `trait_kind!` invocation generates a `Kind_{hash}` trait (and a
matching `InferableBrand_{hash}` trait for brand inference). The `{hash}`
is a deterministic 64-bit hash of the canonical signature (number of
lifetimes and types, type bounds with full path preservation and generic
arguments, output bounds on associated types). Semantically equivalent
signatures always map to the same trait, regardless of parameter names
or formatting.

| Hash               | Signature                        | Used by                          |
| ------------------ | -------------------------------- | -------------------------------- |
| `ad6c20556a82a1f0` | `type Of<A>;`                    | Simple type constructors         |
| `cdc7cd43dac7585f` | `type Of<'a, A: 'a>: 'a;`        | Most functor/monad types         |
| `5b1bcedfd80bdc16` | `type Of<A, B>;`                 | Bifunctor brands (no lifetime)   |
| `266801a817966495` | `type Of<'a, A: 'a, B: 'a>: 'a;` | Bifunctor brands (with lifetime) |

For each hash, both `Kind_{hash}` and `InferableBrand_{hash}` exist.
For example, `OptionBrand` implements `Kind_cdc7cd43dac7585f`, and
`Option<A>` implements `InferableBrand_cdc7cd43dac7585f<OptionBrand, A>`
with `type Marker = Val`.

For the full trait shapes, impl landscape, and Marker invariant, see
[Brand Inference](./brand-inference.md).
