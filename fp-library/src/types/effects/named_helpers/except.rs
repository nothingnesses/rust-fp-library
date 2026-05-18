//! Named Except helpers layered over the Run wrapper primitives.
//!
//! These helpers expose Rust `Result` and `Option` shaped conveniences for the
//! existing `throw` and `handle_with` machinery.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				ExceptBrand,
			},
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Coyoneda,
				effects::{
					except::Except,
					member::Member,
					run::Run,
				},
			},
		},
		fp_macros::*,
	};

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> Run<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Throws unit in an Except row.
		///
		/// `fail()` is the unit-error variant of [`Run::throw`]. It mirrors
		/// PureScript Run's
		/// [`fail`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/src/Run/Except.purs)
		/// helper while using Rust's unit type as the error payload.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("A `Run` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::fail();
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn fail<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<()>, A>, Idx>, {
			Self::throw::<(), Idx>(())
		}

		/// Converts a Rust `Result` into an Except program.
		///
		/// `Ok(value)` becomes a pure program. `Err(error)` becomes
		/// a thrown `ExceptBrand<ErrorType>` effect.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program with a supplied error.
		///
		/// `Some(value)` becomes a pure program. `None` throws the supplied
		/// error. This is Rust's `Option`-shaped analogue of PureScript Run's
		/// [`note`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/src/Run/Except.purs)
		/// helper.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::note::<&'static str, _>("missing", None);
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program that throws unit for `None`.
		///
		/// This is the Rust-named equivalent of PureScript Run's
		/// [`fromJust`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/src/Run/Except.purs)
		/// helper. The name uses `Option` because that is the Rust type being
		/// converted.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::from_option(None);
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<()>, A>, Idx>, {
			Self::note::<(), Idx>((), value)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one Except effect into a Rust `Result`.
		///
		/// A pure result becomes `Ok(result)`. A thrown error becomes
		/// `Err(error)`. Other first-order effects remain in the narrowed
		/// row and continue to be represented by the returned `Run` program.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `Run` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::throw::<&'static str, _>("missing");
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> Run<RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					Coyoneda<
						'static,
						ExceptBrand<ErrorType>,
						Run<R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'static,
					ErrorType,
					Run<RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => Run::pure(Err(error)),
					}
				},
			)
		}
	}
}
