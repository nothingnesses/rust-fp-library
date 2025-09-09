pub trait Defer<'a> {
	fn defer(f: impl 'a + Fn(()) -> Self) -> Self;
}
