//! Test: Invalid associated type name (must be "Of")
//!
//! This test verifies that `impl_kind!` rejects associated type names other than "Of".

use fp_macros::impl_kind;

struct MyBrand;

impl_kind! {
    for MyBrand {
        type Foo<A> = Option<A>;
    }
}

fn main() {}
