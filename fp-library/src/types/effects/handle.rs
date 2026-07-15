//! The generic interpretation surface: the narrowing accumulator runner and
//! the terminal extractor.
//!
//! A narrowing runner eliminates one effect brand from a row and returns the
//! residual program over the remaining cells, so runners stack in any order
//! and handler-ordering semantics (which effect scopes which) fall out of the
//! stacking order. [`handle_accum`] is the generic core: it threads an
//! accumulator through the eliminated effect's operations iteratively, and
//! re-emits every unmatched layer into the residual row with the recursive
//! continuation deferred into `bind`, so the walk is constant-stack per step
//! regardless of program length. [`AccumStep`] is the step abstraction for
//! runners that fork interpretation into owned sub-programs and so re-apply
//! one step at several program result types, which a closure cannot express.
//! [`extract`] closes a fully narrowed pipeline: once every cell is
//! eliminated the residual row is [`CNilBrand`](crate::brands::CNilBrand)
//! and the program is necessarily a pure value. [`run_cont`] is the
//! continuation-passing exit instead: it folds a whole program into a
//! callback target, the shape an external executor consumes.
//!
//! The one-pass handler surface lives on three traits: [`RowHandler`] is
//! the loop (a handler value drives any program over its row to a value or
//! the row abort), [`HandlerPieces`] carries each effect's emitted arm
//! bundle and dispatch across the macro seam, and [`EffectAbort`] carries
//! each effect's abort contribution to the row's abort union.

pub mod multi_shot;

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::CNilBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::LifetimeUnaryKind,
			types::{
				Coyoneda,
				Free,
				effects::coproduct::{
					CoprodUninjector,
					CoproductEmbedder,
				},
			},
		},
		fp_macros::*,
	};

	/// Eliminates one effect brand from a row by threading an accumulator
	/// through its operations; every other layer is re-emitted into the
	/// residual row with the continuation deferred, so unmatched effects run
	/// under whatever interpreter later drives the residual program.
	///
	/// The step function receives the current accumulator and one lowered
	/// operation of the eliminated effect, and returns the new accumulator
	/// and the continuation program (usually by invoking the operation's
	/// resume function). Matched operations are driven iteratively; an
	/// unmatched layer is embedded into the residual row with its holes as
	/// results, and the runner re-enters through `bind`, so native stack use
	/// per step is constant. Sequential threading moves the accumulator, so
	/// no `Clone` bound is needed here; runners that fork interpretation
	/// (nondeterministic choice) clone the accumulator per branch instead.
	///
	/// The two index parameters position the eliminated brand inside the
	/// row's coproduct and the remainder inside the residual row; both are
	/// inferred at every call site (turbofish as
	/// `handle_accum::<EffectBrand, _, _, _, _, _, _>`); the eliminated brand
	/// always needs naming, because it appears in the signature only through
	/// its kind projection, which inference cannot invert.
	#[document_signature]
	///
	#[document_type_parameters(
		"The eliminated effect brand.",
		"The source row brand.",
		"The residual row brand.",
		"The accumulator type.",
		"The program's result type.",
		"The coproduct index locating the eliminated brand (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The initial accumulator.",
		"The program to interpret.",
		"The step function applied to each matched operation."
	)]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the final accumulator paired with the program's result."
	)]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_effect,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::handle::{
	/// 			extract,
	/// 			handle_accum,
	/// 		},
	/// 	},
	/// };
	///
	/// define_effect! {
	/// 	/// A running total.
	/// 	#[handler_state(threaded_by_value)]
	/// 	pub effect Counter {
	/// 		/// Add `amount` to the total, resuming with the new total.
	/// 		fn add(amount: i32) -> i32;
	/// 	}
	/// }
	///
	/// define_row! {
	/// 	/// The one-cell row.
	/// 	pub row CounterRow {
	/// 		CounterBrand,
	/// 	}
	/// }
	///
	/// let program: Free<CounterRow, i32> = add(2).bind(|_| add(3));
	/// let narrowed: Free<CNilBrand, (i32, i32)> =
	/// 	handle_accum::<CounterBrand, _, _, _, _, _, _>(0, program, |s, op| match op {
	/// 		CounterF::Add(amount, resume) => (s + amount, resume(s + amount)),
	/// 	});
	/// assert_eq!(extract(narrowed), (5, 5));
	/// ```
	pub fn handle_accum<EBrand, Row, Narrow, S, A, UninjectIndex, EmbedIndices>(
		mut s: S,
		mut program: Free<Row, A>,
		step: impl Fn(S, <EBrand as LifetimeUnaryKind>::Of<'static, Free<Row, A>>) -> (S, Free<Row, A>)
		+ 'static,
	) -> Free<Narrow, (S, A)>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		EBrand: LifetimeUnaryKind + Functor + 'static,
		S: 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, EBrand, Free<Row, A>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>, {
		loop {
			let layer = match program.resume() {
				Ok(value) => return Free::pure((s, value)),
				Err(layer) => layer,
			};
			let selected: Result<Coyoneda<'static, EBrand, Free<Row, A>>, _> = layer.uninject();
			match selected {
				Ok(op) => {
					let (next_s, next_program) = step(s, op.lower());
					s = next_s;
					program = next_program;
				}
				Err(rest) => {
					let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
						rest.embed();
					// The Box-store `bind` arm must be selected explicitly:
					// `bind` is defined per store, and in generic position the
					// method call is ambiguous until the receiver's store is
					// pinned.
					let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
					return lifted.bind(move |rest_program| handle_accum(s, rest_program, step));
				}
			}
		}
	}

	/// The step for runners that re-apply it at more than one program result
	/// type. [`handle_accum`]'s closure step is applied at one result type
	/// only; a runner that forks interpretation into owned sub-programs (the
	/// scoped
	/// [`handle_choose_accum`](crate::types::effects::choose::handle_choose_accum))
	/// re-applies its step at each sub-program's own result type, which a
	/// closure cannot express because closures are monomorphic, so such a
	/// step is a named type with a generic method. Implementations are
	/// `Clone` because a forking runner uses the step once per branch and
	/// once for the trunk.
	#[document_type_parameters(
		"The eliminated effect brand.",
		"The row brand the interpreted programs run over.",
		"The accumulator type."
	)]
	#[document_parameters("The step value.")]
	pub trait AccumStep<EBrand: LifetimeUnaryKind, Row: WrapDrop + 'static, S>:
		Clone + 'static {
		/// Interprets one lowered operation of the eliminated effect,
		/// returning the new accumulator and the continuation program
		/// (usually by invoking the operation's resume function).
		#[document_signature]
		///
		#[document_type_parameters("The program result type this application interprets at.")]
		///
		#[document_parameters("The current accumulator.", "The lowered operation to interpret.")]
		///
		#[document_returns("The new accumulator paired with the continuation program.")]
		#[document_examples(
			skip_call_check,
			reason = "A step is consumed by a forking runner rather than called directly; the example demonstrates the trait's role by driving `handle_choose_accum` with an implementation."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	define_row,
		/// 	types::{
		/// 		Free,
		/// 		effects::{
		/// 			choose::{
		/// 				ChooseBrand,
		/// 				choose,
		/// 				handle_choose_accum,
		/// 			},
		/// 			handle::extract,
		/// 			state::{
		/// 				StateBrand,
		/// 				StateStep,
		/// 				get,
		/// 				put,
		/// 			},
		/// 		},
		/// 	},
		/// };
		///
		/// define_row! {
		/// 	/// Integer choice alongside integer state.
		/// 	pub row ChoiceStateRow {
		/// 		ChooseBrand<ChoiceStateRow, i32>,
		/// 		StateBrand<i32>,
		/// 	}
		/// }
		///
		/// // `StateStep` implements the trait, so the forking runner applies
		/// // it inside each branch and for the trunk: the left branch's write
		/// // is branch-local and the right branch reads the untouched fork.
		/// let program: Free<ChoiceStateRow, Vec<i32>> =
		/// 	choose(put(10).bind(|()| get()), get()).bind(|values: Vec<i32>| Free::pure(values));
		/// let narrowed: Free<CNilBrand, (i32, Option<Vec<i32>>)> =
		/// 	handle_choose_accum(1, program, StateStep);
		/// assert_eq!(extract(narrowed), (1, Some(vec![10, 1])));
		/// ```
		fn step<T: 'static>(
			&self,
			s: S,
			op: <EBrand as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		) -> (S, Free<Row, T>);
	}

	/// The abort type an effect contributes to a row's abort union: one
	/// variant per no-resume operation, carrying that operation's payloads,
	/// and uninhabited when every operation resumes. `define_effect!` emits
	/// the type and this implementation for every effect brand; the
	/// `define_row!` handler extension builds the row's abort enum out of
	/// its cells' projections. The projection lives on its own trait, apart
	/// from [`HandlerPieces`], because the row abort enum's variants must
	/// name each cell's abort type while that enum is being defined, and
	/// reaching them through a trait parameterised by the row abort itself
	/// would be a definition cycle.
	pub trait EffectAbort {
		/// The effect's abort payload type.
		type Abort;
	}

	/// A value that can drive any program over a row to its outcome: the
	/// one-pass interpretation loop of the handler surface. The handler
	/// struct the `define_row!` handler extension emits implements this by
	/// brand-keyed dispatch over its arm fields, and higher-order
	/// elaboration re-enters interpretation through it, which is why the
	/// method is generic over the program result type (one handler value
	/// drives the top-level program and every owned sub-program, whatever
	/// their result types).
	#[document_type_parameters(
		"The row brand the handled programs run over.",
		"The row's abort union."
	)]
	#[document_parameters("The handler value.")]
	pub trait RowHandler<Row: WrapDrop + 'static, RowAbort> {
		/// Runs a program over the row to its value, or to the row abort a
		/// no-resume operation reified.
		#[document_signature]
		///
		#[document_type_parameters("The program's result type.")]
		///
		#[document_parameters("The program to interpret.")]
		///
		#[document_returns("The program's value, or the abort that ended interpretation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	define_effect,
		/// 	define_row,
		/// 	types::{
		/// 		Free,
		/// 		effects::handle::RowHandler,
		/// 	},
		/// };
		///
		/// define_effect! {
		/// 	/// A running tally.
		/// 	#[handler_state(shared_by_reference)]
		/// 	pub effect Tally {
		/// 		/// Add `amount` to the total, resuming with the new total.
		/// 		fn add(amount: i32) -> i32;
		/// 	}
		/// }
		///
		/// define_row! {
		/// 	/// The one-cell row.
		/// 	#[handlers]
		/// 	pub row TallyRow {
		/// 		TallyBrand,
		/// 	}
		/// }
		///
		/// let total = std::cell::Cell::new(0);
		/// let handlers = TallyRowHandlers {
		/// 	tally: TallyArms {
		/// 		add: Box::new(|amount| {
		/// 			total.set(total.get() + amount);
		/// 			total.get()
		/// 		}),
		/// 	},
		/// };
		/// let program: Free<TallyRow, i32> = add(2).bind(|_| add(3));
		/// assert_eq!(handlers.handle(program).ok(), Some(5));
		/// ```
		fn handle<T: 'static>(
			&self,
			program: Free<Row, T>,
		) -> Result<T, RowAbort>;
	}

	/// The per-effect handler pieces `define_effect!` emits, exposed on the
	/// brand so the `define_row!` handler extension reaches them by
	/// path-resolved projection (a row macro cannot see the effect macros'
	/// operation inventories, so per-effect knowledge crosses the seam as
	/// associated items): the arm bundle a handler stores for the effect,
	/// and the dispatch function that interprets one lowered operation
	/// against it. First-order resumptive operations receive
	/// payloads-to-resume-value arms and dispatch applies the continuation
	/// itself; no-resume operations have no arm and reify into the row
	/// abort through [`EffectAbort`]; higher-order operations receive their
	/// owned sub-programs plus re-entry handles dispatch builds over the
	/// [`RowHandler`], so their arms can interpret sub-programs and
	/// selectively recover from the re-entry's aborts.
	#[document_type_parameters(
		"The row brand the handled programs run over.",
		"The row's abort union."
	)]
	pub trait HandlerPieces<Row: WrapDrop + 'static, RowAbort>:
		EffectAbort + LifetimeUnaryKind + Sized {
		/// The effect's arm bundle over borrowed handler state.
		type Arms<'h>;

		/// Interprets one lowered operation against the arms, returning the
		/// continuation program, or the abort to propagate. A no-resume
		/// operation's payloads become the effect's abort, injected into the
		/// row abort by the caller-supplied injection (the emitted loop
		/// passes the row abort's variant constructor for the cell); the
		/// injection is a parameter rather than a `From` bound because
		/// conversion impls headed by associated-type projections cannot be
		/// proven disjoint from the reflexive `From` impl.
		#[document_signature]
		///
		#[document_type_parameters("The program result type this dispatch interprets at.")]
		///
		#[document_parameters(
			"The lowered operation.",
			"The effect's arm bundle.",
			"The row handler driving interpretation.",
			"The injection from the effect's abort into the row abort."
		)]
		///
		#[document_returns("The continuation program, or the abort that ends interpretation.")]
		#[document_examples(
			skip_call_check,
			reason = "Dispatch is consumed by the emitted one-pass loop rather than called directly; the example demonstrates it through the handler surface it powers."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	define_effect,
		/// 	define_row,
		/// 	types::{
		/// 		Free,
		/// 		effects::handle::RowHandler,
		/// 	},
		/// };
		///
		/// define_effect! {
		/// 	/// A single prompt.
		/// 	#[handler_state(none)]
		/// 	pub effect Prompt {
		/// 		/// Ask for the line behind `key`, resuming with it.
		/// 		fn ask(key: &'static str) -> String;
		/// 	}
		/// }
		///
		/// define_row! {
		/// 	/// The one-cell row.
		/// 	#[handlers]
		/// 	pub row PromptRow {
		/// 		PromptBrand,
		/// 	}
		/// }
		///
		/// let handlers = PromptRowHandlers {
		/// 	prompt: PromptArms {
		/// 		ask: Box::new(|key| format!("{key}!")),
		/// 	},
		/// };
		/// let program: Free<PromptRow, String> = ask("hello");
		/// assert_eq!(handlers.handle(program).ok(), Some("hello!".to_string()));
		/// ```
		fn dispatch<T: 'static>(
			op: <Self as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
			arms: &Self::Arms<'_>,
			handler: &impl RowHandler<Row, RowAbort>,
			inject_abort: impl Fn(<Self as EffectAbort>::Abort) -> RowAbort,
		) -> Result<Free<Row, T>, RowAbort>;
	}

	/// Extracts the value from a fully narrowed program: over the empty row
	/// no operation can be suspended, so the program is necessarily a pure
	/// value and the suspended case is uninhabited.
	#[document_signature]
	///
	#[document_type_parameters("The program's result type.")]
	///
	#[document_parameters("The fully narrowed program.")]
	///
	#[document_returns("The program's value.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::{
	/// 		Free,
	/// 		effects::handle::extract,
	/// 	},
	/// };
	///
	/// let program: Free<CNilBrand, i32> = Free::pure(7);
	/// assert_eq!(extract(program), 7);
	/// ```
	pub fn extract<A: 'static>(program: Free<CNilBrand, A>) -> A {
		match program.resume() {
			Ok(value) => value,
			Err(layer) => match layer {},
		}
	}

	/// Extracts the value from a program via continuation passing, the
	/// purescript-run `runCont` shape: on suspension, `on_suspend` receives
	/// the whole row layer with every continuation already folded into a
	/// `B`-producing application (deferred by the row's `Functor`), and on
	/// completion `on_pure` receives the final value. The callback owns each
	/// step, so it can force a continuation immediately (a synchronous
	/// drive) or store the force and return (a scheduling driver, the
	/// callback-target shape an external executor consumes).
	///
	/// Native stack use grows with the number of continuations forced in one
	/// synchronous chain, because each force re-enters the driver inside the
	/// callback's frame; a scheduling callback that defers each force into
	/// an external loop drives arbitrarily deep programs with constant
	/// native stack.
	#[document_signature]
	///
	#[document_type_parameters(
		"The row brand the program runs over.",
		"The program's result type.",
		"The callback target type."
	)]
	///
	#[document_parameters(
		"The program to drive.",
		"The operation callback, receiving the row layer with its continuations folded to the target.",
		"The completion callback, receiving the final value."
	)]
	///
	#[document_returns("The callback target's value for the whole program.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	define_effect,
	/// 	define_row,
	/// 	types::{
	/// 		Coyoneda,
	/// 		Free,
	/// 		effects::handle::run_cont,
	/// 	},
	/// };
	///
	/// define_effect! {
	/// 	/// Doubles a number.
	/// 	#[handler_state(none)]
	/// 	pub effect Double {
	/// 		/// Resume with twice `value`.
	/// 		fn double(value: i32) -> i32;
	/// 	}
	/// }
	///
	/// define_row! {
	/// 	/// The one-cell row.
	/// 	pub row MathRow {
	/// 		DoubleBrand,
	/// 	}
	/// }
	///
	/// let program: Free<MathRow, i32> = double(2).bind(|four| double(four));
	/// let result = run_cont(
	/// 	program,
	/// 	|layer| {
	/// 		let cell: Coyoneda<'static, DoubleBrand, i32> = match layer.uninject() {
	/// 			Ok(cell) => cell,
	/// 			Err(terminal) => match terminal {},
	/// 		};
	/// 		match cell.lower() {
	/// 			DoubleF::Double(value, resume) => resume(value * 2),
	/// 		}
	/// 	},
	/// 	|value| value,
	/// );
	/// assert_eq!(result, 8);
	/// ```
	pub fn run_cont<Row, A, B>(
		program: Free<Row, A>,
		on_suspend: impl Fn(<Row as LifetimeUnaryKind>::Of<'static, B>) -> B + Clone + 'static,
		on_pure: impl Fn(A) -> B + Clone + 'static,
	) -> B
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: 'static,
		B: 'static, {
		match program.resume() {
			Ok(value) => on_pure(value),
			Err(layer) => {
				let suspend = on_suspend.clone();
				let pure = on_pure.clone();
				let folded: <Row as LifetimeUnaryKind>::Of<'static, B> = <Row as Functor>::map(
					move |next: Free<Row, A>| run_cont(next, suspend.clone(), pure.clone()),
					layer,
				);
				on_suspend(folded)
			}
		}
	}
}

pub use inner::*;
