use crate::v2::classes::{monoid::Monoid, semigroup::Semigroup};

impl Semigroup for String {
    fn append(a: Self, b: Self) -> Self {
        a + &b
    }
}

impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }
}
