use crate::classes::Category;
use std::ops::Deref;

pub trait Function: Category {
	type Inner<'a, A: 'a, B: 'a>: Deref<Target = dyn 'a + Fn(A) -> B>;
}

pub trait ClonableFunction: Function {
	type Inner<'a, A: 'a, B: 'a>: Deref<Target = dyn 'a + Fn(A) -> B> + Clone;
}
