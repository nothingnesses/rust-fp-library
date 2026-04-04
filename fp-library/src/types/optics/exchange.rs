//! The `Exchange` profunctor, used for isomorphisms.
//!
//! `Exchange<A, B, S, T>` wraps a forward function `S -> A` and a backward function `B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::optics::*,
			classes::{
				Profunctor,
				*,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// The `Exchange` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Exchange<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Forward function.
		pub get: <FunctionBrand as CloneableFn>::Of<'a, S, A>,
		/// Backward function.
		pub set: <FunctionBrand as CloneableFn>::Of<'a, B, T>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a>
		Exchange<'a, FunctionBrand, A, B, S, T>
	{
		/// Creates a new `Exchange` instance.
		#[document_signature]
		///
		#[document_parameters("The forward function.", "The backward function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as lift_fn_new,
		/// 	types::optics::Exchange,
		/// };
		///
		/// let exchange = Exchange::<RcFnBrand, _, _, _, _>::new(
		/// 	lift_fn_new::<RcFnBrand, _, _>(|s: String| s.len()),
		/// 	lift_fn_new::<RcFnBrand, _, _>(|n: usize| n.to_string()),
		/// );
		/// assert_eq!((exchange.get)("hello".to_string()), 5);
		/// assert_eq!((exchange.set)(10), "10".to_string());
		/// ```
		pub fn new(
			get: <FunctionBrand as CloneableFn>::Of<'a, S, A>,
			set: <FunctionBrand as CloneableFn>::Of<'a, B, T>,
		) -> Self {
			Exchange {
				get,
				set,
			}
		}
	}

	impl_kind! {
		impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> for ExchangeBrand<FunctionBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Exchange<'a, FunctionBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Profunctor
		for ExchangeBrand<FunctionBrand, A, B>
	{
		/// Maps functions over the input and output of the `Exchange` profunctor.
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
			"The exchange instance to transform."
		)]
		#[document_returns("A transformed `Exchange` instance.")]
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
		/// let exchange: Exchange<RcFnBrand, usize, usize, String, String> =
		/// 	Exchange::<RcFnBrand, _, _, _, _>::new(
		/// 		lift_fn_new::<RcFnBrand, _, _>(|s: String| s.len()),
		/// 		lift_fn_new::<RcFnBrand, _, _>(|n: usize| n.to_string()),
		/// 	);
		///
		/// let transformed = <ExchangeBrand<RcFnBrand, usize, usize> as Profunctor>::dimap(
		/// 	|s: &str| s.to_string(),
		/// 	|s: String| s.len(),
		/// 	exchange,
		/// );
		/// assert_eq!((transformed.get)("hello"), 5);
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a>(
			st: impl Fn(S) -> T + 'a,
			uv: impl Fn(U) -> V + 'a,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>) {
			let get = puv.get;
			let set = puv.set;
			let st = <FunctionBrand as LiftFn>::new(st);
			let uv = <FunctionBrand as LiftFn>::new(uv);
			Exchange::new(
				<FunctionBrand as LiftFn>::new(move |s: S| (*get)((*st)(s))),
				<FunctionBrand as LiftFn>::new(move |b: B| (*uv)((*set)(b))),
			)
		}
	}
}
pub use inner::*;
