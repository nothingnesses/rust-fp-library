//! Data structures that can be traversed and filtered simultaneously in an applicative context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = wither::<OptionBrand, OptionBrand, _, _>(
//! 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
//! 	x,
//! );
//! assert_eq!(y, Some(Some(10)));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for data structures that can be traversed and filtered.
	///
	/// `Witherable` extends [`Filterable`] and [`Traversable`], adding methods for:
	/// *   `wither`: Effectful `filter_map`.
	/// *   `wilt`: Effectful `partition_map`.
	///
	/// ### Laws
	///
	/// `Witherable` instances must satisfy the following laws:
	/// * Identity: `wither(|a| pure(Some(a)), fa) = pure(fa)`.
	/// * Multipass (filter): `wither(p, fa) = map(|r| compact(r), traverse(p, fa))`.
	/// * Multipass (partition): `wilt(p, fa) = map(|r| separate(r), traverse(p, fa))`.
	///
	/// Superclass equivalences:
	/// * `filter_map(p, fa) = unwrap(wither(|a| Identity(p(a)), fa))`.
	/// * `partition_map(p, fa) = unwrap(wilt(|a| Identity(p(a)), fa))`.
	/// * `traverse(f, fa) = wither(|a| map(Some, f(a)), fa)`.
	#[document_examples]
	///
	/// Witherable laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Identity: wither(|a| pure(Some(a)), fa) = pure(fa)
	/// assert_eq!(wither::<OptionBrand, OptionBrand, _, _>(|a| Some(Some(a)), Some(5)), Some(Some(5)),);
	/// assert_eq!(
	/// 	wither::<OptionBrand, OptionBrand, _, _>(|a| Some(Some(a)), None::<i32>),
	/// 	Some(None),
	/// );
	///
	/// // Multipass (filter): wither(p, fa) = map(|r| compact(r), traverse(p, fa))
	/// let p = |a: i32| Some(if a > 2 { Some(a * 2) } else { None });
	/// assert_eq!(
	/// 	wither::<OptionBrand, OptionBrand, _, _>(p, Some(5)),
	/// 	map::<OptionBrand, _, _>(
	/// 		|r| compact::<OptionBrand, _>(r),
	/// 		traverse::<OptionBrand, _, _, OptionBrand>(p, Some(5)),
	/// 	),
	/// );
	///
	/// // Multipass (partition): wilt(p, fa) = map(|r| separate(r), traverse(p, fa))
	/// let p = |a: i32| Some(if a > 2 { Ok(a) } else { Err(a) });
	/// assert_eq!(
	/// 	wilt::<OptionBrand, OptionBrand, _, _, _>(p, Some(5)),
	/// 	map::<OptionBrand, _, _>(
	/// 		|r| separate::<OptionBrand, _, _>(r),
	/// 		traverse::<OptionBrand, _, _, OptionBrand>(p, Some(5)),
	/// 	),
	/// );
	/// ```
	///
	/// Witherable laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Identity: wither(|a| pure(Some(a)), fa) = pure(fa)
	/// assert_eq!(
	/// 	wither::<VecBrand, OptionBrand, _, _>(|a| Some(Some(a)), vec![1, 2, 3]),
	/// 	Some(vec![1, 2, 3]),
	/// );
	///
	/// // Multipass (filter): wither(p, fa) = map(|r| compact(r), traverse(p, fa))
	/// let p = |a: i32| Some(if a > 2 { Some(a * 2) } else { None });
	/// assert_eq!(
	/// 	wither::<VecBrand, OptionBrand, _, _>(p, vec![1, 2, 3, 4, 5]),
	/// 	map::<OptionBrand, _, _>(
	/// 		|r| compact::<VecBrand, _>(r),
	/// 		traverse::<VecBrand, _, _, OptionBrand>(p, vec![1, 2, 3, 4, 5]),
	/// 	),
	/// );
	/// ```
	///
	/// ### Minimal Implementation
	///
	/// A minimal implementation of `Witherable` requires no specific method implementations, as all methods have default implementations based on [`Traversable`] and [`Compactable`](crate::classes::compactable::Compactable).
	///
	/// However, it is recommended to implement [`Witherable::wilt`] and [`Witherable::wither`] to avoid the intermediate structure created by the default implementations (which use [`traverse`](crate::functions::traverse) followed by [`separate`](crate::functions::separate) or [`compact`](crate::functions::compact)).
	pub trait Witherable: Filterable + Traversable {
		/// Partitions a data structure based on a function that returns a [`Result`] in an applicative context.
		///
		/// The default implementation uses [`traverse`](crate::functions::traverse) and [`separate`](crate::functions::separate).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The applicative context.",
			"The type of the elements in the input structure.",
			"The type of the error values.",
			"The type of the success values."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a [`Result`] in an applicative context.",
			"The data structure to partition."
		)]
		///
		#[document_returns("The partitioned data structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Some(5);
		/// let y =
		/// 	wilt::<OptionBrand, OptionBrand, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
		/// assert_eq!(y, Some((None, Some(5))));
		/// ```
		fn wilt<'a, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
			func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
			+ 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			(
				Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
				Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			),
		>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone, {
			M::map(
				|res| Self::separate::<E, O>(res),
				Self::traverse::<A, Result<O, E>, M>(func, ta),
			)
		}

		/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
		///
		/// The default implementation uses [`traverse`](crate::functions::traverse) and [`compact`](crate::functions::compact).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The applicative context.",
			"The type of the elements in the input structure.",
			"The type of the elements in the output structure."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning an [`Option`] in an applicative context.",
			"The data structure to filter and map."
		)]
		///
		#[document_returns(
			"The filtered and mapped data structure wrapped in the applicative context."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = wither::<OptionBrand, OptionBrand, _, _>(
		/// 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
		/// 	x,
		/// );
		/// assert_eq!(y, Some(Some(10)));
		/// ```
		fn wither<'a, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
			func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone, {
			M::map(|opt| Self::compact(opt), Self::traverse::<A, Option<B>, M>(func, ta))
		}
	}

	/// Partitions a data structure based on a function that returns a [`Result`] in an applicative context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Witherable::wilt`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the witherable structure.",
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the error values.",
		"The type of the success values."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a [`Result`] in an applicative context.",
		"The data structure to partition."
	)]
	///
	#[document_returns("The partitioned data structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let x = Some(5);
	/// let y =
	/// 	wilt::<OptionBrand, OptionBrand, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
	/// assert_eq!(y, Some((None, Some(5))));
	/// ```
	pub fn wilt<'a, F: Witherable, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
		func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone, {
		F::wilt::<M, A, E, O>(func, ta)
	}

	/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Witherable::wither`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the witherable structure.",
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning an [`Option`] in an applicative context.",
		"The data structure to filter and map."
	)]
	///
	#[document_returns(
		"The filtered and mapped data structure wrapped in the applicative context."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = wither::<OptionBrand, OptionBrand, _, _>(
	/// 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
	/// 	x,
	/// );
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn wither<'a, F: Witherable, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
		func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
		ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone, {
		F::wither::<M, A, B>(func, ta)
	}
}

pub use inner::*;
