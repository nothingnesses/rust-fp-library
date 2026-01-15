//! Test: Missing "=" in GAT definition
//!
//! This test verifies that `impl_kind!` produces a helpful error when
//! the "=" sign is missing from the GAT definition.

use fp_macros::impl_kind;

struct MyBrand;

impl_kind! {
    for MyBrand {
        type Of<A> Option<A>;
    }
}

fn main() {}
