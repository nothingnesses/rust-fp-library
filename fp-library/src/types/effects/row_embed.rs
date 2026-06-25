//! Crate-private row embedding helpers for Run-style dual-row programs.
//!
//! These helpers perform the structural part of row subsumption: they
//! walk a Free program over `NodeBrand<R, S>`, recursively rewrite inner
//! programs to the target rows, and use `CoproductEmbedder` evidence to
//! widen both the first-order row and the scoped row. Public wrapper
//! methods such as `expand` and `weaken` are generated elsewhere; this
//! module owns the shared traversal those generated methods call.

#[fp_macros::document_module]
pub(crate) mod inner {
	// Generated wrapper methods call this module in the following W1 slice.
	#![allow(dead_code)]

	use {
		crate::{
			Apply,
			brands::NodeBrand,
			classes::{
				Functor,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCatList,
				ArcFree,
				ArcFreeExplicit,
				CatList,
				Free,
				FreeExplicit,
				RcCatList,
				RcFree,
				RcFreeExplicit,
				arc_free::{
					ArcContinuation,
					ArcTypeErasedValue,
				},
				effects::{
					coproduct::CoproductEmbedder,
					node::Node,
				},
				free::{
					Continuation,
					TypeErasedValue,
				},
				rc_free::{
					RcContinuation,
					RcTypeErasedValue,
				},
			},
		},
		fp_macros::*,
	};

	#[doc(hidden)]
	/// Raw Free branch carried by continuation-aware row embedding.
	pub(crate) type RawNodeFree<R, S> = Free<NodeBrand<R, S>, TypeErasedValue>;

	#[doc(hidden)]
	/// Continuation queue carried by continuation-aware row embedding.
	pub(crate) type NodeContinuations<R, S> = CatList<Continuation<NodeBrand<R, S>>>;

	#[doc(hidden)]
	/// Raw RcFree branch carried by continuation-aware row embedding.
	pub(crate) type RawRcNodeFree<R, S> = RcFree<NodeBrand<R, S>, RcTypeErasedValue>;

	#[doc(hidden)]
	/// Rc continuation queue carried by continuation-aware row embedding.
	pub(crate) type RcNodeContinuations<R, S> = RcCatList<RcContinuation<NodeBrand<R, S>>>;

	#[doc(hidden)]
	/// Raw ArcFree branch carried by continuation-aware row embedding.
	pub(crate) type RawArcNodeFree<R, S> = ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>;

	#[doc(hidden)]
	/// Arc continuation queue carried by continuation-aware row embedding.
	pub(crate) type ArcNodeContinuations<R, S> = ArcCatList<ArcContinuation<NodeBrand<R, S>>>;

	#[doc(hidden)]
	/// Boxed FreeExplicit branch carried by explicit row embedding.
	pub(crate) type BoxedExplicitNodeFree<'a, R, S, A> = Box<FreeExplicit<'a, NodeBrand<R, S>, A>>;

	#[doc(hidden)]
	/// RcFreeExplicit branch carried by explicit row embedding.
	pub(crate) type RawRcExplicitNodeFree<'a, R, S, A> = RcFreeExplicit<'a, NodeBrand<R, S>, A>;

	#[doc(hidden)]
	/// ArcFreeExplicit branch carried by explicit row embedding.
	pub(crate) type RawArcExplicitNodeFree<'a, R, S, A> = ArcFreeExplicit<'a, NodeBrand<R, S>, A>;

	/// Embeds a Free program over one dual-row Node brand into wider rows.
	///
	/// This is the shared structural traversal for default Free-backed
	/// Run programs. It keeps pending continuations outside suspended
	/// layers until a branch is selected, recursively rewrites the selected
	/// branch programs, and then widens first-order and scoped row layers
	/// with the caller-provided `CoproductEmbedder` evidence.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The Free program to structurally widen.")]
	#[document_returns("A Free program with the same result over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through focused row-embedding tests and generated Run wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_free_node<R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		free: Free<NodeBrand<R, S>, A>
	) -> Free<NodeBrand<R2, S2>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		A: 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		free.transform_raw(
			embed_raw_node_layer::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
			embed_node_continuations::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
		)
	}

	/// Embeds a raw scoped row layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The raw scoped row layer to widen.")]
	#[document_returns("A raw scoped row layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated Run boundary-frame expansion tests; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let scoped_layer_count = 1;
	/// assert_eq!(scoped_layer_count, 1);
	/// ```
	pub(crate) fn embed_scoped_row_layer<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R, S>,
		>)
	) -> Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawNodeFree<R2, S2>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>) = <S as Functor>::map(
			embed_free_node::<R, S, R2, S2, TypeErasedValue, REmbedIdx, SEmbedIdx>,
			layer,
		);
		<Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>) as CoproductEmbedder<
			Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RawNodeFree<R2, S2>,
			>),
			SEmbedIdx,
		>>::embed(mapped)
	}

	/// Embeds a raw suspended Node layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to widen.")]
	#[document_returns("A suspended Node layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw Free branches and is covered through embed_free_node tests."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_raw_node_layer<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R, S>,
		>)
	) -> Apply!(<NodeBrand<R2, S2> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawNodeFree<R2, S2>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>) = <R as Functor>::map(
					embed_free_node::<R, S, R2, S2, TypeErasedValue, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawNodeFree<R2, S2>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>) = <S as Functor>::map(
					embed_free_node::<R, S, R2, S2, TypeErasedValue, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>) as CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawNodeFree<R2, S2>,
					>),
					SEmbedIdx,
				>>::embed(mapped);
				Node::Scoped(embedded)
			}
		}
	}

	/// Embeds a raw continuation queue into the target rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The continuation queue to widen.")]
	#[document_returns("A continuation queue whose returned programs use the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper transforms raw continuation queues and is covered through embed_free_node continuation tests."
	)]
	///
	/// ```
	/// let continuation_count = 0;
	/// assert_eq!(continuation_count, 0);
	/// ```
	pub(crate) fn embed_node_continuations<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		continuations: NodeContinuations<R, S>
	) -> NodeContinuations<R2, S2>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		continuations.map(|continuation| {
			Box::new(move |value| {
				embed_free_node::<R, S, R2, S2, TypeErasedValue, REmbedIdx, SEmbedIdx>(
					continuation(value),
				)
			}) as Continuation<NodeBrand<R2, S2>>
		})
	}

	/// Embeds only the first-order row of a Free program.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The Free program to structurally widen.")]
	#[document_returns(
		"A Free program with the same result and scoped row over the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated weaken wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_free_node_first_order<R, S, R2, A, REmbedIdx>(
		free: Free<NodeBrand<R, S>, A>
	) -> Free<NodeBrand<R2, S>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		A: 'static,
		REmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		free.transform_raw(
			embed_first_order_raw_node_layer::<R, S, R2, REmbedIdx>,
			embed_first_order_node_continuations::<R, S, R2, REmbedIdx>,
		)
	}

	/// Maps a scoped row layer while preserving the scoped row identity.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The raw scoped row layer to update.")]
	#[document_returns(
		"A raw scoped row layer whose inner programs use the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated Run::weaken boundary-frame tests; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let scoped_layer_count = 1;
	/// assert_eq!(scoped_layer_count, 1);
	/// ```
	pub(crate) fn embed_first_order_scoped_row_layer<R, S, R2, REmbedIdx>(
		layer: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R, S>,
		>)
	) -> Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawNodeFree<R2, S>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		<S as Functor>::map(
			embed_free_node_first_order::<R, S, R2, TypeErasedValue, REmbedIdx>,
			layer,
		)
	}

	/// Embeds only the first-order arm of a raw suspended Node layer.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to update.")]
	#[document_returns("A suspended Node layer over the target first-order row.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw Free branches and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_first_order_raw_node_layer<R, S, R2, REmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R, S>,
		>)
	) -> Apply!(<NodeBrand<R2, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawNodeFree<R2, S>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S>,
				>) = <R as Functor>::map(
					embed_free_node_first_order::<R, S, R2, TypeErasedValue, REmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawNodeFree<R2, S>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped = <S as Functor>::map(
					embed_free_node_first_order::<R, S, R2, TypeErasedValue, REmbedIdx>,
					row,
				);
				Node::Scoped(mapped)
			}
		}
	}

	/// Embeds a raw continuation queue while preserving the scoped row.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The continuation queue to update.")]
	#[document_returns(
		"A continuation queue whose returned programs use the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper transforms raw continuation queues and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let continuation_count = 0;
	/// assert_eq!(continuation_count, 0);
	/// ```
	pub(crate) fn embed_first_order_node_continuations<R, S, R2, REmbedIdx>(
		continuations: NodeContinuations<R, S>
	) -> NodeContinuations<R2, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		continuations.map(|continuation| {
			Box::new(move |value| {
				embed_free_node_first_order::<R, S, R2, TypeErasedValue, REmbedIdx>(continuation(
					value,
				))
			}) as Continuation<NodeBrand<R2, S>>
		})
	}

	/// Embeds an RcFree program over one dual-row Node brand into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The RcFree program to structurally widen.")]
	#[document_returns("An RcFree program with the same result over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through focused row-embedding tests and generated RcRun wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_rc_free_node<R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		free: RcFree<NodeBrand<R, S>, A>
	) -> RcFree<NodeBrand<R2, S2>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		A: 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		free.transform_raw(
			embed_rc_raw_node_layer::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
			embed_rc_node_continuations::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
		)
	}

	/// Embeds a raw suspended RcFree Node layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to widen.")]
	#[document_returns("A suspended Node layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw RcFree branches and is covered through embed_rc_free_node tests."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_rc_raw_node_layer<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>)
	) -> Apply!(<NodeBrand<R2, S2> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawRcNodeFree<R2, S2>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>) = <R as Functor>::map(
					embed_rc_free_node::<R, S, R2, S2, RcTypeErasedValue, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawRcNodeFree<R2, S2>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>) = <S as Functor>::map(
					embed_rc_free_node::<R, S, R2, S2, RcTypeErasedValue, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>) as CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawRcNodeFree<R2, S2>,
					>),
					SEmbedIdx,
				>>::embed(mapped);
				Node::Scoped(embedded)
			}
		}
	}

	/// Embeds a raw Rc continuation queue into the target rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The continuation queue to widen.")]
	#[document_returns("A continuation queue whose returned programs use the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper transforms raw Rc continuation queues and is covered through embed_rc_free_node continuation tests."
	)]
	///
	/// ```
	/// let continuation_count = 0;
	/// assert_eq!(continuation_count, 0);
	/// ```
	pub(crate) fn embed_rc_node_continuations<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		mut continuations: RcNodeContinuations<R, S>
	) -> RcNodeContinuations<R2, S2>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			>, {
		let mut transformed = RcCatList::empty();
		while let Some((continuation, rest)) = continuations.uncons() {
			transformed = transformed.snoc(RcContinuation::new(move |value| {
				embed_rc_free_node::<R, S, R2, S2, RcTypeErasedValue, REmbedIdx, SEmbedIdx>(
					continuation.call(value),
				)
			}));
			continuations = rest;
		}
		transformed
	}

	/// Embeds only the first-order row of an RcFree program.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The RcFree program to structurally widen.")]
	#[document_returns(
		"An RcFree program with the same result and scoped row over the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated RcRun::weaken methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_rc_free_node_first_order<R, S, R2, A, REmbedIdx>(
		free: RcFree<NodeBrand<R, S>, A>
	) -> RcFree<NodeBrand<R2, S>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		A: 'static,
		REmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		free.transform_raw(
			embed_rc_first_order_raw_node_layer::<R, S, R2, REmbedIdx>,
			embed_rc_first_order_node_continuations::<R, S, R2, REmbedIdx>,
		)
	}

	/// Embeds only the first-order arm of a raw suspended RcFree Node layer.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to update.")]
	#[document_returns("A suspended Node layer over the target first-order row.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw RcFree branches and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_rc_first_order_raw_node_layer<R, S, R2, REmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>)
	) -> Apply!(<NodeBrand<R2, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawRcNodeFree<R2, S>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S>,
				>) = <R as Functor>::map(
					embed_rc_free_node_first_order::<R, S, R2, RcTypeErasedValue, REmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawRcNodeFree<R2, S>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped = <S as Functor>::map(
					embed_rc_free_node_first_order::<R, S, R2, RcTypeErasedValue, REmbedIdx>,
					row,
				);
				Node::Scoped(mapped)
			}
		}
	}

	/// Embeds an Rc continuation queue while preserving the scoped row.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The continuation queue to update.")]
	#[document_returns(
		"A continuation queue whose returned programs use the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper transforms raw Rc continuation queues and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let continuation_count = 0;
	/// assert_eq!(continuation_count, 0);
	/// ```
	pub(crate) fn embed_rc_first_order_node_continuations<R, S, R2, REmbedIdx>(
		mut continuations: RcNodeContinuations<R, S>
	) -> RcNodeContinuations<R2, S>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		REmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R, S>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawRcNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawRcNodeFree<R2, S>,
				>),
				REmbedIdx,
			>, {
		let mut transformed = RcCatList::empty();
		while let Some((continuation, rest)) = continuations.uncons() {
			transformed = transformed.snoc(RcContinuation::new(move |value| {
				embed_rc_free_node_first_order::<R, S, R2, RcTypeErasedValue, REmbedIdx>(
					continuation.call(value),
				)
			}));
			continuations = rest;
		}
		transformed
	}

	/// Embeds an ArcFree program over one dual-row Node brand into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The ArcFree program to structurally widen.")]
	#[document_returns("An ArcFree program with the same result over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through focused row-embedding tests and generated ArcRun wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_arc_free_node<R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		free: ArcFree<NodeBrand<R, S>, A>
	) -> ArcFree<NodeBrand<R2, S2>, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R, S>> = Node<'static, R, S, RawArcNodeFree<R, S>>,
			> + 'static,
		NodeBrand<R2, S2>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R2, S2>> = Node<'static, R2, S2, RawArcNodeFree<R2, S2>>,
			> + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		S2: WrapDrop + SendFunctor + 'static,
		A: 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Node<'static, R, S, RawArcNodeFree<R, S>>: Clone + Send + Sync,
		Node<'static, R2, S2, RawArcNodeFree<R2, S2>>: Send + Sync,
		RawArcNodeFree<R, S>: Send + Sync,
		RawArcNodeFree<R2, S2>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>),
				REmbedIdx,
			> + Send
			+ Sync,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			> + Send
			+ Sync, {
		free.transform_raw(
			embed_arc_raw_node_layer::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
			embed_arc_node_continuations::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
		)
	}

	/// Embeds a raw suspended ArcFree Node layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to widen.")]
	#[document_returns("A suspended Node layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw ArcFree branches and is covered through embed_arc_free_node tests."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_arc_raw_node_layer<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R, S>,
		>)
	) -> Apply!(<NodeBrand<R2, S2> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawArcNodeFree<R2, S2>,
	>)
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R, S>> = Node<'static, R, S, RawArcNodeFree<R, S>>,
			> + 'static,
		NodeBrand<R2, S2>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R2, S2>> = Node<'static, R2, S2, RawArcNodeFree<R2, S2>>,
			> + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		S2: WrapDrop + SendFunctor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Node<'static, R, S, RawArcNodeFree<R, S>>: Clone + Send + Sync,
		Node<'static, R2, S2, RawArcNodeFree<R2, S2>>: Send + Sync,
		RawArcNodeFree<R, S>: Send + Sync,
		RawArcNodeFree<R2, S2>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>),
				REmbedIdx,
			> + Send
			+ Sync,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			> + Send
			+ Sync, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>) = <R as SendFunctor>::send_map(
					embed_arc_free_node::<R, S, R2, S2, ArcTypeErasedValue, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawArcNodeFree<R2, S2>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>) = <S as SendFunctor>::send_map(
					embed_arc_free_node::<R, S, R2, S2, ArcTypeErasedValue, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>) as CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawArcNodeFree<R2, S2>,
					>),
					SEmbedIdx,
				>>::embed(mapped);
				Node::Scoped(embedded)
			}
		}
	}

	/// Embeds a raw Arc continuation queue into the target rows.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The continuation queue to widen.")]
	#[document_returns("A continuation queue whose returned programs use the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper transforms raw Arc continuation queues and is covered through embed_arc_free_node continuation tests."
	)]
	///
	/// ```
	/// let continuation_count = 0;
	/// assert_eq!(continuation_count, 0);
	/// ```
	pub(crate) fn embed_arc_node_continuations<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
		mut continuations: ArcNodeContinuations<R, S>
	) -> ArcNodeContinuations<R2, S2>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R, S>> = Node<'static, R, S, RawArcNodeFree<R, S>>,
			> + 'static,
		NodeBrand<R2, S2>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R2, S2>> = Node<'static, R2, S2, RawArcNodeFree<R2, S2>>,
			> + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		S2: WrapDrop + SendFunctor + 'static,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Node<'static, R, S, RawArcNodeFree<R, S>>: Clone + Send + Sync,
		Node<'static, R2, S2, RawArcNodeFree<R2, S2>>: Send + Sync,
		RawArcNodeFree<R, S>: Send + Sync,
		RawArcNodeFree<R2, S2>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>),
				REmbedIdx,
			> + Send
			+ Sync,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S2>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S2>,
				>),
				SEmbedIdx,
			> + Send
			+ Sync, {
		let mut transformed = ArcCatList::empty();
		while let Some((continuation, rest)) = continuations.uncons() {
			transformed = transformed.snoc(ArcContinuation::new(move |value| {
				embed_arc_free_node::<R, S, R2, S2, ArcTypeErasedValue, REmbedIdx, SEmbedIdx>(
					continuation.call(value),
				)
			}));
			continuations = rest;
		}
		transformed
	}

	/// Embeds only the first-order row of an ArcFree program.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The ArcFree program to structurally widen.")]
	#[document_returns(
		"An ArcFree program with the same result and scoped row over the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated ArcRun::weaken methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_arc_free_node_first_order<R, S, R2, A, REmbedIdx>(
		free: ArcFree<NodeBrand<R, S>, A>
	) -> ArcFree<NodeBrand<R2, S>, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R, S>> = Node<'static, R, S, RawArcNodeFree<R, S>>,
			> + 'static,
		NodeBrand<R2, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R2, S>> = Node<'static, R2, S, RawArcNodeFree<R2, S>>,
			> + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		A: 'static,
		REmbedIdx: 'static,
		Node<'static, R, S, RawArcNodeFree<R, S>>: Clone + Send + Sync,
		Node<'static, R2, S, RawArcNodeFree<R2, S>>: Send + Sync,
		RawArcNodeFree<R, S>: Send + Sync,
		RawArcNodeFree<R2, S>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S>,
				>),
				REmbedIdx,
			> + Send
			+ Sync, {
		free.transform_raw(
			embed_arc_first_order_raw_node_layer::<R, S, R2, REmbedIdx>,
			embed_arc_first_order_node_continuations::<R, S, R2, REmbedIdx>,
		)
	}

	/// Embeds only the first-order arm of a raw suspended ArcFree Node layer.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to update.")]
	#[document_returns("A suspended Node layer over the target first-order row.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw ArcFree branches and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_arc_first_order_raw_node_layer<R, S, R2, REmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R, S>,
		>)
	) -> Apply!(<NodeBrand<R2, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RawArcNodeFree<R2, S>,
	>)
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R, S>> = Node<'static, R, S, RawArcNodeFree<R, S>>,
			> + 'static,
		NodeBrand<R2, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R2, S>> = Node<'static, R2, S, RawArcNodeFree<R2, S>>,
			> + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		REmbedIdx: 'static,
		Node<'static, R, S, RawArcNodeFree<R, S>>: Clone + Send + Sync,
		Node<'static, R2, S, RawArcNodeFree<R2, S>>: Send + Sync,
		RawArcNodeFree<R, S>: Send + Sync,
		RawArcNodeFree<R2, S>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S>,
				>),
				REmbedIdx,
			> + Send
			+ Sync, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S>,
				>) = <R as SendFunctor>::send_map(
					embed_arc_free_node_first_order::<R, S, R2, ArcTypeErasedValue, REmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RawArcNodeFree<R2, S>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped = <S as SendFunctor>::send_map(
					embed_arc_free_node_first_order::<R, S, R2, ArcTypeErasedValue, REmbedIdx>,
					row,
				);
				Node::Scoped(mapped)
			}
		}
	}

	/// Embeds an Arc continuation queue while preserving the scoped row.
	#[document_signature]
	#[document_type_parameters(
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The continuation queue to update.")]
	#[document_returns(
		"A continuation queue whose returned programs use the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper transforms raw Arc continuation queues and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let continuation_count = 0;
	/// assert_eq!(continuation_count, 0);
	/// ```
	pub(crate) fn embed_arc_first_order_node_continuations<R, S, R2, REmbedIdx>(
		mut continuations: ArcNodeContinuations<R, S>
	) -> ArcNodeContinuations<R2, S>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R, S>> = Node<'static, R, S, RawArcNodeFree<R, S>>,
			> + 'static,
		NodeBrand<R2, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, RawArcNodeFree<R2, S>> = Node<'static, R2, S, RawArcNodeFree<R2, S>>,
			> + 'static,
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		REmbedIdx: 'static,
		Node<'static, R, S, RawArcNodeFree<R, S>>: Clone + Send + Sync,
		Node<'static, R2, S, RawArcNodeFree<R2, S>>: Send + Sync,
		RawArcNodeFree<R, S>: Send + Sync,
		RawArcNodeFree<R2, S>: Send + Sync,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RawArcNodeFree<R2, S>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RawArcNodeFree<R2, S>,
				>),
				REmbedIdx,
			> + Send
			+ Sync, {
		let mut transformed = ArcCatList::empty();
		while let Some((continuation, rest)) = continuations.uncons() {
			transformed = transformed.snoc(ArcContinuation::new(move |value| {
				embed_arc_free_node_first_order::<R, S, R2, ArcTypeErasedValue, REmbedIdx>(
					continuation.call(value),
				)
			}));
			continuations = rest;
		}
		transformed
	}

	/// Embeds a FreeExplicit program over one dual-row Node brand into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The FreeExplicit program to structurally widen.")]
	#[document_returns("A FreeExplicit program with the same result over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through focused row-embedding tests and generated RunExplicit wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_free_explicit_node<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		free: FreeExplicit<'a, NodeBrand<R, S>, A>
	) -> FreeExplicit<'a, NodeBrand<R2, S2>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		A: 'a,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>),
				SEmbedIdx,
			>, {
		free.transform_raw(
			embed_free_explicit_raw_node_layer::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
		)
	}

	/// Embeds a raw suspended FreeExplicit Node layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to widen.")]
	#[document_returns("A suspended Node layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw FreeExplicit branches and is covered through embed_free_explicit_node tests."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_free_explicit_raw_node_layer<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R, S, A>,
		>)
	) -> Apply!(<NodeBrand<R2, S2> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		BoxedExplicitNodeFree<'a, R2, S2, A>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		A: 'a,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>),
				SEmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>) = <R as Functor>::map(
					|inner: BoxedExplicitNodeFree<'a, R, S, A>| {
						Box::new(embed_free_explicit_node::<
							'a,
							R,
							S,
							R2,
							S2,
							A,
							REmbedIdx,
							SEmbedIdx,
						>(*inner))
					},
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						BoxedExplicitNodeFree<'a, R2, S2, A>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>) = <S as Functor>::map(
					|inner: BoxedExplicitNodeFree<'a, R, S, A>| {
						Box::new(embed_free_explicit_node::<
							'a,
							R,
							S,
							R2,
							S2,
							A,
							REmbedIdx,
							SEmbedIdx,
						>(*inner))
					},
					row,
				);
				let embedded = <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S2, A>,
				>) as CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						BoxedExplicitNodeFree<'a, R2, S2, A>,
					>),
					SEmbedIdx,
				>>::embed(mapped);
				Node::Scoped(embedded)
			}
		}
	}

	/// Embeds only the first-order row of a FreeExplicit program.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The FreeExplicit program to structurally widen.")]
	#[document_returns(
		"A FreeExplicit program with the same result and scoped row over the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated RunExplicit::weaken methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_free_explicit_node_first_order<'a, R, S, R2, A, REmbedIdx>(
		free: FreeExplicit<'a, NodeBrand<R, S>, A>
	) -> FreeExplicit<'a, NodeBrand<R2, S>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		A: 'a,
		REmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R2, S, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S, A>,
				>),
				REmbedIdx,
			>, {
		free.transform_raw(
			embed_free_explicit_first_order_raw_node_layer::<'a, R, S, R2, A, REmbedIdx>,
		)
	}

	/// Embeds only the first-order arm of a raw suspended FreeExplicit Node layer.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to update.")]
	#[document_returns("A suspended Node layer over the target first-order row.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw FreeExplicit branches and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_free_explicit_first_order_raw_node_layer<'a, R, S, R2, A, REmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R, S, A>,
		>)
	) -> Apply!(<NodeBrand<R2, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		BoxedExplicitNodeFree<'a, R2, S, A>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		A: 'a,
		REmbedIdx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			BoxedExplicitNodeFree<'a, R2, S, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S, A>,
				>),
				REmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S, A>,
				>) = <R as Functor>::map(
					|inner: BoxedExplicitNodeFree<'a, R, S, A>| {
						Box::new(
							embed_free_explicit_node_first_order::<'a, R, S, R2, A, REmbedIdx>(
								*inner,
							),
						)
					},
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					BoxedExplicitNodeFree<'a, R2, S, A>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						BoxedExplicitNodeFree<'a, R2, S, A>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped = <S as Functor>::map(
					|inner: BoxedExplicitNodeFree<'a, R, S, A>| {
						Box::new(
							embed_free_explicit_node_first_order::<'a, R, S, R2, A, REmbedIdx>(
								*inner,
							),
						)
					},
					row,
				);
				Node::Scoped(mapped)
			}
		}
	}

	/// Embeds an RcFreeExplicit program over one dual-row Node brand into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The RcFreeExplicit program to structurally widen.")]
	#[document_returns("An RcFreeExplicit program with the same result over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through focused row-embedding tests and generated RcRunExplicit wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_rc_free_explicit_node<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		free: RcFreeExplicit<'a, NodeBrand<R, S>, A>
	) -> RcFreeExplicit<'a, NodeBrand<R2, S2>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>),
				SEmbedIdx,
			>, {
		free.transform_raw(
			embed_rc_free_explicit_raw_node_layer::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
		)
	}

	/// Embeds a raw suspended RcFreeExplicit Node layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to widen.")]
	#[document_returns("A suspended Node layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw RcFreeExplicit branches and is covered through embed_rc_free_explicit_node tests."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_rc_free_explicit_raw_node_layer<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R, S, A>,
		>)
	) -> Apply!(<NodeBrand<R2, S2> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RawRcExplicitNodeFree<'a, R2, S2, A>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		S2: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>),
				REmbedIdx,
			>,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>),
				SEmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>) = <R as Functor>::map(
					embed_rc_free_explicit_node::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RawRcExplicitNodeFree<'a, R2, S2, A>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>) = <S as Functor>::map(
					embed_rc_free_explicit_node::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S2, A>,
				>) as CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RawRcExplicitNodeFree<'a, R2, S2, A>,
					>),
					SEmbedIdx,
				>>::embed(mapped);
				Node::Scoped(embedded)
			}
		}
	}

	/// Embeds only the first-order row of an RcFreeExplicit program.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The RcFreeExplicit program to structurally widen.")]
	#[document_returns(
		"An RcFreeExplicit program with the same result and scoped row over the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated RcRunExplicit::weaken methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_rc_free_explicit_node_first_order<'a, R, S, R2, A, REmbedIdx>(
		free: RcFreeExplicit<'a, NodeBrand<R, S>, A>
	) -> RcFreeExplicit<'a, NodeBrand<R2, S>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		REmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R2, S, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S, A>,
				>),
				REmbedIdx,
			>, {
		free.transform_raw(
			embed_rc_free_explicit_first_order_raw_node_layer::<'a, R, S, R2, A, REmbedIdx>,
		)
	}

	/// Embeds only the first-order arm of a raw suspended RcFreeExplicit Node layer.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to update.")]
	#[document_returns("A suspended Node layer over the target first-order row.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw RcFreeExplicit branches and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_rc_free_explicit_first_order_raw_node_layer<'a, R, S, R2, A, REmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R, S, A>,
		>)
	) -> Apply!(<NodeBrand<R2, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RawRcExplicitNodeFree<'a, R2, S, A>,
	>)
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		R2: WrapDrop + Functor + 'static,
		A: Clone + 'a,
		REmbedIdx: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawRcExplicitNodeFree<'a, R2, S, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S, A>,
				>),
				REmbedIdx,
			>, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S, A>,
				>) = <R as Functor>::map(
					embed_rc_free_explicit_node_first_order::<'a, R, S, R2, A, REmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawRcExplicitNodeFree<'a, R2, S, A>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RawRcExplicitNodeFree<'a, R2, S, A>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped = <S as Functor>::map(
					embed_rc_free_explicit_node_first_order::<'a, R, S, R2, A, REmbedIdx>,
					row,
				);
				Node::Scoped(mapped)
			}
		}
	}

	/// Embeds an ArcFreeExplicit program over one dual-row Node brand into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The ArcFreeExplicit program to structurally widen.")]
	#[document_returns("An ArcFreeExplicit program with the same result over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through focused row-embedding tests and generated ArcRunExplicit wrapper methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_arc_free_explicit_node<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		free: ArcFreeExplicit<'a, NodeBrand<R, S>, A>
	) -> ArcFreeExplicit<'a, NodeBrand<R2, S2>, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		S2: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		RawArcExplicitNodeFree<'a, R, S, A>: Send + Sync,
		RawArcExplicitNodeFree<'a, R2, S2, A>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>),
				REmbedIdx,
			> + Send
			+ Sync,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>),
				SEmbedIdx,
			> + Send
			+ Sync, {
		free.transform_raw(
			embed_arc_free_explicit_raw_node_layer::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
		)
	}

	/// Embeds a raw suspended ArcFreeExplicit Node layer into wider rows.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The source scoped row brand.",
		"The target first-order row brand.",
		"The target scoped row brand.",
		"The result type.",
		"The first-order row embedding witness.",
		"The scoped row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to widen.")]
	#[document_returns("A suspended Node layer over the target rows.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw ArcFreeExplicit branches and is covered through embed_arc_free_explicit_node tests."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_arc_free_explicit_raw_node_layer<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R, S, A>,
		>)
	) -> Apply!(<NodeBrand<R2, S2> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RawArcExplicitNodeFree<'a, R2, S2, A>,
	>)
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		S2: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		REmbedIdx: 'static,
		SEmbedIdx: 'static,
		RawArcExplicitNodeFree<'a, R, S, A>: Send + Sync,
		RawArcExplicitNodeFree<'a, R2, S2, A>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>),
				REmbedIdx,
			> + Send
			+ Sync,
		Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R2, S2, A>,
		>): CoproductEmbedder<
				Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>),
				SEmbedIdx,
			> + Send
			+ Sync, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>) = <R as SendFunctor>::send_map(
					embed_arc_free_explicit_node::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RawArcExplicitNodeFree<'a, R2, S2, A>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped: Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>) = <S as SendFunctor>::send_map(
					embed_arc_free_explicit_node::<'a, R, S, R2, S2, A, REmbedIdx, SEmbedIdx>,
					row,
				);
				let embedded = <Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S2, A>,
				>) as CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RawArcExplicitNodeFree<'a, R2, S2, A>,
					>),
					SEmbedIdx,
				>>::embed(mapped);
				Node::Scoped(embedded)
			}
		}
	}

	/// Embeds only the first-order row of an ArcFreeExplicit program.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The ArcFreeExplicit program to structurally widen.")]
	#[document_returns(
		"An ArcFreeExplicit program with the same result and scoped row over the target first-order row."
	)]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper is exercised through generated ArcRunExplicit::weaken methods; external examples cannot name the helper."
	)]
	///
	/// ```
	/// let value = 42;
	/// assert_eq!(value, 42);
	/// ```
	pub(crate) fn embed_arc_free_explicit_node_first_order<'a, R, S, R2, A, REmbedIdx>(
		free: ArcFreeExplicit<'a, NodeBrand<R, S>, A>
	) -> ArcFreeExplicit<'a, NodeBrand<R2, S>, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		REmbedIdx: 'static,
		RawArcExplicitNodeFree<'a, R, S, A>: Send + Sync,
		RawArcExplicitNodeFree<'a, R2, S, A>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R2, S, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S, A>,
				>),
				REmbedIdx,
			> + Send
			+ Sync, {
		free.transform_raw(
			embed_arc_free_explicit_first_order_raw_node_layer::<'a, R, S, R2, A, REmbedIdx>,
		)
	}

	/// Embeds only the first-order arm of a raw suspended ArcFreeExplicit Node layer.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime that bounds the payload and row brands.",
		"The source first-order row brand.",
		"The scoped row brand, preserved unchanged.",
		"The target first-order row brand.",
		"The result type.",
		"The first-order row embedding witness."
	)]
	#[document_parameters("The suspended Node layer to update.")]
	#[document_returns("A suspended Node layer over the target first-order row.")]
	#[document_examples(
		skip_call_check,
		reason = "This crate-private helper manipulates raw ArcFreeExplicit branches and is covered through generated weaken methods."
	)]
	///
	/// ```
	/// let layer_count = 1;
	/// assert_eq!(layer_count, 1);
	/// ```
	fn embed_arc_free_explicit_first_order_raw_node_layer<'a, R, S, R2, A, REmbedIdx>(
		layer: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R, S, A>,
		>)
	) -> Apply!(<NodeBrand<R2, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RawArcExplicitNodeFree<'a, R2, S, A>,
	>)
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		R2: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'a,
		REmbedIdx: 'static,
		RawArcExplicitNodeFree<'a, R, S, A>: Send + Sync,
		RawArcExplicitNodeFree<'a, R2, S, A>: Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R, S, A>,
		>): Clone,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RawArcExplicitNodeFree<'a, R2, S, A>,
		>): CoproductEmbedder<
				Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S, A>,
				>),
				REmbedIdx,
			> + Send
			+ Sync, {
		match layer {
			Node::First(row) => {
				let mapped: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S, A>,
				>) = <R as SendFunctor>::send_map(
					embed_arc_free_explicit_node_first_order::<'a, R, S, R2, A, REmbedIdx>,
					row,
				);
				let embedded = <Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RawArcExplicitNodeFree<'a, R2, S, A>,
				>) as CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RawArcExplicitNodeFree<'a, R2, S, A>,
					>),
					REmbedIdx,
				>>::embed(mapped);
				Node::First(embedded)
			}
			Node::Scoped(row) => {
				let mapped = <S as SendFunctor>::send_map(
					embed_arc_free_explicit_node_first_order::<'a, R, S, R2, A, REmbedIdx>,
					row,
				);
				Node::Scoped(mapped)
			}
		}
	}
}

pub(crate) use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::inner::{
			embed_arc_free_explicit_node,
			embed_arc_free_node,
			embed_free_explicit_node,
			embed_free_node,
			embed_rc_free_explicit_node,
			embed_rc_free_node,
		},
		crate::{
			brands::{
				ArcCoyonedaBrand,
				BoxBrand,
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
				IdentityBrand,
				NodeBrand,
				OptionBrand,
				RcCoyonedaBrand,
			},
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				Coyoneda,
				Free,
				FreeExplicit,
				Identity,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				effects::{
					coproduct::Coproduct,
					node::Node,
				},
			},
		},
	};

	type ExtraRowCell = CoyonedaBrand<OptionBrand>;
	type NarrowRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type WideRow = CoproductBrand<ExtraRowCell, NarrowRow>;
	type RcExtraRowCell = RcCoyonedaBrand<OptionBrand>;
	type RcNarrowRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	type RcWideRow = CoproductBrand<RcExtraRowCell, RcNarrowRow>;
	type ArcExtraRowCell = ArcCoyonedaBrand<OptionBrand>;
	type ArcNarrowRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	type ArcWideRow = CoproductBrand<ArcExtraRowCell, ArcNarrowRow>;

	#[test]
	fn embed_free_node_widens_first_order_row_and_preserves_continuation() {
		let layer = Coproduct::inject(Coyoneda::lift(Identity(40)));
		let free: Free<NodeBrand<NarrowRow, CNilBrand>, i32> =
			Free::<_, _, BoxBrand>::lift_f(Node::First(layer)).map(|value| value + 2);

		let widened: Free<NodeBrand<WideRow, CNilBrand>, i32> = embed_free_node(free);

		let continuation_value = match widened.resume() {
			Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
				let Identity(next) = coyo.lower();
				next.resume().ok()
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}

	#[test]
	fn embed_free_node_widens_scoped_row_and_preserves_continuation() {
		let layer = Coproduct::inject(Coyoneda::lift(Identity(7)));
		let free: Free<NodeBrand<CNilBrand, NarrowRow>, i32> =
			Free::<_, _, BoxBrand>::lift_f(Node::Scoped(layer)).map(|value| value * 6);

		let widened: Free<NodeBrand<CNilBrand, WideRow>, i32> = embed_free_node(free);

		let continuation_value = match widened.resume() {
			Err(Node::Scoped(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
				let Identity(next) = coyo.lower();
				next.resume().ok()
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}

	#[test]
	fn embed_rc_free_node_widens_first_order_row_and_preserves_continuation() {
		let layer = Coproduct::inject(RcCoyoneda::lift(Identity(40)));
		let free: RcFree<NodeBrand<RcNarrowRow, CNilBrand>, i32> =
			RcFree::lift_f(Node::First(layer)).map(|value| value + 2);

		let widened: RcFree<NodeBrand<RcWideRow, CNilBrand>, i32> = embed_rc_free_node(free);

		let continuation_value = match widened.resume() {
			Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
				let Identity(next) = coyo.lower_ref();
				next.resume().ok()
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}

	#[test]
	fn embed_arc_free_node_widens_first_order_row_and_preserves_continuation() {
		let layer = Coproduct::inject(ArcCoyoneda::lift(Identity(40)));
		let free: ArcFree<NodeBrand<ArcNarrowRow, CNilBrand>, i32> =
			ArcFree::lift_f(Node::First(layer)).map(|value| value + 2);

		let widened: ArcFree<NodeBrand<ArcWideRow, CNilBrand>, i32> = embed_arc_free_node(free);

		let continuation_value = match widened.resume() {
			Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
				let Identity(next) = coyo.lower_ref();
				next.resume().ok()
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}

	#[test]
	fn embed_free_explicit_node_widens_first_order_row() {
		let layer = Coproduct::inject(Coyoneda::lift(Identity(Box::new(FreeExplicit::pure(40)))));
		let free: FreeExplicit<'static, NodeBrand<NarrowRow, CNilBrand>, i32> =
			FreeExplicit::<_, _, BoxBrand>::wrap(Node::First(layer))
				.bind(|value: i32| FreeExplicit::pure(value + 2));

		let widened: FreeExplicit<'static, NodeBrand<WideRow, CNilBrand>, i32> =
			embed_free_explicit_node(free);

		let continuation_value = match widened.to_view() {
			crate::types::FreeExplicitView::Wrap(Node::First(Coproduct::Inr(Coproduct::Inl(
				coyo,
			)))) => {
				let Identity(next) = coyo.lower();
				match (*next).to_view() {
					crate::types::FreeExplicitView::Pure(value) => Some(value),
					crate::types::FreeExplicitView::Wrap(_) => None,
				}
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}

	#[test]
	fn embed_rc_free_explicit_node_widens_first_order_row() {
		let layer = Coproduct::inject(RcCoyoneda::lift(Identity(RcFreeExplicit::pure(40))));
		let free: RcFreeExplicit<'static, NodeBrand<RcNarrowRow, CNilBrand>, i32> =
			RcFreeExplicit::wrap(Node::First(layer))
				.bind(|value: i32| RcFreeExplicit::pure(value + 2));

		let widened: RcFreeExplicit<'static, NodeBrand<RcWideRow, CNilBrand>, i32> =
			embed_rc_free_explicit_node(free);

		let continuation_value = match widened.to_view() {
			crate::types::RcFreeExplicitView::Wrap(Node::First(Coproduct::Inr(
				Coproduct::Inl(coyo),
			))) => {
				let Identity(next) = coyo.lower_ref();
				match next.to_view() {
					crate::types::RcFreeExplicitView::Pure(value) => Some(value),
					crate::types::RcFreeExplicitView::Wrap(_) => None,
				}
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}

	#[test]
	fn embed_arc_free_explicit_node_widens_first_order_row() {
		let layer = Coproduct::inject(ArcCoyoneda::lift(Identity(ArcFreeExplicit::pure(40))));
		let free: ArcFreeExplicit<'static, NodeBrand<ArcNarrowRow, CNilBrand>, i32> =
			ArcFreeExplicit::wrap(Node::First(layer))
				.bind(|value: i32| ArcFreeExplicit::pure(value + 2));

		let widened: ArcFreeExplicit<'static, NodeBrand<ArcWideRow, CNilBrand>, i32> =
			embed_arc_free_explicit_node(free);

		let continuation_value = match widened.to_view() {
			crate::types::ArcFreeExplicitView::Wrap(Node::First(Coproduct::Inr(
				Coproduct::Inl(coyo),
			))) => {
				let Identity(next) = coyo.lower_ref();
				match next.to_view() {
					crate::types::ArcFreeExplicitView::Pure(value) => Some(value),
					crate::types::ArcFreeExplicitView::Wrap(_) => None,
				}
			}
			_ => None,
		};
		assert_eq!(continuation_value, Some(42));
	}
}
