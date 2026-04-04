//! The `Shop` profunctor, used for lenses.
//!
//! `Shop<A, B, S, T>` wraps a getter `S -> A` and a setter `S -> B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::optics::*,
			classes::{
				profunctor::{
					Profunctor,
					Strong,
				},
				*,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// The `Shop` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Shop<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Getter function.
		pub get: <FunctionBrand as CloneableFn>::Of<'a, S, A>,
		/// Setter function.
		pub set: <FunctionBrand as CloneableFn>::Of<'a, (S, B), T>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a> Shop<'a, FunctionBrand, A, B, S, T> {
		/// Creates a new `Shop` instance.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as lift_fn_new,
		/// 	types::optics::Shop,
		/// };
		///
		/// let shop = Shop::<RcFnBrand, i32, i32, (i32, i32), (i32, i32)>::new(
		/// 	lift_fn_new::<RcFnBrand, _, _>(|s: (i32, i32)| s.0),
		/// 	lift_fn_new::<RcFnBrand, _, _>(|(s, b): ((i32, i32), i32)| (b, s.1)),
		/// );
		/// assert_eq!((shop.get)((10, 20)), 10);
		/// assert_eq!((shop.set)(((10, 20), 30)), (30, 20));
		/// ```
		pub fn new(
			get: <FunctionBrand as CloneableFn>::Of<'a, S, A>,
			set: <FunctionBrand as CloneableFn>::Of<'a, (S, B), T>,
		) -> Self {
			Shop {
				get,
				set,
			}
		}
	}

	impl_kind! {
		impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> for ShopBrand<FunctionBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Shop<'a, FunctionBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Profunctor
		for ShopBrand<FunctionBrand, A, B>
	{
		/// Maps functions over the input and output of the `Shop` profunctor.
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
			"The shop instance to transform."
		)]
		#[document_returns("A transformed `Shop` instance.")]
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
		/// // Shop is usually used internally by Lens optics
		/// let shop = Shop::<RcFnBrand, i32, i32, (i32, i32), (i32, i32)>::new(
		/// 	lift_fn_new::<RcFnBrand, _, _>(|s: (i32, i32)| s.0),
		/// 	lift_fn_new::<RcFnBrand, _, _>(|(s, b): ((i32, i32), i32)| (b, s.1)),
		/// );
		/// let transformed = <ShopBrand<RcFnBrand, i32, i32> as Profunctor>::dimap(
		/// 	|s: (i32, i32)| s,
		/// 	|t: (i32, i32)| t,
		/// 	shop,
		/// );
		/// assert_eq!((transformed.get)((10, 20)), 10);
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
			let st_2 = st.clone();
			let uv_2 = uv.clone();
			Shop::new(
				<FunctionBrand as LiftFn>::new(move |s: S| (*get)((*st)(s))),
				<FunctionBrand as LiftFn>::new(move |(s, b): (S, B)| {
					(*uv_2)((*set)(((*st_2)(s), b)))
				}),
			)
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Strong
		for ShopBrand<FunctionBrand, A, B>
	{
		/// Lifts the `Shop` profunctor to operate on the first component of a tuple.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		#[document_parameters("The shop instance to transform.")]
		#[document_returns("A transformed `Shop` instance that operates on tuples.")]
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
		/// let shop = Shop::<RcFnBrand, i32, i32, i32, i32>::new(
		/// 	lift_fn_new::<RcFnBrand, _, _>(|s| s),
		/// 	lift_fn_new::<RcFnBrand, _, _>(|(_, b)| b),
		/// );
		/// let first_shop = <ShopBrand<RcFnBrand, i32, i32> as Strong>::first::<i32, i32, i32>(shop);
		/// assert_eq!((first_shop.get)((42, 10)), 42);
		/// ```
		fn first<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
			let get = pab.get;
			let set = pab.set;
			Shop::new(
				<FunctionBrand as LiftFn>::new(move |(s, _): (S, C)| (*get)(s)),
				<FunctionBrand as LiftFn>::new(move |((s, c), b): ((S, C), B)| ((*set)((s, b)), c)),
			)
		}
	}
}
pub use inner::*;
