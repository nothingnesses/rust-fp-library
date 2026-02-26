//! The `Stall` profunctor, used for affine traversals.
//!
//! `Stall<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a setter function `S -> B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				CloneableFn,
				profunctor::{
					Choice,
					Profunctor,
					Strong,
				},
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// The `Stall` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Stall<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Preview function: tries to extract the focus.
		pub get: <FnBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
		/// Setter function.
		pub set: <FnBrand as CloneableFn>::Of<'a, (S, B), T>,
		pub(crate) _phantom: PhantomData<(A, B)>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Stall<'a, FnBrand, A, B, S, T> {
		/// Creates a new `Stall` instance.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The setter function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as cloneable_fn_new,
		/// 	types::optics::Stall,
		/// };
		///
		/// let stall = Stall::<RcFnBrand, i32, i32, (i32, i32), (i32, i32)>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: (i32, i32)| Ok(s.0)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(s, b): ((i32, i32), i32)| (b, s.1)),
		/// );
		/// assert_eq!((stall.get)((10, 20)), Ok(10));
		/// assert_eq!((stall.set)(((10, 20), 30)), (30, 20));
		/// ```
		pub fn new(
			get: <FnBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
			set: <FnBrand as CloneableFn>::Of<'a, (S, B), T>,
		) -> Self {
			Stall {
				get,
				set,
				_phantom: PhantomData,
			}
		}
	}

	/// Brand for the `Stall` profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	pub struct StallBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

	impl_kind! {
		impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for StallBrand<FnBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Stall<'a, FnBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for StallBrand<FnBrand, A, B>
	{
		/// Maps functions over the input and output of the `Stall` profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the new structure.",
			"The target type of the new structure.",
			"The source type of the original structure.",
			"The target type of the original structure.",
			"The type of the function to apply to the input.",
			"The type of the function to apply to the output."
		)]
		///
		#[document_parameters(
			"The function to apply to the input.",
			"The function to apply to the output.",
			"The stall instance to transform."
		)]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Stall is usually used internally by AffineTraversal optics
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a, FuncST, FuncUV>(
			st: FuncST,
			uv: FuncUV,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>)
		where
			FuncST: Fn(S) -> T + 'a,
			FuncUV: Fn(U) -> V + 'a, {
			let get = puv.get;
			let set = puv.set;
			let st = <FnBrand as CloneableFn>::new(st);
			let uv = <FnBrand as CloneableFn>::new(uv);
			let st_2 = st.clone();
			let uv_2 = uv.clone();
			Stall::new(
				<FnBrand as CloneableFn>::new(move |s: S| (*get)((*st)(s)).map_err(|u| (*uv)(u))),
				<FnBrand as CloneableFn>::new(move |(s, b): (S, B)| {
					(*uv_2)((*set)(((*st_2)(s), b)))
				}),
			)
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Strong for StallBrand<FnBrand, A, B> {
		/// Lifts the `Stall` profunctor to operate on the first component of a tuple.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The stall instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Stall is usually used internally by AffineTraversal optics
		/// ```
		fn first<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
			let get = pab.get;
			let set = pab.set;
			Stall::new(
				<FnBrand as CloneableFn>::new(move |(s, c): (S, C)| (*get)(s).map_err(|t| (t, c))),
				<FnBrand as CloneableFn>::new(move |((s, c), b): ((S, C), B)| ((*set)((s, b)), c)),
			)
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Choice for StallBrand<FnBrand, A, B> {
		/// Lifts the `Stall` profunctor to operate on the left component of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The stall instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Stall is usually used internally by AffineTraversal optics
		/// ```
		fn left<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
		{
			let get = pab.get;
			let set = pab.set;
			Stall::new(
				<FnBrand as CloneableFn>::new(move |r: Result<C, S>| match r {
					Err(s) => (*get)(s).map_err(Err),
					Ok(c) => Err(Ok(c)),
				}),
				<FnBrand as CloneableFn>::new(move |(r, b): (Result<C, S>, B)| match r {
					Err(s) => Err((*set)((s, b))),
					Ok(c) => Ok(c),
				}),
			)
		}

		/// Lifts the `Stall` profunctor to operate on the right component of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The stall instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Stall is usually used internally by AffineTraversal optics
		/// ```
		fn right<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
		{
			let get = pab.get;
			let set = pab.set;
			Stall::new(
				<FnBrand as CloneableFn>::new(move |r: Result<S, C>| match r {
					Ok(s) => (*get)(s).map_err(Ok),
					Err(c) => Err(Err(c)),
				}),
				<FnBrand as CloneableFn>::new(move |(r, b): (Result<S, C>, B)| match r {
					Ok(s) => Ok((*set)((s, b))),
					Err(c) => Err(c),
				}),
			)
		}
	}
}
pub use inner::*;
