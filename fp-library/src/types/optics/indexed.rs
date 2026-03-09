//! The `Indexed` profunctor wrapper.
//!
//! `Indexed<'a, P, I, A, B>` wraps a profunctor `P` to carry an index `I` alongside the focus `A`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				applicative::Applicative,
				profunctor::{
					Choice,
					Profunctor,
					Strong,
					Wander,
					wander::TraversalFunc,
				},
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// The `Indexed` profunctor wrapper.
	///
	/// `Indexed<'a, P, I, A, B>` wraps a profunctor `P` that operates on `(I, A)` and `B`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The underlying profunctor brand.",
		"The index type.",
		"The focus type.",
		"The target focus type."
	)]
	pub struct Indexed<'a, P, I, A, B>
	where
		P: Profunctor,
		I: 'a,
		A: 'a,
		B: 'a, {
		/// The underlying profunctor value.
		pub inner: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, (I, A), B>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The underlying profunctor brand.",
		"The index type.",
		"The focus type.",
		"The target focus type."
	)]
	impl<'a, P, I, A, B> Indexed<'a, P, I, A, B>
	where
		P: Profunctor,
		I: 'a,
		A: 'a,
		B: 'a,
	{
		/// Creates a new `Indexed` instance.
		#[document_signature]
		#[document_parameters("The underlying profunctor value.")]
		#[document_returns("A new `Indexed` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	types::optics::Indexed,
		/// };
		/// let f = |(i, a): (usize, i32)| a + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// assert_eq!((indexed.inner)((10, 32)), 42);
		/// ```
		pub fn new(
			inner: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, (I, A), B>)
		) -> Self {
			Self {
				inner,
			}
		}
	}

	/// Brand for the `Indexed` profunctor wrapper.
	#[document_type_parameters("The underlying profunctor brand.", "The index type.")]
	pub struct IndexedBrand<P, I>(PhantomData<(P, I)>);

	impl_kind! {
		impl<P: Profunctor + 'static, I: 'static> for IndexedBrand<P, I> {
			#[document_default]
			type Of<'a, A: 'a, B: 'a>: 'a = Indexed<'a, P, I, A, B>;
		}
	}

	#[document_type_parameters("The underlying profunctor brand.", "The index type.")]
	impl<P: Profunctor + 'static, I: 'static> Profunctor for IndexedBrand<P, I> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The new input type.",
			"The original input type.",
			"The original output type.",
			"The new output type.",
			"The type of the contravariant function.",
			"The type of the covariant function."
		)]
		#[document_parameters(
			"The contravariant function to apply to the input.",
			"The covariant function to apply to the output.",
			"The indexed profunctor instance."
		)]
		#[document_returns("A transformed `Indexed` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::profunctor::*,
		/// 	types::optics::{
		/// 		Indexed,
		/// 		IndexedBrand,
		/// 	},
		/// };
		/// let f = |(i, a): (usize, i32)| a + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let transformed = <IndexedBrand<RcFnBrand, usize> as Profunctor>::dimap(
		/// 	|a: i32| a * 2,
		/// 	|b: i32| b - 1,
		/// 	indexed,
		/// );
		/// assert_eq!((transformed.inner)((10, 16)), 41); // (10 + (16 * 2)) - 1 = 41
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, FuncAB, FuncCD>(
			ab: FuncAB,
			cd: FuncCD,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>)
		where
			FuncAB: Fn(A) -> B + 'a,
			FuncCD: Fn(C) -> D + 'a, {
			Indexed::new(P::dimap(move |(i, a)| (i, ab(a)), cd, pbc.inner))
		}
	}

	#[document_type_parameters("The underlying profunctor brand.", "The index type.")]
	impl<P: Strong + 'static, I: 'static> Strong for IndexedBrand<P, I> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the second component (threaded through unchanged)."
		)]
		#[document_parameters("The indexed profunctor instance to lift.")]
		#[document_returns("A transformed `Indexed` instance that operates on pairs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::profunctor::*,
		/// 	types::optics::{Indexed, IndexedBrand},
		/// };
		/// let f = |(i, a): (usize, i32)| a + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let transformed = <IndexedBrand<RcFnBrand, usize> as Strong>::first::<i32, i32, i32>(indexed);
		/// assert_eq!((transformed.inner)((10, (16, 100))), (26, 100)); // (10 + 16) = 26, 100 threaded through
		/// ```
		fn first<'a, A: 'a, B: 'a, C>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
			Indexed::new(P::dimap(
				|(i, (a, c))| ((i, a), c),
				|(b, c)| (b, c),
				P::first::<(I, A), B, C>(pab.inner),
			))
		}

		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the first component (threaded through unchanged)."
		)]
		#[document_parameters("The indexed profunctor instance to lift.")]
		#[document_returns("A transformed `Indexed` instance that operates on pairs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::profunctor::*,
		/// 	types::optics::{Indexed, IndexedBrand},
		/// };
		/// let f = |(i, b): (usize, i32)| b + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let transformed = <IndexedBrand<RcFnBrand, usize> as Strong>::second::<i32, i32, i32>(indexed);
		/// assert_eq!((transformed.inner)((10, (100, 16))), (100, 26)); // (10 + 16) = 26, 100 threaded through
		/// ```
		fn second<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
			Indexed::new(P::dimap(
				|(i, (c, a))| (c, (i, a)),
				|(c, b)| (c, b),
				P::second::<(I, A), B, C>(pab.inner),
			))
		}
	}

	#[document_type_parameters("The underlying profunctor brand.", "The index type.")]
	impl<P: Choice + 'static, I: 'static> Choice for IndexedBrand<P, I> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the alternative variant (threaded through unchanged)."
		)]
		#[document_parameters("The indexed profunctor instance to lift.")]
		#[document_returns("A transformed `Indexed` instance that operates on `Result` types.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::profunctor::*,
		/// 	types::optics::{
		/// 		Indexed,
		/// 		IndexedBrand,
		/// 	},
		/// };
		/// let f = |(i, a): (usize, i32)| a + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let transformed = <IndexedBrand<RcFnBrand, usize> as Choice>::left::<i32, i32, i32>(indexed);
		/// assert_eq!((transformed.inner)((10, Err(32))), Err(42));
		/// ```
		fn left<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		{
			Indexed::new(P::dimap(
				|(i, r)| match r {
					Err(a) => Err((i, a)),
					Ok(c) => Ok(c),
				},
				|r| match r {
					Err(b) => Err(b),
					Ok(c) => Ok(c),
				},
				P::left::<(I, A), B, C>(pab.inner),
			))
		}

		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the alternative variant (threaded through unchanged)."
		)]
		#[document_parameters("The indexed profunctor instance to lift.")]
		#[document_returns("A transformed `Indexed` instance that operates on `Result` types.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::profunctor::*,
		/// 	types::optics::{
		/// 		Indexed,
		/// 		IndexedBrand,
		/// 	},
		/// };
		/// let f = |(i, b): (usize, i32)| b + (i as i32);
		/// let indexed = Indexed::<RcFnBrand, usize, i32, i32>::new(std::rc::Rc::new(f));
		/// let transformed = <IndexedBrand<RcFnBrand, usize> as Choice>::right::<i32, i32, i32>(indexed);
		/// assert_eq!((transformed.inner)((10, Ok(32))), Ok(42));
		/// ```
		fn right<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		{
			Indexed::new(P::dimap(
				|(i, r)| match r {
					Ok(a) => Ok((i, a)),
					Err(c) => Err(c),
				},
				|r| match r {
					Ok(b) => Ok(b),
					Err(c) => Err(c),
				},
				P::right::<(I, A), B, C>(pab.inner),
			))
		}
	}

	#[document_type_parameters("The underlying profunctor brand.", "The index type.")]
	impl<P: Wander + 'static, I: Clone + 'static> Wander for IndexedBrand<P, I> {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The source type of the focus.",
			"The target type of the focus.",
			"The type of the traversal function."
		)]
		#[document_parameters("The traversal function.", "The indexed profunctor instance.")]
		#[document_returns("A transformed `Indexed` instance that operates on structures.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 	},
		/// 	classes::{
		/// 		optics::IndexedTraversalOptic,
		/// 		profunctor::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// // Use an indexed traversal over a Vec to demonstrate wandering with index
		/// let traversal: IndexedTraversal<
		/// 	RcBrand,
		/// 	usize,
		/// 	Vec<i32>,
		/// 	Vec<i32>,
		/// 	i32,
		/// 	i32,
		/// 	Traversed<VecBrand>,
		/// > = IndexedTraversal::traversed();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::<RcFnBrand, _, _, _>::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&traversal, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone, TFunc>(
			traversal: TFunc,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		where
			TFunc: TraversalFunc<'a, S, T, A, B> + 'a, {
			struct IWanderAdapter<'a, I, S, T, A, B, TFunc> {
				traversal: TFunc,
				_phantom: PhantomData<&'a (I, S, T, A, B)>,
			}

			impl<'a, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a, TFunc>
				TraversalFunc<'a, (I, S), T, (I, A), B> for IWanderAdapter<'a, I, S, T, A, B, TFunc>
			where
				TFunc: TraversalFunc<'a, S, T, A, B> + 'a,
			{
				fn apply<M: Applicative>(
					&self,
					f: Box<
						dyn Fn((I, A)) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>)
							+ 'a,
					>,
					(i, s): (I, S),
				) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, T>)
				where
					Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>): Clone, {
					let i_clone = i.clone();
					self.traversal.apply::<M>(Box::new(move |a| f((i_clone.clone(), a))), s)
				}
			}

			Indexed::new(P::wander(
				IWanderAdapter {
					traversal,
					_phantom: PhantomData,
				},
				pab.inner,
			))
		}
	}
}

pub use inner::*;
