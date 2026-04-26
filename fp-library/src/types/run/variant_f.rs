//! Open sum of first-order effect functors, encoded as a nested
//! [`Coproduct`](crate::types::run::coproduct::Coproduct) row.
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
//! crate::types::run::coproduct::Coproduct::Inl) /
//! [`Inr`](crate::types::run::coproduct::Coproduct::Inr) variants without
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
			classes::Functor,
			impl_kind,
			kinds::*,
			types::run::coproduct::Coproduct,
		},
		fp_macros::*,
	};

	impl_kind! {
		for CNilBrand {
			type Of<'a, A: 'a>: 'a = crate::types::run::coproduct::CNil;
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
		/// 	types::run::coproduct::Coproduct,
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
		/// 	types::run::coproduct::Coproduct,
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
}

pub use inner::*;

#[cfg(test)]
#[expect(clippy::panic, reason = "Tests panic on unreachable Coproduct branches for clarity.")]
mod tests {
	#[expect(
		unused_imports,
		reason = "Kind is referenced via the Kind!(...) macro below, which rustc does not detect as a direct use."
	)]
	use fp_macros::Kind;
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
			types::run::coproduct::Coproduct,
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
