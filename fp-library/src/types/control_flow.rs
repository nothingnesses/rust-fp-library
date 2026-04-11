//! Helpers and type class implementations for [`ControlFlow`](core::ops::ControlFlow) in tail-recursive computations.
//!
//! Used by [`MonadRec`](crate::classes::monad_rec::MonadRec) to implement stack-safe tail recursion. [`ControlFlow::Continue`](core::ops::ControlFlow::Continue) continues iteration, while [`ControlFlow::Break`](core::ops::ControlFlow::Break) terminates with a result.
//!
//! ### Examples
//!
//! ```
//! use core::ops::ControlFlow;
//!
//! // Count down from n to 0, accumulating the sum
//! fn sum_to_zero(
//! 	n: i32,
//! 	acc: i32,
//! ) -> ControlFlow<i32, (i32, i32)> {
//! 	if n <= 0 { ControlFlow::Break(acc) } else { ControlFlow::Continue((n - 1, acc + n)) }
//! }
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ControlFlowBrand,
				ControlFlowBreakAppliedBrand,
				ControlFlowContinueAppliedBrand,
			},
			classes::*,
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	/// Static helper methods for [`ControlFlow`] values.
	///
	/// These methods mirror the inherent methods that were previously on the
	/// `Step` type, now provided as associated functions on the brand.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// [`ControlFlow`] has multiple higher-kinded representations:
	/// - [`ControlFlowBrand`]: fully polymorphic over both continue and break types (bifunctor).
	/// - [`ControlFlowContinueAppliedBrand<C>`](crate::brands::ControlFlowContinueAppliedBrand): the continue type is fixed, polymorphic over the break type (functor over break).
	/// - [`ControlFlowBreakAppliedBrand<B>`](crate::brands::ControlFlowBreakAppliedBrand): the break type is fixed, polymorphic over the continue type (functor over continue).
	impl ControlFlowBrand {
		/// Returns `true` if this is a `Continue` variant.
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.")]
		///
		#[document_parameters("The control flow value.")]
		///
		#[document_returns("`true` if the value is `Continue`, `false` otherwise.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		/// assert!(ControlFlowBrand::is_continue(&cf));
		/// ```
		pub fn is_continue<B, C>(cf: &ControlFlow<B, C>) -> bool {
			matches!(cf, ControlFlow::Continue(_))
		}

		/// Returns `true` if this is a `Break` variant.
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.")]
		///
		#[document_parameters("The control flow value.")]
		///
		#[document_returns("`true` if the value is `Break`, `false` otherwise.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Break(1);
		/// assert!(ControlFlowBrand::is_break(&cf));
		/// ```
		pub fn is_break<B, C>(cf: &ControlFlow<B, C>) -> bool {
			matches!(cf, ControlFlow::Break(_))
		}

		/// Maps a function over the `Continue` variant.
		#[document_signature]
		///
		#[document_type_parameters(
			"The break type.",
			"The original continue type.",
			"The new continue type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the continue value."
		)]
		///
		#[document_returns("A new `ControlFlow` with the continue value transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		/// let mapped = ControlFlowBrand::map_continue(cf, |x| x + 1);
		/// assert_eq!(mapped, ControlFlow::Continue(2));
		/// ```
		pub fn map_continue<B, C, C2>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(C) -> C2,
		) -> ControlFlow<B, C2> {
			match cf {
				ControlFlow::Continue(c) => ControlFlow::Continue(f(c)),
				ControlFlow::Break(b) => ControlFlow::Break(b),
			}
		}

		/// Maps a function over the `Break` variant.
		#[document_signature]
		///
		#[document_type_parameters(
			"The original break type.",
			"The continue type.",
			"The new break type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the break value."
		)]
		///
		#[document_returns("A new `ControlFlow` with the break value transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Break(1);
		/// let mapped = ControlFlowBrand::map_break(cf, |x| x + 1);
		/// assert_eq!(mapped, ControlFlow::Break(2));
		/// ```
		pub fn map_break<B, C, B2>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(B) -> B2,
		) -> ControlFlow<B2, C> {
			match cf {
				ControlFlow::Continue(c) => ControlFlow::Continue(c),
				ControlFlow::Break(b) => ControlFlow::Break(f(b)),
			}
		}

		/// Applies functions to both variants (bifunctor map).
		#[document_signature]
		///
		#[document_type_parameters(
			"The original break type.",
			"The original continue type.",
			"The new break type.",
			"The new continue type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the continue value.",
			"The function to apply to the break value."
		)]
		///
		#[document_returns("A new `ControlFlow` with both values transformed.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		/// let mapped = ControlFlowBrand::bimap(cf, |x| x + 1, |x| x * 2);
		/// assert_eq!(mapped, ControlFlow::Continue(2));
		/// ```
		pub fn bimap<B, C, B2, C2>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(C) -> C2,
			g: impl FnOnce(B) -> B2,
		) -> ControlFlow<B2, C2> {
			match cf {
				ControlFlow::Continue(c) => ControlFlow::Continue(f(c)),
				ControlFlow::Break(b) => ControlFlow::Break(g(b)),
			}
		}

		/// Folds the control flow from right to left using two step functions.
		///
		/// See [`Bifoldable::bi_fold_right`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The break type.",
			"The continue type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The step function for the Continue variant.",
			"The step function for the Break variant.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(ControlFlowBrand::bi_fold_right(x, |c, acc| acc - c, |b, acc| acc + b, 10), 7);
		/// ```
		pub fn bi_fold_right<B, C, Acc>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(C, Acc) -> Acc,
			g: impl FnOnce(B, Acc) -> Acc,
			z: Acc,
		) -> Acc {
			match cf {
				ControlFlow::Continue(c) => f(c, z),
				ControlFlow::Break(b) => g(b, z),
			}
		}

		/// Folds the control flow from left to right using two step functions.
		///
		/// See [`Bifoldable::bi_fold_left`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The break type.",
			"The continue type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The step function for the Continue variant.",
			"The step function for the Break variant.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Break(5);
		/// assert_eq!(ControlFlowBrand::bi_fold_left(x, |acc, c| acc - c, |acc, b| acc + b, 10), 15);
		/// ```
		pub fn bi_fold_left<B, C, Acc>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(Acc, C) -> Acc,
			g: impl FnOnce(Acc, B) -> Acc,
			z: Acc,
		) -> Acc {
			match cf {
				ControlFlow::Continue(c) => f(z, c),
				ControlFlow::Break(b) => g(z, b),
			}
		}

		/// Maps the value to a monoid depending on the variant.
		///
		/// See [`Bifoldable::bi_fold_map`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.", "The monoid type.")]
		///
		#[document_parameters(
			"The control flow value.",
			"The function mapping the Continue value to the monoid.",
			"The function mapping the Break value to the monoid."
		)]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(
		/// 	ControlFlowBrand::bi_fold_map(x, |c: i32| c.to_string(), |b: i32| b.to_string()),
		/// 	"3"
		/// );
		/// ```
		pub fn bi_fold_map<B, C, M>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(C) -> M,
			g: impl FnOnce(B) -> M,
		) -> M {
			match cf {
				ControlFlow::Continue(c) => f(c),
				ControlFlow::Break(b) => g(b),
			}
		}

		/// Folds the Break value, returning `initial` for Continue.
		///
		/// See [`Foldable::fold_right`] for the type class version
		/// (via [`ControlFlowContinueAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters(
			"The break type.",
			"The continue type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the Break value and the accumulator.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, ()> = ControlFlow::Break(5);
		/// assert_eq!(ControlFlowBrand::fold_right(x, |b, acc| b + acc, 10), 15);
		/// ```
		pub fn fold_right<B, C, Acc>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(B, Acc) -> Acc,
			initial: Acc,
		) -> Acc {
			match cf {
				ControlFlow::Continue(_) => initial,
				ControlFlow::Break(b) => f(b, initial),
			}
		}

		/// Folds the Break value from the left, returning `initial` for Continue.
		///
		/// See [`Foldable::fold_left`] for the type class version
		/// (via [`ControlFlowContinueAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters(
			"The break type.",
			"The continue type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the accumulator and the Break value.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, ()> = ControlFlow::Break(5);
		/// assert_eq!(ControlFlowBrand::fold_left(x, |acc, b| acc + b, 10), 15);
		/// ```
		pub fn fold_left<B, C, Acc>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(Acc, B) -> Acc,
			initial: Acc,
		) -> Acc {
			match cf {
				ControlFlow::Continue(_) => initial,
				ControlFlow::Break(b) => f(initial, b),
			}
		}

		/// Maps the Break value to a monoid, returning `M::empty()` for Continue.
		///
		/// See [`Foldable::fold_map`] for the type class version
		/// (via [`ControlFlowContinueAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.", "The monoid type.")]
		///
		#[document_parameters("The control flow value.", "The mapping function.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, ()> = ControlFlow::Break(5);
		/// assert_eq!(ControlFlowBrand::fold_map(x, |b: i32| b.to_string()), "5".to_string());
		/// ```
		pub fn fold_map<B, C, M: Monoid>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(B) -> M,
		) -> M {
			match cf {
				ControlFlow::Continue(_) => M::empty(),
				ControlFlow::Break(b) => f(b),
			}
		}

		/// Chains the Break value into a new computation, passing through Continue.
		///
		/// See [`Semimonad::bind`] for the type class version
		/// (via [`ControlFlowContinueAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters(
			"The original break type.",
			"The continue type.",
			"The type of the resulting break value."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the Break value."
		)]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Break(5);
		/// let y = ControlFlowBrand::bind_break(x, |b| ControlFlow::Break(b * 2));
		/// assert_eq!(y, ControlFlow::Break(10));
		/// ```
		pub fn bind_break<B, C, B2>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(B) -> ControlFlow<B2, C>,
		) -> ControlFlow<B2, C> {
			match cf {
				ControlFlow::Continue(c) => ControlFlow::Continue(c),
				ControlFlow::Break(b) => f(b),
			}
		}

		/// Chains the Continue value into a new computation, passing through Break.
		///
		/// See [`Semimonad::bind`] for the type class version
		/// (via [`ControlFlowBreakAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters(
			"The break type.",
			"The original continue type.",
			"The type of the resulting continue value."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function to apply to the Continue value."
		)]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(5);
		/// let y = ControlFlowBrand::bind_continue(x, |c| ControlFlow::Continue(c * 2));
		/// assert_eq!(y, ControlFlow::Continue(10));
		/// ```
		pub fn bind_continue<B, C, C2>(
			cf: ControlFlow<B, C>,
			f: impl FnOnce(C) -> ControlFlow<B, C2>,
		) -> ControlFlow<B, C2> {
			match cf {
				ControlFlow::Continue(c) => f(c),
				ControlFlow::Break(b) => ControlFlow::Break(b),
			}
		}

		/// Extracts the `Break` value, returning `None` if this is a `Continue`.
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.")]
		///
		#[document_parameters("The control flow value.")]
		///
		#[document_returns("`Some(b)` if `Break(b)`, `None` if `Continue(_)`.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Break(42);
		/// assert_eq!(ControlFlowBrand::break_val(cf), Some(42));
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		/// assert_eq!(ControlFlowBrand::break_val(cf), None);
		/// ```
		pub fn break_val<B, C>(cf: ControlFlow<B, C>) -> Option<B> {
			match cf {
				ControlFlow::Break(b) => Some(b),
				ControlFlow::Continue(_) => None,
			}
		}

		/// Extracts the `Continue` value, returning `None` if this is `Break`.
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.")]
		///
		#[document_parameters("The control flow value.")]
		///
		#[document_returns("`Some(c)` if `Continue(c)`, `None` if `Break(_)`.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Continue(7);
		/// assert_eq!(ControlFlowBrand::continue_val(cf), Some(7));
		///
		/// let cf: ControlFlow<i32, i32> = ControlFlow::Break(42);
		/// assert_eq!(ControlFlowBrand::continue_val(cf), None);
		/// ```
		pub fn continue_val<B, C>(cf: ControlFlow<B, C>) -> Option<C> {
			match cf {
				ControlFlow::Continue(c) => Some(c),
				ControlFlow::Break(_) => None,
			}
		}

		/// Swaps the type parameters, mapping `Continue(c)` to `Break(c)` and `Break(b)` to `Continue(b)`.
		#[document_signature]
		///
		#[document_type_parameters("The break type.", "The continue type.")]
		///
		#[document_parameters("The control flow value.")]
		///
		#[document_returns("A new `ControlFlow` with the variants swapped.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::ControlFlowBrand,
		/// };
		///
		/// let cf: ControlFlow<&str, i32> = ControlFlow::Continue(1);
		/// assert_eq!(ControlFlowBrand::swap(cf), ControlFlow::Break(1));
		///
		/// let cf: ControlFlow<&str, i32> = ControlFlow::Break("hello");
		/// assert_eq!(ControlFlowBrand::swap(cf), ControlFlow::Continue("hello"));
		/// ```
		pub fn swap<B, C>(cf: ControlFlow<B, C>) -> ControlFlow<C, B> {
			match cf {
				ControlFlow::Continue(c) => ControlFlow::Break(c),
				ControlFlow::Break(b) => ControlFlow::Continue(b),
			}
		}

		/// Traverses the control flow with two effectful functions.
		///
		/// See [`Bitraversable::bi_traverse`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The break type.",
			"The continue type.",
			"The output type for the Continue value.",
			"The output type for the Break value.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The control flow value.",
			"The function for the Continue value.",
			"The function for the Break value."
		)]
		///
		#[document_returns("The transformed control flow wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::brands::{
		/// 		ControlFlowBrand,
		/// 		OptionBrand,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// let y = ControlFlowBrand::bi_traverse::<_, _, _, _, OptionBrand>(
		/// 	x,
		/// 	|c| Some(c + 1),
		/// 	|b| Some(b * 2),
		/// );
		/// assert_eq!(y, Some(ControlFlow::Continue(4)));
		/// ```
		pub fn bi_traverse<'a, B: 'a, C: 'a, C2: 'a + Clone, B2: 'a + Clone, F: Applicative>(
			cf: ControlFlow<B, C>,
			f: impl Fn(C) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C2>) + 'a,
			g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B2>) + 'a,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B2, C2>>)
		where
			ControlFlow<B2, C2>: Clone, {
			match cf {
				ControlFlow::Continue(c) => F::map(|c2| ControlFlow::Continue(c2), f(c)),
				ControlFlow::Break(b) => F::map(|b2| ControlFlow::Break(b2), g(b)),
			}
		}
	}

	// HKT Branding

	impl_kind! {
		for ControlFlowBrand {
			type Of<C, B> = ControlFlow<B, C>;
		}
	}

	impl_kind! {
		for ControlFlowBrand {
			type Of<'a, C: 'a, B: 'a>: 'a = ControlFlow<B, C>;
		}
	}

	impl Bifunctor for ControlFlowBrand {
		/// Maps functions over the values in the control flow.
		///
		/// This method applies one function to the continue value and another to the break value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the continue value.",
			"The type of the mapped continue value.",
			"The type of the break value.",
			"The type of the mapped break value."
		)]
		///
		#[document_parameters(
			"The function to apply to the continue value.",
			"The function to apply to the break value.",
			"The control flow to map over."
		)]
		///
		#[document_returns("A new control flow containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x = ControlFlow::<i32, i32>::Continue(1);
		/// assert_eq!(
		/// 	bimap::<ControlFlowBrand, _, _, _, _, _, _>((|c| c + 1, |b: i32| b * 2), x),
		/// 	ControlFlow::Continue(2)
		/// );
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			ControlFlowBrand::bimap(p, f, g)
		}
	}

	impl RefBifunctor for ControlFlowBrand {
		/// Maps functions over the values in the control flow by reference.
		///
		/// This method applies one function to a reference of the continue value and another
		/// to a reference of the break value, producing a new control flow with mapped values.
		/// The original control flow is borrowed, not consumed.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the continue value.",
			"The type of the mapped continue value.",
			"The type of the break value.",
			"The type of the mapped break value."
		)]
		///
		#[document_parameters(
			"The function to apply to a reference of the continue value.",
			"The function to apply to a reference of the break value.",
			"The control flow to map over by reference."
		)]
		///
		#[document_returns("A new control flow containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ref_bifunctor::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x = ControlFlow::<i32, i32>::Continue(1);
		/// assert_eq!(
		/// 	ref_bimap::<ControlFlowBrand, _, _, _, _>(|c| *c + 1, |b: &i32| *b * 2, &x),
		/// 	ControlFlow::Continue(2)
		/// );
		///
		/// let y = ControlFlow::<i32, i32>::Break(3);
		/// assert_eq!(
		/// 	ref_bimap::<ControlFlowBrand, _, _, _, _>(|c| *c + 1, |b: &i32| *b * 2, &y),
		/// 	ControlFlow::Break(6)
		/// );
		/// ```
		fn ref_bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(&A) -> B + 'a,
			g: impl Fn(&C) -> D + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			match p {
				ControlFlow::Continue(a) => ControlFlow::Continue(f(a)),
				ControlFlow::Break(c) => ControlFlow::Break(g(c)),
			}
		}
	}

	impl RefBifoldable for ControlFlowBrand {
		/// Folds the control flow from right to left by reference using two step functions.
		///
		/// Applies `f` to a reference of the Continue value or `g` to a reference of the
		/// Break value, returning the folded result without consuming the control flow.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Continue value.",
			"The type of the Break value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for a reference to the Continue variant.",
			"The step function for a reference to the Break variant.",
			"The initial accumulator.",
			"The control flow to fold by reference."
		)]
		///
		#[document_returns("The folded result.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, ControlFlowBrand, _, _, _, _, _>(
		/// 		(|c: &i32, acc| acc - *c, |b: &i32, acc| acc + *b),
		/// 		10,
		/// 		&x,
		/// 	),
		/// 	7
		/// );
		///
		/// let y: ControlFlow<i32, i32> = ControlFlow::Break(5);
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, ControlFlowBrand, _, _, _, _, _>(
		/// 		(|c: &i32, acc| acc - *c, |b: &i32, acc| acc + *b),
		/// 		10,
		/// 		&y,
		/// 	),
		/// 	15
		/// );
		/// ```
		fn ref_bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(&A, C) -> C + 'a,
			g: impl Fn(&B, C) -> C + 'a,
			z: C,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p {
				ControlFlow::Continue(a) => f(a, z),
				ControlFlow::Break(b) => g(b, z),
			}
		}
	}

	impl RefBitraversable for ControlFlowBrand {
		/// Traverses a control flow by reference with two effectful functions.
		///
		/// Applies `f` to a reference of the Continue value or `g` to a reference of
		/// the Break value, wrapping the result in the applicative context `F`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the Continue value.",
			"The type of the Break value.",
			"The output type for Continue.",
			"The output type for Break.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to a reference of the Continue value.",
			"The function applied to a reference of the Break value.",
			"The control flow to traverse by reference."
		)]
		///
		#[document_returns(
			"`f(&a)` wrapped in context for `Continue(a)`, or `g(&b)` wrapped in context for `Break(b)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(
		/// 	bi_traverse::<RcFnBrand, ControlFlowBrand, _, _, _, _, OptionBrand, _, _>(
		/// 		(|c: &i32| Some(c + 1), |b: &i32| Some(b * 2)),
		/// 		&x,
		/// 	),
		/// 	Some(ControlFlow::Continue(4))
		/// );
		///
		/// let y: ControlFlow<i32, i32> = ControlFlow::Break(5);
		/// assert_eq!(
		/// 	bi_traverse::<RcFnBrand, ControlFlowBrand, _, _, _, _, OptionBrand, _, _>(
		/// 		(|c: &i32| Some(c + 1), |b: &i32| Some(b * 2)),
		/// 		&y,
		/// 	),
		/// 	Some(ControlFlow::Break(10))
		/// );
		/// ```
		fn ref_bi_traverse<
			'a,
			FnBrand,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
			match p {
				ControlFlow::Continue(a) => F::map(|c| ControlFlow::Continue(c), f(a)),
				ControlFlow::Break(b) => F::map(|d| ControlFlow::Break(d), g(b)),
			}
		}
	}

	impl Bifoldable for ControlFlowBrand {
		/// Folds the control flow from right to left using two step functions.
		///
		/// Applies `f` to the Continue value or `g` to the Break value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Continue value.",
			"The type of the Break value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for the Continue variant.",
			"The step function for the Break variant.",
			"The initial accumulator.",
			"The control flow to fold."
		)]
		///
		#[document_returns("The folded result.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, ControlFlowBrand, _, _, _, _, _>(
		/// 		(|c, acc| acc - c, |b, acc| acc + b),
		/// 		10,
		/// 		x,
		/// 	),
		/// 	7
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(A, C) -> C + 'a,
			g: impl Fn(B, C) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			ControlFlowBrand::bi_fold_right(p, f, g, z)
		}

		/// Folds the control flow from left to right using two step functions.
		///
		/// Applies `f` to the Continue value or `g` to the Break value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Continue value.",
			"The type of the Break value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for the Continue variant.",
			"The step function for the Break variant.",
			"The initial accumulator.",
			"The control flow to fold."
		)]
		///
		#[document_returns("The folded result.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Break(5);
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, ControlFlowBrand, _, _, _, _, _>(
		/// 		(|acc, c| acc - c, |acc, b| acc + b),
		/// 		10,
		/// 		x,
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, A) -> C + 'a,
			g: impl Fn(C, B) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			ControlFlowBrand::bi_fold_left(p, f, g, z)
		}

		/// Maps the value to a monoid depending on the variant.
		///
		/// Applies `f` if Continue, `g` if Break.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Continue value.",
			"The type of the Break value.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping the Continue value to the monoid.",
			"The function mapping the Break value to the monoid.",
			"The control flow to fold."
		)]
		///
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, ControlFlowBrand, _, _, _, _, _>(
		/// 		(|c: i32| c.to_string(), |b: i32| b.to_string()),
		/// 		x,
		/// 	),
		/// 	"3".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(A) -> M + 'a,
			g: impl Fn(B) -> M + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			ControlFlowBrand::bi_fold_map(p, f, g)
		}
	}

	impl Bitraversable for ControlFlowBrand {
		/// Traverses the control flow with two effectful functions.
		///
		/// Applies `f` to the Continue value or `g` to the Break value,
		/// wrapping the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the Continue value.",
			"The type of the Break value.",
			"The output type for Continue.",
			"The output type for Break.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to the Continue value.",
			"The function applied to the Break value.",
			"The control flow to traverse."
		)]
		///
		#[document_returns("The transformed control flow wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let x: ControlFlow<i32, i32> = ControlFlow::Continue(3);
		/// assert_eq!(
		/// 	bi_traverse::<RcFnBrand, ControlFlowBrand, _, _, _, _, OptionBrand, _, _>(
		/// 		(|c: i32| Some(c + 1), |b: i32| Some(b * 2)),
		/// 		x,
		/// 	),
		/// 	Some(ControlFlow::Continue(4))
		/// );
		/// ```
		fn bi_traverse<
			'a,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		{
			ControlFlowBrand::bi_traverse::<_, _, C, D, F>(p, f, g)
		}
	}

	// ControlFlowContinueAppliedBrand<ContinueType> (Functor over B, the Break type)

	impl_kind! {
		impl<ContinueType: 'static> for ControlFlowContinueAppliedBrand<ContinueType> {
			type Of<'a, B: 'a>: 'a = ControlFlow<B, ContinueType>;
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: 'static> Functor for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Maps a function over the break value in the control flow.
		///
		/// This method applies a function to the break value inside the control flow, producing a new control flow with the transformed break value. The continue value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the break value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the break value.",
			"The control flow to map over."
		)]
		///
		#[document_returns(
			"A new control flow containing the result of applying the function to the break value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	map_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32| x * 2,
		/// 		ControlFlow::<i32, i32>::Break(5)
		/// 	),
		/// 	ControlFlow::Break(10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ControlFlowBrand::map_break(fa, func)
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> Lift for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Lifts a binary function into the control flow context.
		///
		/// This method lifts a binary function to operate on values within the control flow context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first control flow.",
			"The second control flow."
		)]
		///
		#[document_returns(
			"`Break(f(a, b))` if both are `Break`, otherwise the first continue encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	lift2_explicit::<ControlFlowContinueAppliedBrand<()>, _, _, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		ControlFlow::Break(1),
		/// 		ControlFlow::Break(2)
		/// 	),
		/// 	ControlFlow::Break(3)
		/// );
		/// assert_eq!(
		/// 	lift2_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		ControlFlow::Break(1),
		/// 		ControlFlow::Continue(2)
		/// 	),
		/// 	ControlFlow::Continue(2)
		/// );
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			match (fa, fb) {
				(ControlFlow::Break(a), ControlFlow::Break(b)) => ControlFlow::Break(func(a, b)),
				(ControlFlow::Continue(e), _) => ControlFlow::Continue(e),
				(_, ControlFlow::Continue(e)) => ControlFlow::Continue(e),
			}
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: 'static> Pointed for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Wraps a value in a control flow.
		///
		/// This method wraps a value in the `Break` variant of a `ControlFlow`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Break(a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(pure::<ControlFlowContinueAppliedBrand<()>, _>(5), ControlFlow::<_, ()>::Break(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			ControlFlow::Break(a)
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> ApplyFirst for ControlFlowContinueAppliedBrand<ContinueType> {}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> ApplySecond for ControlFlowContinueAppliedBrand<ContinueType> {}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> Semiapplicative
		for ControlFlowContinueAppliedBrand<ContinueType>
	{
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in a control flow to a value wrapped in a control flow.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The control flow containing the function.",
			"The control flow containing the value."
		)]
		///
		#[document_returns(
			"`Break(f(a))` if both are `Break`, otherwise the first continue encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let f: ControlFlow<_, ()> = ControlFlow::Break(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	apply::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _>(f, ControlFlow::Break(5)),
		/// 	ControlFlow::Break(10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(ControlFlow::Break(f), ControlFlow::Break(a)) => ControlFlow::Break(f(a)),
				(ControlFlow::Continue(e), _) => ControlFlow::Continue(e),
				(_, ControlFlow::Continue(e)) => ControlFlow::Continue(e),
			}
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> Semimonad for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Chains control flow computations.
		///
		/// This method chains two computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first control flow.",
			"The function to apply to the value inside the control flow."
		)]
		///
		#[document_returns(
			"The result of applying `f` to the value if `ma` is `Break`, otherwise the original continue."
		)]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	bind_explicit::<ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
		/// 		ControlFlow::Break(5),
		/// 		|x| { ControlFlow::Break(x * 2) }
		/// 	),
		/// 	ControlFlow::<_, ()>::Break(10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ControlFlowBrand::bind_break(ma, func)
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: 'static> Foldable for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Folds the control flow from the right.
		///
		/// This method performs a right-associative fold of the control flow.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The folding function.",
			"The initial value.",
			"The control flow to fold."
		)]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Break(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
		/// 		|x, acc| x + acc,
		/// 		0,
		/// 		ControlFlow::Break(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		ControlFlow::Continue(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			ControlFlowBrand::fold_right(fa, func, initial)
		}

		/// Folds the control flow from the left.
		///
		/// This method performs a left-associative fold of the control flow.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The folding function.",
			"The initial value.",
			"The control flow to fold."
		)]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Break(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
		/// 		|acc, x| acc + x,
		/// 		0,
		/// 		ControlFlow::Break(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		ControlFlow::Continue(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			ControlFlowBrand::fold_left(fa, func, initial)
		}

		/// Maps the value to a monoid and returns it.
		///
		/// This method maps the element of the control flow to a monoid and then returns it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The control flow to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Break(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		ControlFlow::Break(5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		ControlFlow::Continue(1)
		/// 	),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			ControlFlowBrand::fold_map(fa, func)
		}
	}

	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> Traversable for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Traverses the control flow with an applicative function.
		///
		/// This method maps the element of the control flow to a computation, evaluates it, and combines the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The control flow to traverse.")]
		///
		#[document_returns("The control flow wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, OptionBrand, _, _>(
		/// 		|x| Some(x * 2),
		/// 		ControlFlow::Break(5)
		/// 	),
		/// 	Some(ControlFlow::Break(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _, OptionBrand, _, _>(
		/// 		|x: i32| Some(x * 2),
		/// 		ControlFlow::Continue(1)
		/// 	),
		/// 	Some(ControlFlow::Continue(1))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				ControlFlow::Break(a) => F::map(|b| ControlFlow::Break(b), func(a)),
				ControlFlow::Continue(e) => F::pure(ControlFlow::Continue(e)),
			}
		}

		/// Sequences a control flow of applicative.
		///
		/// This method evaluates the computation inside the control flow and accumulates the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The control flow containing the applicative value.")]
		///
		#[document_returns("The control flow wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	sequence::<ControlFlowContinueAppliedBrand<()>, _, OptionBrand>(ControlFlow::Break(Some(
		/// 		5
		/// 	))),
		/// 	Some(ControlFlow::Break(5))
		/// );
		/// assert_eq!(
		/// 	sequence::<ControlFlowContinueAppliedBrand<i32>, i32, OptionBrand>(ControlFlow::<
		/// 		Option<i32>,
		/// 		i32,
		/// 	>::Continue(1)),
		/// 	Some(ControlFlow::<i32, i32>::Continue(1))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				ControlFlow::Break(fa) => F::map(|a| ControlFlow::Break(a), fa),
				ControlFlow::Continue(e) => F::pure(ControlFlow::Continue(e)),
			}
		}
	}

	// ControlFlowBreakAppliedBrand<BreakType> (Functor over C, the Continue type)

	impl_kind! {
		impl<BreakType: 'static> for ControlFlowBreakAppliedBrand<BreakType> {
			type Of<'a, C: 'a>: 'a = ControlFlow<BreakType, C>;
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: 'static> Functor for ControlFlowBreakAppliedBrand<BreakType> {
		/// Maps a function over the continue value in the control flow.
		///
		/// This method applies a function to the continue value inside the control flow, producing a new control flow with the transformed continue value. The break value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the continue value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the continue value.",
			"The control flow to map over."
		)]
		///
		#[document_returns(
			"A new control flow containing the result of applying the function to the continue value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	map_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32| x * 2,
		/// 		ControlFlow::<i32, i32>::Continue(5)
		/// 	),
		/// 	ControlFlow::Continue(10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ControlFlowBrand::map_continue(fa, func)
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> Lift for ControlFlowBreakAppliedBrand<BreakType> {
		/// Lifts a binary function into the control flow context (over continue).
		///
		/// This method lifts a binary function to operate on continue values within the control flow context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first continue value.",
			"The type of the second continue value.",
			"The type of the result continue value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the continues.",
			"The first control flow.",
			"The second control flow."
		)]
		///
		#[document_returns(
			"`Continue(f(a, b))` if both are `Continue`, otherwise the first break encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	lift2_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		ControlFlow::Continue(1),
		/// 		ControlFlow::Continue(2)
		/// 	),
		/// 	ControlFlow::Continue(3)
		/// );
		/// assert_eq!(
		/// 	lift2_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		ControlFlow::Continue(1),
		/// 		ControlFlow::Break(2)
		/// 	),
		/// 	ControlFlow::Break(2)
		/// );
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			match (fa, fb) {
				(ControlFlow::Continue(a), ControlFlow::Continue(b)) =>
					ControlFlow::Continue(func(a, b)),
				(ControlFlow::Break(t), _) => ControlFlow::Break(t),
				(_, ControlFlow::Break(t)) => ControlFlow::Break(t),
			}
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: 'static> Pointed for ControlFlowBreakAppliedBrand<BreakType> {
		/// Wraps a value in a control flow (as continue).
		///
		/// This method wraps a value in the `Continue` variant of a `ControlFlow`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Continue(a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(pure::<ControlFlowBreakAppliedBrand<()>, _>(5), ControlFlow::<(), _>::Continue(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			ControlFlow::Continue(a)
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> ApplyFirst for ControlFlowBreakAppliedBrand<BreakType> {}

	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> ApplySecond for ControlFlowBreakAppliedBrand<BreakType> {}

	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> Semiapplicative for ControlFlowBreakAppliedBrand<BreakType> {
		/// Applies a wrapped function to a wrapped value (over continue).
		///
		/// This method applies a function wrapped in a control flow (as continue) to a value wrapped in a control flow (as continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The control flow containing the function (in Continue).",
			"The control flow containing the value (in Continue)."
		)]
		///
		#[document_returns(
			"`Continue(f(a))` if both are `Continue`, otherwise the first break encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let f: ControlFlow<(), _> =
		/// 	ControlFlow::Continue(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	apply::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _>(f, ControlFlow::Continue(5)),
		/// 	ControlFlow::Continue(10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(ControlFlow::Continue(f), ControlFlow::Continue(a)) => ControlFlow::Continue(f(a)),
				(ControlFlow::Break(t), _) => ControlFlow::Break(t),
				(_, ControlFlow::Break(t)) => ControlFlow::Break(t),
			}
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> Semimonad for ControlFlowBreakAppliedBrand<BreakType> {
		/// Chains control flow computations (over continue).
		///
		/// This method chains two computations, where the second computation depends on the result of the first (over continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first control flow.",
			"The function to apply to the continue value."
		)]
		///
		#[document_returns(
			"The result of applying `f` to the continue if `ma` is `Continue`, otherwise the original break."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	bind_explicit::<ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
		/// 		ControlFlow::Continue(5),
		/// 		|x| { ControlFlow::Continue(x * 2) }
		/// 	),
		/// 	ControlFlow::<(), _>::Continue(10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ControlFlowBrand::bind_continue(ma, func)
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: 'static> Foldable for ControlFlowBreakAppliedBrand<BreakType> {
		/// Folds the control flow from the right (over continue).
		///
		/// This method performs a right-associative fold of the control flow (over continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The folding function.",
			"The initial value.",
			"The control flow to fold."
		)]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Continue(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		ControlFlow::Continue(1)
		/// 	),
		/// 	1
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		ControlFlow::Break(())
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa {
				ControlFlow::Continue(e) => func(e, initial),
				ControlFlow::Break(_) => initial,
			}
		}

		/// Folds the control flow from the left (over continue).
		///
		/// This method performs a left-associative fold of the control flow (over continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The folding function.",
			"The initial value.",
			"The control flow to fold."
		)]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Continue(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		ControlFlow::Continue(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		ControlFlow::Break(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa {
				ControlFlow::Continue(e) => func(initial, e),
				ControlFlow::Break(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it (over continue).
		///
		/// This method maps the element of the control flow to a monoid and then returns it (over continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The control flow to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Continue(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		ControlFlow::Continue(5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		ControlFlow::Break(1)
		/// 	),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			match fa {
				ControlFlow::Continue(e) => func(e),
				ControlFlow::Break(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> Traversable for ControlFlowBreakAppliedBrand<BreakType> {
		/// Traverses the control flow with an applicative function (over continue).
		///
		/// This method maps the element of the control flow to a computation, evaluates it, and combines the result into an applicative context (over continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The control flow to traverse.")]
		///
		#[document_returns("The control flow wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, OptionBrand, _, _>(
		/// 		|x| Some(x * 2),
		/// 		ControlFlow::Continue(5)
		/// 	),
		/// 	Some(ControlFlow::Continue(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _, OptionBrand, _, _>(
		/// 		|x: i32| Some(x * 2),
		/// 		ControlFlow::Break(1)
		/// 	),
		/// 	Some(ControlFlow::Break(1))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				ControlFlow::Continue(e) => F::map(|b| ControlFlow::Continue(b), func(e)),
				ControlFlow::Break(t) => F::pure(ControlFlow::Break(t)),
			}
		}

		/// Sequences a control flow of applicative (over continue).
		///
		/// This method evaluates the computation inside the control flow and accumulates the result into an applicative context (over continue).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The control flow containing the applicative value.")]
		///
		#[document_returns("The control flow wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// assert_eq!(
		/// 	sequence::<ControlFlowBreakAppliedBrand<()>, _, OptionBrand>(ControlFlow::Continue(Some(
		/// 		5
		/// 	))),
		/// 	Some(ControlFlow::Continue(5))
		/// );
		/// assert_eq!(
		/// 	sequence::<ControlFlowBreakAppliedBrand<i32>, i32, OptionBrand>(ControlFlow::<
		/// 		i32,
		/// 		Option<i32>,
		/// 	>::Break(1)),
		/// 	Some(ControlFlow::<i32, i32>::Break(1))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				ControlFlow::Continue(fe) => F::map(|e| ControlFlow::Continue(e), fe),
				ControlFlow::Break(t) => F::pure(ControlFlow::Break(t)),
			}
		}
	}

	/// [`MonadRec`] implementation for [`ControlFlowContinueAppliedBrand`].
	#[document_type_parameters("The continue type.")]
	impl<ContinueType: Clone + 'static> MonadRec for ControlFlowContinueAppliedBrand<ContinueType> {
		/// Performs tail-recursive monadic computation over [`ControlFlow`] (break channel).
		///
		/// Iteratively applies the step function. If the function returns [`ControlFlow::Continue`],
		/// the computation short-circuits with that continue value. If it returns
		/// `Break(ControlFlow::Continue(a))`, the loop continues with the new state. If it returns
		/// `Break(ControlFlow::Break(b))`, the computation completes with `Break(b)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns(
			"The result of the computation, or a continue if the step function returned `Continue`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<ControlFlowContinueAppliedBrand<&str>, _, _>(
		/// 	|n| {
		/// 		if n < 10 {
		/// 			ControlFlow::Break(ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			ControlFlow::Break(ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, ControlFlow::Break(10));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut current = initial;
			loop {
				match func(current) {
					ControlFlow::Continue(l) => return ControlFlow::Continue(l),
					ControlFlow::Break(ControlFlow::Continue(next)) => current = next,
					ControlFlow::Break(ControlFlow::Break(b)) => return ControlFlow::Break(b),
				}
			}
		}
	}

	/// [`MonadRec`] implementation for [`ControlFlowBreakAppliedBrand`].
	#[document_type_parameters("The break type.")]
	impl<BreakType: Clone + 'static> MonadRec for ControlFlowBreakAppliedBrand<BreakType> {
		/// Performs tail-recursive monadic computation over [`ControlFlow`] (continue channel).
		///
		/// Iteratively applies the step function. If the function returns [`ControlFlow::Break`],
		/// the computation short-circuits with that break value. If it returns
		/// `Continue(ControlFlow::Continue(a))`, the loop continues with the new state. If it returns
		/// `Continue(ControlFlow::Break(b))`, the computation completes with `Continue(b)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns(
			"The result of the computation, or a break if the step function returned `Break`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<ControlFlowBreakAppliedBrand<&str>, _, _>(
		/// 	|n| {
		/// 		if n < 10 {
		/// 			ControlFlow::Continue(ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			ControlFlow::Continue(ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, ControlFlow::Continue(10));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut current = initial;
			loop {
				match func(current) {
					ControlFlow::Break(d) => return ControlFlow::Break(d),
					ControlFlow::Continue(ControlFlow::Continue(next)) => current = next,
					ControlFlow::Continue(ControlFlow::Break(b)) =>
						return ControlFlow::Continue(b),
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::*,
		},
		core::ops::ControlFlow,
		quickcheck::{
			Arbitrary,
			Gen,
		},
		quickcheck_macros::quickcheck,
	};

	impl<B: Arbitrary, C: Arbitrary> Arbitrary for ControlFlowWrapper<B, C> {
		fn arbitrary(g: &mut Gen) -> Self {
			if bool::arbitrary(g) {
				ControlFlowWrapper(ControlFlow::Continue(C::arbitrary(g)))
			} else {
				ControlFlowWrapper(ControlFlow::Break(B::arbitrary(g)))
			}
		}
	}

	/// Newtype wrapper for `ControlFlow` to implement `Arbitrary`.
	#[derive(Clone, Copy, Debug, PartialEq, Eq)]
	struct ControlFlowWrapper<B, C>(ControlFlow<B, C>);

	impl<B, C> ControlFlowWrapper<B, C> {
		fn into_inner(self) -> ControlFlow<B, C> {
			self.0
		}
	}

	/// Tests the `is_continue` method.
	///
	/// Verifies that `is_continue` returns true for `Continue` variants and false for `Break` variants.
	#[test]
	fn test_is_continue() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		assert!(ControlFlowBrand::is_continue(&cf));
		assert!(!ControlFlowBrand::is_break(&cf));
	}

	/// Tests the `is_break` method.
	///
	/// Verifies that `is_break` returns true for `Break` variants and false for `Continue` variants.
	#[test]
	fn test_is_break() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Break(1);
		assert!(ControlFlowBrand::is_break(&cf));
		assert!(!ControlFlowBrand::is_continue(&cf));
	}

	/// Tests the `map_continue` method.
	///
	/// Verifies that `map_continue` transforms the value inside a `Continue` variant and leaves a `Break` variant unchanged.
	#[test]
	fn test_map_continue() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		let mapped = ControlFlowBrand::map_continue(cf, |x| x + 1);
		assert_eq!(mapped, ControlFlow::Continue(2));

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(1);
		let mapped_brk = ControlFlowBrand::map_continue(brk, |x| x + 1);
		assert_eq!(mapped_brk, ControlFlow::Break(1));
	}

	/// Tests the `map_break` method.
	///
	/// Verifies that `map_break` transforms the value inside a `Break` variant and leaves a `Continue` variant unchanged.
	#[test]
	fn test_map_break() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Break(1);
		let mapped = ControlFlowBrand::map_break(cf, |x| x + 1);
		assert_eq!(mapped, ControlFlow::Break(2));

		let cont: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		let mapped_cont = ControlFlowBrand::map_break(cont, |x| x + 1);
		assert_eq!(mapped_cont, ControlFlow::Continue(1));
	}

	/// Tests the `bimap` method.
	///
	/// Verifies that `bimap` transforms the value inside both `Continue` and `Break` variants using the appropriate function.
	#[test]
	fn test_bimap() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		let mapped = ControlFlowBrand::bimap(cf, |x| x + 1, |x| x * 2);
		assert_eq!(mapped, ControlFlow::Continue(2));

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(1);
		let mapped_brk = ControlFlowBrand::bimap(brk, |x| x + 1, |x| x * 2);
		assert_eq!(mapped_brk, ControlFlow::Break(2));
	}

	/// Tests `Functor` implementation for `ControlFlowContinueAppliedBrand`.
	#[test]
	fn test_functor_with_continue() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Break(5);
		assert_eq!(
			map_explicit::<ControlFlowContinueAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, cf),
			ControlFlow::Break(10)
		);

		let cont: ControlFlow<i32, i32> = ControlFlow::Continue(5);
		assert_eq!(
			map_explicit::<ControlFlowContinueAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, cont),
			ControlFlow::Continue(5)
		);
	}

	/// Tests `Functor` implementation for `ControlFlowBreakAppliedBrand`.
	#[test]
	fn test_functor_with_break() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(5);
		assert_eq!(
			map_explicit::<ControlFlowBreakAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, cf),
			ControlFlow::Continue(10)
		);

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(5);
		assert_eq!(
			map_explicit::<ControlFlowBreakAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, brk),
			ControlFlow::Break(5)
		);
	}

	/// Tests `Bifunctor` implementation for `ControlFlowBrand`.
	#[test]
	fn test_bifunctor() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(5);
		assert_eq!(
			bimap::<ControlFlowBrand, _, _, _, _, _, _>((|c| c + 1, |b| b * 2), cf),
			ControlFlow::Continue(6)
		);

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(5);
		assert_eq!(
			bimap::<ControlFlowBrand, _, _, _, _, _, _>((|c| c + 1, |b| b * 2), brk),
			ControlFlow::Break(10)
		);
	}

	// Functor Laws for ControlFlowContinueAppliedBrand

	#[quickcheck]
	fn functor_identity_with_continue(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		map_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_with_continue(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(compose(f, g), x)
			== map_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
				f,
				map_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(g, x),
			)
	}

	// Functor Laws for ControlFlowBreakAppliedBrand

	#[quickcheck]
	fn functor_identity_with_break(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		map_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_with_break(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(compose(f, g), x)
			== map_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
				f,
				map_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(g, x),
			)
	}

	// Bifunctor Laws for ControlFlowBrand

	#[quickcheck]
	fn bifunctor_identity(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		bimap::<ControlFlowBrand, _, _, _, _, _, _>((identity, identity), x) == x
	}

	#[quickcheck]
	fn bifunctor_composition(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<ControlFlowBrand, _, _, _, _, _, _>((compose(f, g), compose(h, i)), x)
			== bimap::<ControlFlowBrand, _, _, _, _, _, _>(
				(f, h),
				bimap::<ControlFlowBrand, _, _, _, _, _, _>((g, i), x),
			)
	}

	// Lift Tests

	/// Tests the `lift2` function for `ControlFlowContinueAppliedBrand`.
	///
	/// Verifies that `lift2` correctly combines two `ControlFlow` values using a binary function,
	/// handling `Break` and `Continue` variants according to the `Lift` implementation.
	#[test]
	fn test_lift2_with_continue() {
		let s1: ControlFlow<i32, i32> = ControlFlow::Break(1);
		let s2: ControlFlow<i32, i32> = ControlFlow::Break(2);
		let s3: ControlFlow<i32, i32> = ControlFlow::Continue(3);

		assert_eq!(
			lift2_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _, _, _>(
				|x, y| x + y,
				s1,
				s2
			),
			ControlFlow::Break(3)
		);
		assert_eq!(
			lift2_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _, _, _>(
				|x, y| x + y,
				s1,
				s3
			),
			ControlFlow::Continue(3)
		);
	}

	/// Tests the `lift2` function for `ControlFlowBreakAppliedBrand`.
	///
	/// Verifies that `lift2` correctly combines two `ControlFlow` values using a binary function,
	/// handling `Break` and `Continue` variants according to the `Lift` implementation.
	#[test]
	fn test_lift2_with_break() {
		let s1: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		let s2: ControlFlow<i32, i32> = ControlFlow::Continue(2);
		let s3: ControlFlow<i32, i32> = ControlFlow::Break(3);

		assert_eq!(
			lift2_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _, _, _>(
				|x, y| x + y,
				s1,
				s2
			),
			ControlFlow::Continue(3)
		);
		assert_eq!(
			lift2_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _, _, _>(
				|x, y| x + y,
				s1,
				s3
			),
			ControlFlow::Break(3)
		);
	}

	// Pointed Tests

	/// Tests the `pure` function for `ControlFlowContinueAppliedBrand`.
	///
	/// Verifies that `pure` wraps a value into a `ControlFlow::Break` variant.
	#[test]
	fn test_pointed_with_continue() {
		assert_eq!(
			pure::<ControlFlowContinueAppliedBrand<()>, _>(5),
			ControlFlow::<_, ()>::Break(5)
		);
	}

	/// Tests the `pure` function for `ControlFlowBreakAppliedBrand`.
	///
	/// Verifies that `pure` wraps a value into a `ControlFlow::Continue` variant.
	#[test]
	fn test_pointed_with_break() {
		assert_eq!(
			pure::<ControlFlowBreakAppliedBrand<()>, _>(5),
			ControlFlow::<(), _>::Continue(5)
		);
	}

	// Semiapplicative Tests

	/// Tests the `apply` function for `ControlFlowContinueAppliedBrand`.
	///
	/// Verifies that `apply` correctly applies a wrapped function to a wrapped value,
	/// handling `Break` and `Continue` variants.
	#[test]
	fn test_apply_with_continue() {
		let f = pure::<ControlFlowContinueAppliedBrand<()>, _>(lift_fn_new::<RcFnBrand, _, _>(
			|x: i32| x * 2,
		));
		let x = pure::<ControlFlowContinueAppliedBrand<()>, _>(5);
		assert_eq!(
			apply::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _>(f, x),
			ControlFlow::Break(10)
		);

		let cont: ControlFlow<_, i32> = ControlFlow::Continue(1);
		let f_cont = pure::<ControlFlowContinueAppliedBrand<i32>, _>(
			lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),
		);
		assert_eq!(
			apply::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _>(f_cont, cont),
			ControlFlow::Continue(1)
		);
	}

	/// Tests the `apply` function for `ControlFlowBreakAppliedBrand`.
	///
	/// Verifies that `apply` correctly applies a wrapped function to a wrapped value,
	/// handling `Break` and `Continue` variants.
	#[test]
	fn test_apply_with_break() {
		let f = pure::<ControlFlowBreakAppliedBrand<()>, _>(lift_fn_new::<RcFnBrand, _, _>(
			|x: i32| x * 2,
		));
		let x = pure::<ControlFlowBreakAppliedBrand<()>, _>(5);
		assert_eq!(
			apply::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _>(f, x),
			ControlFlow::Continue(10)
		);

		let brk: ControlFlow<i32, _> = ControlFlow::Break(1);
		let f_brk = pure::<ControlFlowBreakAppliedBrand<i32>, _>(lift_fn_new::<RcFnBrand, _, _>(
			|x: i32| x * 2,
		));
		assert_eq!(
			apply::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _>(f_brk, brk),
			ControlFlow::Break(1)
		);
	}

	// Semimonad Tests

	/// Tests the `bind` function for `ControlFlowContinueAppliedBrand`.
	///
	/// Verifies that `bind` correctly chains computations, handling `Break` and `Continue` variants.
	#[test]
	fn test_bind_with_continue() {
		let x = pure::<ControlFlowContinueAppliedBrand<()>, _>(5);
		assert_eq!(
			bind_explicit::<ControlFlowContinueAppliedBrand<()>, _, _, _, _>(x, |i| pure::<
				ControlFlowContinueAppliedBrand<()>,
				_,
			>(i * 2)),
			ControlFlow::Break(10)
		);

		let cont: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		assert_eq!(
			bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(cont, |i| pure::<
				ControlFlowContinueAppliedBrand<i32>,
				_,
			>(i * 2)),
			ControlFlow::Continue(1)
		);
	}

	/// Tests the `bind` function for `ControlFlowBreakAppliedBrand`.
	///
	/// Verifies that `bind` correctly chains computations, handling `Break` and `Continue` variants.
	#[test]
	fn test_bind_with_break() {
		let x = pure::<ControlFlowBreakAppliedBrand<()>, _>(5);
		assert_eq!(
			bind_explicit::<ControlFlowBreakAppliedBrand<()>, _, _, _, _>(x, |i| pure::<
				ControlFlowBreakAppliedBrand<()>,
				_,
			>(i * 2)),
			ControlFlow::Continue(10)
		);

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(1);
		assert_eq!(
			bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(brk, |i| pure::<
				ControlFlowBreakAppliedBrand<i32>,
				_,
			>(i * 2)),
			ControlFlow::Break(1)
		);
	}

	// Foldable Tests

	/// Tests `Foldable` methods for `ControlFlowContinueAppliedBrand`.
	///
	/// Verifies `fold_right`, `fold_left`, and `fold_map` behavior for `Break` and `Continue` variants.
	#[test]
	fn test_foldable_with_continue() {
		let x = pure::<ControlFlowContinueAppliedBrand<()>, _>(5);
		assert_eq!(
			fold_right::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
				|a, b| a + b,
				10,
				x
			),
			15
		);
		assert_eq!(
			fold_left::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
				|b, a| b + a,
				10,
				x
			),
			15
		);
		assert_eq!(
			fold_map::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, _, _>(
				|a: i32| a.to_string(),
				x
			),
			"5"
		);

		let cont: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		assert_eq!(
			fold_right::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
				|a, b| a + b,
				10,
				cont
			),
			10
		);
	}

	/// Tests `Foldable` methods for `ControlFlowBreakAppliedBrand`.
	///
	/// Verifies `fold_right`, `fold_left`, and `fold_map` behavior for `Break` and `Continue` variants.
	#[test]
	fn test_foldable_with_break() {
		let x = pure::<ControlFlowBreakAppliedBrand<()>, _>(5);
		assert_eq!(
			fold_right::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
				|a, b| a + b,
				10,
				x
			),
			15
		);
		assert_eq!(
			fold_left::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
				|b, a| b + a,
				10,
				x
			),
			15
		);
		assert_eq!(
			fold_map::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, _, _>(
				|a: i32| a.to_string(),
				x
			),
			"5"
		);

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(1);
		assert_eq!(
			fold_right::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
				|a, b| a + b,
				10,
				brk
			),
			10
		);
	}

	// Traversable Tests

	/// Tests the `traverse` function for `ControlFlowContinueAppliedBrand`.
	///
	/// Verifies that `traverse` correctly maps and sequences effects over `ControlFlow`.
	#[test]
	fn test_traversable_with_continue() {
		let x = pure::<ControlFlowContinueAppliedBrand<()>, _>(5);
		assert_eq!(
			traverse::<RcFnBrand, ControlFlowContinueAppliedBrand<()>, _, _, OptionBrand, _, _>(
				|a| Some(a * 2),
				x
			),
			Some(ControlFlow::Break(10))
		);

		let cont: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		assert_eq!(
			traverse::<RcFnBrand, ControlFlowContinueAppliedBrand<i32>, _, _, OptionBrand, _, _>(
				|a| Some(a * 2),
				cont
			),
			Some(ControlFlow::Continue(1))
		);
	}

	/// Tests the `traverse` function for `ControlFlowBreakAppliedBrand`.
	///
	/// Verifies that `traverse` correctly maps and sequences effects over `ControlFlow`.
	#[test]
	fn test_traversable_with_break() {
		let x = pure::<ControlFlowBreakAppliedBrand<()>, _>(5);
		assert_eq!(
			traverse::<RcFnBrand, ControlFlowBreakAppliedBrand<()>, _, _, OptionBrand, _, _>(
				|a| Some(a * 2),
				x
			),
			Some(ControlFlow::Continue(10))
		);

		let brk: ControlFlow<i32, i32> = ControlFlow::Break(1);
		assert_eq!(
			traverse::<RcFnBrand, ControlFlowBreakAppliedBrand<i32>, _, _, OptionBrand, _, _>(
				|a| Some(a * 2),
				brk
			),
			Some(ControlFlow::Break(1))
		);
	}

	// Monad Laws for ControlFlowContinueAppliedBrand

	/// Verifies the Left Identity law for `ControlFlowContinueAppliedBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_with_continue(a: i32) -> bool {
		let f = |x: i32| pure::<ControlFlowContinueAppliedBrand<i32>, _>(x.wrapping_mul(2));
		bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
			pure::<ControlFlowContinueAppliedBrand<i32>, _>(a),
			f,
		) == f(a)
	}

	/// Verifies the Right Identity law for `ControlFlowContinueAppliedBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_with_continue(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
			x,
			pure::<ControlFlowContinueAppliedBrand<i32>, _>,
		) == x
	}

	/// Verifies the Associativity law for `ControlFlowContinueAppliedBrand` Monad.
	#[quickcheck]
	fn monad_associativity_with_continue(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		let f = |x: i32| pure::<ControlFlowContinueAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<ControlFlowContinueAppliedBrand<i32>, _>(x.wrapping_add(1));
		bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(
			bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(x, f),
			g,
		) == bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(x, |a| {
			bind_explicit::<ControlFlowContinueAppliedBrand<i32>, _, _, _, _>(f(a), g)
		})
	}

	// Monad Laws for ControlFlowBreakAppliedBrand

	/// Verifies the Left Identity law for `ControlFlowBreakAppliedBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_with_break(a: i32) -> bool {
		let f = |x: i32| pure::<ControlFlowBreakAppliedBrand<i32>, _>(x.wrapping_mul(2));
		bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
			pure::<ControlFlowBreakAppliedBrand<i32>, _>(a),
			f,
		) == f(a)
	}

	/// Verifies the Right Identity law for `ControlFlowBreakAppliedBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_with_break(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
			x,
			pure::<ControlFlowBreakAppliedBrand<i32>, _>,
		) == x
	}

	/// Verifies the Associativity law for `ControlFlowBreakAppliedBrand` Monad.
	#[quickcheck]
	fn monad_associativity_with_break(x: ControlFlowWrapper<i32, i32>) -> bool {
		let x = x.into_inner();
		let f = |x: i32| pure::<ControlFlowBreakAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<ControlFlowBreakAppliedBrand<i32>, _>(x.wrapping_add(1));
		bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(
			bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(x, f),
			g,
		) == bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(x, |a| {
			bind_explicit::<ControlFlowBreakAppliedBrand<i32>, _, _, _, _>(f(a), g)
		})
	}

	// Applicative and Monad marker trait verification

	/// Verifies that `ControlFlowContinueAppliedBrand` satisfies the `Applicative` trait.
	#[test]
	fn test_applicative_continue_applied() {
		fn assert_applicative<B: crate::classes::Applicative>() {}
		assert_applicative::<ControlFlowContinueAppliedBrand<i32>>();
	}

	/// Verifies that `ControlFlowBreakAppliedBrand` satisfies the `Applicative` trait.
	#[test]
	fn test_applicative_break_applied() {
		fn assert_applicative<B: crate::classes::Applicative>() {}
		assert_applicative::<ControlFlowBreakAppliedBrand<i32>>();
	}

	/// Verifies that `ControlFlowContinueAppliedBrand` satisfies the `Monad` trait.
	#[test]
	fn test_monad_continue_applied() {
		fn assert_monad<B: crate::classes::Monad>() {}
		assert_monad::<ControlFlowContinueAppliedBrand<i32>>();
	}

	/// Verifies that `ControlFlowBreakAppliedBrand` satisfies the `Monad` trait.
	#[test]
	fn test_monad_break_applied() {
		fn assert_monad<B: crate::classes::Monad>() {}
		assert_monad::<ControlFlowBreakAppliedBrand<i32>>();
	}

	// MonadRec tests for ControlFlowContinueAppliedBrand

	/// Tests the MonadRec identity law for `ControlFlowContinueAppliedBrand`:
	/// `tail_rec_m(|a| Break(Break(a)), x) == Break(x)`.
	#[quickcheck]
	fn monad_rec_continue_applied_identity(x: i32) -> bool {
		tail_rec_m::<ControlFlowContinueAppliedBrand<()>, _, _>(
			|a| ControlFlow::Break(ControlFlow::Break(a)),
			x,
		) == ControlFlow::Break(x)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`
	/// on the break channel of `ControlFlowContinueAppliedBrand`.
	#[test]
	fn monad_rec_continue_applied_sum_range() {
		let result = tail_rec_m::<ControlFlowContinueAppliedBrand<&str>, _, _>(
			|(n, acc)| {
				if n == 0 {
					ControlFlow::Break(ControlFlow::Break(acc))
				} else {
					ControlFlow::Break(ControlFlow::Continue((n - 1, acc + n)))
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, ControlFlow::Break(5050));
	}

	/// Tests that `tail_rec_m` short-circuits on `Continue` for `ControlFlowContinueAppliedBrand`.
	#[test]
	fn monad_rec_continue_applied_short_circuit() {
		let result = tail_rec_m::<ControlFlowContinueAppliedBrand<&str>, _, _>(
			|n| {
				if n == 5 {
					ControlFlow::Continue("stopped")
				} else {
					ControlFlow::Break(ControlFlow::Continue(n + 1))
				}
			},
			0,
		);
		assert_eq!(result, ControlFlow::<i32, &str>::Continue("stopped"));
	}

	/// Tests stack safety: `tail_rec_m` handles large iteration counts
	/// for `ControlFlowContinueAppliedBrand`.
	#[test]
	fn monad_rec_continue_applied_stack_safety() {
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<ControlFlowContinueAppliedBrand<()>, _, _>(
			|acc| {
				if acc < iterations {
					ControlFlow::Break(ControlFlow::Continue(acc + 1))
				} else {
					ControlFlow::Break(ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, ControlFlow::Break(iterations));
	}

	// MonadRec tests for ControlFlowBreakAppliedBrand

	/// Tests the MonadRec identity law for `ControlFlowBreakAppliedBrand`:
	/// `tail_rec_m(|a| Continue(Break(a)), x) == Continue(x)`.
	#[quickcheck]
	fn monad_rec_break_applied_identity(x: i32) -> bool {
		tail_rec_m::<ControlFlowBreakAppliedBrand<()>, _, _>(
			|a| ControlFlow::Continue(ControlFlow::Break(a)),
			x,
		) == ControlFlow::Continue(x)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`
	/// on the continue channel of `ControlFlowBreakAppliedBrand`.
	#[test]
	fn monad_rec_break_applied_sum_range() {
		let result = tail_rec_m::<ControlFlowBreakAppliedBrand<&str>, _, _>(
			|(n, acc)| {
				if n == 0 {
					ControlFlow::Continue(ControlFlow::Break(acc))
				} else {
					ControlFlow::Continue(ControlFlow::Continue((n - 1, acc + n)))
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, ControlFlow::Continue(5050));
	}

	/// Tests that `tail_rec_m` short-circuits on `Break` for `ControlFlowBreakAppliedBrand`.
	#[test]
	fn monad_rec_break_applied_short_circuit() {
		let result = tail_rec_m::<ControlFlowBreakAppliedBrand<&str>, _, _>(
			|n| {
				if n == 5 {
					ControlFlow::Break("stopped")
				} else {
					ControlFlow::Continue(ControlFlow::Continue(n + 1))
				}
			},
			0,
		);
		assert_eq!(result, ControlFlow::<&str, i32>::Break("stopped"));
	}

	/// Tests stack safety: `tail_rec_m` handles large iteration counts
	/// for `ControlFlowBreakAppliedBrand`.
	#[test]
	fn monad_rec_break_applied_stack_safety() {
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<ControlFlowBreakAppliedBrand<()>, _, _>(
			|acc| {
				if acc < iterations {
					ControlFlow::Continue(ControlFlow::Continue(acc + 1))
				} else {
					ControlFlow::Continue(ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, ControlFlow::Continue(iterations));
	}

	// MonadRec marker trait verification

	/// Verifies that `ControlFlowContinueAppliedBrand` satisfies the `MonadRec` trait.
	#[test]
	fn test_monad_rec_continue_applied() {
		fn assert_monad_rec<B: crate::classes::MonadRec>() {}
		assert_monad_rec::<ControlFlowContinueAppliedBrand<i32>>();
	}

	/// Verifies that `ControlFlowBreakAppliedBrand` satisfies the `MonadRec` trait.
	#[test]
	fn test_monad_rec_break_applied() {
		fn assert_monad_rec<B: crate::classes::MonadRec>() {}
		assert_monad_rec::<ControlFlowBreakAppliedBrand<i32>>();
	}

	/// Tests the `break_val` method.
	///
	/// Verifies that `break_val` returns `Some(b)` for `Break(b)` and `None` for `Continue(_)`.
	#[test]
	fn test_break_val() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Break(42);
		assert_eq!(ControlFlowBrand::break_val(cf), Some(42));

		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(1);
		assert_eq!(ControlFlowBrand::break_val(cf), None);
	}

	/// Tests the `continue_val` method.
	///
	/// Verifies that `continue_val` returns `Some(c)` for `Continue(c)` and `None` for `Break(_)`.
	#[test]
	fn test_continue_val() {
		let cf: ControlFlow<i32, i32> = ControlFlow::Continue(7);
		assert_eq!(ControlFlowBrand::continue_val(cf), Some(7));

		let cf: ControlFlow<i32, i32> = ControlFlow::Break(42);
		assert_eq!(ControlFlowBrand::continue_val(cf), None);
	}

	/// Tests the `swap` method.
	///
	/// Verifies that `swap` maps `Continue(c)` to `Break(c)` and `Break(b)` to `Continue(b)`.
	#[test]
	fn test_swap() {
		let cf: ControlFlow<&str, i32> = ControlFlow::Continue(1);
		assert_eq!(ControlFlowBrand::swap(cf), ControlFlow::Break(1));

		let cf: ControlFlow<&str, i32> = ControlFlow::Break("hello");
		assert_eq!(ControlFlowBrand::swap(cf), ControlFlow::Continue("hello"));
	}

	/// Property test: `break_val` and `continue_val` are complementary.
	#[quickcheck]
	fn break_and_continue_val_complementary(x: ControlFlowWrapper<i32, i32>) -> bool {
		let cf = x.into_inner();
		ControlFlowBrand::break_val(cf).is_some() != ControlFlowBrand::continue_val(cf).is_some()
	}

	/// Property test: swapping twice is identity.
	#[quickcheck]
	fn swap_involution(x: ControlFlowWrapper<i32, i32>) -> bool {
		let cf = x.into_inner();
		ControlFlowBrand::swap(ControlFlowBrand::swap(cf)) == cf
	}
}
