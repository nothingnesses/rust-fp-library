//! The `Grating` profunctor, used for grates.
//!
//! `Grating<A, B, S, T>` wraps a function `((S -> A) -> B) -> T`.

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
			document_parameters,
			document_return,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// The `Grating` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Grating<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Grating function.
		pub run: <FnBrand as CloneableFn>::Of<
			'a,
			<FnBrand as CloneableFn>::Of<'a, <FnBrand as CloneableFn>::Of<'a, S, A>, B>,
			T,
		>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Grating<'a, FnBrand, A, B, S, T> {
		/// Creates a new `Grating` instance.
		#[document_signature]
		///
		#[document_parameters("The grating function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::Grating,
		/// };
		///
		/// let grating =
		/// 	Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(cloneable_fn_new::<RcFnBrand, _, _>(
		/// 		|f: std::rc::Rc<dyn Fn(std::rc::Rc<dyn Fn((i32, i32)) -> i32>) -> i32>| {
		/// 			let get_x = cloneable_fn_new::<RcFnBrand, _, _>(|(x, _)| x);
		/// 			let get_y = cloneable_fn_new::<RcFnBrand, _, _>(|(_, y)| y);
		/// 			f(get_x) + f(get_y)
		/// 		},
		/// 	));
		/// ```
		pub fn new(
			run: <FnBrand as CloneableFn>::Of<
				'a,
				<FnBrand as CloneableFn>::Of<'a, <FnBrand as CloneableFn>::Of<'a, S, A>, B>,
				T,
			>
		) -> Self {
			Grating {
				run,
			}
		}
	}

	/// Brand for the `Grating` profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function."
	)]
	pub struct GratingBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

	impl_kind! {
		impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for GratingBrand<FnBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Grating<'a, FnBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for GratingBrand<FnBrand, A, B>
	{
		/// Maps functions over the input and output of the `Grating` profunctor.
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
			"The grating instance to transform."
		)]
		///
		#[document_return("A transformed `Grating` instance.")]
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
		/// // Grating is usually used internally by Grate optics
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a, FuncST, FuncUV>(
			st: FuncST,
			uv: FuncUV,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>)
		where
			FuncST: Fn(S) -> T + 'a,
			FuncUV: Fn(U) -> V + 'a, {
			let run = puv.run;
			let st = <FnBrand as CloneableFn>::new(st);
			let uv = <FnBrand as CloneableFn>::new(uv);
			Grating::<FnBrand, A, B, S, V>::new(<FnBrand as CloneableFn>::new(
				move |f: <FnBrand as CloneableFn>::Of<
					'a,
					<FnBrand as CloneableFn>::Of<'a, S, A>,
					B,
				>| {
					let st = st.clone();
					let uv = uv.clone();
					(*uv)((*run)(<FnBrand as CloneableFn>::new(
						move |g: <FnBrand as CloneableFn>::Of<'a, T, A>| {
							let st = st.clone();
							f(<FnBrand as CloneableFn>::new(move |s| g((*st)(s))))
						},
					)))
				},
			))
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Closed<FnBrand>
		for GratingBrand<FnBrand, A, B>
	{
		/// Lifts the `Grating` profunctor to operate on functions.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the function input."
		)]
		///
		#[document_parameters("The grating instance to transform.")]
		///
		#[document_return("A transformed `Grating` instance that operates on functions.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let grating =
		/// 	Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(cloneable_fn_new::<RcFnBrand, _, _>(
		/// 		|f: std::rc::Rc<dyn Fn(std::rc::Rc<dyn Fn((i32, i32)) -> i32>) -> i32>| {
		/// 			let get_x = cloneable_fn_new::<RcFnBrand, _, _>(|(x, _)| x);
		/// 			let get_y = cloneable_fn_new::<RcFnBrand, _, _>(|(_, y)| y);
		/// 			f(get_x) + f(get_y)
		/// 		},
		/// 	));
		///
		/// let closed_grating = <GratingBrand<RcFnBrand, i32, i32> as Closed<RcFnBrand>>::closed::<
		/// 	(i32, i32),
		/// 	i32,
		/// 	String,
		/// >(grating);
		///
		/// let run_closed = closed_grating.run;
		/// type GetterFn = std::rc::Rc<dyn Fn(std::rc::Rc<dyn Fn(String) -> (i32, i32)>) -> i32>;
		/// let result_fn = run_closed(cloneable_fn_new::<RcFnBrand, _, _>(|getter: GetterFn| {
		/// 	// getter: (String -> (i32, i32)) -> i32
		/// 	// We provide a function that produces a pair from a string
		/// 	getter(cloneable_fn_new::<RcFnBrand, _, _>(|s: String| (s.len() as i32, 10)))
		/// }));
		///
		/// assert_eq!(result_fn("hello".to_string()), 5 + 10);
		/// ```
		fn closed<'a, S: 'a, T: 'a, X: 'a + Clone>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, X, S>, <FnBrand as CloneableFn>::Of<'a, X, T>>)
		{
			let run = pab.run;
			Grating::<
				FnBrand,
				A,
				B,
				<FnBrand as CloneableFn>::Of<'a, X, S>,
				<FnBrand as CloneableFn>::Of<'a, X, T>,
			>::new(<FnBrand as CloneableFn>::new(
				move |g: <FnBrand as CloneableFn>::Of<
					'a,
					<FnBrand as CloneableFn>::Of<'a, <FnBrand as CloneableFn>::Of<'a, X, S>, A>,
					B,
				>| {
					let run = run.clone();
					<FnBrand as CloneableFn>::new(move |x: X| {
						let g = g.clone();
						let x = x.clone();
						(*run)(<FnBrand as CloneableFn>::new(
							move |h: <FnBrand as CloneableFn>::Of<'a, S, A>| {
								let x = x.clone();
								g(<FnBrand as CloneableFn>::new(
									move |k: <FnBrand as CloneableFn>::Of<'a, X, S>| {
										h(k(x.clone()))
									},
								))
							},
						))
					})
				},
			))
		}
	}
}
pub use inner::*;
