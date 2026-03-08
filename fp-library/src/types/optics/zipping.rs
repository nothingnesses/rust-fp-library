//! The `Zipping` profunctor, used for zipping two structures through a grate.
//!
//! `Zipping<FnBrand, S, T>` wraps a cloneable function `Fn((S, S)) -> T`, enabling
//! point-wise combination of two structures via a grate optic.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				CloneableFn,
				profunctor::{
					Closed,
					Profunctor,
				},
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

	/// The `Zipping` profunctor.
	///
	/// Wraps a cloneable function `Fn((S, S)) -> T`. As a profunctor, `dimap f g z`
	/// pre-composes both inputs with `f` (by pairing) and post-composes the output with `g`.
	/// The `Closed` instance enables lifting `Zipping` over function types, which is the
	/// key step in the `zip_with_of` operation for grate optics.
	///
	/// Matches PureScript's `newtype Zipping a b = Zipping (a -> a -> b)`.
	#[document_type_parameters(
		"The lifetime of the function.",
		"The cloneable function brand.",
		"The input type (contravariant).",
		"The output type (covariant)."
	)]
	pub struct Zipping<'a, FunctionBrand: CloneableFn, S: 'a, T: 'a> {
		/// The binary function that zips two `S` values (as a pair) into a `T`.
		pub run: <FunctionBrand as CloneableFn>::Of<'a, (S, S), T>,
	}

	#[document_type_parameters(
		"The lifetime of the function.",
		"The cloneable function brand.",
		"The input type.",
		"The output type."
	)]
	impl<'a, FunctionBrand: CloneableFn, S: 'a, T: 'a> Zipping<'a, FunctionBrand, S, T> {
		/// Creates a new `Zipping` instance wrapping a binary function.
		#[document_signature]
		///
		#[document_parameters(
			"The binary function to wrap, taking a pair `(S, S)` and returning `T`."
		)]
		///
		#[document_returns("A new instance of the type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	types::optics::Zipping,
		/// };
		///
		/// let z = Zipping::<RcFnBrand, i32, i32>::new(|(a, b)| a + b);
		/// assert_eq!((z.run)((1, 2)), 3);
		/// ```
		pub fn new(f: impl Fn((S, S)) -> T + 'a) -> Self {
			Zipping {
				run: <FunctionBrand as CloneableFn>::new(f),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the function.",
		"The cloneable function brand.",
		"The input type.",
		"The output type."
	)]
	#[document_parameters("The zipping instance.")]
	impl<'a, FunctionBrand: CloneableFn, S: 'a, T: 'a> Clone for Zipping<'a, FunctionBrand, S, T> {
		#[document_signature]
		#[document_returns("A new `Zipping` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	types::optics::Zipping,
		/// };
		///
		/// let z = Zipping::<RcFnBrand, i32, i32>::new(|(a, b)| a + b);
		/// let z2 = z.clone();
		/// assert_eq!((z2.run)((3, 4)), 7);
		/// ```
		fn clone(&self) -> Self {
			Zipping {
				run: self.run.clone(),
			}
		}
	}

	/// Brand for the `Zipping` profunctor.
	#[document_type_parameters("The cloneable function brand.")]
	pub struct ZippingBrand<FunctionBrand>(PhantomData<FunctionBrand>);

	impl_kind! {
		impl<FunctionBrand: CloneableFn + 'static> for ZippingBrand<FunctionBrand> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Zipping<'a, FunctionBrand, S, T>;
		}
	}

	#[document_type_parameters("The cloneable function brand.")]
	impl<FunctionBrand: CloneableFn + 'static> Profunctor for ZippingBrand<FunctionBrand> {
		/// Maps over both arguments of the `Zipping` profunctor.
		///
		/// Matches PureScript's `dimap f g (Zipping z) = Zipping \a1 a2 -> g (z (f a1) (f a2))`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The new input type.",
			"The original input type.",
			"The original output type.",
			"The new output type.",
			"The type of the contravariant function.",
			"The type of the covariant function."
		)]
		///
		#[document_parameters(
			"The contravariant function to pre-compose on both inputs.",
			"The covariant function to post-compose on the output.",
			"The zipping instance to transform."
		)]
		#[document_returns("A transformed `Zipping` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::profunctor::Profunctor,
		/// 	types::optics::{
		/// 		Zipping,
		/// 		ZippingBrand,
		/// 	},
		/// };
		///
		/// let z = Zipping::<RcFnBrand, i32, i32>::new(|(a, b)| a + b);
		/// // dimap (*2) (+1) z = \a1 a2 -> (a1*2 + a2*2) + 1
		/// let z2 = <ZippingBrand<RcFnBrand> as Profunctor>::dimap(|x: i32| x * 2, |y: i32| y + 1, z);
		/// assert_eq!((z2.run)((3, 4)), 15); // (3*2 + 4*2) + 1 = 15
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, FuncAB, FuncCD>(
			ab: FuncAB,
			cd: FuncCD,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>)
		where
			FuncAB: Fn(A) -> B + 'a,
			FuncCD: Fn(C) -> D + 'a, {
			Zipping::new(move |(a1, a2)| cd((*pbc.run)((ab(a1), ab(a2)))))
		}
	}

	#[document_type_parameters("The cloneable function brand.")]
	impl<FunctionBrand: CloneableFn + 'static> Closed<FunctionBrand> for ZippingBrand<FunctionBrand> {
		/// Lifts `Zipping` to operate on functions.
		///
		/// Matches PureScript's `closed (Zipping z) = Zipping \f1 f2 x -> z (f1 x) (f2 x)`.
		///
		/// Given `Zipping<S, T>` (a binary function `Fn((S, S)) -> T`), produces
		/// `Zipping<FnBrand(X, S), FnBrand(X, T)>` where the result takes two `X -> S`
		/// functions (wrapped in `FnBrand`) and returns an `X -> T` function that applies
		/// both to the same `x` and combines the results.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type of the profunctor.",
			"The target type of the profunctor.",
			"The shared input type for the lifted functions."
		)]
		///
		#[document_parameters("The zipping instance to lift.")]
		#[document_returns("A transformed `Zipping` instance that operates on functions.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::{
		/// 		CloneableFn,
		/// 		profunctor::{
		/// 			Closed,
		/// 			Profunctor,
		/// 		},
		/// 	},
		/// 	types::optics::{
		/// 		Zipping,
		/// 		ZippingBrand,
		/// 	},
		/// };
		///
		/// let z = Zipping::<RcFnBrand, i32, i32>::new(|(a, b)| a + b);
		/// let zc = <ZippingBrand<RcFnBrand> as Closed<RcFnBrand>>::closed::<i32, i32, String>(z);
		/// // (zc.run)(f1, f2)(x) = f1(x) + f2(x)
		/// let f1 = <RcFnBrand as CloneableFn>::new(|s: String| s.len() as i32);
		/// let f2 = <RcFnBrand as CloneableFn>::new(|s: String| s.len() as i32 * 2);
		/// let result = (zc.run)((f1, f2));
		/// assert_eq!(result("hi".to_string()), 6); // 2 + 4
		/// ```
		fn closed<'a, S: 'a, T: 'a, X: 'a + Clone>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, <FunctionBrand as CloneableFn>::Of<'a, X, S>, <FunctionBrand as CloneableFn>::Of<'a, X, T>>)
		{
			Zipping::new(
				move |(f1, f2): (
					<FunctionBrand as CloneableFn>::Of<'a, X, S>,
					<FunctionBrand as CloneableFn>::Of<'a, X, S>,
				)| {
					let run = pab.run.clone();
					<FunctionBrand as CloneableFn>::new(move |x: X| {
						(*run)((f1(x.clone()), f2(x.clone())))
					})
				},
			)
		}
	}
}
pub use inner::*;
