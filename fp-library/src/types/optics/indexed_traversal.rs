//! Indexed traversal optics.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				FnBrand,
				optics::*,
			},
			classes::{
				UnsizedCoercible,
				applicative::Applicative,
				monoid::Monoid,
				optics::*,
				profunctor::{
					Wander,
					wander::TraversalFunc,
				},
				traversable_with_index::TraversableWithIndex,
			},
			kinds::*,
			types::optics::Indexed,
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// A polymorphic indexed traversal.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The traversal function type."
	)]
	pub struct IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + 'a, {
		/// The underlying indexed traversal function.
		pub traversal: F,
		pub(crate) _phantom: PhantomData<(&'a (I, S, T, A, B), PointerBrand)>,
	}

	/// A wrapper struct for the `traversed` constructor.
	#[derive(Clone)]
	pub struct Traversed<Brand>(pub std::marker::PhantomData<Brand>);

	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The brand of the traversable structure.",
		"The type of the elements in the structure.",
		"The type of the elements in the result."
	)]
	#[document_parameters("The traversed struct.")]
	impl<'a, I, Brand, A, B>
		IndexedTraversalFunc<
			'a,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>),
			A,
			B,
		> for Traversed<Brand>
	where
		Brand: TraversableWithIndex<I>,
		A: 'a + Clone,
		B: 'a + Clone,
		I: 'a,
		Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>): Clone,
	{
		#[document_signature]
		#[document_type_parameters("The applicative context.")]
		#[document_parameters("The traversal function.", "The structure to traverse.")]
		#[document_returns("The traversed structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalFunc,
		/// 	types::optics::*,
		/// };
		///
		/// let traversal = Traversed::<VecBrand>(std::marker::PhantomData);
		/// let s = vec![10, 20, 30];
		/// let f = Box::new(|i: usize, a: i32| -> Option<i32> { Some(a + i as i32) });
		///
		/// let result: Option<Vec<i32>> = IndexedTraversalFunc::apply::<OptionBrand>(&traversal, f, s);
		///
		/// assert_eq!(result, Some(vec![10, 21, 32]));
		/// ```
		fn apply<M: Applicative>(
			&self,
			f: Box<dyn Fn(I, A) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>) + 'a>,
			s: Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>)>)
		where
			Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>): Clone, {
			// IMPORTANT: The turbofish `::<A, B, M>` is **required** here. Do not remove it.
			//
			// Root cause: The return type of this method contains a nested associated type
			// projection: `<M as Kind>::Of<'a, <Brand as Kind>::Of<'a, B>>`. When the old
			// trait solver (stable Rust) encounters this without explicit type annotations,
			// it enters an infinite loop during type normalization — `cargo check` hangs
			// indefinitely. This is not a recursion depth issue (`recursion_limit` has no
			// effect); it is an infinite loop in the solver's normalization phase when it
			// tries to resolve the inner projection `<Brand as Kind>::Of<'a, B>` inside the
			// outer projection `<M as Kind>::Of<'a, _>` with both `M` and `Brand` being
			// generic parameters.
			//
			// The turbofish explicitly provides the type parameters to `traverse_with_index`,
			// preventing the solver from needing to infer them through the nested projection.
			//
			// The new trait solver (`-Znext-solver`) handles this correctly without hanging,
			// but still requires the turbofish for type inference (E0283 without it).
			Brand::traverse_with_index::<A, B, M>(f, s)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The brand of the traversable structure.",
		"The type of the elements in the structure.",
		"The type of the elements in the result."
	)]
	impl<'a, PointerBrand, I, Brand, A, B>
		IndexedTraversal<
			'a,
			PointerBrand,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>),
			A,
			B,
			Traversed<Brand>,
		>
	where
		Brand: TraversableWithIndex<I>,
		A: 'a + Clone,
		B: 'a + Clone,
		I: 'a,
		Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, B>): Clone,
	{
		/// Create an indexed traversal from a `TraversableWithIndex`.
		#[document_signature]
		#[document_returns("A new `IndexedTraversal` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::optics_indexed_fold_map,
		/// 	types::optics::*,
		/// };
		/// let t: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let v = vec![10, 20, 30];
		/// let s =
		/// 	optics_indexed_fold_map::<RcBrand, _, _, _, String>(&t, |i, x| format!("{}:{}", i, x), v);
		/// assert_eq!(s, "0:101:202:30".to_string());
		/// ```
		pub fn traversed() -> Self {
			IndexedTraversal::new(Traversed(std::marker::PhantomData))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The brand of the traversable structure.",
		"The type of the elements in the structure."
	)]
	impl<'a, PointerBrand, I, Brand, A>
		IndexedTraversalPrime<
			'a,
			PointerBrand,
			I,
			Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>),
			A,
			Traversed<Brand>,
		>
	where
		Brand: TraversableWithIndex<I>,
		A: 'a + Clone,
		I: 'a,
		Apply!(<Brand as Kind!( type Of<'c, T: 'c>: 'c; )>::Of<'a, A>): Clone,
	{
		/// Create a monomorphic indexed traversal from a `TraversableWithIndex`.
		#[document_signature]
		#[document_returns("A new `IndexedTraversalPrime` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::optics_indexed_over,
		/// 	types::optics::*,
		/// };
		/// let t: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let v = vec![10, 20, 30];
		/// let v2 = optics_indexed_over::<RcBrand, _, _, _>(&t, v, |i, x| x + i as i32);
		/// assert_eq!(v2, vec![10, 21, 32]);
		/// ```
		pub fn traversed() -> Self {
			IndexedTraversalPrime::new(Traversed(std::marker::PhantomData))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, PointerBrand, I, S, T, A, B, F> Clone
		for IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
	{
		#[document_signature]
		#[document_returns("A new `IndexedTraversal` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let cloned = l.clone();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn clone(&self) -> Self {
			IndexedTraversal {
				traversal: self.traversal.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The traversal function type."
	)]
	impl<'a, PointerBrand, I, S, T, A, B, F> IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + 'a,
	{
		/// Create a new indexed traversal.
		#[document_signature]
		#[document_parameters("The traversal function.")]
		#[document_returns("A new `IndexedTraversal` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::new(Traversed(std::marker::PhantomData));
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		pub fn new(traversal: F) -> Self {
			IndexedTraversal {
				traversal,
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, PointerBrand, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone, F>
		IndexedTraversalOptic<'a, I, S, T, A, B> for IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Indexed<'a, Q, I, A, B>,
		) -> Apply!(<Q as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
			struct IWanderAdapter<'a, I, S, T, A, B, F> {
				traversal: F,
				_phantom: PhantomData<&'a (I, S, T, A, B)>,
			}

			impl<'a, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a, F>
				TraversalFunc<'a, S, T, (I, A), B> for IWanderAdapter<'a, I, S, T, A, B, F>
			where
				F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
			{
				fn apply<M: Applicative>(
					&self,
					f: Box<
						dyn Fn((I, A)) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>)
							+ 'a,
					>,
					s: S,
				) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, T>)
				where
					Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>): Clone, {
					self.traversal.apply::<M>(Box::new(move |i, a| f((i, a))), s)
				}
			}

			Q::wander(
				IWanderAdapter {
					traversal: self.traversal.clone(),
					_phantom: PhantomData,
				},
				pab.inner,
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The source type of the structure.",
		"The focus type.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, PointerBrand, I: Clone + 'a, S: 'a, A: 'a + Clone, F> IndexedFoldOptic<'a, I, S, A>
		for IndexedTraversal<'a, PointerBrand, I, S, S, A, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedFoldOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(_, x)| x.to_string());
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
		/// assert_eq!(result.run(vec![10, 20]), "1020".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + Clone + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The reference-counted pointer type for the lens.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, Q, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone, PointerBrand, F>
		IndexedSetterOptic<'a, Q, I, S, T, A, B> for IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedSetterOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::<RcFnBrand, _, _, _>::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> = IndexedSetterOptic::evaluate(&l, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn evaluate(
			&self,
			pab: Indexed<'a, FnBrand<Q>, I, A, B>,
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The original pointer type.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, P: Wander, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone, PointerBrand, F>
		IndexedOpticAdapter<'a, P, I, S, T, A, B> for IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		FnBrand,
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let _unindexed = optics_un_index::<FnBrand<RcBrand>, _, _, _, _, _>(&l);
		/// assert_eq!(optics_indexed_over::<RcBrand, _, _, _>(&l, vec![1, 2], |_i, x| x + 1), vec![2, 3]);
		/// ```
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The original pointer type.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, P: Wander, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone, PointerBrand, F>
		IndexedOpticAdapterDiscardsFocus<'a, P, I, S, T, A, B>
		for IndexedTraversal<'a, PointerBrand, I, S, T, A, B, F>
	where
		F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		FnBrand,
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversal<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversal::traversed();
		/// let _as_index = optics_as_index::<FnBrand<RcBrand>, _, _, _, _, _>(&l);
		/// assert_eq!(optics_indexed_over::<RcBrand, _, _, _>(&l, vec![1, 2], |_i, x| x + 1), vec![2, 3]);
		/// ```
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, P, I, A, B>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	/// A monomorphic indexed traversal.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The traversal function type."
	)]
	pub struct IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + 'a, {
		/// The underlying indexed traversal function.
		pub traversal: F,
		pub(crate) _phantom: PhantomData<(&'a (I, S, A), PointerBrand)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, PointerBrand, I, S, A, F> Clone for IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_returns(
			"A new `IndexedTraversalPrime` instance that is a copy of the original."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let cloned = l.clone();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn clone(&self) -> Self {
			IndexedTraversalPrime {
				traversal: self.traversal.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The traversal function type."
	)]
	impl<'a, PointerBrand, I, S, A, F> IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + 'a,
	{
		/// Create a new monomorphic indexed traversal.
		#[document_signature]
		#[document_parameters("The traversal function.")]
		#[document_returns("A new `IndexedTraversalPrime` instance.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::new(Traversed(std::marker::PhantomData));
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		pub fn new(traversal: F) -> Self {
			IndexedTraversalPrime {
				traversal,
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, PointerBrand, I: Clone + 'a, S: 'a, A: 'a + Clone, F>
		IndexedTraversalOptic<'a, I, S, S, A, A> for IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedTraversalOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Indexed<'a, Q, I, A, A>,
		) -> Apply!(<Q as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
			struct IWanderAdapter<'a, I, S, A, F> {
				traversal: F,
				_phantom: PhantomData<&'a (I, S, A)>,
			}

			impl<'a, I: Clone + 'a, S: 'a, A: 'a, F> TraversalFunc<'a, S, S, (I, A), A>
				for IWanderAdapter<'a, I, S, A, F>
			where
				F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
			{
				fn apply<M: Applicative>(
					&self,
					f: Box<
						dyn Fn((I, A)) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, A>)
							+ 'a,
					>,
					s: S,
				) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, S>)
				where
					Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, A>): Clone, {
					self.traversal.apply::<M>(Box::new(move |i, a| f((i, a))), s)
				}
			}

			Q::wander(
				IWanderAdapter {
					traversal: self.traversal.clone(),
					_phantom: PhantomData,
				},
				pab.inner,
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, PointerBrand, I: Clone + 'a, S: 'a, A: 'a + Clone, F> IndexedFoldOptic<'a, I, S, A>
		for IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedFoldOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let f = Forget::<RcBrand, String, (usize, i32), i32>::new(|(_, x)| x.to_string());
		/// let pab = Indexed::new(f);
		/// let result = IndexedFoldOptic::evaluate::<String, RcBrand>(&l, pab);
		/// assert_eq!(result.run(vec![10, 20]), "1020".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + Clone + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Indexed<'a, ForgetBrand<Q, R>, I, A, A>,
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The reference-counted pointer type for the lens.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, Q, I: Clone + 'a, S: 'a, A: 'a + Clone, PointerBrand, F>
		IndexedSetterOptic<'a, Q, I, S, S, A, A> for IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::optics::IndexedSetterOptic,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let f = std::rc::Rc::new(|(i, x): (usize, i32)| x + (i as i32))
		/// 	as std::rc::Rc<dyn Fn((usize, i32)) -> i32>;
		/// let pab = Indexed::<RcFnBrand, _, _, _>::new(f);
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> = IndexedSetterOptic::evaluate(&l, pab);
		/// assert_eq!(result(vec![10, 20]), vec![10, 21]);
		/// ```
		fn evaluate(
			&self,
			pab: Indexed<'a, FnBrand<Q>, I, A, A>,
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, S>) {
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The type of the structure.",
		"The focus type.",
		"The original pointer type.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, P: Wander, I: Clone + 'a, S: 'a, A: 'a + Clone, PointerBrand, F>
		IndexedOpticAdapter<'a, P, I, S, S, A, A> for IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		FnBrand,
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let _unindexed = optics_un_index::<FnBrand<RcBrand>, _, _, _, _, _>(&l);
		/// assert_eq!(optics_indexed_over::<RcBrand, _, _, _>(&l, vec![1, 2], |_i, x| x + 1), vec![2, 3]);
		/// ```
		fn evaluate_indexed(
			&self,
			pab: Indexed<'a, P, I, A, A>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The index type.",
		"The type of the structure.",
		"The focus type.",
		"The original pointer type.",
		"The traversal function type."
	)]
	#[document_parameters("The indexed traversal instance.")]
	impl<'a, P: Wander, I: Clone + 'a, S: 'a, A: 'a + Clone, PointerBrand, F>
		IndexedOpticAdapterDiscardsFocus<'a, P, I, S, S, A, A>
		for IndexedTraversalPrime<'a, PointerBrand, I, S, A, F>
	where
		F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
	{
		#[document_signature]
		#[document_parameters("The indexed profunctor value.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		FnBrand,
		/// 		RcBrand,
		/// 		VecBrand,
		/// 		optics::*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		/// let l: IndexedTraversalPrime<RcBrand, usize, Vec<i32>, i32, Traversed<VecBrand>> =
		/// 	IndexedTraversalPrime::traversed();
		/// let _as_index = optics_as_index::<FnBrand<RcBrand>, _, _, _, _, _>(&l);
		/// assert_eq!(optics_indexed_over::<RcBrand, _, _, _>(&l, vec![1, 2], |_i, x| x + 1), vec![2, 3]);
		/// ```
		fn evaluate_indexed_discards_focus(
			&self,
			pab: Indexed<'a, P, I, A, A>,
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			IndexedTraversalOptic::evaluate(self, pab)
		}
	}
}

pub use inner::*;
