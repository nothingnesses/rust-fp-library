## Higher-Kinded Types (HKT)

Since Rust doesn't support HKTs directly (i.e., it's not possible to use `Option` in `impl Functor for Option`, instead of `Option<T>`), this library uses **Lightweight Higher-Kinded Polymorphism** (also known as the "Brand" pattern or type-level defunctionalization).

Each type constructor has a corresponding `Brand` type (e.g., `OptionBrand` for `Option`). These brands implement the `Kind` traits, which map the brand and generic arguments back to the concrete type. The library provides macros to simplify this process.

```rust
use fp_library::{
	impl_kind,
	kinds::*,
};

pub struct OptionBrand;

impl_kind! {
	#[no_inferable_brand]
	for OptionBrand {
		type Of<'a, A: 'a>: 'a = Option<A>;
	}
}
```
