//! Test: Missing "for" keyword
//!
//! This test verifies that `impl_kind!` produces a helpful error when
//! the "for" keyword is missing.

use fp_macros::impl_kind;

struct MyBrand;

impl_kind! {
    MyBrand {
        type Of<A> = Option<A>;
    }
}

fn main() {}
