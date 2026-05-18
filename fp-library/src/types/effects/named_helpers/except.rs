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
				NodeBrand,
			},
			classes::{
				Functor,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				Coyoneda,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					arc_run_explicit::ArcRunExplicit,
					except::Except,
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
				},
				rc_free::RcTypeErasedValue,
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

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> RunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Throws unit in an Except row.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("A `RunExplicit` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::fail();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn fail<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<()>, A>, Idx>, {
			Self::throw::<(), Idx>(())
		}

		/// Converts a Rust `Result` into an Except program.
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
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program with a supplied error.
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
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::note::<&'static str, _>("missing", None);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::from_option(None);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<()>, A>, Idx>, {
			Self::note::<(), Idx>((), value)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` program to interpret.")]
	impl<'a, R, A> RunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one Except effect into a Rust `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::throw::<&'static str, _>("missing");
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> RunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					Coyoneda<
						'a,
						ExceptBrand<ErrorType>,
						RunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'a,
					ErrorType,
					RunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => RunExplicit::pure(Err(error)),
					}
				},
			)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> RcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		/// Throws unit in an Except row.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `RcRunExplicit` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> = RcRunExplicit::fail();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn fail<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			Self::throw::<(), Idx>(())
		}

		/// Converts a Rust `Result` into an Except program.
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
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program with a supplied error.
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
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::note::<&'static str, _>("missing", None);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> = RcRunExplicit::from_option(None);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			Self::note::<(), Idx>((), value)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` program to interpret.")]
	impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		/// Interprets one Except effect into a Rust `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::throw::<&'static str, _>("missing");
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> RcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					RcCoyoneda<
						'a,
						ExceptBrand<ErrorType>,
						RcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'a,
					ErrorType,
					RcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => RcRunExplicit::pure(Err(error)),
					}
				},
			)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> ArcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
	{
		/// Throws unit in an Except row.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `ArcRunExplicit` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> = ArcRunExplicit::fail();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn fail<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			Self::throw::<(), Idx>(())
		}

		/// Converts a Rust `Result` into an Except program.
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
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + Send + Sync + 'static, Idx>(
			result: Result<A, ErrorType>
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program with a supplied error.
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
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::note::<&'static str, _>("missing", None);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + Send + Sync + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> = ArcRunExplicit::from_option(None);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			Self::note::<(), Idx>((), value)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` program to interpret.")]
	impl<'a, R, A> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
	{
		/// Interprets one Except effect into a Rust `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::throw::<&'static str, _>("missing");
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + Send + Sync + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusExcept, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					ArcCoyoneda<
						'a,
						ExceptBrand<ErrorType>,
						ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'a,
					ErrorType,
					ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => ArcRunExplicit::pure(Err(error)),
					}
				},
			)
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> ArcRun<R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Clone + Send + Sync + 'static,
	{
		/// Throws unit in an Except row.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `ArcRun` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::fail();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn fail<Idx>() -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			Self::throw::<(), Idx>(())
		}

		/// Converts a Rust `Result` into an Except program.
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
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + Send + Sync + 'static, Idx>(
			result: Result<A, ErrorType>
		) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program with a supplied error.
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
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::note::<&'static str, _>("missing", None);
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + Send + Sync + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::from_option(None);
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			Self::note::<(), Idx>((), value)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `ArcRun` program to interpret.")]
	impl<R, A> ArcRun<R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Clone + Send + Sync + 'static,
	{
		/// Interprets one Except effect into a Rust `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::throw::<&'static str, _>("missing");
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> ArcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusExcept, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<
						'static,
						ArcFree<NodeBrand<RMinusExcept, CNilBrand>, ArcTypeErasedValue>,
					>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusExcept, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					ArcCoyoneda<
						'static,
						ExceptBrand<ErrorType>,
						ArcRun<R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Result<A, ErrorType>>,
									>
								),
		>,{
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'static,
					ErrorType,
					ArcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => ArcRun::pure(Err(error)),
					}
				},
			)
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		/// Throws unit in an Except row.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `RcRun` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::fail();
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn fail<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			Self::throw::<(), Idx>(())
		}

		/// Converts a Rust `Result` into an Except program.
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program with a supplied error.
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
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::note::<&'static str, _>("missing", None);
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}

		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::from_option(None);
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			Self::note::<(), Idx>((), value)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` program to interpret.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		/// Interprets one Except effect into a Rust `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::throw::<&'static str, _>("missing");
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> RcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusExcept, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					RcCoyoneda<
						'static,
						ExceptBrand<ErrorType>,
						RcRun<R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'static,
					ErrorType,
					RcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => RcRun::pure(Err(error)),
					}
				},
			)
		}
	}
}
