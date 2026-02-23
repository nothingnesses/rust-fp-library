//! The `Exchange` profunctor, used for isomorphisms.
//!
//! `Exchange<A, B, S, T>` wraps a forward function `S -> A` and a backward function `B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{Apply, classes::Profunctor, impl_kind, kinds::*},
		fp_macros::{document_parameters, document_signature, document_type_parameters},
		std::marker::PhantomData,
	};

	/// The `Exchange` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Exchange<'a, A, B, S, T> {
		/// Forward function.
		pub get: Box<dyn Fn(S) -> A + 'a>,
		/// Backward function.
		pub set: Box<dyn Fn(B) -> T + 'a>,
		pub(crate) _phantom: PhantomData<(A, B)>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, A, B, S, T> Exchange<'a, A, B, S, T> {
		/// Creates a new `Exchange` instance.
		#[document_signature]
		///
		#[document_parameters("The forward function.", "The backward function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::optics::Exchange;
		///
		/// let exchange = Exchange::new(|s: String| s.len(), |n: usize| n.to_string());
		/// assert_eq!((exchange.get)("hello".to_string()), 5);
		/// assert_eq!((exchange.set)(10), "10".to_string());
		/// ```
		pub fn new(
			get: impl Fn(S) -> A + 'a,
			set: impl Fn(B) -> T + 'a,
		) -> Self {
			Exchange { get: Box::new(get), set: Box::new(set), _phantom: PhantomData }
		}
	}

	/// Brand for the `Exchange` profunctor.
	#[document_type_parameters(
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function."
	)]
	pub struct ExchangeBrand<A, B>(PhantomData<(A, B)>);

	impl_kind! {
		impl<A: 'static, B: 'static> for ExchangeBrand<A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Exchange<'a, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The type of the value produced by the forward function.",
		"The type of the value consumed by the backward function."
	)]
	impl<A: 'static, B: 'static> Profunctor for ExchangeBrand<A, B> {
		/// Maps functions over the input and output of the `Exchange` profunctor.
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
			"The exchange instance to transform."
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
		/// let exchange: Exchange<usize, String, String, usize> =
		/// 	Exchange::new(|s: String| s.len(), |n: usize| n.to_string());
		///
		/// let transformed = Profunctor::dimap(
		/// 	|s: &str| s.to_string(),
		/// 	|s: String| s.len(),
		/// 	exchange
		/// );
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
			let get = puv.get;
			let set = puv.set;
			Exchange::new(move |s| get(st(s)), move |b| uv(set(b)))
		}
	}
}
pub use inner::*;
