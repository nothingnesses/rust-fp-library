#[allow(
	unused_imports,
	reason = "Each scoped-handler child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// Standard Writer handler that transforms each selected action log before accumulation.
	///
	/// `WriterPreHandler` carries only type-level row witnesses. The handler
	/// semantics are intentionally named: it applies `censor` before the
	/// selected action's `Tell` values reach the surrounding Writer handler.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct WriterPreHandler<Idx, RMinusWriter, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusWriter, EmbedIndices)>,
	);

	/// Standard Writer handler that accumulates a selected action log before transforming it.
	///
	/// `WriterPostHandler` carries only type-level row witnesses. The handler
	/// semantics are intentionally named: it observes the selected action's
	/// `Tell` values, accumulates them, and then applies `censor` to the
	/// aggregate before re-emitting it.
	#[derive(Clone, Copy, Debug, Default)]
	#[expect(
		clippy::type_complexity,
		reason = "The fn marker carries row-witness type parameters without making auto-traits depend on them."
	)]
	pub struct WriterPostHandler<Idx, RMinusWriter, EmbedIndices>(
		PhantomData<fn() -> (Idx, RMinusWriter, EmbedIndices)>,
	);

	/// Constructs a [`WriterPreHandler`] without naming its private field.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::effects::standard_scoped_handlers::{
	/// 		WriterPreHandler,
	/// 		writer_pre_handler,
	/// 	},
	/// };
	///
	/// fn accepts_pre_handler<Idx, RMinusWriter, EmbedIndices>(
	/// 	_handler: WriterPreHandler<Idx, RMinusWriter, EmbedIndices>
	/// ) -> &'static str {
	/// 	"writer pre handler"
	/// }
	///
	/// let handler = writer_pre_handler::<(), CNilBrand, ()>();
	/// assert_eq!(accepts_pre_handler(handler), "writer pre handler");
	/// ```
	pub const fn writer_pre_handler<Idx, RMinusWriter, EmbedIndices>()
	-> WriterPreHandler<Idx, RMinusWriter, EmbedIndices> {
		WriterPreHandler(PhantomData)
	}

	/// Constructs a [`WriterPostHandler`] without naming its private field.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::effects::standard_scoped_handlers::{
	/// 		WriterPostHandler,
	/// 		writer_post_handler,
	/// 	},
	/// };
	///
	/// fn accepts_post_handler<Idx, RMinusWriter, EmbedIndices>(
	/// 	_handler: WriterPostHandler<Idx, RMinusWriter, EmbedIndices>
	/// ) -> &'static str {
	/// 	"writer post handler"
	/// }
	///
	/// let handler = writer_post_handler::<(), CNilBrand, ()>();
	/// assert_eq!(accepts_post_handler(handler), "writer post handler");
	/// ```
	pub const fn writer_post_handler<Idx, RMinusWriter, EmbedIndices>()
	-> WriterPostHandler<Idx, RMinusWriter, EmbedIndices> {
		WriterPostHandler(PhantomData)
	}
}

pub use inner::*;
