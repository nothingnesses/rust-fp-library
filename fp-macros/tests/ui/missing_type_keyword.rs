//! Test: Missing "type" keyword in GAT definition
//!
//! This test verifies that `impl_kind!` produces a helpful error when
//! the "type" keyword is missing from the GAT definition.

use fp_macros::impl_kind;

struct MyBrand;

impl_kind! {
    for MyBrand {
        Of<A> = Option<A>;
    }
}

fn main() {}
