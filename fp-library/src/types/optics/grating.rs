//! The `Grating` profunctor, used for grates.
//!
//! `Grating<A, B, S, T>` wraps a function `((S -> A) -> B) -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{CloneableFn, Closed, Profunctor},
			impl_kind,
			kinds::*,
		},
		fp_macros::{document_parameters, document_signature, document_type_parameters},
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
		pub run:
			<FnBrand as CloneableFn>::Of<'a, Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>, T>,
		pub(crate) _phantom: PhantomData<(A, B)>,
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
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as cloneable_fn_new,
		/// 	types::optics::Grating,
		/// };
		///
		/// let grating = Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(cloneable_fn_new::<RcFnBrand, _, _>(|f| {
		/// 	f(Box::new(|(x, _)| x)) + f(Box::new(|(_, y)| y))
		/// }));
		/// ```
		pub fn new(
			run: <FnBrand as CloneableFn>::Of<
				'a,
				Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>,
				T,
			>
		) -> Self {
			Grating { run, _phantom: PhantomData }
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
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
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
			FuncUV: Fn(U) -> V + 'a,
		{
			let run = puv.run;
			let st = <FnBrand as CloneableFn>::new(st);
			let uv = <FnBrand as CloneableFn>::new(uv);
			Grating::<FnBrand, A, B, S, V>::new(<FnBrand as CloneableFn>::new(
				move |f: Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>| {
					let st = st.clone();
					let uv = uv.clone();
					(*uv)((*run)(Box::new(move |g| {
						let st = st.clone();
						f(Box::new(move |s| g((*st)(s))))
					})))
				},
			))
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the inner function.",
		"The type of the value consumed by the inner function."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Closed
		for GratingBrand<FnBrand, A, B>
	{
		/// Lifts the `Grating` profunctor to operate on functions.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The type of the function input.",
			"The source type of the structure.",
			"The target type of the structure."
		)]
		///
		#[document_parameters("The grating instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::optics::*,
		/// };
		///
		/// // Grating::closed is currently unimplemented
		/// ```
		fn closed<'a, X: 'a, S: 'a, T: 'a>(
			_pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Box<dyn Fn(X) -> S + 'a>, Box<dyn Fn(X) -> T + 'a>>)
		{
			// This is currently unimplemented because the profunctor encoding of Grate
			// requires cloning the input X to support structure reconstruction,
			// which cannot be expressed within the current trait constraints.
			panic!(
				"Grating::closed is not yet implemented for all X. Please use concrete Grate types instead."
			)
		}
	}
}
pub use inner::*;
