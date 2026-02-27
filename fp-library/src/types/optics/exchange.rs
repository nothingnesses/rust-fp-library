//! The `Exchange` profunctor, used for isomorphisms.
//!
//! `Exchange<A, B, S, T>` wraps a forward function `S -> A` and a backward function `B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				CloneableFn,
				Profunctor,
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

	/// The `Exchange` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Exchange<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Forward function.
		pub get: <FnBrand as CloneableFn>::Of<'a, S, A>,
		/// Backward function.
		pub set: <FnBrand as CloneableFn>::Of<'a, B, T>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Exchange<'a, FnBrand, A, B, S, T> {
		/// Creates a new `Exchange` instance.
		#[document_signature]
		///
		#[document_parameters("The forward function.", "The backward function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as cloneable_fn_new,
		/// 	types::optics::Exchange,
		/// };
		///
		/// let exchange = Exchange::<RcFnBrand, _, _, _, _>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: String| s.len()),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|n: usize| n.to_string()),
		/// );
		/// assert_eq!((exchange.get)("hello".to_string()), 5);
		/// assert_eq!((exchange.set)(10), "10".to_string());
		/// ```
		pub fn new(
			get: <FnBrand as CloneableFn>::Of<'a, S, A>,
			set: <FnBrand as CloneableFn>::Of<'a, B, T>,
		) -> Self {
			Exchange {
				get,
				set,
			}
		}
	}

	/// Brand for the `Exchange` profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function."
	)]
	pub struct ExchangeBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

	impl_kind! {
		impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for ExchangeBrand<FnBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Exchange<'a, FnBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for ExchangeBrand<FnBrand, A, B>
	{
		/// Maps functions over the input and output of the `Exchange` profunctor.
		#[document_signature]
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
			"The exchange instance to transform."
		)]
		#[document_return("A transformed `Exchange` instance.")]
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
		/// let exchange: Exchange<RcFnBrand, usize, usize, String, String> =
		/// 	Exchange::<RcFnBrand, _, _, _, _>::new(
		/// 		cloneable_fn_new::<RcFnBrand, _, _>(|s: String| s.len()),
		/// 		cloneable_fn_new::<RcFnBrand, _, _>(|n: usize| n.to_string()),
		/// 	);
		///
		/// let transformed = <ExchangeBrand<RcFnBrand, usize, usize> as Profunctor>::dimap(
		/// 	|s: &str| s.to_string(),
		/// 	|s: String| s.len(),
		/// 	exchange,
		/// );
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
			Exchange::new(
				<FnBrand as CloneableFn>::new(move |s: S| (*get)((*st)(s))),
				<FnBrand as CloneableFn>::new(move |b: B| (*uv)((*set)(b))),
			)
		}
	}
}
pub use inner::*;
