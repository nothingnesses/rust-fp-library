//! Open sum of first-order effect functors, encoded as a nested
//! [`Coproduct`](crate::types::effects::coproduct::Coproduct) row.
//!
//! This module is the Rust counterpart to PureScript's `VariantF` from
//! [`Data.Functor.Variant`](https://github.com/purescript-deprecated/purescript-variant).
//! Where PureScript carries a runtime `Mapper f` dictionary alongside each
//! `VariantFRep` so its `Functor (VariantF r)` impl can dispatch to the
//! active variant's `map`, the Rust port adopts the static option from
//! [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! section 4.2: each effect is wrapped in
//! [`CoyonedaBrand`](crate::brands::CoyonedaBrand) at lift time, the
//! Coproduct row is encoded at the brand level via
//! [`CoproductBrand`](crate::brands::CoproductBrand) /
//! [`CNilBrand`](crate::brands::CNilBrand), and `Functor` dispatches via
//! type-level pattern-matching on the [`Inl`](
//! crate::types::effects::coproduct::Coproduct::Inl) /
//! [`Inr`](crate::types::effects::coproduct::Coproduct::Inr) variants without
//! a runtime dictionary.
//!
//! The conceptual type is also exposed as the alias [`VariantF<H, T>`] so
//! call sites that read the surrounding plan / decisions docs can use the
//! canonical name.
//!
//! ## Effect-row shape
//!
//! The macro `effects![E1, E2, E3]` (Phase 2 step 8) lowers a list of
//! effect types into a `CoproductBrand` chain whose head and tail brands
//! are [`CoyonedaBrand`](crate::brands::CoyonedaBrand)-wrapped, terminated
//! by `CNilBrand`. The recursion implements
//! [`Functor`](crate::classes::Functor) via the impls below:
//! `CoproductBrand<H, T>` requires `H: Functor + 'static, T: Functor + 'static`,
//! and `CNilBrand`'s `map` is the uninhabited `match` base case.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				CoproductBrand,
			},
			classes::{
				Extract,
				Functor,
				RefFunctor,
				SendFunctor,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::effects::coproduct::Coproduct,
		},
		fp_macros::*,
	};

	impl_kind! {
		for CNilBrand {
			type Of<'a, A: 'a>: 'a = crate::types::effects::coproduct::CNil;
		}
	}

	impl_kind! {
		impl<H: Kind_cdc7cd43dac7585f + 'static, T: Kind_cdc7cd43dac7585f + 'static>
			for CoproductBrand<H, T> {
			type Of<'a, A: 'a>: 'a = Coproduct<
				Apply!(<H as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
				Apply!(<T as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			>;
		}
	}

	/// Conceptual alias for the [`CoproductBrand`] chain, matching the
	/// vocabulary used by
	/// [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
	/// section 5.1 (`VariantF<Effects>` = open sum of first-order effect
	/// functors, encoded as a nested Coproduct).
	///
	/// The alias has no behaviour beyond [`CoproductBrand`]; using it at
	/// call sites communicates intent ("this is an effect row") without
	/// introducing a separate brand.
	pub type VariantF<H, T> = CoproductBrand<H, T>;

	impl Functor for CNilBrand {
		/// Functor `map` for the empty row. The argument's type is
		/// `CNil`, which is uninhabited; the body matches it
		/// exhaustively without producing a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply (unused; the empty row carries no value).",
			"The empty-row functor instance (uninhabited)."
		)]
		///
		#[document_returns("Unreachable; the body matches the uninhabited input exhaustively.")]
		///
		#[document_examples]
		///
		/// ```
		/// // CNil is uninhabited, so `CNilBrand::map` is never called
		/// // at runtime. The example demonstrates `CNilBrand` serving
		/// // as the base case of a `CoproductBrand` row whose head
		/// // carries the runtime value, exercising the recursive
		/// // dispatch that bottoms out at `CNilBrand`.
		/// use fp_library::{
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		OptionBrand,
		/// 	},
		/// 	classes::Functor,
		/// 	types::effects::coproduct::Coproduct,
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<OptionBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let value: Row = Coproduct::inject(Some(7));
		/// let mapped = <CoproductBrand<OptionBrand, CNilBrand> as Functor>::map(|x: i32| x * 2, value);
		/// assert!(matches!(mapped, Coproduct::Inl(Some(14))));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			_func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {}
		}
	}

	impl SendFunctor for CNilBrand {
		/// Thread-safe `send_map` for the empty row. The argument's
		/// type is [`CNil`](crate::types::effects::coproduct::CNil),
		/// which is uninhabited; the body matches it exhaustively
		/// without producing a result. Parallel to
		/// [`Functor::map`](crate::classes::Functor::map) for
		/// [`CNilBrand`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor. Must be `Send + Sync`.",
			"The type of the result(s) of applying the function. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to apply (unused; the empty row carries no value). Must be `Send + Sync`.",
			"The empty-row functor instance (uninhabited)."
		)]
		///
		#[document_returns("Unreachable; the body matches the uninhabited input exhaustively.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	classes::SendFunctor,
		/// 	types::effects::coproduct::CNil,
		/// };
		///
		/// // Constructing a `CNil` value is impossible (uninhabited),
		/// // so this example only demonstrates the type signature.
		/// fn _send_map_signature_check<A: Send + Sync + 'static, B: Send + Sync + 'static>(
		/// 	cnil: CNil
		/// ) -> CNil {
		/// 	<CNilBrand as SendFunctor>::send_map::<A, B>(|_| panic!("unreachable"), cnil)
		/// }
		/// assert!(true);
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			_func: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {}
		}
	}

	#[document_type_parameters(
		"The brand of the head effect in the row.",
		"The brand of the tail row (typically another `CoproductBrand` or `CNilBrand`)."
	)]
	impl<H, T> Functor for CoproductBrand<H, T>
	where
		H: Functor + 'static,
		T: Functor + 'static,
	{
		/// Functor `map` for a non-empty effect row. Dispatches to the
		/// head brand `H` if the runtime tag is
		/// [`Inl`](Coproduct::Inl), or recurses into the tail brand `T`
		/// if it is [`Inr`](Coproduct::Inr).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::coproduct::Coproduct,
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<OptionBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let value: Row = Coproduct::inject(Some(10));
		/// let mapped = <CoproductBrand<OptionBrand, CNilBrand> as Functor>::map(|x: i32| x + 1, value);
		/// assert!(matches!(mapped, Coproduct::Inl(Some(11))));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Coproduct::Inl(h) => Coproduct::Inl(<H as Functor>::map(func, h)),
				Coproduct::Inr(t) => Coproduct::Inr(<T as Functor>::map(func, t)),
			}
		}
	}

	#[document_type_parameters(
		"The brand of the head effect in the row.",
		"The brand of the tail row (typically another `CoproductBrand` or `CNilBrand`)."
	)]
	impl<H, T> SendFunctor for CoproductBrand<H, T>
	where
		H: SendFunctor + 'static,
		T: SendFunctor + 'static,
	{
		/// Thread-safe `send_map` for a non-empty effect row.
		/// Dispatches to the head brand `H` if the runtime tag is
		/// [`Inl`](Coproduct::Inl), or recurses into the tail brand
		/// `T` if it is [`Inr`](Coproduct::Inr). Parallel to
		/// [`Functor::map`](crate::classes::Functor::map) for
		/// [`CoproductBrand`], with the closure `Send + Sync`
		/// requirement propagated to both branches.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor. Must be `Send + Sync`.",
			"The type of the result(s) of applying the function. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s). Must be `Send + Sync`.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::SendFunctor,
		/// 	types::effects::coproduct::Coproduct,
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<OptionBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let value: Row = Coproduct::inject(Some(10));
		/// let mapped =
		/// 	<CoproductBrand<OptionBrand, CNilBrand> as SendFunctor>::send_map(|x: i32| x + 1, value);
		/// assert!(matches!(mapped, Coproduct::Inl(Some(11))));
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			func: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Coproduct::Inl(h) => Coproduct::Inl(<H as SendFunctor>::send_map(func, h)),
				Coproduct::Inr(t) => Coproduct::Inr(<T as SendFunctor>::send_map(func, t)),
			}
		}
	}

	impl WrapDrop for CNilBrand {
		/// Drop-time decomposition for the empty effect row. The argument
		/// has type `CNil`, which is uninhabited; the body matches it
		/// exhaustively without producing a result. The outer `Free`
		/// `Drop` never reaches this method on a row whose tail has been
		/// narrowed to `CNilBrand` because no inhabitant of the row
		/// could have selected the empty tail; the impl exists to
		/// satisfy the trait at the type level.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type the empty row would yield."
		)]
		///
		#[document_parameters("The empty-row functor instance (uninhabited).")]
		///
		#[document_returns("Unreachable; the body matches the uninhabited input exhaustively.")]
		///
		#[document_examples]
		///
		/// ```
		/// // CNilBrand's `WrapDrop::drop` is unreachable at runtime: the
		/// // input type `CNil` is uninhabited, so no value can be passed.
		/// // This sketch only exercises the type-level resolution; the
		/// // assertion ties the bound check into a runtime test.
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	classes::WrapDrop,
		/// };
		/// fn requires_wrap_drop<F: WrapDrop>() {}
		/// requires_wrap_drop::<CNilBrand>();
		/// assert!(true);
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {}
		}
	}

	#[document_type_parameters(
		"The brand of the head effect in the row.",
		"The brand of the tail row (typically another `CoproductBrand` or `CNilBrand`)."
	)]
	impl<H, T> WrapDrop for CoproductBrand<H, T>
	where
		H: WrapDrop + 'static,
		T: WrapDrop + 'static,
	{
		/// Drop-time decomposition for a non-empty effect row. Dispatches
		/// to the head brand `H` if the runtime tag is
		/// [`Inl`](Coproduct::Inl), or recurses into the tail brand `T`
		/// if it is [`Inr`](Coproduct::Inr). The recursion bottoms out
		/// at [`CNilBrand`]'s uninhabited base case, so a fully-resolved
		/// row delegates to the active head brand's policy and returns
		/// whatever that brand returns.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type the layer would yield.")]
		///
		#[document_parameters("The Coproduct functor layer.")]
		///
		#[document_returns("The active head or tail brand's `WrapDrop::drop` result for `fa`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::WrapDrop,
		/// 	types::{
		/// 		Identity,
		/// 		effects::coproduct::Coproduct,
		/// 	},
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<IdentityBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let row: Row = Coproduct::inject(Identity(42));
		/// assert_eq!(<CoproductBrand<IdentityBrand, CNilBrand> as WrapDrop>::drop(row), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				Coproduct::Inl(h) => <H as WrapDrop>::drop::<X>(h),
				Coproduct::Inr(t) => <T as WrapDrop>::drop::<X>(t),
			}
		}
	}

	impl RefFunctor for CNilBrand {
		/// `RefFunctor::ref_map` for the empty row. The argument's type
		/// is `CNil`, which is uninhabited; the body matches it
		/// exhaustively without producing a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply by reference (unused; the empty row carries no value).",
			"The empty-row functor instance (uninhabited)."
		)]
		///
		#[document_returns("Unreachable; the body matches the uninhabited input exhaustively.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::RefFunctor,
		/// 	types::{
		/// 		Identity,
		/// 		effects::coproduct::Coproduct,
		/// 	},
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<IdentityBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let value: Row = Coproduct::inject(Identity(7));
		/// let mapped =
		/// 	<CoproductBrand<IdentityBrand, CNilBrand> as RefFunctor>::ref_map(|x: &i32| *x * 2, &value);
		/// assert!(matches!(mapped, Coproduct::Inl(Identity(14))));
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			_func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match *fa {}
		}
	}

	#[document_type_parameters(
		"The brand of the head effect in the row.",
		"The brand of the tail row (typically another `CoproductBrand` or `CNilBrand`)."
	)]
	impl<H, T> RefFunctor for CoproductBrand<H, T>
	where
		H: RefFunctor + 'static,
		T: RefFunctor + 'static,
	{
		/// `RefFunctor::ref_map` for a non-empty effect row. Dispatches
		/// to the head brand `H` if the runtime tag is
		/// [`Inl`](Coproduct::Inl), or recurses into the tail brand `T`
		/// if it is [`Inr`](Coproduct::Inr). Mirrors the
		/// [`Functor`](crate::classes::Functor) impl above with `&self`
		/// receivers.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply by reference to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
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
		/// 		effects::coproduct::Coproduct,
		/// 	},
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<IdentityBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let value: Row = Coproduct::inject(Identity(10));
		/// let mapped =
		/// 	<CoproductBrand<IdentityBrand, CNilBrand> as RefFunctor>::ref_map(|x: &i32| *x + 1, &value);
		/// assert!(matches!(mapped, Coproduct::Inl(Identity(11))));
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Coproduct::Inl(h) => Coproduct::Inl(<H as RefFunctor>::ref_map(func, h)),
				Coproduct::Inr(t) => Coproduct::Inr(<T as RefFunctor>::ref_map(func, t)),
			}
		}
	}

	impl Extract for CNilBrand {
		/// `Extract::extract` for the empty row. The argument's type is
		/// `CNil`, which is uninhabited; the body matches it
		/// exhaustively without producing a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type the empty row would yield."
		)]
		///
		#[document_parameters("The empty-row functor instance (uninhabited).")]
		///
		#[document_returns("Unreachable; the body matches the uninhabited input exhaustively.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	classes::Extract,
		/// };
		/// fn requires_extract<F: Extract>() {}
		/// requires_extract::<CNilBrand>();
		/// assert_eq!(2 + 2, 4);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {}
		}
	}

	#[document_type_parameters(
		"The brand of the head effect in the row.",
		"The brand of the tail row (typically another `CoproductBrand` or `CNilBrand`)."
	)]
	impl<H, T> Extract for CoproductBrand<H, T>
	where
		H: Extract + 'static,
		T: Extract + 'static,
	{
		/// `Extract::extract` for a non-empty effect row. Dispatches
		/// to the head brand `H` if the runtime tag is
		/// [`Inl`](Coproduct::Inl), or recurses into the tail brand `T`
		/// if it is [`Inr`](Coproduct::Inr). The recursion bottoms out
		/// at [`CNilBrand`]'s uninhabited base case.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type yielded by the active brand."
		)]
		///
		#[document_parameters("The Coproduct functor layer.")]
		///
		#[document_returns("The active head or tail brand's `extract` result for `fa`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::Extract,
		/// 	types::{
		/// 		Identity,
		/// 		effects::coproduct::Coproduct,
		/// 	},
		/// };
		///
		/// type Row =
		/// 	<CoproductBrand<IdentityBrand, CNilBrand> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<
		/// 		'static,
		/// 		i32,
		/// 	>;
		///
		/// let row: Row = Coproduct::inject(Identity(42));
		/// let value = <CoproductBrand<IdentityBrand, CNilBrand> as Extract>::extract::<i32>(row);
		/// assert_eq!(value, 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				Coproduct::Inl(h) => <H as Extract>::extract::<A>(h),
				Coproduct::Inr(t) => <T as Extract>::extract::<A>(t),
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
				OptionBrand,
				VecBrand,
			},
			classes::Functor,
			kinds::*,
			types::effects::coproduct::Coproduct,
		},
	};

	type TwoEffectRow<A> = Apply!(
		<CoproductBrand<OptionBrand, CoproductBrand<VecBrand, CNilBrand>>
			as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>
	);

	#[test]
	fn kind_resolves_two_effect_row() {
		let row: TwoEffectRow<i32> = Coproduct::inject(Some(7));
		match row {
			Coproduct::Inl(opt) => assert_eq!(opt, Some(7)),
			Coproduct::Inr(_) => panic!("expected the head Inl branch"),
		}
	}

	#[test]
	fn functor_dispatches_to_head_branch() {
		let row: TwoEffectRow<i32> = Coproduct::inject(Some(10));
		let mapped: TwoEffectRow<i32> = <CoproductBrand<
			OptionBrand,
			CoproductBrand<VecBrand, CNilBrand>,
		> as Functor>::map(|x: i32| x + 1, row);
		match mapped {
			Coproduct::Inl(opt) => assert_eq!(opt, Some(11)),
			Coproduct::Inr(_) => panic!("expected head Inl branch"),
		}
	}

	#[test]
	fn functor_dispatches_to_tail_branch() {
		let row: TwoEffectRow<i32> = Coproduct::inject(vec![1_i32, 2, 3]);
		let mapped: TwoEffectRow<i32> = <CoproductBrand<
			OptionBrand,
			CoproductBrand<VecBrand, CNilBrand>,
		> as Functor>::map(|x: i32| x * 10, row);
		match mapped {
			Coproduct::Inl(_) => panic!("expected tail Inr branch"),
			Coproduct::Inr(rest) => match rest {
				Coproduct::Inl(v) => assert_eq!(v, vec![10, 20, 30]),
				Coproduct::Inr(_) => panic!("expected the second-position Inl branch"),
			},
		}
	}

	#[test]
	fn variant_f_alias_resolves_to_coproduct_brand() {
		// VariantF<H, T> is a type alias for CoproductBrand<H, T>; the
		// two should be interchangeable in any position.
		let _row: Apply!(
			<VariantF<OptionBrand, CNilBrand>
				as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, i32>
		) = Coproduct::inject(Some(0_i32));
	}
}
