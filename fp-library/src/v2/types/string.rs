use crate::{
    hkt::Kind1L0T,
    v2::classes::{monoid::Monoid, semigroup::Semigroup},
};

impl Kind1L0T for String {
    type Output<'a> = String;
}

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
