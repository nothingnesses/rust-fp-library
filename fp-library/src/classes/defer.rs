pub trait Defer<'a>: Clone {
	fn defer(f: impl 'a + Fn(()) -> Self) -> Self;
}
