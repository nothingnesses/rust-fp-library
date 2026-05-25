#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::super::prelude::*;

	/// Same-row Writer `Tell` transformer used by pre-applying `censor`.
	pub(crate) struct BoxWriterPreRewriter<'a, W>
	where
		W: 'a, {
		pub(crate) censor: Box<dyn 'a + Fn(W) -> W>,
	}

	/// Same-row Writer `Tell` transformer used by Rc pre-applying `censor`.
	pub(crate) struct RcWriterPreRewriter<'a, W>
	where
		W: 'a, {
		pub(crate) censor: Rc<dyn 'a + Fn(W) -> W>,
	}

	/// Same-row Writer `Tell` transformer used by Arc pre-applying `censor`.
	pub(crate) struct ArcWriterPreRewriter<'a, W>
	where
		W: 'a, {
		pub(crate) censor: Arc<dyn 'a + Fn(W) -> W + Send + Sync>,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer pre-censor rewriter.")]
	impl<R, S, W> RunFirstOrderRewriter<WriterBrand<W>, R, S> for BoxWriterPreRewriter<'static, W>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: 'static,
	{
		/// Rewrites one default-wrapper Writer operation by applying the stored censor to its log.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns(
			"The Writer operation with its log transformed and next program preserved."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn rewrite<T: 'static>(
			&self,
			effect: Writer<'static, W, Run<R, S, T>>,
		) -> Writer<'static, W, Run<R, S, T>> {
			match effect {
				Writer::Tell(log, next, marker) => Writer::Tell((self.censor)(log), next, marker),
			}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer pre-censor rewriter.")]
	impl<R, S, W> RcRunFirstOrderRewriter<WriterBrand<W>, R, S> for RcWriterPreRewriter<'static, W>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: 'static,
	{
		/// Rewrites one Rc-wrapper Writer operation by applying the stored censor to its log.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns(
			"The Writer operation with its log transformed and next program preserved."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn rewrite<T: Clone + 'static>(
			&self,
			effect: Writer<'static, W, RcRun<R, S, T>>,
		) -> Writer<'static, W, RcRun<R, S, T>> {
			match effect {
				Writer::Tell(log, next, marker) => Writer::Tell((self.censor)(log), next, marker),
			}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer pre-censor rewriter.")]
	impl<R, S, W> ArcRunFirstOrderRewriter<WriterBrand<W>, R, S> for ArcWriterPreRewriter<'static, W>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		W: Send + Sync + 'static,
	{
		/// Rewrites one Arc-wrapper Writer operation by applying the stored censor to its log.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns(
			"The Writer operation with its log transformed and next program preserved."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn rewrite<T: Clone + Send + Sync + 'static>(
			&self,
			effect: Writer<'static, W, ArcRun<R, S, T>>,
		) -> Writer<'static, W, ArcRun<R, S, T>> {
			match effect {
				Writer::Tell(log, next, marker) => Writer::Tell((self.censor)(log), next, marker),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer pre-censor rewriter.")]
	impl<'a, R, S, W> RunExplicitFirstOrderRewriter<'a, WriterBrand<W>, R, S>
		for BoxWriterPreRewriter<'a, W>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: 'static,
	{
		/// Rewrites one single-shot Explicit Writer operation by applying the stored censor to its log.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns(
			"The Writer operation with its log transformed and next program preserved."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn rewrite<T: 'a>(
			&self,
			effect: Writer<'a, W, RunExplicit<'a, R, S, T>>,
		) -> Writer<'a, W, RunExplicit<'a, R, S, T>> {
			match effect {
				Writer::Tell(log, next, marker) => Writer::Tell((self.censor)(log), next, marker),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer pre-censor rewriter.")]
	impl<'a, R, S, W> RcRunExplicitFirstOrderRewriter<'a, WriterBrand<W>, R, S>
		for RcWriterPreRewriter<'a, W>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: 'static,
	{
		/// Rewrites one Rc Explicit Writer operation by applying the stored censor to its log.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns(
			"The Writer operation with its log transformed and next program preserved."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn rewrite<T: Clone + 'a>(
			&self,
			effect: Writer<'a, W, RcRunExplicit<'a, R, S, T>>,
		) -> Writer<'a, W, RcRunExplicit<'a, R, S, T>> {
			match effect {
				Writer::Tell(log, next, marker) => Writer::Tell((self.censor)(log), next, marker),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer pre-censor rewriter.")]
	impl<'a, R, S, W> ArcRunExplicitFirstOrderRewriter<'a, WriterBrand<W>, R, S>
		for ArcWriterPreRewriter<'a, W>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		W: Send + Sync + 'static,
	{
		/// Rewrites one Arc Explicit Writer operation by applying the stored censor to its log.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns(
			"The Writer operation with its log transformed and next program preserved."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// let censor = |log: &'static str| if log == "inner" { "censored" } else { log };
		/// assert_eq!(censor("inner"), "censored");
		/// ```
		fn rewrite<T: Clone + Send + Sync + 'a>(
			&self,
			effect: Writer<'a, W, ArcRunExplicit<'a, R, S, T>>,
		) -> Writer<'a, W, ArcRunExplicit<'a, R, S, T>> {
			match effect {
				Writer::Tell(log, next, marker) => Writer::Tell((self.censor)(log), next, marker),
			}
		}
	}
}

pub(crate) use inner::*;
