//! The `Const` functor, which ignores its second type parameter.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				apply_first::ApplyFirst,
				apply_second::ApplySecond,
				cloneable_fn::CloneableFn,
				functor::Functor,
				lift::Lift,
				monoid::Monoid,
				pointed::Pointed,
				semiapplicative::Semiapplicative,
				semigroup::Semigroup,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{
			document_examples,
			document_parameters,
			document_returns,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// The `Const` functor.
	///
	/// `Const<R, A>` stores a value of type `R` and ignores the type `A`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The stored type.",
		"The ignored type."
	)]
	#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct Const<'a, R, A>(pub R, pub PhantomData<&'a A>);

	#[document_type_parameters(
		"The lifetime of the values.",
		"The stored type.",
		"The ignored type."
	)]
	impl<'a, R, A> Const<'a, R, A> {
		/// Creates a new `Const` instance.
		#[document_signature]
		#[document_parameters("The value to store.")]
		#[document_returns("A new `Const` instance.")]
		#[document_examples(
			r#"use fp_library::types::const_val::Const;

let c: Const<i32, String> = Const::new(42);
assert_eq!(c.0, 42);"#
		)]
		pub fn new(r: R) -> Self {
			Const(r, PhantomData)
		}
	}

	/// Brand for the `Const` functor.
	pub struct ConstBrand<R>(PhantomData<R>);

	impl_kind! {
		impl<R: 'static> for ConstBrand<R> {
			type Of<'a, A: 'a>: 'a = Const<'a, R, A>;
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static> Functor for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type.",
			"The output type.",
			"The function type."
		)]
		#[document_parameters(
			"The function to map (ignored).",
			"The `Const` instance to map over."
		)]
		#[document_returns("A new `Const` instance with the same stored value.")]
		#[document_examples(
			r#"use fp_library::{classes::functor::Functor, types::const_val::{Const, ConstBrand}};

let c: Const<i32, String> = Const::new(42);
let mapped = ConstBrand::map(|s: String| s.len(), c);
assert_eq!(mapped.0, 42);"#
		)]
		fn map<'a, A: 'a, B: 'a, F>(
			_f: F,
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>)
		where
			F: Fn(A) -> B + 'a, {
			Const::new(fa.0)
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Semigroup> Lift for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The first input type.",
			"The second input type.",
			"The output type.",
			"The function type."
		)]
		#[document_parameters(
			"The function to lift (ignored).",
			"The first `Const` instance.",
			"The second `Const` instance."
		)]
		#[document_returns("A new `Const` instance with the combined stored values.")]
		#[document_examples(
			r#"use fp_library::{classes::lift::Lift, types::const_val::{Const, ConstBrand}};

let c1: Const<String, i32> = Const::new("Hello".to_string());
let c2: Const<String, i32> = Const::new(" World".to_string());
let lifted = ConstBrand::lift2(|a: i32, b: i32| a + b, c1, c2);
assert_eq!(lifted.0, "Hello World");"#
		)]
		fn lift2<'a, A, B, C, Func>(
			_func: Func,
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, C>)
		where
			Func: Fn(A, B) -> C + 'a,
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			Const::new(R::append(fa.0, fb.0))
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Semigroup> Semiapplicative for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters(
			"The `Const` instance containing a function.",
			"The `Const` instance containing a value."
		)]
		#[document_returns("A new `Const` instance with the combined stored values.")]
		#[document_examples(r#"use fp_library::{brands::RcFnBrand, classes::{semiapplicative::Semiapplicative, cloneable_fn::CloneableFn}, types::const_val::{Const, ConstBrand}};

let c1 = Const::<String, _>::new("Hello".to_string());
let c2 = Const::<String, i32>::new(" World".to_string());
let applied = ConstBrand::<String>::apply::<RcFnBrand, i32, i32>(c1, c2);
assert_eq!(applied.0, "Hello World");"#)]
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>) {
			Const::new(R::append(ff.0, fa.0))
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Semigroup> ApplyFirst for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The first type.",
			"The second type."
		)]
		#[document_parameters("The first `Const` instance.", "The second `Const` instance.")]
		#[document_returns("A new `Const` instance with the combined stored values.")]
		#[document_examples(r#"use fp_library::{classes::apply_first::ApplyFirst, types::const_val::{Const, ConstBrand}};

let c1: Const<String, i32> = Const::new("Hello".to_string());
let c2: Const<String, i32> = Const::new(" World".to_string());
let applied = ConstBrand::apply_first(c1, c2);
assert_eq!(applied.0, "Hello World");"#)]
		fn apply_first<'a, A: 'a, B: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>) {
			Const::new(R::append(fa.0, fb.0))
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Semigroup> ApplySecond for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The first type.",
			"The second type."
		)]
		#[document_parameters("The first `Const` instance.", "The second `Const` instance.")]
		#[document_returns("A new `Const` instance with the combined stored values.")]
		#[document_examples(r#"use fp_library::{classes::apply_second::ApplySecond, types::const_val::{Const, ConstBrand}};

let c1: Const<String, i32> = Const::new("Hello".to_string());
let c2: Const<String, i32> = Const::new(" World".to_string());
let applied = ConstBrand::apply_second(c1, c2);
assert_eq!(applied.0, "Hello World");"#)]
		fn apply_second<'a, A: 'a, B: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>) {
			Const::new(R::append(fa.0, fb.0))
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Monoid> Pointed for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters("The lifetime of the values.", "The type to wrap (ignored).")]
		#[document_parameters("The value to wrap (ignored).")]
		#[document_returns("A new `Const` instance with the empty value of the stored type.")]
		#[document_examples(
			r#"use fp_library::{classes::pointed::Pointed, types::const_val::{Const, ConstBrand}};

let c: Const<String, i32> = ConstBrand::pure(42);
assert_eq!(c.0, "".to_string());"#
		)]
		fn pure<'a, A: 'a>(_a: A) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>) {
			Const::new(R::empty())
		}
	}
}
pub use inner::*;

// `A` is only `PhantomData<&'a A>` — always Clone/Copy — so we only need `R: Clone`/`R: Copy`.
impl<'a, R: Clone, A> Clone for Const<'a, R, A> {
	fn clone(&self) -> Self {
		Const(self.0.clone(), std::marker::PhantomData)
	}
}
impl<'a, R: Copy, A> Copy for Const<'a, R, A> {}
