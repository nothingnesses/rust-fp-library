//! Test: Missing semicolon in GAT definition
//!
//! This test verifies that `impl_kind!` produces a helpful error when
//! the semicolon is missing at the end of the GAT definition.

use fp_macros::impl_kind;

struct MyBrand;

impl_kind! {
    for MyBrand {
        type Of<A> = Option<A>
    }
}

fn main() {}
