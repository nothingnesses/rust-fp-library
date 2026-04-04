//! The `Grating` profunctor, used for grates.
//!
//! `Grating<A, B, S, T>` wraps a function `((S -> A) -> B) -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::optics::*,
			classes::{
				profunctor::{
					Closed,
					Profunctor,
				},
				*,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
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
	pub struct Grating<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Grating function.
		pub run: <FunctionBrand as CloneableFn>::Of<
			'a,
			<FunctionBrand as CloneableFn>::Of<'a, <FunctionBrand as CloneableFn>::Of<'a, S, A>, B>,
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
	impl<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a> Grating<'a, FunctionBrand, A, B, S, T> {
		/// Creates a new `Grating` instance.
		#[document_signature]
		///
		#[document_parameters("The grating function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::Grating,
		/// };
		///
		/// let grating =
		/// 	Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(lift_fn_new::<RcFnBrand, _, _>(
		/// 		|f: std::rc::Rc<dyn Fn(std::rc::Rc<dyn Fn((i32, i32)) -> i32>) -> i32>| {
		/// 			let get_x = lift_fn_new::<RcFnBrand, _, _>(|(x, _)| x);
		/// 			let get_y = lift_fn_new::<RcFnBrand, _, _>(|(_, y)| y);
		/// 			f(get_x) + f(get_y)
		/// 		},
		/// 	));
		/// let result = (grating.run)(lift_fn_new::<RcFnBrand, _, _>(
		/// 	|g: std::rc::Rc<dyn Fn((i32, i32)) -> i32>| g((10, 20)),
		/// ));
		/// assert_eq!(result, 30);
		/// ```
		pub fn new(
			run: <FunctionBrand as CloneableFn>::Of<
				'a,
				<FunctionBrand as CloneableFn>::Of<
					'a,
					<FunctionBrand as CloneableFn>::Of<'a, S, A>,
					B,
				>,
				T,
			>
		) -> Self {
			Grating {
				run,
			}
		}
	}

	impl_kind! {
		impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> for GratingBrand<FunctionBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Grating<'a, FunctionBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Profunctor
		for GratingBrand<FunctionBrand, A, B>
	{
		/// Maps functions over the input and output of the `Grating` profunctor.
		#[document_signature]
		///
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
			"The grating instance to transform."
		)]
		///
		#[document_returns("A transformed `Grating` instance.")]
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
		/// 		cloneable_fn::new as lift_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Grating is usually used internally by Grate optics
		/// let grating =
		/// 	Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(lift_fn_new::<RcFnBrand, _, _>(
		/// 		|f: std::rc::Rc<dyn Fn(std::rc::Rc<dyn Fn((i32, i32)) -> i32>) -> i32>| {
		/// 			let get_x = lift_fn_new::<RcFnBrand, _, _>(|(x, _)| x);
		/// 			let get_y = lift_fn_new::<RcFnBrand, _, _>(|(_, y)| y);
		/// 			f(get_x) + f(get_y)
		/// 		},
		/// 	));
		/// let transformed = <GratingBrand<RcFnBrand, i32, i32> as Profunctor>::dimap(
		/// 	|s: (i32, i32)| s,
		/// 	|t: i32| t,
		/// 	grating,
		/// );
		/// let result = (transformed.run)(lift_fn_new::<RcFnBrand, _, _>(
		/// 	|g: std::rc::Rc<dyn Fn((i32, i32)) -> i32>| g((10, 20)),
		/// ));
		/// assert_eq!(result, 30);
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a>(
			st: impl Fn(S) -> T + 'a,
			uv: impl Fn(U) -> V + 'a,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>) {
			let run = puv.run;
			let st = <FunctionBrand as LiftFn>::new(st);
			let uv = <FunctionBrand as LiftFn>::new(uv);
			Grating::<FunctionBrand, A, B, S, V>::new(<FunctionBrand as LiftFn>::new(
				move |f: <FunctionBrand as CloneableFn>::Of<
					'a,
					<FunctionBrand as CloneableFn>::Of<'a, S, A>,
					B,
				>| {
					let st = st.clone();
					let uv = uv.clone();
					(*uv)((*run)(<FunctionBrand as LiftFn>::new(
						move |g: <FunctionBrand as CloneableFn>::Of<'a, T, A>| {
							let st = st.clone();
							f(<FunctionBrand as LiftFn>::new(move |s| g((*st)(s))))
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
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Closed<FunctionBrand>
		for GratingBrand<FunctionBrand, A, B>
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
		#[document_returns("A transformed `Grating` instance that operates on functions.")]
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
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let grating =
		/// 	Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(lift_fn_new::<RcFnBrand, _, _>(
		/// 		|f: std::rc::Rc<dyn Fn(std::rc::Rc<dyn Fn((i32, i32)) -> i32>) -> i32>| {
		/// 			let get_x = lift_fn_new::<RcFnBrand, _, _>(|(x, _)| x);
		/// 			let get_y = lift_fn_new::<RcFnBrand, _, _>(|(_, y)| y);
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
		/// let result_fn = run_closed(lift_fn_new::<RcFnBrand, _, _>(|getter: GetterFn| {
		/// 	// getter: (String -> (i32, i32)) -> i32
		/// 	// We provide a function that produces a pair from a string
		/// 	getter(lift_fn_new::<RcFnBrand, _, _>(|s: String| (s.len() as i32, 10)))
		/// }));
		///
		/// assert_eq!(result_fn("hello".to_string()), 5 + 10);
		/// ```
		fn closed<'a, S: 'a, T: 'a, X: 'a + Clone>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, <FunctionBrand as CloneableFn>::Of<'a, X, S>, <FunctionBrand as CloneableFn>::Of<'a, X, T>>)
		{
			let run = pab.run;
			Grating::<
				FunctionBrand,
				A,
				B,
				<FunctionBrand as CloneableFn>::Of<'a, X, S>,
				<FunctionBrand as CloneableFn>::Of<'a, X, T>,
			>::new(<FunctionBrand as LiftFn>::new(
				move |g: <FunctionBrand as CloneableFn>::Of<
					'a,
					<FunctionBrand as CloneableFn>::Of<
						'a,
						<FunctionBrand as CloneableFn>::Of<'a, X, S>,
						A,
					>,
					B,
				>| {
					let run = run.clone();
					<FunctionBrand as LiftFn>::new(move |x: X| {
						let g = g.clone();
						let x = x.clone();
						(*run)(<FunctionBrand as LiftFn>::new(
							move |h: <FunctionBrand as CloneableFn>::Of<'a, S, A>| {
								let x = x.clone();
								g(<FunctionBrand as LiftFn>::new(
									move |k: <FunctionBrand as CloneableFn>::Of<'a, X, S>| {
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
