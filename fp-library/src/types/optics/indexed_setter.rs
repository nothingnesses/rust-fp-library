//! Indexed setter optics.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				UnsizedCoercible,
				functor_with_index::FunctorWithIndex,
				optics::*,
			},
			kinds::*,
			types::optics::Indexed,
		},
		fp_macros::{
			document_examples,
			document_parameters,
			document_returns,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// A trait for indexed setter functions.
	pub trait IndexedSetterFunc<'a, I, S, T, A, B> {
		/// Apply the indexed setter function.
		fn apply(
			&self,
			f: Box<dyn Fn(I, A) -> B + 'a>,
			s: S,
		) -> T;
	}

	/// A wrapper struct for the `mapped` constructor.
	#[derive(Clone)]
	pub struct Mapped<Brand>(std::marker::PhantomData<Brand>);

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The brand of the functor.",
		"The type of the elements in the structure.",
		"The type of the elements in the result."
	)]
	#[document_parameters("The mapped struct.")]
	impl<'a, I, Brand, A, B>
		IndexedSetterFunc<
			'a,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>),
			A,
			B,
		> for Mapped<Brand>
	where
		Brand: FunctorWithIndex<I>,
		A: 'a,
		B: 'a,
		I: 'a,
	{
		#[document_signature]
		#[document_parameters("The map function.", "The structure to map.")]
		#[document_returns("The mapped structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::VecBrand,
	types::optics::indexed_setter::Mapped,
	classes::optics::indexed_setter::IndexedSetterFunc,
};

let mapper = Mapped::<VecBrand>(std::marker::PhantomData);
let s = vec![10, 20, 30];
let f = Box::new(|i: usize, a: i32| a + i as i32);

let result: Vec<i32> = IndexedSetterFunc::apply(
	&mapper,
	f,
	s
);

assert_eq!(result, vec![10, 21, 32]);
"#
		)]
		fn apply(
			&self,
			f: Box<dyn Fn(I, A) -> B + 'a>,
			s: Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>) {
			Brand::map_with_index(f, s)
		}
	}

	/// A polymorphic indexed setter.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The setter function type."
	)]
	pub struct IndexedSetter<'a, P, I, S, T, A, B, F>
	where
		F: IndexedSetterFunc<'a, I, S, T, A, B> + 'a, {
		/// The setter function.
		pub setter_fn: F,
		pub(crate) _phantom: PhantomData<(&'a (I, S, T, A, B), P)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The setter function type."
	)]
	#[document_parameters("The indexed setter instance.")]
	impl<'a, P, I, S, T, A, B, F> Clone for IndexedSetter<'a, P, I, S, T, A, B, F>
	where
		F: IndexedSetterFunc<'a, I, S, T, A, B> + Clone + 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedSetter` instance that is a copy of the original.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
#[derive(Clone)]
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetter<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MySetter> =
	IndexedSetter::new(MySetter);
let cloned = l.clone();
assert_eq!(cloned.over(vec![10, 20], |i, x| x + (i as i32)), vec![10, 21]);
"#
		)]
		fn clone(&self) -> Self {
			IndexedSetter {
				setter_fn: self.setter_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The setter function type."
	)]
	#[document_parameters("The indexed setter instance.")]
	impl<'a, P, I, S, T, A, B, F> IndexedSetter<'a, P, I, S, T, A, B, F>
	where
		F: IndexedSetterFunc<'a, I, S, T, A, B> + 'a,
	{
		/// Create a new indexed setter.
		#[document_signature]
		#[document_parameters("The setter function.")]
		#[document_returns("A new `IndexedSetter` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetter<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MySetter> =
	IndexedSetter::new(MySetter);
assert_eq!(l.over(vec![10, 20], |i, x| x + (i as i32)), vec![10, 21]);
"#
		)]
		pub fn new(setter_fn: F) -> Self {
			IndexedSetter {
				setter_fn,
				_phantom: PhantomData,
			}
		}

		/// Update the focus using an indexed function.
		#[document_signature]
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetter<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MySetter> =
	IndexedSetter::new(MySetter);
assert_eq!(l.over(vec![10, 20], |i, x| x + (i as i32)), vec![10, 21]);
"#
		)]
		pub fn over(
			&self,
			s: S,
			f: impl Fn(I, A) -> B + 'a,
		) -> T {
			self.setter_fn.apply(Box::new(f), s)
		}

		/// Set the focus.
		#[document_signature]
		#[document_parameters("The structure to update.", "The new focus value.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetter<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MySetter> =
	IndexedSetter::new(MySetter);
assert_eq!(l.set(vec![10, 20], 42), vec![42, 42]);
"#
		)]
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T
		where
			B: Clone + 'a, {
			self.over(s, move |_, _| b.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The brand of the functor.",
		"The type of the elements in the structure.",
		"The type of the elements in the result."
	)]
	impl<'a, P, I, Brand, A, B>
		IndexedSetter<
			'a,
			P,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>),
			A,
			B,
			Mapped<Brand>,
		>
	where
		Brand: FunctorWithIndex<I>,
		A: 'a,
		B: 'a,
		I: 'a,
	{
		/// Create an indexed setter from a `FunctorWithIndex`.
		#[document_signature]
		#[document_returns("A new `IndexedSetter` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::{RcBrand, VecBrand},
	types::optics::IndexedSetter,
	functions::optics_indexed_set,
};
let l: IndexedSetter<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _> =
	IndexedSetter::mapped::<VecBrand>();
let v = vec![10, 20];
let s = optics_indexed_set::<RcBrand, _, _, _, _>(&l, v, 99);
assert_eq!(s, vec![99, 99]);
"#
		)]
		pub fn mapped() -> Self {
			IndexedSetter::new(Mapped(std::marker::PhantomData))
		}
	}

	/// A monomorphic indexed setter.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The setter function type."
	)]
	pub struct IndexedSetterPrime<'a, P, I, S, A, F>
	where
		F: IndexedSetterFunc<'a, I, S, S, A, A> + 'a, {
		/// The setter function.
		pub setter_fn: F,
		pub(crate) _phantom: PhantomData<(&'a (I, S, A), P)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The setter function type."
	)]
	#[document_parameters("The indexed setter instance.")]
	impl<'a, P, I, S, A, F> Clone for IndexedSetterPrime<'a, P, I, S, A, F>
	where
		F: IndexedSetterFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedSetterPrime` instance that is a copy of the original.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
#[derive(Clone)]
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetterPrime<RcBrand, usize, Vec<i32>, i32, MySetter> =
	IndexedSetterPrime::new(MySetter);
let cloned = l.clone();
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	IndexedSetterOptic::evaluate::<RcBrand>(&cloned, pab);
assert_eq!(result(vec![10, 20]), vec![10, 21]);
"#
		)]
		fn clone(&self) -> Self {
			IndexedSetterPrime {
				setter_fn: self.setter_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The setter function type."
	)]
	#[document_parameters("The indexed setter instance.")]
	impl<'a, P, I, S, A, F> IndexedSetterPrime<'a, P, I, S, A, F>
	where
		F: IndexedSetterFunc<'a, I, S, S, A, A> + 'a,
	{
		/// Create a new monomorphic indexed setter.
		#[document_signature]
		#[document_parameters("The setter function.")]
		#[document_returns("A new `IndexedSetterPrime` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetterPrime<RcBrand, usize, Vec<i32>, i32, MySetter> =
	IndexedSetterPrime::new(MySetter);
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
assert_eq!(result(vec![10, 20]), vec![10, 21]);
"#
		)]
		pub fn new(setter_fn: F) -> Self {
			IndexedSetterPrime {
				setter_fn,
				_phantom: PhantomData,
			}
		}

		/// Update the focus using an indexed function.
		#[document_signature]
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetterPrime<RcBrand, usize, Vec<i32>, i32, MySetter> =
	IndexedSetterPrime::new(MySetter);
assert_eq!(l.over(vec![10, 20], |i, x| x + (i as i32)), vec![10, 21]);
"#
		)]
		pub fn over(
			&self,
			s: S,
			f: impl Fn(I, A) -> A + 'a,
		) -> S {
			self.setter_fn.apply(Box::new(f), s)
		}

		/// Set the focus.
		#[document_signature]
		#[document_parameters("The structure to update.", "The new focus value.")]
		#[document_returns("The updated structure.")]
		#[document_examples(
			r#"use fp_library::{
	brands::RcBrand,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetterPrime<RcBrand, usize, Vec<i32>, i32, MySetter> =
	IndexedSetterPrime::new(MySetter);
assert_eq!(l.set(vec![10, 20], 42), vec![42, 42]);
"#
		)]
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S
		where
			A: Clone + 'a, {
			self.over(s, move |_, _| a.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The brand of the functor.",
		"The type of the elements in the structure."
	)]
	impl<'a, P, I, Brand, A>
		IndexedSetterPrime<
			'a,
			P,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			A,
			Mapped<Brand>,
		>
	where
		Brand: FunctorWithIndex<I>,
		A: 'a,
		I: 'a,
	{
		/// Create a monomorphic indexed setter from a `FunctorWithIndex`.
		#[document_signature]
		#[document_returns("A new `IndexedSetterPrime` instance.")]
		#[document_examples(
			r#"use fp_library::{
	brands::{RcBrand, VecBrand},
	types::optics::IndexedSetterPrime,
	functions::optics_indexed_over,
};
let l: IndexedSetterPrime<RcBrand, usize, Vec<i32>, i32, _> =
	IndexedSetterPrime::mapped::<VecBrand>();
let v = vec![10, 20];
let s = optics_indexed_over::<RcBrand, _, _, _, _, _>(&l, v, |i, x| x + i as i32);
assert_eq!(s, vec![10, 21]);
"#
		)]
		pub fn mapped() -> Self {
			IndexedSetterPrime::new(Mapped(std::marker::PhantomData))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The reference-counted pointer type for the lens.",
		"The setter function type."
	)]
	#[document_parameters("The indexed setter instance.")]
	impl<'a, Q, I: 'a, S: 'a, A: 'a, P, F> IndexedSetterOptic<'a, Q, I, S, S, A, A>
		for IndexedSetterPrime<'a, P, I, S, A, F>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
		F: IndexedSetterFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetterPrime<RcBrand, usize, Vec<i32>, i32, MySetter> =
	IndexedSetterPrime::new(MySetter);
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
assert_eq!(result(vec![10, 20]), vec![10, 21]);
"#
		)]
		fn evaluate(
			&self,
			pab: Indexed<'a, FnBrand<Q>, I, A, A>,
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
			let setter_fn = self.setter_fn.clone();
			<FnBrand<Q> as CloneableFn>::new(move |s: S| {
				let pab_fn = pab.inner.clone();
				setter_fn.apply(Box::new(move |i, a| pab_fn((i, a))), s)
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The reference-counted pointer type for the lens.",
		"The setter function type."
	)]
	#[document_parameters("The indexed setter instance.")]
	impl<'a, Q, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, P, F> IndexedSetterOptic<'a, Q, I, S, T, A, B>
		for IndexedSetter<'a, P, I, S, T, A, B, F>
	where
		F: IndexedSetterFunc<'a, I, S, T, A, B> + Clone + 'a,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::optics::*,
	types::optics::*,
};
struct MySetter;
impl<'a> IndexedSetterFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MySetter {
	fn apply(&self, f: Box<dyn Fn(usize, i32) -> i32 + 'a>, s: Vec<i32>) -> Vec<i32> {
		s.into_iter().enumerate().map(|(i, x)| f(i, x)).collect()
	}
}
let l: IndexedSetter<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MySetter> =
	IndexedSetter::new(MySetter);
let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32)) as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
let pab = Indexed::new(f);
let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
assert_eq!(result(vec![10, 20]), vec![10, 21]);
"#
		)]
		fn evaluate(
			&self,
			pab: Indexed<'a, FnBrand<Q>, I, A, B>,
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
			let setter_fn = self.setter_fn.clone();
			<FnBrand<Q> as CloneableFn>::new(move |s: S| {
				let pab_fn = pab.inner.clone();
				setter_fn.apply(Box::new(move |i, a| pab_fn((i, a))), s)
			})
		}
	}
}

pub use inner::*;
