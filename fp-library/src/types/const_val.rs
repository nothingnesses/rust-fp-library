//! The `Const` functor, which ignores its second type parameter.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ConstBrand,
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
		fp_macros::*,
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
	#[document_parameters("The `Const` instance.")]
	impl<'a, R, A> Const<'a, R, A> {
		/// Creates a new `Const` instance.
		#[document_signature]
		#[document_parameters("The value to store.")]
		#[document_returns("A new `Const` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::const_val::Const;
		///
		/// let c: Const<i32, String> = Const::new(42);
		/// assert_eq!(c.0, 42);
		/// ```
		pub fn new(r: R) -> Self {
			Const(r, PhantomData)
		}

		/// Maps over the phantom type parameter, preserving the stored value.
		///
		/// Since `Const` ignores its second type parameter, the function is never called.
		/// This is the inherent method form of [`Functor::map`](crate::classes::functor::Functor::map).
		#[document_signature]
		#[document_type_parameters("The new phantom type.")]
		#[document_parameters("The function to map (ignored).")]
		#[document_returns(
			"A new `Const` instance with the same stored value but a different phantom type."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::const_val::Const;
		///
		/// let c: Const<i32, String> = Const::new(42);
		/// let mapped: Const<i32, bool> = c.map(|s: String| s.is_empty());
		/// assert_eq!(mapped.0, 42);
		/// ```
		pub fn map<B>(
			self,
			_f: impl FnOnce(A) -> B,
		) -> Const<'a, R, B> {
			Const::new(self.0)
		}

		/// Combines two `Const` values by appending their stored values, discarding the phantom types.
		///
		/// This is the inherent method form of [`Lift::lift2`](crate::classes::lift::Lift::lift2).
		#[document_signature]
		#[document_type_parameters("The second phantom type.", "The result phantom type.")]
		#[document_parameters("The other `Const` instance.", "The function to lift (ignored).")]
		#[document_returns("A new `Const` instance with the appended stored values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::const_val::Const;
		///
		/// let c1: Const<String, i32> = Const::new("Hello".to_string());
		/// let c2: Const<String, i32> = Const::new(" World".to_string());
		/// let lifted = c1.lift2(c2, |a: i32, b: i32| a + b);
		/// assert_eq!(lifted.0, "Hello World");
		/// ```
		pub fn lift2<B, C>(
			self,
			other: Const<'a, R, B>,
			_f: impl FnOnce(A, B) -> C,
		) -> Const<'a, R, C>
		where
			R: Semigroup, {
			Const::new(R::append(self.0, other.0))
		}

		/// Combines the stored values of two `Const` instances, keeping the phantom type of the first.
		///
		/// This is the inherent method form of [`ApplyFirst::apply_first`](crate::classes::apply_first::ApplyFirst::apply_first).
		#[document_signature]
		#[document_type_parameters("The phantom type of the second `Const` instance.")]
		#[document_parameters("The second `Const` instance.")]
		#[document_returns("A new `Const` instance with the appended stored values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::const_val::Const;
		///
		/// let c1: Const<String, i32> = Const::new("Hello".to_string());
		/// let c2: Const<String, bool> = Const::new(" World".to_string());
		/// let result = c1.apply_first(c2);
		/// assert_eq!(result.0, "Hello World");
		/// ```
		pub fn apply_first<B>(
			self,
			other: Const<'a, R, B>,
		) -> Const<'a, R, A>
		where
			R: Semigroup, {
			Const::new(R::append(self.0, other.0))
		}

		/// Combines the stored values of two `Const` instances, keeping the phantom type of the second.
		///
		/// This is the inherent method form of [`ApplySecond::apply_second`](crate::classes::apply_second::ApplySecond::apply_second).
		#[document_signature]
		#[document_type_parameters("The phantom type of the second `Const` instance.")]
		#[document_parameters("The second `Const` instance.")]
		#[document_returns("A new `Const` instance with the appended stored values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::const_val::Const;
		///
		/// let c1: Const<String, i32> = Const::new("Hello".to_string());
		/// let c2: Const<String, bool> = Const::new(" World".to_string());
		/// let result = c1.apply_second(c2);
		/// assert_eq!(result.0, "Hello World");
		/// ```
		pub fn apply_second<B>(
			self,
			other: Const<'a, R, B>,
		) -> Const<'a, R, B>
		where
			R: Semigroup, {
			Const::new(R::append(self.0, other.0))
		}

		/// Creates a `Const` with the monoidal identity, ignoring the given value.
		///
		/// This is the inherent method form of [`Pointed::pure`](crate::classes::pointed::Pointed::pure).
		#[document_signature]
		#[document_parameters("The value to wrap (ignored).")]
		#[document_returns("A new `Const` instance with the empty value of the stored type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::const_val::Const;
		///
		/// let c: Const<String, i32> = Const::pure(42);
		/// assert_eq!(c.0, "".to_string());
		/// ```
		pub fn pure(_a: A) -> Self
		where
			R: Monoid, {
			Const::new(R::empty())
		}
	}

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
			"The output type."
		)]
		#[document_parameters(
			"The function to map (ignored).",
			"The `Const` instance to map over."
		)]
		#[document_returns("A new `Const` instance with the same stored value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ConstBrand,
		/// 	classes::functor::Functor,
		/// 	types::const_val::Const,
		/// };
		///
		/// let c: Const<i32, String> = Const::new(42);
		/// let mapped = ConstBrand::map(|s: String| s.len(), c);
		/// assert_eq!(mapped.0, 42);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			_f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>) {
			fa.map(_f)
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Semigroup> Lift for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The first input type.",
			"The second input type.",
			"The output type."
		)]
		#[document_parameters(
			"The function to lift (ignored).",
			"The first `Const` instance.",
			"The second `Const` instance."
		)]
		#[document_returns("A new `Const` instance with the combined stored values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ConstBrand,
		/// 	classes::lift::Lift,
		/// 	types::const_val::Const,
		/// };
		///
		/// let c1: Const<String, i32> = Const::new("Hello".to_string());
		/// let c2: Const<String, i32> = Const::new(" World".to_string());
		/// let lifted = ConstBrand::lift2(|a: i32, b: i32| a + b, c1, c2);
		/// assert_eq!(lifted.0, "Hello World");
		/// ```
		fn lift2<'a, A, B, C>(
			_func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			fa.lift2(fb, _func)
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
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ConstBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::CloneableFn,
		/// 		semiapplicative::Semiapplicative,
		/// 	},
		/// 	types::const_val::Const,
		/// };
		///
		/// let c1 = Const::<String, _>::new("Hello".to_string());
		/// let c2 = Const::<String, i32>::new(" World".to_string());
		/// let applied = ConstBrand::<String>::apply::<RcFnBrand, i32, i32>(c1, c2);
		/// assert_eq!(applied.0, "Hello World");
		/// ```
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
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ConstBrand,
		/// 	classes::apply_first::ApplyFirst,
		/// 	types::const_val::Const,
		/// };
		///
		/// let c1: Const<String, i32> = Const::new("Hello".to_string());
		/// let c2: Const<String, i32> = Const::new(" World".to_string());
		/// let applied = ConstBrand::apply_first(c1, c2);
		/// assert_eq!(applied.0, "Hello World");
		/// ```
		fn apply_first<'a, A: 'a, B: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>) {
			fa.apply_first(fb)
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
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ConstBrand,
		/// 	classes::apply_second::ApplySecond,
		/// 	types::const_val::Const,
		/// };
		///
		/// let c1: Const<String, i32> = Const::new("Hello".to_string());
		/// let c2: Const<String, i32> = Const::new(" World".to_string());
		/// let applied = ConstBrand::apply_second(c1, c2);
		/// assert_eq!(applied.0, "Hello World");
		/// ```
		fn apply_second<'a, A: 'a, B: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>) {
			fa.apply_second(fb)
		}
	}

	#[document_type_parameters("The stored type.")]
	impl<R: 'static + Monoid> Pointed for ConstBrand<R> {
		#[document_signature]
		#[document_type_parameters("The lifetime of the values.", "The type to wrap (ignored).")]
		#[document_parameters("The value to wrap (ignored).")]
		#[document_returns("A new `Const` instance with the empty value of the stored type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ConstBrand,
		/// 	classes::pointed::Pointed,
		/// 	types::const_val::Const,
		/// };
		///
		/// let c: Const<String, i32> = ConstBrand::pure(42);
		/// assert_eq!(c.0, "".to_string());
		/// ```
		fn pure<'a, A: 'a>(_a: A) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>) {
			Const::pure(_a)
		}
	}
}
pub use inner::*;

// `A` is only `PhantomData<&'a A>` - always Clone/Copy - so we only need `R: Clone`/`R: Copy`.
impl<'a, R: Clone, A> Clone for Const<'a, R, A> {
	fn clone(&self) -> Self {
		Const(self.0.clone(), std::marker::PhantomData)
	}
}
impl<'a, R: Copy, A> Copy for Const<'a, R, A> {}
