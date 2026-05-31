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
				WrapDrop,
			},
			kinds::*,
			types::{
				CatList,
				Free,
				effects::{
					coproduct::CoproductEmbedder,
					node::Node,
				},
				free::{
					Continuation,
					TypeErasedValue,
				},
			},
		},
		fp_macros::*,
	};

	#[doc(hidden)]
	/// Raw Free branch carried by continuation-aware row embedding.
	type RawNodeFree<R, S> = Free<NodeBrand<R, S>, TypeErasedValue>;

	#[doc(hidden)]
	/// Continuation queue carried by continuation-aware row embedding.
	type NodeContinuations<R, S> = CatList<Continuation<NodeBrand<R, S>>>;

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
			embed_continuations::<R, S, R2, S2, REmbedIdx, SEmbedIdx>,
		)
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
	fn embed_continuations<R, S, R2, S2, REmbedIdx, SEmbedIdx>(
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
}

#[cfg(test)]
mod tests {
	use {
		super::inner::embed_free_node,
		crate::{
			brands::{
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
				IdentityBrand,
				NodeBrand,
				OptionBrand,
			},
			types::{
				Coyoneda,
				Free,
				Identity,
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

	#[test]
	fn embed_free_node_widens_first_order_row_and_preserves_continuation() {
		let layer = Coproduct::inject(Coyoneda::lift(Identity(40)));
		let free: Free<NodeBrand<NarrowRow, CNilBrand>, i32> =
			Free::lift_f(Node::First(layer)).map(|value| value + 2);

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
			Free::lift_f(Node::Scoped(layer)).map(|value| value * 6);

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
}
