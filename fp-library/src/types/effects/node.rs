//! Dispatch node for Run-style programs: a layer that is either a
//! first-order effect from the row `R` or a scoped-effect constructor
//! from the row `S`.
//!
//! The conceptual identity is
//!
//! ```text
//! Run<R, S, A> = FreeFamily<NodeBrand<R, S>, A>
//! ```
//!
//! [`Node`] is the runtime-tagged enum stored in the Free family's
//! `Wrap` arm; [`NodeBrand`](crate::brands::NodeBrand) is the
//! brand-level type constructor that resolves
//! `Of<'a, A>` to `Node<'a, R, S, A>`.
//!
//! ## Why two rows
//!
//! The dual-row architecture (heftia's pattern, decided in
//! [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! section 4.5) keeps first-order algebraic effects (Reader, State,
//! Choose, ...) separate from higher-order scoped effects (Catch,
//! Local, Bracket, Span). The first-order row uses
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped effect
//! functors so any effect type becomes a [`Functor`](crate::classes::Functor)
//! for free; the scoped row holds concrete constructor types interpreted
//! via manual case dispatch and does not require [`Functor`](crate::classes::Functor).
//!
//! In Phase 2 step 4a (this commit) the scoped row is structurally a
//! second [`CoproductBrand`](crate::brands::CoproductBrand) chain whose
//! tail is `CNilBrand`; Phase 4 will populate it with the standard
//! scoped constructors (`Catch`, `Local`, ...).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::NodeBrand,
			classes::{
				Extract,
				Functor,
				RefFunctor,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Tagged dispatch between a first-order effect layer and a scoped
	/// effect layer.
	///
	/// `Node<'a, R, S, A>` is the value the Free family's `Wrap` arm
	/// stores. `R` is the first-order row brand and `S` is the scoped
	/// row brand; both resolve via their `Kind` projection at the
	/// active lifetime `'a` and result type `A`.
	#[document_type_parameters(
		"The lifetime of the layer and its inner Free continuations.",
		"The first-order row brand (typically a `CoproductBrand` of `CoyonedaBrand`-wrapped effects, terminated by `CNilBrand`).",
		"The scoped-effect row brand (typically `CNilBrand` for first-order-only programs; Phase 4 populates it with scoped constructors).",
		"The result type of the layer's continuation."
	)]
	pub enum Node<'a, R, S, A>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'a, {
		/// A first-order effect layer drawn from the row `R`.
		First(Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)),
		/// A scoped-effect layer drawn from the row `S`.
		Scoped(Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)),
	}

	impl_kind! {
		impl<R: Kind_cdc7cd43dac7585f + 'static, S: Kind_cdc7cd43dac7585f + 'static>
			for NodeBrand<R, S> {
			type Of<'a, A: 'a>: 'a = Node<'a, R, S, A>;
		}
	}

	#[document_type_parameters("The first-order row brand.", "The scoped-effect row brand.")]
	impl<R, S> Functor for NodeBrand<R, S>
	where
		R: Functor + 'static,
		S: Functor + 'static,
	{
		/// `Functor::map` for a [`Node`] layer dispatches by variant:
		/// `First` recurses into `R::map`; `Scoped` recurses into
		/// `S::map`. Both row brands satisfy [`Functor`] in the
		/// canonical Run shape (the first-order row via the
		/// `CoproductBrand`-of-`CoyonedaBrand`-of-effects recursion
		/// from [Phase 2 step 2](crate::types::effects::variant_f); the
		/// scoped row will satisfy it once Phase 4 lands).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The current result type.",
			"The new result type."
		)]
		///
		#[document_parameters("The function to apply to the result.", "The Node layer.")]
		///
		#[document_returns("A Node layer with the function applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::node::Node,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let node: Node<'static, FirstRow, Scoped, i32> = Node::First(
		/// 	fp_library::types::effects::coproduct::Coproduct::inject(fp_library::types::Identity(7)),
		/// );
		/// let mapped: Node<'static, FirstRow, Scoped, i32> =
		/// 	<NodeBrand<FirstRow, Scoped> as Functor>::map(|x: i32| x + 1, node);
		/// match mapped {
		/// 	Node::First(c) => match c {
		/// 		fp_library::types::effects::coproduct::Coproduct::Inl(fp_library::types::Identity(
		/// 			x,
		/// 		)) => assert_eq!(x, 8),
		/// 		fp_library::types::effects::coproduct::Coproduct::Inr(_) => panic!("expected head Inl"),
		/// 	},
		/// 	Node::Scoped(_) => panic!("expected First variant"),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Node::First(r) => Node::First(<R as Functor>::map(func, r)),
				Node::Scoped(s) => Node::Scoped(<S as Functor>::map(func, s)),
			}
		}
	}

	#[document_type_parameters("The first-order row brand.", "The scoped-effect row brand.")]
	impl<R, S> WrapDrop for NodeBrand<R, S>
	where
		R: WrapDrop + 'static,
		S: WrapDrop + 'static,
	{
		/// Drop-time decomposition for a [`Node`] layer dispatches by
		/// variant: `First` delegates to `R::drop`; `Scoped` delegates
		/// to `S::drop`. Whatever the active row brand returns flows
		/// out unchanged; the typical Run row's
		/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand) tip returns
		/// `None`, so a Run-typical program drops via recursive drop
		/// on the layer (sound per the Wrap-depth probe documented on
		/// [`WrapDrop`]).
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type the layer would yield.")]
		///
		#[document_parameters("The Node layer being decomposed.")]
		///
		#[document_returns(
			"The active row brand's `WrapDrop::drop` result for the variant's payload."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let node: Node<'static, FirstRow, Scoped, i32> = Node::First(Coproduct::inject(Identity(42)));
		/// assert_eq!(<NodeBrand<FirstRow, Scoped> as WrapDrop>::drop(node), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				Node::First(r) => <R as WrapDrop>::drop::<X>(r),
				Node::Scoped(s) => <S as WrapDrop>::drop::<X>(s),
			}
		}
	}

	#[document_type_parameters("The first-order row brand.", "The scoped-effect row brand.")]
	impl<R, S> RefFunctor for NodeBrand<R, S>
	where
		R: RefFunctor + 'static,
		S: RefFunctor + 'static,
	{
		/// `RefFunctor::ref_map` for a [`Node`] layer dispatches by
		/// variant: `First` recurses into `R::ref_map`; `Scoped`
		/// recurses into `S::ref_map`. Mirrors the
		/// [`Functor`](crate::classes::Functor) impl above with `&self`
		/// receivers; required by Phase 2 step 4b's
		/// `RunExplicitBrand` Ref-hierarchy delegation through
		/// [`FreeExplicitBrand`](crate::brands::FreeExplicitBrand)'s
		/// `RefFunctor` impl.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The current result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply by reference to the result.",
			"The Node layer."
		)]
		///
		#[document_returns("A Node layer with the function applied to its result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::node::Node,
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let node: Node<'static, FirstRow, Scoped, i32> = Node::First(
		/// 	fp_library::types::effects::coproduct::Coproduct::inject(fp_library::types::Identity(7)),
		/// );
		/// let mapped: Node<'static, FirstRow, Scoped, i32> =
		/// 	<NodeBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x + 1, &node);
		/// match mapped {
		/// 	Node::First(c) => match c {
		/// 		fp_library::types::effects::coproduct::Coproduct::Inl(fp_library::types::Identity(
		/// 			x,
		/// 		)) => assert_eq!(x, 8),
		/// 		fp_library::types::effects::coproduct::Coproduct::Inr(_) => panic!("expected head Inl"),
		/// 	},
		/// 	Node::Scoped(_) => panic!("expected First variant"),
		/// }
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Node::First(r) => Node::First(<R as RefFunctor>::ref_map(func, r)),
				Node::Scoped(s) => Node::Scoped(<S as RefFunctor>::ref_map(func, s)),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the layer and its inner Free continuations.",
		"The first-order row brand.",
		"The scoped-effect row brand.",
		"The result type of the layer's continuation."
	)]
	#[document_parameters("The Node layer to clone.")]
	impl<'a, R, S, A> Clone for Node<'a, R, S, A>
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'a,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		/// Clones a [`Node`] by delegating to the active row brand's
		/// payload [`Clone`]. Required so the
		/// [`Rc`](std::rc::Rc)- and [`Arc`](std::sync::Arc)-backed
		/// [`Free`](crate::types::FreeExplicit) substrates can clone
		/// shared inner state when their outer refcounts are not
		/// unique.
		#[document_signature]
		///
		#[document_returns("A clone of the Node layer.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let node: Node<'static, FirstRow, Scoped, i32> = Node::First(Coproduct::inject(Identity(7)));
		/// let cloned = node.clone();
		/// assert!(matches!(cloned, Node::First(Coproduct::Inl(Identity(7)))));
		/// ```
		fn clone(&self) -> Self {
			match self {
				Node::First(r) => Node::First(r.clone()),
				Node::Scoped(s) => Node::Scoped(s.clone()),
			}
		}
	}

	#[document_type_parameters("The first-order row brand.", "The scoped-effect row brand.")]
	impl<R, S> Extract for NodeBrand<R, S>
	where
		R: Extract + 'static,
		S: Extract + 'static,
	{
		/// `Extract::extract` for a [`Node`] layer dispatches by
		/// variant: `First` recurses into `R::extract`; `Scoped`
		/// recurses into `S::extract`. Mirrors the
		/// [`Functor`](crate::classes::Functor) impl above. Used by
		/// [`FreeExplicit::evaluate`](crate::types::FreeExplicit) to
		/// pull a value out of a Run-shaped program when the underlying
		/// row brands themselves implement [`Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type yielded by the layer.")]
		///
		#[document_parameters("The Node layer being extracted.")]
		///
		#[document_returns("The active row brand's `extract` result for the variant's payload.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::{
		/// 		Identity,
		/// 		effects::{
		/// 			coproduct::Coproduct,
		/// 			node::Node,
		/// 		},
		/// 	},
		/// };
		///
		/// type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let node: Node<'static, FirstRow, Scoped, i32> = Node::First(Coproduct::inject(Identity(42)));
		/// let value = <NodeBrand<FirstRow, Scoped> as Extract>::extract::<i32>(node);
		/// assert_eq!(value, 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				Node::First(r) => <R as Extract>::extract::<A>(r),
				Node::Scoped(s) => <S as Extract>::extract::<A>(s),
			}
		}
	}
}

pub use inner::*;

#[cfg(test)]
#[expect(clippy::panic, reason = "Tests panic on unreachable Coproduct branches for clarity.")]
mod tests {
	use {
		super::*,
		crate::{
			Apply,
			brands::{
				CNilBrand,
				CoproductBrand,
				IdentityBrand,
				NodeBrand,
			},
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Identity,
				effects::coproduct::Coproduct,
			},
		},
	};

	type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
	type Scoped = CNilBrand;
	type NodeAt<'a, A> =
		Apply!(<NodeBrand<FirstRow, Scoped> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);

	#[test]
	fn first_variant_holds_first_row_layer() {
		let node: NodeAt<'static, i32> = Node::First(Coproduct::inject(Identity(7)));
		match node {
			Node::First(Coproduct::Inl(Identity(x))) => assert_eq!(x, 7),
			Node::First(Coproduct::Inr(_)) | Node::Scoped(_) => panic!("expected head Inl branch"),
		}
	}

	#[test]
	fn functor_dispatches_to_first_row() {
		let node: NodeAt<'static, i32> = Node::First(Coproduct::inject(Identity(10)));
		let mapped: NodeAt<'static, i32> =
			<NodeBrand<FirstRow, Scoped> as Functor>::map(|x: i32| x + 1, node);
		match mapped {
			Node::First(Coproduct::Inl(Identity(x))) => assert_eq!(x, 11),
			Node::First(Coproduct::Inr(_)) | Node::Scoped(_) => panic!("expected head Inl branch"),
		}
	}

	#[test]
	fn wrap_drop_first_delegates_to_row_brand() {
		let node: NodeAt<'static, i32> = Node::First(Coproduct::inject(Identity(42)));
		assert_eq!(<NodeBrand<FirstRow, Scoped> as WrapDrop>::drop(node), Some(42));
	}
}
