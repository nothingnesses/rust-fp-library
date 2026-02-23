use fp_library::classes::{Monoid, Semigroup};

// Monoid for testing (Sum of i64 using wrapping_add to avoid overflow)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sum(pub i64);

impl Semigroup for Sum {
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Sum(a.0.wrapping_add(b.0))
	}
}

impl Monoid for Sum {
	fn empty() -> Self {
		Sum(0)
	}
}

// i64 is Send + Sync, so Sum is Send + Sync automatically.
