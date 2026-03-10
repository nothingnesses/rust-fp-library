//! The `Stall` profunctor, used for affine traversals.
//!
//! `Stall<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a setter function `S -> B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::optics::*,
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
		fp_macros::*,
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
	pub struct Stall<'a, FunctionBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Preview function: tries to extract the focus.
		pub get: <FunctionBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
		/// Setter function.
		pub set: <FunctionBrand as CloneableFn>::Of<'a, (S, B), T>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FunctionBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a>
		Stall<'a, FunctionBrand, A, B, S, T>
	{
		/// Creates a new `Stall` instance.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The setter function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
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
			get: <FunctionBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
			set: <FunctionBrand as CloneableFn>::Of<'a, (S, B), T>,
		) -> Self {
			Stall {
				get,
				set,
			}
		}
	}

	impl_kind! {
		impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> for StallBrand<FunctionBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Stall<'a, FunctionBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for StallBrand<FunctionBrand, A, B>
	{
		/// Maps functions over the input and output of the `Stall` profunctor.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the new structure.",
			"The target type of the new structure.",
			"The source type of the original structure.",
			"The target type of the original structure."
		)]
		///
		#[document_parameters(
			"The function to apply to the input.",
			"The function to apply to the output.",
			"The stall instance to transform."
		)]
		#[document_returns("A transformed `Stall` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Stall is usually used internally by AffineTraversal optics
		/// let stall = Stall::<RcFnBrand, i32, i32, (i32, i32), (i32, i32)>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: (i32, i32)| Ok(s.0)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(s, b): ((i32, i32), i32)| (b, s.1)),
		/// );
		/// let transformed = <StallBrand<RcFnBrand, i32, i32> as Profunctor>::dimap(
		/// 	|s: (i32, i32)| s,
		/// 	|t: (i32, i32)| t,
		/// 	stall,
		/// );
		/// assert_eq!((transformed.get)((10, 20)), Ok(10));
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a>(
			st: impl Fn(S) -> T + 'a,
			uv: impl Fn(U) -> V + 'a,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>) {
			let get = puv.get;
			let set = puv.set;
			let st = <FunctionBrand as CloneableFn>::new(st);
			let uv = <FunctionBrand as CloneableFn>::new(uv);
			let st_2 = st.clone();
			let uv_2 = uv.clone();
			Stall::new(
				<FunctionBrand as CloneableFn>::new(move |s: S| {
					(*get)((*st)(s)).map_err(|u| (*uv)(u))
				}),
				<FunctionBrand as CloneableFn>::new(move |(s, b): (S, B)| {
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
	impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> Strong
		for StallBrand<FunctionBrand, A, B>
	{
		/// Lifts the `Stall` profunctor to operate on the first component of a tuple.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The stall instance to transform.")]
		#[document_returns("A transformed `Stall` instance that operates on tuples.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let stall = Stall::<RcFnBrand, i32, i32, i32, i32>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: i32| Ok(s)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(_s, b): (i32, i32)| b),
		/// );
		/// let lifted = <StallBrand<RcFnBrand, i32, i32> as Strong>::first::<i32, i32, String>(stall);
		/// assert_eq!((lifted.get)((10, "hi".to_string())), Ok(10));
		/// ```
		fn first<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
			let get = pab.get;
			let set = pab.set;
			Stall::new(
				<FunctionBrand as CloneableFn>::new(move |(s, c): (S, C)| {
					(*get)(s).map_err(|t| (t, c))
				}),
				<FunctionBrand as CloneableFn>::new(move |((s, c), b): ((S, C), B)| {
					((*set)((s, b)), c)
				}),
			)
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the setter."
	)]
	impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> Choice
		for StallBrand<FunctionBrand, A, B>
	{
		/// Lifts the `Stall` profunctor to operate on the left component of a `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The stall instance to transform.")]
		#[document_returns(
			"A transformed `Stall` instance that operates on the left component of a `Result`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let stall = Stall::<RcFnBrand, i32, i32, i32, i32>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: i32| Ok(s)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(_s, b): (i32, i32)| b),
		/// );
		/// let lifted = <StallBrand<RcFnBrand, i32, i32> as Choice>::left::<i32, i32, String>(stall);
		/// assert!((lifted.get)(Err(10)).is_ok());
		/// ```
		fn left<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
		{
			let get = pab.get;
			let set = pab.set;
			Stall::new(
				<FunctionBrand as CloneableFn>::new(move |r: Result<C, S>| match r {
					Err(s) => (*get)(s).map_err(Err),
					Ok(c) => Err(Ok(c)),
				}),
				<FunctionBrand as CloneableFn>::new(move |(r, b): (Result<C, S>, B)| match r {
					Err(s) => Err((*set)((s, b))),
					Ok(c) => Ok(c),
				}),
			)
		}

		/// Lifts the `Stall` profunctor to operate on the right component of a `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The stall instance to transform.")]
		#[document_returns(
			"A transformed `Stall` instance that operates on the right component of a `Result`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let stall = Stall::<RcFnBrand, i32, i32, i32, i32>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s| Ok(s)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(_, b)| b),
		/// );
		/// let lifted = <StallBrand<RcFnBrand, i32, i32> as Choice>::right::<i32, i32, i32>(stall);
		/// assert_eq!((lifted.get)(Ok(42)), Ok(42));
		/// ```
		fn right<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
		{
			let get = pab.get;
			let set = pab.set;
			Stall::new(
				<FunctionBrand as CloneableFn>::new(move |r: Result<S, C>| match r {
					Ok(s) => (*get)(s).map_err(Ok),
					Err(c) => Err(Err(c)),
				}),
				<FunctionBrand as CloneableFn>::new(move |(r, b): (Result<S, C>, B)| match r {
					Ok(s) => Ok((*set)((s, b))),
					Err(c) => Err(c),
				}),
			)
		}
	}
}
pub use inner::*;
