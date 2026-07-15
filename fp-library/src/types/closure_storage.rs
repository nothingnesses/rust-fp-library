//! FS-1 substrate: the unified closure storage.
//!
//! This module lives in core `crate::types` (not under the
//! `effects`-feature-gated subsystem) because the core `Free<F, A, Store>`
//! carries `Store: ClosureStorage` in its definition and must compile with
//! the `effects` feature off; [`ClosureStorage`] has no effects-specific
//! dependencies (it impls only for the core store brands
//! `BoxBrand`/`RcBrand`/`ArcBrand`). `ClosureStorage` is `pub` because the
//! public `Free<F, A, Store>` carries the bound in its (effective-public)
//! signature, so a `pub(crate)` bound would trip the `private_bounds` lint;
//! this mirrors `LazyConfig`, the analogous public store-axis trait for
//! `Lazy`. Users do not name `Store` directly (they use the `Free<F, A>`
//! Box default).
//!
//! [`ClosureStorage`] unifies the per-pointer closure storage. One
//! associated `Stored` type carries the callable *kind* per store, a
//! one-shot `Box<dyn FnOnce>` for `BoxBrand`, a reusable `Rc<dyn Fn>` for
//! `RcBrand`, and a thread-safe `Arc<dyn Fn + Send + Sync>` for `ArcBrand`,
//! and a single by-value [`call_once`](ClosureStorage::call_once) bridges
//! all three: it consumes the `FnOnce` for Box (running it once) and
//! borrows the `Fn` through the owned pointer for Rc/Arc (which the
//! multi-shot path clones first). A companion
//! [`from_fn`](ClosureStorage::from_fn) constructs a stored callable from a
//! capture-free (`Fn + Send + Sync`) body for any store, which the
//! interpreter core uses for the downcast/unbox continuations it builds
//! inline (those capture nothing, so the strict bound is harmless). This is
//! the substrate's continuation-storage axis; the row-cell pointer axis (a
//! sibling storage for `Coyoneda`) is
//! [`CoyoStore`](crate::types::coyo_store::CoyoStore).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::BoxBrand,
//! 	types::closure_storage::ClosureStorage,
//! };
//!
//! let stored = <BoxBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
//! assert_eq!(<BoxBrand as ClosureStorage>::call_once(stored, 41), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::{
				ArcBrand,
				BoxBrand,
				RcBrand,
			},
			types::{
				arc_cat_list::ArcCatList,
				cat_list::CatList,
				rc_cat_list::RcCatList,
			},
		},
		fp_macros::*,
		std::{
			any::Any,
			rc::Rc,
			sync::Arc,
		},
	};

	/// One closure-storage class over the pointer stores. `Stored<'a, I, O>`
	/// is the stored callable for a store, carrying its callable kind
	/// (`FnOnce` for Box, `Fn` for Rc/Arc), and
	/// [`call_once`](ClosureStorage::call_once) invokes it by value so one
	/// signature serves both kinds.
	pub trait ClosureStorage: 'static {
		/// The stored callable from `I` to `O` for this store.
		type Stored<'a, I: 'a, O: 'a>: 'a;

		/// The type-erased value for this store: `Box<dyn Any>` for the
		/// one-shot Box spine, `Rc<dyn Any>` / `Arc<dyn Any + Send + Sync>`
		/// for the multi-shot Rc/Arc spines. Multi-shot effects re-invoke a
		/// continuation, so the erased value lives behind a shareable
		/// pointer and an owned value is recovered per call (cloning the
		/// cell when shared); the one-shot Box value is moved once.
		type Erased: 'static;

		/// The continuation queue for this store: the by-value
		/// [`CatList`](crate::types::CatList) for the one-shot Box spine,
		/// and the refcounted [`RcCatList`](crate::types::RcCatList) /
		/// [`ArcCatList`](crate::types::ArcCatList) (O(1) `Clone`) for the
		/// multi-shot Rc/Arc spines, which clone the queue per branch.
		/// Bounded only by `Default` (the empty queue), which is all the
		/// construction-free accessors (`mem::take`) and the iterative
		/// `Drop` need, so it serves a one-shot store whose element is a
		/// non-`Clone` `FnOnce` as well as the multi-shot stores; the
		/// catenable-queue operations the multi-shot stepping needs are
		/// required of the queue at that use site, where the element is
		/// always a `Clone` `Rc`/`Arc` continuation.
		type Queue<C>: Default;

		/// Invokes the stored callable once, consuming it. For Box this
		/// consumes the owned `FnOnce`; for Rc/Arc it borrows the `Fn`
		/// through the owned pointer, which then drops (the multi-shot path
		/// clones the pointer beforehand).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The stored callable to invoke.", "The input to pass to it.")]
		#[document_returns("The callable's output.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::closure_storage::ClosureStorage,
		/// };
		///
		/// let stored: <BoxBrand as ClosureStorage>::Stored<'static, i32, i32> = Box::new(|x| x + 1);
		/// assert_eq!(<BoxBrand as ClosureStorage>::call_once(stored, 41), 42);
		/// ```
		fn call_once<'a, I: 'a, O: 'a>(
			stored: Self::Stored<'a, I, O>,
			input: I,
		) -> O;

		/// Stores a capture-free (or capture-`Send + Sync`) closure for any
		/// store.
		///
		/// The bound is the strictest of the three stores
		/// (`Fn + Send + Sync`), which is sound for every impl: Box stores
		/// it as `Box<dyn FnOnce>`, Rc as `Rc<dyn Fn>`, and Arc as
		/// `Arc<dyn Fn + Send + Sync>`. The interpreter core uses this for
		/// the downcast/unbox continuations it builds inline, which capture
		/// nothing, so the strict bound is harmless. Continuations that move
		/// a non-`Send` capture (a user `FnOnce` in `bind`/`map`) are
		/// constructed per-store instead, not through this bridge.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The closure to store.")]
		#[document_returns("The stored callable for this store.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::closure_storage::ClosureStorage,
		/// };
		///
		/// let stored = <RcBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		/// assert_eq!(<RcBrand as ClosureStorage>::call_once(stored, 41), 42);
		/// ```
		fn from_fn<'a, I: 'a, O: 'a>(
			f: impl Fn(I) -> O + Send + Sync + 'a
		) -> Self::Stored<'a, I, O>;
	}

	impl ClosureStorage for BoxBrand {
		type Erased = Box<dyn Any>;
		type Queue<C> = CatList<C>;
		type Stored<'a, I: 'a, O: 'a> = Box<dyn FnOnce(I) -> O + 'a>;

		/// Consumes the owned `FnOnce`, running it once.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The stored callable to invoke.", "The input to pass to it.")]
		#[document_returns("The callable's output.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::closure_storage::ClosureStorage,
		/// };
		///
		/// // The Box store holds a genuine `FnOnce`: the closure may move a
		/// // non-`Copy` capture out when it runs.
		/// let captured = String::from("hi");
		/// let stored: <BoxBrand as ClosureStorage>::Stored<'static, (), usize> =
		/// 	Box::new(move |()| captured.into_bytes().len());
		/// assert_eq!(<BoxBrand as ClosureStorage>::call_once(stored, ()), 2);
		/// ```
		fn call_once<'a, I: 'a, O: 'a>(
			stored: Self::Stored<'a, I, O>,
			input: I,
		) -> O {
			stored(input)
		}

		/// Stores the closure as a one-shot `Box<dyn FnOnce>`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The closure to store.")]
		#[document_returns("The boxed one-shot callable.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::closure_storage::ClosureStorage,
		/// };
		///
		/// let stored = <BoxBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		/// assert_eq!(<BoxBrand as ClosureStorage>::call_once(stored, 41), 42);
		/// ```
		fn from_fn<'a, I: 'a, O: 'a>(
			f: impl Fn(I) -> O + Send + Sync + 'a
		) -> Self::Stored<'a, I, O> {
			Box::new(f)
		}
	}

	impl ClosureStorage for RcBrand {
		type Erased = Rc<dyn Any>;
		type Queue<C> = RcCatList<C>;
		type Stored<'a, I: 'a, O: 'a> = Rc<dyn Fn(I) -> O + 'a>;

		/// Borrows the `Fn` through the owned `Rc`, which then drops; the
		/// multi-shot path clones the pointer beforehand to re-invoke.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The stored callable to invoke.", "The input to pass to it.")]
		#[document_returns("The callable's output.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::RcBrand,
		/// 		types::closure_storage::ClosureStorage,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// // The Rc store is multi-shot: clone the stored `Fn`, then invoke
		/// // each clone independently.
		/// let stored: <RcBrand as ClosureStorage>::Stored<'static, i32, i32> = Rc::new(|x| x + 1);
		/// let first = <RcBrand as ClosureStorage>::call_once(Rc::clone(&stored), 10);
		/// let second = <RcBrand as ClosureStorage>::call_once(stored, 20);
		/// assert_eq!((first, second), (11, 21));
		/// ```
		fn call_once<'a, I: 'a, O: 'a>(
			stored: Self::Stored<'a, I, O>,
			input: I,
		) -> O {
			(*stored)(input)
		}

		/// Stores the closure as a re-callable `Rc<dyn Fn>`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The closure to store.")]
		#[document_returns("The Rc-wrapped re-callable callable.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::closure_storage::ClosureStorage,
		/// };
		///
		/// let stored = <RcBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		/// assert_eq!(<RcBrand as ClosureStorage>::call_once(stored, 41), 42);
		/// ```
		fn from_fn<'a, I: 'a, O: 'a>(
			f: impl Fn(I) -> O + Send + Sync + 'a
		) -> Self::Stored<'a, I, O> {
			Rc::new(f)
		}
	}

	impl ClosureStorage for ArcBrand {
		type Erased = Arc<dyn Any + Send + Sync>;
		type Queue<C> = ArcCatList<C>;
		type Stored<'a, I: 'a, O: 'a> = Arc<dyn Fn(I) -> O + Send + Sync + 'a>;

		/// Borrows the `Fn` through the owned `Arc`, which then drops; the
		/// multi-shot path clones the pointer beforehand to re-invoke.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The stored callable to invoke.", "The input to pass to it.")]
		#[document_returns("The callable's output.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::ArcBrand,
		/// 		types::closure_storage::ClosureStorage,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let stored: <ArcBrand as ClosureStorage>::Stored<'static, i32, i32> = Arc::new(|x| x * 2);
		/// assert_eq!(<ArcBrand as ClosureStorage>::call_once(stored, 21), 42);
		/// ```
		fn call_once<'a, I: 'a, O: 'a>(
			stored: Self::Stored<'a, I, O>,
			input: I,
		) -> O {
			(*stored)(input)
		}

		/// Stores the closure as a thread-safe
		/// `Arc<dyn Fn + Send + Sync>`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the callable.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The closure to store.")]
		#[document_returns("The Arc-wrapped thread-safe callable.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ArcBrand,
		/// 	types::closure_storage::ClosureStorage,
		/// };
		///
		/// let stored = <ArcBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		/// assert_eq!(<ArcBrand as ClosureStorage>::call_once(stored, 41), 42);
		/// ```
		fn from_fn<'a, I: 'a, O: 'a>(
			f: impl Fn(I) -> O + Send + Sync + 'a
		) -> Self::Stored<'a, I, O> {
			Arc::new(f)
		}
	}

	/// The multi-shot stores. Their continuations are re-callable `Fn`
	/// pointers and their erased values are shareable, so a suspended
	/// program can be re-run, the property `Choose`/`Amb` need. `BoxBrand`
	/// is one-shot and is deliberately not a member; it has its own arm. The
	/// unified multi-shot interpreter is generic over this trait, so one
	/// body serves both `Rc` and `Arc`.
	pub trait MultiShotStore: ClosureStorage {}

	impl MultiShotStore for RcBrand {}

	impl MultiShotStore for ArcBrand {}

	/// Carries a store's per-value bound, plus the erase/recover operations
	/// over the store's erased cell.
	///
	/// The bound lives on the impl, not as a supertrait: `'static` (no
	/// `Clone`) for the one-shot `Box` store, `Clone + 'static` for `Rc`,
	/// and `Clone + Send + Sync + 'static` for `Arc`. So generic code
	/// bounded by `A: ValueFor<S>` gets the right per-store bound through
	/// the impl, and a single `Free::pure` (and the other value-erasing
	/// constructors) serves every store rather than colliding as separate
	/// per-store associated functions. The operations live here rather than
	/// as generic methods on the store because a generic `fn erase<A>(..)`
	/// on the store could not see the impl's `Send + Sync` (or `Clone`)
	/// bound: a bound `A: ValueFor<ArcBrand>` hands generic code only the
	/// trait's supertraits, not the impl's bounds; placing them here lets
	/// each impl body see its own bound.
	#[document_type_parameters("The store brand whose erased cell is targeted.")]
	#[document_parameters("The value to erase.")]
	pub trait ValueFor<S: ClosureStorage>: 'static {
		/// Erases a value into the store's erased cell (`Box::new` /
		/// `Rc::new` / `Arc::new`).
		#[document_signature]
		#[document_parameters]
		#[document_returns("The store's erased cell holding the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// let erased = <i32 as ValueFor<BoxBrand>>::erase(6);
		/// assert_eq!(<i32 as ValueFor<BoxBrand>>::recover(erased), 6);
		/// ```
		fn erase(self) -> S::Erased;

		/// Recovers an owned value from the erased cell: for the one-shot
		/// `Box` store move it out; for the multi-shot `Rc`/`Arc` stores
		/// move out when uniquely owned and clone when the cell is shared.
		///
		/// # Panics
		///
		/// Panics if the erased cell does not hold a value of this type; the
		/// pairing is maintained by the substrate's construction invariant,
		/// so a mismatch is an internal bug.
		#[document_signature]
		#[document_parameters("The erased cell to recover from.")]
		#[document_returns("The recovered owned value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// let erased = <i32 as ValueFor<RcBrand>>::erase(7);
		/// assert_eq!(<i32 as ValueFor<RcBrand>>::recover(erased), 7);
		/// ```
		fn recover(erased: S::Erased) -> Self;
	}

	#[document_type_parameters("The type of the value.")]
	#[document_parameters("The value to erase.")]
	impl<A: 'static> ValueFor<BoxBrand> for A {
		/// Erases by boxing; the one-shot store requires no `Clone`, so a
		/// non-`Clone` value is accepted.
		#[document_signature]
		#[document_parameters]
		#[document_returns("The boxed erased cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// // No `Clone` bound: a non-`Clone` value round-trips.
		/// struct NotClone(i32);
		/// let erased = <NotClone as ValueFor<BoxBrand>>::erase(NotClone(5));
		/// let recovered: NotClone = <NotClone as ValueFor<BoxBrand>>::recover(erased);
		/// assert_eq!(recovered.0, 5);
		/// ```
		fn erase(self) -> <BoxBrand as ClosureStorage>::Erased {
			Box::new(self)
		}

		/// Recovers by moving the value out of the box.
		#[document_signature]
		#[document_parameters("The erased cell to recover from.")]
		#[document_returns("The recovered owned value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// let erased = <i32 as ValueFor<BoxBrand>>::erase(6);
		/// assert_eq!(<i32 as ValueFor<BoxBrand>>::recover(erased), 6);
		/// ```
		fn recover(erased: <BoxBrand as ClosureStorage>::Erased) -> Self {
			match erased.downcast::<A>() {
				Ok(boxed) => *boxed,
				Err(_) => value_type_invariant(),
			}
		}
	}

	#[document_type_parameters("The type of the value.")]
	#[document_parameters("The value to erase.")]
	impl<A: Clone + 'static> ValueFor<RcBrand> for A {
		/// Erases into an `Rc` cell; the multi-shot store requires `Clone`
		/// so a shared cell can hand out owned values, but not `Send`, which
		/// is what distinguishes it from the Arc store.
		#[document_signature]
		#[document_parameters]
		#[document_returns("The Rc-wrapped erased cell.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::RcBrand,
		/// 		types::closure_storage::ValueFor,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// // The Rc store accepts a non-`Send` value, which the Arc store's
		/// // bound rejects at compile time.
		/// let erased = <Rc<i32> as ValueFor<RcBrand>>::erase(Rc::new(5));
		/// let recovered: Rc<i32> = <Rc<i32> as ValueFor<RcBrand>>::recover(erased);
		/// assert_eq!(*recovered, 5);
		/// ```
		fn erase(self) -> <RcBrand as ClosureStorage>::Erased {
			Rc::new(self)
		}

		/// Recovers by moving out when the cell is uniquely owned and
		/// cloning when it is shared.
		#[document_signature]
		#[document_parameters("The erased cell to recover from.")]
		#[document_returns("The recovered owned value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// let erased = <i32 as ValueFor<RcBrand>>::erase(7);
		/// assert_eq!(<i32 as ValueFor<RcBrand>>::recover(erased), 7);
		/// ```
		fn recover(erased: <RcBrand as ClosureStorage>::Erased) -> Self {
			match erased.downcast::<A>() {
				Ok(rc) => Rc::try_unwrap(rc).unwrap_or_else(|shared| (*shared).clone()),
				Err(_) => value_type_invariant(),
			}
		}
	}

	#[document_type_parameters("The type of the value.")]
	#[document_parameters("The value to erase.")]
	impl<A: Clone + Send + Sync + 'static> ValueFor<ArcBrand> for A {
		/// Erases into an `Arc` cell; the thread-safe store's bound adds
		/// `Send + Sync` on top of the multi-shot `Clone`.
		#[document_signature]
		#[document_parameters]
		#[document_returns("The Arc-wrapped erased cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ArcBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// let erased = <i32 as ValueFor<ArcBrand>>::erase(8);
		/// assert_eq!(<i32 as ValueFor<ArcBrand>>::recover(erased), 8);
		/// ```
		fn erase(self) -> <ArcBrand as ClosureStorage>::Erased {
			Arc::new(self)
		}

		/// Recovers by moving out when the cell is uniquely owned and
		/// cloning when it is shared.
		#[document_signature]
		#[document_parameters("The erased cell to recover from.")]
		#[document_returns("The recovered owned value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ArcBrand,
		/// 	types::closure_storage::ValueFor,
		/// };
		///
		/// let erased = <i32 as ValueFor<ArcBrand>>::erase(8);
		/// assert_eq!(<i32 as ValueFor<ArcBrand>>::recover(erased), 8);
		/// ```
		fn recover(erased: <ArcBrand as ClosureStorage>::Erased) -> Self {
			match erased.downcast::<A>() {
				Ok(arc) => Arc::try_unwrap(arc).unwrap_or_else(|shared| (*shared).clone()),
				Err(_) => value_type_invariant(),
			}
		}
	}

	/// Diverges on the erase/recover pairing invariant: the erased value's
	/// concrete type is maintained by the substrate's construction
	/// invariant, so a downcast mismatch is an internal bug, not a
	/// recoverable condition.
	#[document_signature]
	#[document_type_parameters("The type the caller expected to recover.")]
	#[document_returns("Never returns; this function always panics.")]
	#[document_examples(
		skip_call_check,
		reason = "Direct-call validation is skipped because value_type_invariant is a private divergence helper; the public ValueFor::recover reaches it when the pairing invariant is violated, as the example demonstrates."
	)]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::BoxBrand,
	/// 		types::closure_storage::ValueFor,
	/// 	},
	/// 	std::panic::catch_unwind,
	/// };
	///
	/// // Recovering at the wrong type violates the pairing invariant and
	/// // panics.
	/// let outcome = catch_unwind(|| {
	/// 	let erased = <String as ValueFor<BoxBrand>>::erase(String::from("mismatch"));
	/// 	let _: i32 = <i32 as ValueFor<BoxBrand>>::recover(erased);
	/// });
	/// assert!(outcome.is_err());
	/// ```
	fn value_type_invariant<A>() -> A {
		#[expect(
			clippy::panic,
			reason = "the erased value's concrete type is maintained by the substrate's construction invariant; a mismatch is an internal bug, not a recoverable condition."
		)]
		{
			panic!("Type mismatch recovering an erased value (substrate invariant violated)")
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::brands::{
			ArcBrand,
			BoxBrand,
			RcBrand,
		},
		std::{
			rc::Rc,
			sync::Arc,
		},
	};

	fn assert_send_sync<T: Send + Sync>() {}

	// The Box store holds a genuine `FnOnce`: the closure moves a non-`Copy`
	// capture out when it runs, which an `Fn`-only store could not hold.
	#[test]
	fn box_store_holds_a_genuine_fnonce() {
		let captured = String::from("hi");
		let stored: <BoxBrand as ClosureStorage>::Stored<'static, (), usize> =
			Box::new(move |()| captured.into_bytes().len());
		assert_eq!(<BoxBrand as ClosureStorage>::call_once(stored, ()), 2);
	}

	// The Rc store is multi-shot: clone the stored `Fn`, then `call_once` each
	// clone independently.
	#[test]
	fn rc_store_is_multi_shot_via_clone() {
		let stored: <RcBrand as ClosureStorage>::Stored<'static, i32, i32> = Rc::new(|x| x + 1);
		let first = <RcBrand as ClosureStorage>::call_once(Rc::clone(&stored), 10);
		let second = <RcBrand as ClosureStorage>::call_once(stored, 20);
		assert_eq!((first, second), (11, 21));
	}

	// The Arc store is statically `Send + Sync` and invokes correctly.
	#[test]
	fn arc_store_is_send_sync_and_invokes() {
		assert_send_sync::<<ArcBrand as ClosureStorage>::Stored<'static, i32, i32>>();
		let stored: <ArcBrand as ClosureStorage>::Stored<'static, i32, i32> = Arc::new(|x| x * 2);
		assert_eq!(<ArcBrand as ClosureStorage>::call_once(stored, 21), 42);
	}

	// `from_fn` builds the same capture-free continuation for every store. This
	// is how the interpreter core's downcast/unbox continuations stay generic.
	#[test]
	fn from_fn_builds_a_capture_free_continuation_for_every_store() {
		let kb = <BoxBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		assert_eq!(<BoxBrand as ClosureStorage>::call_once(kb, 41), 42);

		let kr = <RcBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		assert_eq!(<RcBrand as ClosureStorage>::call_once(kr, 41), 42);

		let ka = <ArcBrand as ClosureStorage>::from_fn(|x: i32| x + 1);
		assert_eq!(<ArcBrand as ClosureStorage>::call_once(ka, 41), 42);
	}

	// A continuation built by the Arc store's `from_fn` is statically
	// `Send + Sync`, so an Arc-store stepped value carrying it stays `Send + Sync`.
	#[test]
	fn arc_from_fn_continuation_is_send_sync() {
		let ka = <ArcBrand as ClosureStorage>::from_fn(|x: i32| x * 2);
		assert_send_sync::<<ArcBrand as ClosureStorage>::Stored<'static, i32, i32>>();
		assert_eq!(<ArcBrand as ClosureStorage>::call_once(ka, 21), 42);
	}

	// `ValueFor` erase/recover round-trips through the store's cell for every
	// store; one bound (`A: ValueFor<S>`) serves all three, which is what lets a
	// single `Free::pure` construct over Box, Rc, and Arc.
	#[test]
	fn value_for_round_trips_box_rc_and_arc() {
		assert_eq!(<i32 as ValueFor<BoxBrand>>::recover(<i32 as ValueFor<BoxBrand>>::erase(6)), 6);
		assert_eq!(<i32 as ValueFor<RcBrand>>::recover(<i32 as ValueFor<RcBrand>>::erase(7)), 7);
		assert_eq!(<i32 as ValueFor<ArcBrand>>::recover(<i32 as ValueFor<ArcBrand>>::erase(8)), 8);
	}

	// The one-shot `Box` store's `ValueFor` requires no `Clone` (it moves the
	// value out), unlike the multi-shot stores; a non-`Clone` value round-trips.
	#[test]
	fn box_value_for_accepts_non_clone() {
		struct NotClone(i32);
		let recovered: NotClone = <NotClone as ValueFor<BoxBrand>>::recover(
			<NotClone as ValueFor<BoxBrand>>::erase(NotClone(5)),
		);
		assert_eq!(recovered.0, 5);
	}

	// The Arc store's erased cell is statically `Send + Sync`, so a structure
	// holding it is `Send + Sync` by inference (once its queue is also Arc-backed).
	#[test]
	fn arc_value_for_erased_is_send_sync() {
		assert_send_sync::<<ArcBrand as ClosureStorage>::Erased>();
	}

	// The Rc store accepts a non-`Send` value (`Rc<i32>`), which the Arc store's
	// bound rejects at compile time; the reason both multi-shot stores exist.
	#[test]
	fn rc_value_for_accepts_non_send() {
		let recovered: Rc<i32> = <Rc<i32> as ValueFor<RcBrand>>::recover(<Rc<i32> as ValueFor<
			RcBrand,
		>>::erase(Rc::new(5)));
		assert_eq!(*recovered, 5);
	}
}
