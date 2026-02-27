//! The `Shop` profunctor, used for lenses.
//!
//! `Shop<A, B, S, T>` wraps a getter `S -> A` and a setter `S -> B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				CloneableFn,
				profunctor::{
					Profunctor,
					Strong,
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

	/// The `Shop` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Shop<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Getter function.
		pub get: <FnBrand as CloneableFn>::Of<'a, S, A>,
		/// Setter function.
		pub set: <FnBrand as CloneableFn>::Of<'a, (S, B), T>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Shop<'a, FnBrand, A, B, S, T> {
		/// Creates a new `Shop` instance.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as cloneable_fn_new,
		/// 	types::optics::Shop,
		/// };
		///
		/// let shop = Shop::<RcFnBrand, i32, i32, (i32, i32), (i32, i32)>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: (i32, i32)| s.0),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(s, b): ((i32, i32), i32)| (b, s.1)),
		/// );
		/// assert_eq!((shop.get)((10, 20)), 10);
		/// assert_eq!((shop.set)(((10, 20), 30)), (30, 20));
		/// ```
		pub fn new(
			get: <FnBrand as CloneableFn>::Of<'a, S, A>,
			set: <FnBrand as CloneableFn>::Of<'a, (S, B), T>,
		) -> Self {
			Shop {
				get,
				set,
			}
		}
	}

	/// Brand for the `Shop` profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter."
	)]
	pub struct ShopBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

	impl_kind! {
		impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for ShopBrand<FnBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Shop<'a, FnBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the getter.",
		"The type of the value consumed by the setter."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for ShopBrand<FnBrand, A, B>
	{
		/// Maps functions over the input and output of the `Shop` profunctor.
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
			"The shop instance to transform."
		)]
		#[document_return("A transformed `Shop` instance.")]
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
		/// // Shop is usually used internally by Lens optics
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
			Shop::new(
				<FnBrand as CloneableFn>::new(move |s: S| (*get)((*st)(s))),
				<FnBrand as CloneableFn>::new(move |(s, b): (S, B)| {
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
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Strong for ShopBrand<FnBrand, A, B> {
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
		#[document_return("A transformed `Shop` instance that operates on tuples.")]
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
		/// // Shop is usually used internally by Lens optics
		/// ```
		fn first<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
			let get = pab.get;
			let set = pab.set;
			Shop::new(
				<FnBrand as CloneableFn>::new(move |(s, _): (S, C)| (*get)(s)),
				<FnBrand as CloneableFn>::new(move |((s, c), b): ((S, C), B)| ((*set)((s, b)), c)),
			)
		}
	}
}
pub use inner::*;
