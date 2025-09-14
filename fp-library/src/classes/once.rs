use crate::{hkt::Kind0L1T, make_type_apply};

pub trait Once: Kind0L1T {
	type Output<A>;

	fn new<A>() -> ApplyOnce<Self, A>;

	fn get<A>(a: &ApplyOnce<Self, A>) -> Option<&A>;

	fn get_mut<A>(a: &mut ApplyOnce<Self, A>) -> Option<&mut A>;

	fn set<A>(
		a: &ApplyOnce<Self, A>,
		value: A,
	) -> Result<(), A>;

	fn get_or_init<A, B: FnOnce() -> A>(
		a: &ApplyOnce<Self, A>,
		f: B,
	) -> &A;

	fn into_inner<A>(a: ApplyOnce<Self, A>) -> Option<A>;

	fn take<A>(a: &mut ApplyOnce<Self, A>) -> Option<A>;
}

make_type_apply!(ApplyOnce, Once, (), (A), "* -> *");
