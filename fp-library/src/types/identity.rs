//! Trivial wrapper that contains a single value.
//!
//! The simplest possible container type, often used as a base case for higher-kinded types or when a container is required but no additional effect is needed.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::IdentityBrand,
			classes::*,
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	/// Wraps a value.
	///
	/// The `Identity` type represents a trivial wrapper around a value. It is the simplest possible container.
	/// It is often used as a base case for higher-kinded types or when a container is required but no additional effect is needed.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`IdentityBrand`](crate::brands::IdentityBrand),
	/// which is fully polymorphic over the wrapped value type.
	///
	/// ### Serialization
	///
	/// This type supports serialization and deserialization via [`serde`](https://serde.rs) when the `serde` feature is enabled.
	#[document_type_parameters("The type of the wrapped value.")]
	///
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Identity<A>(
		/// The wrapped value.
		pub A,
	);

	impl_kind! {
		for IdentityBrand {
			type Of<'a, A: 'a>: 'a = Identity<A>;
		}
	}

	#[document_type_parameters("The type of the wrapped value.")]
	#[document_parameters("The identity instance.")]
	impl<A> Identity<A> {
		/// Maps a function over the value in the identity.
		///
		/// This is the inherent version of [`Functor::map`], accepting
		/// `FnOnce` instead of `Fn` since it consumes `self`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of applying the function.")]
		///
		#[document_parameters("The function to apply.")]
		///
		#[document_returns("A new identity containing the result of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Identity(5);
		/// let y = x.map(|i| i * 2);
		/// assert_eq!(y, Identity(10));
		/// ```
		pub fn map<B>(
			self,
			f: impl FnOnce(A) -> B,
		) -> Identity<B> {
			Identity(f(self.0))
		}

		/// Lifts a binary function to operate on two identities.
		///
		/// See [`Lift::lift2`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the other identity's value.",
			"The return type of the function."
		)]
		///
		#[document_parameters("The other identity.", "The binary function to apply.")]
		///
		#[document_returns("A new identity containing the result of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Identity(1);
		/// let y = Identity(2);
		/// let z = x.lift2(y, |a, b| a + b);
		/// assert_eq!(z, Identity(3));
		/// ```
		pub fn lift2<B, C>(
			self,
			other: Identity<B>,
			f: impl FnOnce(A, B) -> C,
		) -> Identity<C> {
			Identity(f(self.0, other.0))
		}

		/// Applies a wrapped function to a value.
		///
		/// See [`Semiapplicative::apply`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The return type of the wrapped function.")]
		///
		#[document_parameters("The identity containing the function.")]
		///
		#[document_returns("A new identity containing the result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let f = Identity(|x: i32| x * 2);
		/// let x = Identity(5);
		/// let y = x.apply(f);
		/// assert_eq!(y, Identity(10));
		/// ```
		pub fn apply<B>(
			self,
			ff: Identity<impl FnOnce(A) -> B>,
		) -> Identity<B> {
			Identity(ff.0(self.0))
		}

		/// Chains identity computations.
		///
		/// This is the inherent version of [`Semimonad::bind`], accepting
		/// `FnOnce` instead of `Fn` since it consumes `self`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the chained computation.")]
		///
		#[document_parameters("The function to apply to the value inside the identity.")]
		///
		#[document_returns("The result of applying `f` to the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Identity(5);
		/// let y = x.bind(|i| Identity(i * 2));
		/// assert_eq!(y, Identity(10));
		/// ```
		pub fn bind<B>(
			self,
			f: impl FnOnce(A) -> Identity<B>,
		) -> Identity<B> {
			f(self.0)
		}

		/// Folds the identity from the right.
		///
		/// See [`Foldable::fold_right`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The type of the accumulator.")]
		///
		#[document_parameters(
			"The function to apply to the element and the accumulator.",
			"The initial value of the accumulator."
		)]
		///
		#[document_returns("The final accumulator value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Identity(5);
		/// let y = x.fold_right(|a, b| a + b, 10);
		/// assert_eq!(y, 15);
		/// ```
		pub fn fold_right<B>(
			self,
			f: impl FnOnce(A, B) -> B,
			initial: B,
		) -> B {
			f(self.0, initial)
		}

		/// Folds the identity from the left.
		///
		/// See [`Foldable::fold_left`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The type of the accumulator.")]
		///
		#[document_parameters(
			"The function to apply to the accumulator and the element.",
			"The initial value of the accumulator."
		)]
		///
		#[document_returns("The final accumulator value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Identity(5);
		/// let y = x.fold_left(|b, a| b + a, 10);
		/// assert_eq!(y, 15);
		/// ```
		pub fn fold_left<B>(
			self,
			f: impl FnOnce(B, A) -> B,
			initial: B,
		) -> B {
			f(initial, self.0)
		}

		/// Maps the value to a monoid and returns it.
		///
		/// See [`Foldable::fold_map`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type.")]
		///
		#[document_parameters("The mapping function.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Identity(5);
		/// let y = x.fold_map(|a: i32| a.to_string());
		/// assert_eq!(y, "5".to_string());
		/// ```
		pub fn fold_map<M>(
			self,
			f: impl FnOnce(A) -> M,
		) -> M {
			f(self.0)
		}
	}

	#[document_type_parameters("The lifetime of the values.", "The type of the wrapped value.")]
	#[document_parameters("The identity instance.")]
	impl<'a, A: 'a> Identity<A> {
		/// Traverses the identity with an applicative function.
		///
		/// See [`Traversable::traverse`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the resulting identity.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply, returning a value in an applicative context."
		)]
		///
		#[document_returns("The identity wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = x.traverse::<_, OptionBrand>(|a| Some(a * 2));
		/// assert_eq!(y, Some(Identity(10)));
		/// ```
		pub fn traverse<B: 'a + Clone, F: Applicative>(
			self,
			f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Identity<B>>)
		where
			Identity<B>: Clone, {
			F::map(|b| Identity(b), f(self.0))
		}

		/// Sequences an identity containing an applicative value.
		///
		/// See [`Traversable::sequence`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The inner type wrapped in the applicative context.",
			"The applicative context."
		)]
		///
		#[document_returns("The identity wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(Some(5));
		/// let y: Option<Identity<i32>> = x.sequence::<i32, OptionBrand>();
		/// assert_eq!(y, Some(Identity(5)));
		/// ```
		pub fn sequence<InnerA: 'a + Clone, F: Applicative>(
			self
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Identity<InnerA>>)
		where
			A: Into<Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, InnerA>)>,
			Identity<InnerA>: Clone, {
			F::map(|a| Identity(a), self.0.into())
		}
	}

	impl Functor for IdentityBrand {
		/// Maps a function over the value in the identity.
		///
		/// This method applies a function to the value inside the identity, producing a new identity with the transformed value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the identity.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply.", "The identity to map over.")]
		///
		#[document_returns("A new identity containing the result of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = map::<IdentityBrand, _, _, _>(|i| i * 2, x);
		/// assert_eq!(y, Identity(10));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	impl Lift for IdentityBrand {
		/// Lifts a binary function into the identity context.
		///
		/// This method lifts a binary function to operate on values within the identity context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first identity's value.",
			"The type of the second identity's value.",
			"The return type of the function."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first identity.",
			"The second identity."
		)]
		///
		#[document_returns("A new identity containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(1);
		/// let y = Identity(2);
		/// let z = lift2::<IdentityBrand, _, _, _>(|a, b| a + b, x, y);
		/// assert_eq!(z, Identity(3));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: 'a,
			B: 'a,
			C: 'a, {
			fa.lift2(fb, func)
		}
	}

	impl Pointed for IdentityBrand {
		/// Wraps a value in an identity.
		///
		/// This method wraps a value in an identity context.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An identity containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = pure::<IdentityBrand, _>(5);
		/// assert_eq!(x, Identity(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Identity(a) // Identity constructor is already equivalent to pure
		}
	}

	impl ApplyFirst for IdentityBrand {}
	impl ApplySecond for IdentityBrand {}

	impl Semiapplicative for IdentityBrand {
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in an identity to a value wrapped in an identity.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The identity containing the function.",
			"The identity containing the value."
		)]
		///
		#[document_returns("A new identity containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Identity(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let x = Identity(5);
		/// let y = apply::<RcFnBrand, IdentityBrand, _, _>(f, x);
		/// assert_eq!(y, Identity(10));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.apply(ff.map(|f| move |a| f(a)))
		}
	}

	impl Semimonad for IdentityBrand {
		/// Chains identity computations.
		///
		/// This method chains two identity computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first identity.",
			"The function to apply to the value inside the identity."
		)]
		///
		#[document_returns("The result of applying `f` to the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = bind::<IdentityBrand, _, _>(x, |i| Identity(i * 2));
		/// assert_eq!(y, Identity(10));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	impl Foldable for IdentityBrand {
		/// Folds the identity from the right.
		///
		/// This method performs a right-associative fold of the identity. Since `Identity` contains only one element, this is equivalent to applying the function to the element and the initial value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The identity to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = fold_right::<RcFnBrand, IdentityBrand, _, _>(|a, b| a + b, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			fa.fold_right(func, initial)
		}

		/// Folds the identity from the left.
		///
		/// This method performs a left-associative fold of the identity. Since `Identity` contains only one element, this is equivalent to applying the function to the initial value and the element.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = fold_left::<RcFnBrand, IdentityBrand, _, _>(|b, a| b + a, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			fa.fold_left(func, initial)
		}

		/// Maps the value to a monoid and returns it.
		///
		/// This method maps the element of the identity to a monoid.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The identity to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = fold_map::<RcFnBrand, IdentityBrand, _, _>(|a: i32| a.to_string(), x);
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			fa.fold_map(func)
		}
	}

	impl Traversable for IdentityBrand {
		/// Traverses the identity with an applicative function.
		///
		/// This method maps the element of the identity to a computation, evaluates it, and wraps the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The identity to traverse."
		)]
		///
		#[document_returns("The identity wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(5);
		/// let y = traverse::<IdentityBrand, _, _, OptionBrand>(|a| Some(a * 2), x);
		/// assert_eq!(y, Some(Identity(10)));
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			ta.traverse::<B, F>(func)
		}

		/// Sequences an identity of applicative.
		///
		/// This method evaluates the computation inside the identity and wraps the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The identity containing the applicative value.")]
		///
		#[document_returns("The result of the traversal.")]
		///
		/// # Returns
		///
		/// The identity wrapped in the applicative context.
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Identity(Some(5));
		/// let y = sequence::<IdentityBrand, _, OptionBrand>(x);
		/// assert_eq!(y, Some(Identity(5)));
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			ta.traverse::<A, F>(|a| a)
		}
	}

	impl MonadRec for IdentityBrand {
		/// Performs tail-recursive monadic computation over [`Identity`].
		///
		/// Since `Identity` has no effect, this simply loops on the inner value
		/// until the step function returns [`ControlFlow::Break`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("An identity containing the result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<IdentityBrand, _, _>(
		/// 	|n| {
		/// 		if n < 10 {
		/// 			Identity(ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			Identity(ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, Identity(10));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut current = initial;
			loop {
				match func(current).0 {
					ControlFlow::Continue(next) => current = next,
					ControlFlow::Break(b) => return Identity(b),
				}
			}
		}
	}

	impl Extract for IdentityBrand {
		/// Extracts the inner value from an `Identity` by unwrapping it.
		///
		/// Since `Identity` is a trivial wrapper, extraction simply returns the
		/// contained value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the identity."
		)]
		///
		#[document_parameters("The identity to extract from.")]
		///
		#[document_returns("The inner value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let id = Identity(42);
		/// assert_eq!(extract::<IdentityBrand, _>(id), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			fa.0
		}
	}

	impl Extend for IdentityBrand {
		/// Extends a local computation to the `Identity` context.
		///
		/// Applies the function to the entire `Identity` and wraps the result in
		/// a new `Identity`. Since `Identity` contains exactly one value, extending
		/// is equivalent to applying the function and re-wrapping.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the identity.",
			"The result type of the extension function."
		)]
		///
		#[document_parameters(
			"The function that consumes an `Identity` and produces a value.",
			"The identity to extend over."
		)]
		///
		#[document_returns("A new identity containing the result of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let id = Identity(5);
		/// let result = extend::<IdentityBrand, _, _>(|w: Identity<i32>| w.0 * 2, id);
		/// assert_eq!(result, Identity(10));
		/// ```
		fn extend<'a, A: 'a + Clone, B: 'a>(
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Identity(f(wa))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::inner::Identity,
		crate::{
			brands::{
				IdentityBrand,
				OptionBrand,
				RcFnBrand,
			},
			classes::*,
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		let x = Identity(x);
		map::<IdentityBrand, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let x = Identity(x);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<IdentityBrand, _, _, _>(compose(f, g), x)
			== map::<IdentityBrand, _, _, _>(f, map::<IdentityBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: i32) -> bool {
		let v = Identity(v);
		apply::<RcFnBrand, IdentityBrand, _, _>(
			pure::<IdentityBrand, _>(<RcFnBrand as LiftFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, IdentityBrand, _, _>(
			pure::<IdentityBrand, _>(<RcFnBrand as LiftFn>::new(f)),
			pure::<IdentityBrand, _>(x),
		) == pure::<IdentityBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: i32,
		u_val: i32,
		v_val: i32,
	) -> bool {
		let w = Identity(w);
		let v_fn = move |x: i32| x.wrapping_mul(v_val);
		let u_fn = move |x: i32| x.wrapping_add(u_val);

		let v = pure::<IdentityBrand, _>(<RcFnBrand as LiftFn>::new(v_fn));
		let u = pure::<IdentityBrand, _>(<RcFnBrand as LiftFn>::new(u_fn));

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, IdentityBrand, _, _>(v.clone(), w);
		let rhs = apply::<RcFnBrand, IdentityBrand, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let composed = move |x| u_fn(v_fn(x));
		let uv = pure::<IdentityBrand, _>(<RcFnBrand as LiftFn>::new(composed));

		let lhs = apply::<RcFnBrand, IdentityBrand, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<IdentityBrand, _>(<RcFnBrand as LiftFn>::new(f));

		let lhs = apply::<RcFnBrand, IdentityBrand, _, _>(u.clone(), pure::<IdentityBrand, _>(y));

		let rhs_fn = <RcFnBrand as LiftFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, IdentityBrand, _, _>(pure::<IdentityBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Identity(x.wrapping_mul(2));
		bind::<IdentityBrand, _, _>(pure::<IdentityBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: i32) -> bool {
		let m = Identity(m);
		bind::<IdentityBrand, _, _>(m, pure::<IdentityBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: i32) -> bool {
		let m = Identity(m);
		let f = |x: i32| Identity(x.wrapping_mul(2));
		let g = |x: i32| Identity(x.wrapping_add(1));
		bind::<IdentityBrand, _, _>(bind::<IdentityBrand, _, _>(m, f), g)
			== bind::<IdentityBrand, _, _>(m, |x| bind::<IdentityBrand, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests the `map` function.
	#[test]
	fn map_test() {
		assert_eq!(map::<IdentityBrand, _, _, _>(|x: i32| x + 1, Identity(1)), Identity(2));
	}

	/// Tests the `bind` function.
	#[test]
	fn bind_test() {
		assert_eq!(bind::<IdentityBrand, _, _>(Identity(1), |x| Identity(x + 1)), Identity(2));
	}

	/// Tests the `fold_right` function.
	#[test]
	fn fold_right_test() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, IdentityBrand, _, _>(
				|x: i32, acc| x + acc,
				0,
				Identity(1)
			),
			1
		);
	}

	/// Tests the `fold_left` function.
	#[test]
	fn fold_left_test() {
		assert_eq!(
			crate::classes::foldable::fold_left::<RcFnBrand, IdentityBrand, _, _>(
				|acc, x: i32| acc + x,
				0,
				Identity(1)
			),
			1
		);
	}

	/// Tests the `traverse` function.
	#[test]
	fn traverse_test() {
		assert_eq!(
			crate::classes::traversable::traverse::<IdentityBrand, _, _, OptionBrand>(
				|x: i32| Some(x + 1),
				Identity(1)
			),
			Some(Identity(2))
		);
	}

	// MonadRec tests

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<IdentityBrand, _, _>(|a| Identity(ControlFlow::Break(a)), x) == Identity(x)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`.
	#[test]
	fn monad_rec_sum_range() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result = tail_rec_m::<IdentityBrand, _, _>(
			|(n, acc)| {
				if n == 0 {
					Identity(ControlFlow::Break(acc))
				} else {
					Identity(ControlFlow::Continue((n - 1, acc + n)))
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, Identity(5050));
	}

	/// Tests stack safety: `tail_rec_m` handles large iteration counts.
	#[test]
	fn monad_rec_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<IdentityBrand, _, _>(
			|acc| {
				if acc < iterations {
					Identity(ControlFlow::Continue(acc + 1))
				} else {
					Identity(ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, Identity(iterations));
	}

	// Extract / Extend / Comonad Laws

	/// Extract pure-extract law: `extract(pure(x)) == x`.
	#[quickcheck]
	fn extract_pure(x: i32) -> bool {
		use crate::classes::extract::extract;
		extract::<IdentityBrand, _>(pure::<IdentityBrand, _>(x)) == x
	}

	/// Comonad left identity: `extract(extend(f, wa)) == f(wa)`.
	#[quickcheck]
	fn comonad_left_identity(x: i32) -> bool {
		use crate::classes::{
			extend::extend,
			extract::extract,
		};
		let f = |w: Identity<i32>| w.0.wrapping_mul(3);
		let wa = Identity(x);
		extract::<IdentityBrand, _>(extend::<IdentityBrand, _, _>(f, wa)) == f(wa)
	}

	/// Comonad right identity: `extend(extract, wa) == wa`.
	#[quickcheck]
	fn comonad_right_identity(x: i32) -> bool {
		use crate::classes::{
			extend::extend,
			extract::extract,
		};
		extend::<IdentityBrand, _, _>(extract::<IdentityBrand, _>, Identity(x)) == Identity(x)
	}

	/// Extend associativity: `extend(f, extend(g, w)) == extend(|w| f(extend(g, w)), w)`.
	#[quickcheck]
	fn extend_associativity(x: i32) -> bool {
		use crate::classes::extend::extend;
		let g = |w: Identity<i32>| w.0.wrapping_mul(2);
		let f = |w: Identity<i32>| w.0.wrapping_add(1);
		let wa = Identity(x);
		let lhs = extend::<IdentityBrand, _, _>(f, extend::<IdentityBrand, _, _>(g, wa));
		let rhs = extend::<IdentityBrand, _, _>(
			|w: Identity<i32>| f(extend::<IdentityBrand, _, _>(g, w)),
			wa,
		);
		lhs == rhs
	}

	/// Map-extract law: `extract(map(f, wa)) == f(extract(wa))`.
	#[quickcheck]
	fn comonad_map_extract(x: i32) -> bool {
		use crate::classes::extract::extract;
		let f = |a: i32| a.wrapping_mul(5);
		let wa = Identity(x);
		extract::<IdentityBrand, _>(map::<IdentityBrand, _, _, _>(f, wa))
			== f(extract::<IdentityBrand, _>(wa))
	}

	/// Tests basic `extract` on `Identity`.
	#[test]
	fn extract_test() {
		use crate::classes::extract::extract;
		assert_eq!(extract::<IdentityBrand, _>(Identity(42)), 42);
	}

	/// Tests basic `extend` on `Identity`.
	#[test]
	fn extend_test() {
		use crate::classes::extend::extend;
		let result = extend::<IdentityBrand, _, _>(|w: Identity<i32>| w.0 * 2, Identity(21));
		assert_eq!(result, Identity(42));
	}
}
