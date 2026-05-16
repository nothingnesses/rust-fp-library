use {
	super::*,
	crate::{
		Apply,
		brands::{
			BoxBracketExplicitBrand,
			BoxBrand,
			BoxCatchBrand,
			BoxLocalBrand,
			BoxReaderBrand,
			BoxRefLocalBrand,
			BoxSpanBrand,
			BoxWriterListenBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			ExceptBrand,
			IdentityBrand,
			NodeBrand,
			RcBrand,
			RunExplicitBrand,
		},
		classes::{
			Functor,
			Pointed,
			RefCountedPointer,
			RefFunctor,
			RefPointed,
			RefSemimonad,
			Semimonad,
			ToDynFnOnce,
			WrapDrop,
		},
		impl_kind,
		kinds::{
			InferableBrand_266801a817966495,
			Kind_266801a817966495,
			Kind_cdc7cd43dac7585f,
		},
		types::{
			FreeExplicit,
			effects::{
				bracket::BoxBracketExplicit,
				catch::BoxCatch,
				coproduct::{
					CNil,
					Coproduct,
					Here,
				},
				except::Except,
				handlers::HandlersNil,
				interpreter::{
					ExplicitBoundaryOf,
					ExplicitBoundaryTypes,
					ScopedBoundaryTypes,
					ScopedContinuation,
				},
				local::BoxLocal,
				node::Node,
				reader::BoxReader,
				ref_local::BoxRefLocal,
				span::BoxSpan,
				standard_scoped_handlers::{
					bracket_handler,
					catch_handler,
					local_handler,
					ref_bracket_handler,
					ref_local_handler,
					span_handler,
				},
				writer::BoxWriterListen,
			},
		},
	},
	core::{
		cell::RefCell,
		marker::PhantomData,
	},
	std::rc::Rc,
};

type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
type IdentityFirstOrderRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type Scoped = CNilBrand;
type RunAlias<'a, A> = RunExplicit<'a, FirstRow, Scoped, A>;
type EmptyRunExplicit<'a, A> = RunExplicit<'a, CNilBrand, CNilBrand, A>;
type BoxReaderRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type BoxReaderRowMinusReader = CNilBrand;
type BoxReaderRunExplicit<'a, A> = RunExplicit<'a, BoxReaderRow, CNilBrand, A>;
type BoxExceptRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type BoxExceptRowMinusExcept = CNilBrand;
type BoxExceptRunExplicit<'a, A> = RunExplicit<'a, BoxExceptRow, CNilBrand, A>;
type LocalBoundaryScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
type LocalBoundarySBrand = BoxLocalBrand<BoxBrand, i32>;
type LocalBoundaryRunExplicit<'a, A> = RunExplicit<'a, BoxReaderRow, LocalBoundaryScopedRow, A>;
type LocalBoundaryLayer<'a, A> =
	Coproduct<BoxLocal<'a, BoxBrand, i32, LocalBoundaryRunExplicit<'a, A>>, CNil>;
type RefLocalBoundaryScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
type RefLocalBoundarySBrand = BoxRefLocalBrand<BoxBrand, i32>;
type RefLocalBoundaryRunExplicit<'a, A> =
	RunExplicit<'a, BoxReaderRow, RefLocalBoundaryScopedRow, A>;
type RefLocalBoundaryLayer<'a, A> =
	Coproduct<BoxRefLocal<'a, BoxBrand, i32, RefLocalBoundaryRunExplicit<'a, A>>, CNil>;
type CatchBoundaryScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
type CatchBoundarySBrand = BoxCatchBrand<BoxBrand, &'static str>;
type CatchBoundaryRunExplicit<'a, A> = RunExplicit<'a, BoxExceptRow, CatchBoundaryScopedRow, A>;
type CatchBoundaryLayer<'a, A> =
	Coproduct<BoxCatch<'a, BoxBrand, &'static str, CatchBoundaryRunExplicit<'a, A>>, CNil>;
type BorrowedSpanScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
type BorrowedSpanSBrand = BoxSpanBrand<BoxBrand, &'static str>;
type BorrowedSpanRunExplicit<'a, A> = RunExplicit<'a, CNilBrand, BorrowedSpanScopedRow, A>;
type BorrowedSpanLayer<'a, A> =
	Coproduct<BoxSpan<'a, BoxBrand, &'static str, BorrowedSpanRunExplicit<'a, A>>, CNil>;
type BorrowedSpanFreeCell<'a, A> =
	Box<FreeExplicit<'a, NodeBrand<CNilBrand, BorrowedSpanScopedRow>, A>>;
type WriterListenScopedRow = CoproductBrand<BoxWriterListenBrand<BoxBrand, String, i32>, CNilBrand>;
type WriterListenSBrand = BoxWriterListenBrand<BoxBrand, String, i32>;
type WriterListenRunExplicit<'a, A> = RunExplicit<'a, CNilBrand, WriterListenScopedRow, A>;
type WriterListenLayer<'a, A> =
	Coproduct<BoxWriterListen<'a, BoxBrand, String, i32, WriterListenRunExplicit<'a, A>>, CNil>;
type IdentitySpanRunExplicit<'a, A> =
	RunExplicit<'a, IdentityFirstOrderRow, BorrowedSpanScopedRow, A>;
type DelayedBorrowedSpanPeel<'a, Action, Final, K> = Result<
	BorrowedSpanRunExplicit<'a, Final>,
	(BorrowedSpanLayer<'a, Action>, DelayedBorrowedSpanContinuation<'a, Action, Final, K>),
>;
type TypedBorrowedSpanActionProgram<'a, Action> = BorrowedSpanRunExplicit<'a, Action>;
type TypedBorrowedSpanFinalProgram<'a, Final> = BorrowedSpanRunExplicit<'a, Final>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BoundaryBracketRow;

type BoundaryBracketUnderlyingRow = CoproductBrand<
	BoxBracketExplicitBrand<BoxBrand, NodeBrand<CNilBrand, BoundaryBracketRow>, i32, i32>,
	CNilBrand,
>;
type BoundaryBracketSBrand =
	BoxBracketExplicitBrand<BoxBrand, NodeBrand<CNilBrand, BoundaryBracketRow>, i32, i32>;
type BoundaryBracketRunExplicit<'a, A> = RunExplicit<'a, CNilBrand, BoundaryBracketRow, A>;
type BoundaryBracketLayer<'a> = Coproduct<
	BoxBracketExplicit<'a, BoxBrand, NodeBrand<CNilBrand, BoundaryBracketRow>, i32, i32>,
	CNil,
>;

impl_kind! {
	impl for BoundaryBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<BoundaryBracketUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for BoundaryBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<BoundaryBracketUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for BoundaryBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<BoundaryBracketUnderlyingRow as Functor>::map(f, fa)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TypedBorrowedSpanBoundaryBrand;

impl_kind! {
	impl for TypedBorrowedSpanBoundaryBrand {
		type Of<'a, Action: 'a, Final: 'a>: 'a =
			TypedBorrowedSpanBoundary<'a, Action, Final>;
	}
}

impl<'a, Action: 'a, Final: 'a> ScopedBoundaryTypes<'a, Action, Final>
	for TypedBorrowedSpanBoundaryBrand
{
	type ActionProgram = TypedBorrowedSpanActionProgram<'a, Action>;
	type FinalProgram = TypedBorrowedSpanFinalProgram<'a, Final>;
}

// Test-only substrate boundary prototype for the B49 Option B path.
// Unlike the old direct `RunExplicit<Final>` shape, this boundary keeps
// the selected action program and the final program as separate type-level
// slots before any production representation is chosen.
enum TypedBorrowedSpanBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a, {
	Scoped {
		layer: BorrowedSpanLayer<'a, Action>,
		continuation: TypedBorrowedSpanContinuation<'a, Action, Final>,
	},
}

struct TypedBorrowedSpanContinuation<'a, Action, Final>
where
	Action: 'a,
	Final: 'a, {
	outer: Rc<dyn Fn(Action) -> TypedBorrowedSpanFinalProgram<'a, Final> + 'a>,
	result: PhantomData<fn(Action) -> Final>,
}

// Keeps a scoped source program and its outer continuation as a typed
// frame instead of immediately distributing the continuation through
// `RunExplicit::bind`. This is the substrate boundary the carrier
// interpreter path needs: a scoped row projection at the selected action
// program type plus a separately typed outer continuation.
struct DelayedBorrowedSpanFrame<'a, Action, Final, K>
where
	Action: 'a,
	Final: 'a,
	K: Fn(Action) -> BorrowedSpanRunExplicit<'a, Final> + 'a, {
	source: BorrowedSpanRunExplicit<'a, Action>,
	continuation: DelayedBorrowedSpanContinuation<'a, Action, Final, K>,
}

struct DelayedBorrowedSpanContinuation<'a, Action, Final, K>
where
	Action: 'a,
	Final: 'a,
	K: Fn(Action) -> BorrowedSpanRunExplicit<'a, Final> + 'a, {
	outer: Rc<K>,
	result: PhantomData<fn(Action) -> Final>,
}

impl<'a, Action, Final> TypedBorrowedSpanBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a,
{
	fn bind<Next>(
		self,
		f: impl Fn(Final) -> TypedBorrowedSpanFinalProgram<'a, Next> + 'a,
	) -> TypedBorrowedSpanBoundary<'a, Action, Next>
	where
		Next: 'a, {
		let f = Rc::new(f);
		let Self::Scoped {
			layer,
			continuation,
		} = self;
		let outer = continuation.outer.clone();
		let composed = move |action_value: Action| {
			let f = f.clone();
			outer(action_value).bind(move |final_value| f(final_value))
		};

		TypedBorrowedSpanBoundary::Scoped {
			layer,
			continuation: TypedBorrowedSpanContinuation {
				outer: Rc::new(composed),
				result: PhantomData,
			},
		}
	}
}

impl<'a, Action> TypedBorrowedSpanBoundary<'a, Action, Action>
where
	Action: 'a,
{
	fn span(
		tag: &'static str,
		action: TypedBorrowedSpanActionProgram<'a, Action>,
	) -> Self {
		let layer = Coproduct::Inl(BoxSpan::Span {
			tag,
			action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
		});
		Self::Scoped {
			layer,
			continuation: TypedBorrowedSpanContinuation {
				outer: Rc::new(RunExplicit::pure),
				result: PhantomData,
			},
		}
	}
}

impl<'a, Action, Final> TypedBorrowedSpanContinuation<'a, Action, Final>
where
	Action: 'a,
	Final: 'a,
{
	fn resume_with_post_action(
		self,
		action: TypedBorrowedSpanActionProgram<'a, Action>,
		post_action: impl Fn(Action) -> TypedBorrowedSpanActionProgram<'a, Action> + 'a,
	) -> TypedBorrowedSpanFinalProgram<'a, Final> {
		let outer = self.outer.clone();
		action.bind(move |action_value| {
			let outer = outer.clone();
			post_action(action_value).bind(move |post_value| outer(post_value))
		})
	}
}

impl<'a, Action, Final, K> DelayedBorrowedSpanFrame<'a, Action, Final, K>
where
	Action: 'a,
	Final: 'a,
	K: Fn(Action) -> BorrowedSpanRunExplicit<'a, Final> + 'a,
{
	fn new(
		source: BorrowedSpanRunExplicit<'a, Action>,
		outer: K,
	) -> Self {
		Self {
			source,
			continuation: DelayedBorrowedSpanContinuation {
				outer: Rc::new(outer),
				result: PhantomData,
			},
		}
	}

	fn peel_scoped(self) -> DelayedBorrowedSpanPeel<'a, Action, Final, K> {
		match self.source.peel() {
			Ok(value) => Ok(self.continuation.resume(value)),
			Err(Node::Scoped(layer)) => Err((layer, self.continuation)),
			Err(Node::First(cnil)) => match cnil {},
		}
	}
}

impl<'a, Action, Final, K> DelayedBorrowedSpanContinuation<'a, Action, Final, K>
where
	Action: 'a,
	Final: 'a,
	K: Fn(Action) -> BorrowedSpanRunExplicit<'a, Final> + 'a,
{
	fn resume(
		self,
		action_value: Action,
	) -> BorrowedSpanRunExplicit<'a, Final> {
		(self.outer)(action_value)
	}

	fn resume_with_post_action(
		self,
		action: BorrowedSpanRunExplicit<'a, Action>,
		post_action: impl Fn(Action) -> BorrowedSpanRunExplicit<'a, Action> + 'a,
	) -> BorrowedSpanRunExplicit<'a, Final> {
		let outer = self.outer.clone();
		action.bind(move |action_value| {
			let outer = outer.clone();
			post_action(action_value).bind(move |post_value| outer(post_value))
		})
	}
}

fn explicit_scoped_continuation<'a, Action, Final, K>(
	action: EmptyRunExplicit<'a, Action>,
	outer: K,
) -> RunExplicitScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
where
	Action: 'a,
	Final: 'a,
	K: Fn(Action) -> EmptyRunExplicit<'a, Final> + 'a, {
	RunExplicitScopedContinuation {
		action,
		outer: <RcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

fn explicit_action_supplied_scoped_continuation<'a, Action, Final, K>(
	outer: K
) -> RunExplicitActionSuppliedScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
where
	Action: 'a,
	Final: 'a,
	K: Fn(Action) -> EmptyRunExplicit<'a, Final> + 'a, {
	RunExplicitActionSuppliedScopedContinuation {
		outer: <RcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

fn ordinary_borrowed_span_program<'a, R, A>(
	tag: &'static str,
	action: RunExplicit<'a, R, BorrowedSpanScopedRow, A>,
) -> RunExplicit<'a, R, BorrowedSpanScopedRow, A>
where
	R: WrapDrop + Functor + 'static,
	A: 'a, {
	let action_free = Box::new(action.into_free_explicit());
	let layer = Coproduct::Inl(BoxSpan::Span {
		tag,
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
	});

	RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(layer)))
}

#[test]
fn from_and_into_round_trip() {
	let free: FreeExplicit<'_, _, i32> = FreeExplicit::pure(42);
	let run: RunAlias<'_, i32> = RunExplicit::from_free_explicit(free);
	let _back = run.into_free_explicit();
}

#[test]
fn brand_pure_evaluates() {
	let run: RunAlias<'_, _> = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(7);
	assert_eq!(run.into_free_explicit().evaluate(), 7);
}

#[test]
fn brand_map_evaluates() {
	let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(10);
	let mapped = <RunExplicitBrand<FirstRow, Scoped> as Functor>::map(|x: i32| x * 3, run);
	assert_eq!(mapped.into_free_explicit().evaluate(), 30);
}

#[test]
fn brand_bind_evaluates() {
	let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(2);
	let chained = <RunExplicitBrand<FirstRow, Scoped> as Semimonad>::bind(run, |x: i32| {
		<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(x + 5)
	});
	assert_eq!(chained.into_free_explicit().evaluate(), 7);
}

#[test]
fn brand_ref_pure_evaluates() {
	let value = 11;
	let run: RunAlias<'_, _> = <RunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
	assert_eq!(run.into_free_explicit().evaluate(), 11);
}

#[test]
fn brand_ref_map_evaluates() {
	let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(4);
	let mapped =
		<RunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 5, &run);
	assert_eq!(mapped.into_free_explicit().evaluate(), 20);
}

#[test]
fn brand_ref_bind_evaluates() {
	let run = <RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(8);
	let chained =
		<RunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
			<RunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
		});
	assert_eq!(chained.into_free_explicit().evaluate(), 9);
}

#[test]
fn non_static_payload() {
	let s = String::from("hello");
	let r: &str = &s;
	let run: RunExplicit<'_, FirstRow, Scoped, &str> =
		RunExplicit::from_free_explicit(FreeExplicit::pure(r));
	assert_eq!(run.into_free_explicit().evaluate(), "hello");
}

#[test]
fn pure_then_peel_returns_value() {
	let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::pure(42);
	assert!(matches!(run.peel(), Ok(42)));
}

#[test]
fn core_pure_send_extract_and_first_order_interpretation_stay_ordinary() {
	use crate::types::{
		Identity,
		effects::{
			coproduct::Coproduct,
			node::Node,
		},
	};

	let pure_program: EmptyRunExplicit<'_, i32> =
		RunExplicit::pure(40).bind(|value| RunExplicit::pure(value + 2));
	assert_eq!(pure_program.extract(), 42);

	let layer = Coproduct::inject(Identity(7));
	let sent_program: RunExplicit<'_, FirstRow, Scoped, i32> =
		RunExplicit::send(Node::First(layer));
	let sent_step = sent_program.peel();
	assert!(
		matches!(sent_step, Err(Node::First(Coproduct::Inl(Identity(_))))),
		"send should suspend a first-order Identity layer"
	);
	let Err(Node::First(Coproduct::Inl(Identity(next)))) = sent_step else {
		return;
	};
	assert!(matches!(next.peel(), Ok(7)));

	type Prog = RunExplicit<'static, IdentityFirstOrderRow, CNilBrand, i32>;
	let lifted: Prog = RunExplicit::lift::<IdentityBrand, _>(Identity(41))
		.bind(|value| RunExplicit::pure(value + 1));
	let interpreted = lifted.handle(
		crate::handlers! {
			IdentityBrand: |op: Identity<Prog>| op.0,
		},
		crate::types::effects::scoped_nt(),
	);
	assert_eq!(interpreted, 42);
}

#[test]
fn send_produces_suspended_program() {
	use crate::types::{
		Identity,
		effects::{
			coproduct::Coproduct,
			node::Node,
		},
	};
	let layer = Coproduct::inject(Identity(7));
	let run: RunExplicit<'_, FirstRow, Scoped, i32> = RunExplicit::send(Node::First(layer));
	assert!(run.peel().is_err());
}

#[test]
fn ordinary_scoped_interpretation_stays_on_plain_run_explicit() {
	let action: BorrowedSpanRunExplicit<'static, i32> = RunExplicit::pure(41);
	let program: BorrowedSpanRunExplicit<'static, i32> =
		ordinary_borrowed_span_program("request", action)
			.bind(|value| RunExplicit::pure(value + 1));

	let narrowed: EmptyRunExplicit<'static, i32> = program
		.handle_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
			|span: BoxSpan<'static, BoxBrand, &'static str, EmptyRunExplicit<'static, i32>>| {
				match span {
					BoxSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn interpose_stays_on_plain_run_explicit_through_scoped_layers() {
	use crate::types::Identity;

	let action: IdentitySpanRunExplicit<'static, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let program: IdentitySpanRunExplicit<'static, i32> =
		ordinary_borrowed_span_program("request", action);
	let interposed: IdentitySpanRunExplicit<'static, i32> = program
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<IdentitySpanRunExplicit<'static, i32>>| RunExplicit::pure(99),
		);

	let without_span: RunExplicit<'static, IdentityFirstOrderRow, CNilBrand, i32> =
		interposed.handle_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
			|span: BoxSpan<
				'static,
				BoxBrand,
				&'static str,
				RunExplicit<'static, IdentityFirstOrderRow, CNilBrand, i32>,
			>| {
				match span {
					BoxSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.handle(
		crate::handlers! {
			IdentityBrand: |op: Identity<RunExplicit<'static, IdentityFirstOrderRow, CNilBrand, i32>>| op.0,
		},
		crate::types::effects::scoped_nt(),
	);

	assert_eq!(result, 99);
}

#[test]
fn from_erased_round_trips_pure() {
	use crate::{
		brands::CoyonedaBrand,
		types::effects::run::Run,
	};
	type CoyoFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	let run: Run<CoyoFirstRow, CNilBrand, i32> = Run::pure(42);
	let explicit: RunExplicit<'static, CoyoFirstRow, CNilBrand, i32> = RunExplicit::from(run);
	assert!(matches!(explicit.peel(), Ok(42)));
}

#[test]
fn from_erased_preserves_suspended_layer() {
	use crate::{
		brands::CoyonedaBrand,
		types::{
			Coyoneda,
			Identity,
			effects::{
				coproduct::Coproduct,
				node::Node,
				run::Run,
			},
		},
	};
	type CoyoFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
	let layer = Coproduct::inject(coyo);
	let run: Run<CoyoFirstRow, CNilBrand, i32> = Run::send(Node::First(layer));
	let explicit: RunExplicit<'static, CoyoFirstRow, CNilBrand, i32> = RunExplicit::from(run);
	assert!(explicit.peel().is_err());
}

#[test]
fn bind_chains_pure_values() {
	let run: RunAlias<'_, i32> =
		RunExplicit::pure(2).bind(|x| RunExplicit::pure(x + 1)).bind(|x| RunExplicit::pure(x * 10));
	assert_eq!(run.into_free_explicit().evaluate(), 30);
}

#[test]
fn scoped_continuation_resumes_action_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(explicit_scoped_continuation(
		EmptyRunExplicit::pure(40),
		|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value * 10)
		},
	));

	let result: EmptyRunExplicit<'_, i32> = carrier.resume_explicit(&HandlersNil);

	assert_eq!(result.extract(), 400);
	assert_eq!(events.into_inner(), vec!["outer"]);
}

#[test]
fn scoped_continuation_transforms_action_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(explicit_scoped_continuation(
		EmptyRunExplicit::pure(40),
		|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value * 10)
		},
	));

	let result: EmptyRunExplicit<'_, i32> =
		carrier.resume_explicit_with_post_action(&HandlersNil, |value| {
			events.borrow_mut().push("post");
			EmptyRunExplicit::pure(value + 1)
		});

	assert_eq!(result.extract(), 410);
	assert_eq!(events.into_inner(), vec!["post", "outer"]);
}

#[test]
fn scoped_continuation_transforms_action_program_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(explicit_scoped_continuation(
		EmptyRunExplicit::pure(40),
		|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value * 10)
		},
	));

	let result: EmptyRunExplicit<'_, i32> =
		carrier.resume_explicit_with_action_transform(&HandlersNil, |action| {
			action.bind(|value| {
				events.borrow_mut().push("transform");
				EmptyRunExplicit::pure(value + 1)
			})
		});

	assert_eq!(result.extract(), 410);
	assert_eq!(events.into_inner(), vec!["transform", "outer"]);
}

#[test]
fn scoped_continuation_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let carrier = ScopedContinuation::new(explicit_scoped_continuation(
		EmptyRunExplicit::pure(label.as_str()),
		|value: &str| EmptyRunExplicit::pure(value.len()),
	));

	let result: EmptyRunExplicit<'_, usize> =
		carrier.resume_explicit_with_post_action(&HandlersNil, EmptyRunExplicit::pure);

	assert_eq!(result.extract(), label.len());
}

#[test]
fn scoped_continuation_action_transform_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let carrier = ScopedContinuation::new(explicit_scoped_continuation(
		EmptyRunExplicit::pure(label.as_str()),
		|value: &str| EmptyRunExplicit::pure(value.len()),
	));

	let result: EmptyRunExplicit<'_, usize> = carrier
		.resume_explicit_with_action_transform(&HandlersNil, |action| {
			action.bind(EmptyRunExplicit::pure)
		});

	assert_eq!(result.extract(), label.len());
}

#[test]
fn action_supplied_scoped_continuation_runs_supplied_action_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(explicit_action_supplied_scoped_continuation(|value| {
		events.borrow_mut().push("outer");
		EmptyRunExplicit::pure(value * 10)
	}));

	let result: EmptyRunExplicit<'_, i32> =
		carrier.resume_explicit_with_supplied_action(&HandlersNil, || {
			EmptyRunExplicit::pure(40).bind(|value| {
				events.borrow_mut().push("supplied-action");
				EmptyRunExplicit::pure(value + 1)
			})
		});

	assert_eq!(result.extract(), 410);
	assert_eq!(events.into_inner(), vec!["supplied-action", "outer"]);
}

#[test]
fn bracket_carrier_layer_stores_lifecycle_cells_and_continuation() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitBracketCarrierLayer::<BoxBrand, i32, i32, _, _, _, _>::new(
		|| {
			events.borrow_mut().push("acquire");
			7
		},
		|resource: Box<i32>| {
			events.borrow_mut().push("body");
			(*resource, *resource + 35)
		},
		|resource: Box<i32>| {
			events.borrow_mut().push("release");
			assert_eq!(*resource, 7);
		},
		ScopedContinuation::new(explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value)
		})),
	);

	let (acquire, body, release, continuation) = layer.into_parts();
	let resource = acquire();
	let (resource, body_result) = body(Box::new(resource));
	release(Box::new(resource));
	let result: EmptyRunExplicit<'_, i32> = continuation
		.resume_explicit_with_supplied_action(&HandlersNil, || EmptyRunExplicit::pure(body_result));

	assert_eq!(result.extract(), 42);
	assert_eq!(events.into_inner(), vec!["acquire", "body", "release", "outer"]);
}

#[test]
fn ref_bracket_carrier_layer_stores_pointer_clone_lifecycle_cells() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitRefBracketCarrierLayer::<RcBrand, i32, i32, _, _, _, _>::new(
		|| {
			events.borrow_mut().push("acquire");
			7
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("body");
			assert_eq!(std::rc::Rc::strong_count(&resource), 2);
			*resource + 35
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("release");
			assert_eq!(std::rc::Rc::strong_count(&resource), 1);
			assert_eq!(*resource, 7);
		},
		ScopedContinuation::new(explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value)
		})),
	);

	let (acquire, body, release, continuation) = layer.into_parts();
	let resource = std::rc::Rc::new(acquire());
	let release_resource = std::rc::Rc::clone(&resource);
	let body_result = body(resource);
	release(release_resource);
	let result: EmptyRunExplicit<'_, i32> = continuation
		.resume_explicit_with_supplied_action(&HandlersNil, || EmptyRunExplicit::pure(body_result));

	assert_eq!(result.extract(), 42);
	assert_eq!(events.into_inner(), vec!["acquire", "body", "release", "outer"]);
}

#[test]
fn bracket_carrier_dispatcher_runs_lifecycle_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitBracketCarrierLayer::<BoxBrand, i32, i32, _, _, _, _>::new(
		|| {
			events.borrow_mut().push("acquire");
			EmptyRunExplicit::pure(7)
		},
		|resource: Box<i32>| {
			events.borrow_mut().push("body");
			EmptyRunExplicit::pure((*resource, *resource + 35))
		},
		|resource: Box<i32>| {
			events.borrow_mut().push("release");
			assert_eq!(*resource, 7);
			EmptyRunExplicit::pure(())
		},
		ScopedContinuation::new(explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value)
		})),
	);

	let result: EmptyRunExplicit<'_, i32> =
		bracket_handler().dispatch_run_explicit_bracket_carrier(layer, &HandlersNil);

	assert_eq!(result.extract(), 42);
	assert_eq!(events.into_inner(), vec!["acquire", "body", "release", "outer"]);
}

#[test]
fn ref_bracket_carrier_dispatcher_runs_lifecycle_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitRefBracketCarrierLayer::<RcBrand, i32, i32, _, _, _, _>::new(
		|| {
			events.borrow_mut().push("acquire");
			EmptyRunExplicit::pure(7)
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("body");
			assert_eq!(std::rc::Rc::strong_count(&resource), 2);
			EmptyRunExplicit::pure(*resource + 35)
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("release");
			assert_eq!(std::rc::Rc::strong_count(&resource), 1);
			assert_eq!(*resource, 7);
			EmptyRunExplicit::pure(())
		},
		ScopedContinuation::new(explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value)
		})),
	);

	let result: EmptyRunExplicit<'_, i32> =
		ref_bracket_handler().dispatch_run_explicit_ref_bracket_carrier(layer, &HandlersNil);

	assert_eq!(result.extract(), 42);
	assert_eq!(events.into_inner(), vec!["acquire", "body", "release", "outer"]);
}

#[test]
fn span_carrier_layer_stores_tag_and_continuation_cell() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitSpanCarrierLayer::new(
		"request",
		ScopedContinuation::new(explicit_scoped_continuation(
			EmptyRunExplicit::pure(40),
			|value| {
				events.borrow_mut().push("outer");
				EmptyRunExplicit::pure(value * 10)
			},
		)),
	);

	assert_eq!(layer.tag(), &"request");

	let (tag, continuation) = layer.into_parts();
	let result: EmptyRunExplicit<'_, i32> =
		continuation.resume_explicit_with_post_action(&HandlersNil, |value| {
			events.borrow_mut().push("post");
			EmptyRunExplicit::pure(value + 1)
		});

	assert_eq!(tag, "request");
	assert_eq!(result.extract(), 410);
	assert_eq!(events.into_inner(), vec!["post", "outer"]);
}

#[test]
fn span_carrier_layer_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let layer = RunExplicitSpanCarrierLayer::new(
		"request",
		ScopedContinuation::new(explicit_scoped_continuation(
			EmptyRunExplicit::pure(label.as_str()),
			|value: &str| EmptyRunExplicit::pure(value.len()),
		)),
	);

	let (tag, continuation) = layer.into_parts();
	let result: EmptyRunExplicit<'_, usize> =
		continuation.resume_explicit_with_post_action(&HandlersNil, EmptyRunExplicit::pure);

	assert_eq!(tag, "request");
	assert_eq!(result.extract(), label.len());
}

#[test]
fn delayed_typed_span_frame_exposes_action_row_and_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let action: BorrowedSpanRunExplicit<'_, &str> = RunExplicit::pure(label.as_str());
	let action_free = Box::new(action.into_free_explicit());
	let span: BoxSpan<'_, BoxBrand, &'static str, BorrowedSpanFreeCell<'_, &str>> = BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
	};
	let layer = Coproduct::Inl(span);
	let source: BorrowedSpanRunExplicit<'_, &str> =
		RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(layer)));
	let frame = DelayedBorrowedSpanFrame::new(source, |value: &str| {
		events.borrow_mut().push("outer");
		RunExplicit::pure(value.len())
	});

	let step = frame.peel_scoped();
	assert!(step.is_err(), "expected suspended scoped Span layer");
	let Err((layer, continuation)) = step else {
		return;
	};
	let action_program = match layer {
		Coproduct::Inl(BoxSpan::Span {
			tag,
			action,
		}) => {
			assert_eq!(tag, "request");
			action(())
		}
		Coproduct::Inr(rest) => match rest {},
	};

	let result = continuation.resume_with_post_action(action_program, |value| {
		events.borrow_mut().push("post");
		RunExplicit::pure(value)
	});

	let result_value = match result.peel() {
		Ok(value) => Some(value),
		Err(Node::First(cnil)) => match cnil {},
		Err(Node::Scoped(_)) => None,
	};
	assert_eq!(result_value, Some(label.len()));
	assert_eq!(*events.borrow(), vec!["post", "outer"]);
}

#[test]
fn typed_span_substrate_boundary_separates_action_and_final_programs() {
	fn require_private_boundary_protocol<'a, Action: 'a, Final: 'a>(
		boundary: ExplicitBoundaryOf<'a, TypedBorrowedSpanBoundaryBrand, Action, Final>
	) -> ExplicitBoundaryOf<'a, TypedBorrowedSpanBoundaryBrand, Action, Final>
	where
		TypedBorrowedSpanBoundaryBrand: ExplicitBoundaryTypes<
				'a,
				Action,
				Final,
				ActionProgram = TypedBorrowedSpanActionProgram<'a, Action>,
				FinalProgram = TypedBorrowedSpanFinalProgram<'a, Final>,
			>, {
		boundary
	}

	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let action: TypedBorrowedSpanActionProgram<'_, &str> = RunExplicit::pure(label.as_str());
	let boundary: ExplicitBoundaryOf<'_, TypedBorrowedSpanBoundaryBrand, &str, usize> =
		TypedBorrowedSpanBoundary::span("request", action).bind(|value: &str| {
			events.borrow_mut().push("outer");
			RunExplicit::pure(value.len())
		});
	let boundary = require_private_boundary_protocol(boundary);
	let TypedBorrowedSpanBoundary::Scoped {
		layer,
		continuation,
	} = boundary;

	let action_program: TypedBorrowedSpanActionProgram<'_, &str> = match layer {
		Coproduct::Inl(BoxSpan::Span {
			tag,
			action,
		}) => {
			assert_eq!(tag, "request");
			action(())
		}
		Coproduct::Inr(rest) => match rest {},
	};

	let final_program: TypedBorrowedSpanFinalProgram<'_, usize> = continuation
		.resume_with_post_action(action_program, |value| {
			events.borrow_mut().push("post");
			RunExplicit::pure(value)
		});
	let final_step = final_program.peel();
	assert!(final_step.is_ok());
	let Ok(final_value) = final_step else {
		return;
	};

	assert_eq!(final_value, label.len());
	assert_eq!(*events.borrow(), vec!["post", "outer"]);
}

#[test]
fn run_explicit_boundary_separates_action_layer_and_final_continuation() {
	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let action: BorrowedSpanRunExplicit<'_, &str> = RunExplicit::pure(label.as_str());
	let layer: BorrowedSpanLayer<'_, &str> = Coproduct::Inl(BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
	});

	let boundary: RunExplicitBoundary<
		'_,
		CNilBrand,
		BorrowedSpanScopedRow,
		BorrowedSpanSBrand,
		Here,
		&str,
		usize,
		_,
	> = RunExplicitBoundary::new(layer, |value: &str| BorrowedSpanRunExplicit::pure(value.len()))
		.map(|length| length + 1);
	let (layer, continuation) = boundary.into_parts();
	let action_program: BorrowedSpanRunExplicit<'_, &str> = match layer {
		Coproduct::Inl(BoxSpan::Span {
			tag,
			action,
		}) => {
			assert_eq!(tag, "request");
			action(())
		}
		Coproduct::Inr(rest) => match rest {},
	};

	let final_program: BorrowedSpanRunExplicit<'_, usize> = continuation
		.resume_explicit_with_supplied_action(&HandlersNil, || {
			action_program.bind(|value| {
				events.borrow_mut().push("supplied-action");
				BorrowedSpanRunExplicit::pure(value)
			})
		});
	let final_step = final_program.peel();
	assert!(final_step.is_ok());
	let Ok(final_value) = final_step else {
		return;
	};

	assert_eq!(final_value, label.len() + 1);
	assert_eq!(*events.borrow(), vec!["supplied-action"]);
}

#[test]
fn writer_listen_boundary_keeps_action_slot_before_final_continuation() {
	let events = RefCell::new(Vec::new());
	let action: WriterListenRunExplicit<'_, i32> = RunExplicit::pure(41);
	let layer: WriterListenLayer<'_, i32> = Coproduct::Inl(BoxWriterListen::Listen {
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
		result: PhantomData,
	});

	let boundary: RunExplicitBoundary<
		'_,
		CNilBrand,
		WriterListenScopedRow,
		WriterListenSBrand,
		Here,
		i32,
		String,
		_,
		(i32, String),
	> = RunExplicitBoundary::new(layer, |(value, log): (i32, String)| {
		WriterListenRunExplicit::pure((value, log))
	})
	.map(|(value, log)| (value + 1, log.len()))
	.bind(|(value, log_len)| {
		events.borrow_mut().push("outer");
		WriterListenRunExplicit::pure(format!("{value}:{log_len}"))
	});
	let (layer, continuation) = boundary.into_parts();
	let action_program: WriterListenRunExplicit<'_, i32> = match layer {
		Coproduct::Inl(BoxWriterListen::Listen {
			action,
			result: _,
		}) => action(()),
		Coproduct::Inr(rest) => match rest {},
	};

	let final_program: WriterListenRunExplicit<'_, String> = continuation
		.resume_explicit_with_supplied_action(&HandlersNil, || {
			action_program.bind(|value| {
				events.borrow_mut().push("selected-action");
				WriterListenRunExplicit::pure((value, String::from("log")))
			})
		});
	let final_step = final_program.peel();
	assert!(final_step.is_ok());
	let Ok(final_value) = final_step else {
		return;
	};

	assert_eq!(final_value, "42:3");
	assert_eq!(*events.borrow(), vec!["selected-action", "outer"]);
}

#[test]
fn local_carrier_layer_stores_modifier_and_continuation_cell() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitLocalCarrierLayer::<i32, _, _>::new(
		|env| {
			events.borrow_mut().push("modify");
			env + 1
		},
		ScopedContinuation::new(explicit_scoped_continuation(EmptyRunExplicit::pure(1), |value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value * 10)
		})),
	);

	let (modify, continuation) = layer.into_parts();
	let local_env = modify(39);
	let result: EmptyRunExplicit<'_, i32> =
		continuation.resume_explicit_with_action_transform(&HandlersNil, |action| {
			action.bind(|value| {
				events.borrow_mut().push("transform");
				EmptyRunExplicit::pure(value + local_env)
			})
		});

	assert_eq!(local_env, 40);
	assert_eq!(result.extract(), 410);
	assert_eq!(events.into_inner(), vec!["modify", "transform", "outer"]);
}

#[test]
fn ref_local_carrier_layer_borrows_environment_for_modifier() {
	let events = RefCell::new(Vec::new());
	let inherited = String::from("root");
	let layer = RunExplicitRefLocalCarrierLayer::<String, _, _>::new(
		|env: &String| {
			events.borrow_mut().push("modify");
			format!("{}-local", env)
		},
		ScopedContinuation::new(explicit_scoped_continuation(EmptyRunExplicit::pure(2), |value| {
			events.borrow_mut().push("outer");
			EmptyRunExplicit::pure(value * 10)
		})),
	);

	let (modify, continuation) = layer.into_parts();
	let local_env = modify(&inherited);
	let local_len = local_env.len() as i32;
	let result: EmptyRunExplicit<'_, i32> =
		continuation.resume_explicit_with_action_transform(&HandlersNil, |action| {
			action.bind(|value| {
				events.borrow_mut().push("transform");
				EmptyRunExplicit::pure(value + local_len)
			})
		});

	assert_eq!(local_env, "root-local");
	assert_eq!(result.extract(), 120);
	assert_eq!(events.into_inner(), vec!["modify", "transform", "outer"]);
}

#[test]
fn local_carrier_dispatcher_interposes_reader_before_outer_continuation() {
	const LABEL: &str = "borrowed-value";
	let action: BoxReaderRunExplicit<'static, &'static str> =
		RunExplicit::<BoxReaderRow, CNilBrand, i32>::ask::<_>().bind(|env| {
			assert_eq!(env, 11);
			RunExplicit::pure(LABEL)
		});
	let layer = RunExplicitLocalCarrierLayer::<i32, _, _>::new(
		|env| env + 1,
		ScopedContinuation::new(RunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(|value: &str| {
				BoxReaderRunExplicit::pure(value.len())
			}),
			result: PhantomData,
		}),
	);

	let program: BoxReaderRunExplicit<'static, usize> =
		local_handler::<_, BoxReaderRowMinusReader, _>()
			.dispatch_run_explicit_local_carrier(layer, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxReaderRunExplicit<'static, usize>>| match op {
					BoxReader::Ask(k) => k(10),
				},
			},
			crate::types::effects::scoped_nt(),
		);

	assert_eq!(result, LABEL.len());
}

#[test]
fn ref_local_carrier_dispatcher_borrows_reader_environment() {
	let action: BoxReaderRunExplicit<'static, i32> =
		RunExplicit::<BoxReaderRow, CNilBrand, i32>::ask::<_>()
			.bind(|env| RunExplicit::pure(env * 2));
	let layer = RunExplicitRefLocalCarrierLayer::<i32, _, _>::new(
		|env: &i32| *env + 5,
		ScopedContinuation::new(RunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(|value| {
				BoxReaderRunExplicit::pure(value + 1)
			}),
			result: PhantomData,
		}),
	);

	let program: BoxReaderRunExplicit<'static, i32> =
		ref_local_handler::<_, BoxReaderRowMinusReader, _>()
			.dispatch_run_explicit_ref_local_carrier(layer, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxReaderRunExplicit<'static, i32>>| match op {
					BoxReader::Ask(k) => k(10),
				},
			},
			crate::types::effects::scoped_nt(),
		);

	assert_eq!(result, 31);
}

#[test]
fn catch_carrier_dispatcher_recovers_before_outer_continuation() {
	let action: BoxExceptRunExplicit<'static, i32> =
		RunExplicit::throw::<&'static str, _>("from-action");
	let layer = RunExplicitCatchCarrierLayer::<&'static str, _, _>::new(
		|err| {
			assert_eq!(err, "from-action");
			BoxExceptRunExplicit::pure(41)
		},
		ScopedContinuation::new(RunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(|value| {
				BoxExceptRunExplicit::pure(value + 1)
			}),
			result: PhantomData,
		}),
	);

	let program: BoxExceptRunExplicit<'static, i32> =
		catch_handler::<_, BoxExceptRowMinusExcept, _>()
			.dispatch_run_explicit_catch_carrier(layer, &HandlersNil);
	let result = program.handle(
		crate::handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BoxExceptRunExplicit<'static, i32>>| {
				BoxExceptRunExplicit::pure(-1)
			},
		},
		crate::types::effects::scoped_nt(),
	);

	assert_eq!(result, 42);
}

#[test]
fn catch_carrier_dispatcher_preserves_recovery_rethrow() {
	let action: BoxExceptRunExplicit<'static, i32> =
		RunExplicit::throw::<&'static str, _>("from-action");
	let layer = RunExplicitCatchCarrierLayer::<&'static str, _, _>::new(
		|err| {
			assert_eq!(err, "from-action");
			BoxExceptRunExplicit::throw::<&'static str, _>("from-recovery")
		},
		ScopedContinuation::new(RunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(|value| {
				BoxExceptRunExplicit::pure(value + 100)
			}),
			result: PhantomData,
		}),
	);

	let program: BoxExceptRunExplicit<'static, i32> =
		catch_handler::<_, BoxExceptRowMinusExcept, _>()
			.dispatch_run_explicit_catch_carrier(layer, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				ExceptBrand<&'static str>: |op: Except<'_, &'static str, BoxExceptRunExplicit<'static, i32>>| match op {
					Except::Throw(err, _) => {
						assert_eq!(err, "from-recovery");
						BoxExceptRunExplicit::pure(42)
					},
				},
			},
			crate::types::effects::scoped_nt(),
		);

	assert_eq!(result, 42);
}

#[test]
fn span_handler_consumes_carrier_layer_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let layer = RunExplicitSpanCarrierLayer::new(
		"request",
		ScopedContinuation::new(explicit_scoped_continuation(
			EmptyRunExplicit::pure(label.as_str()),
			|value: &str| {
				events.borrow_mut().push("outer");
				EmptyRunExplicit::pure(value.len())
			},
		)),
	);

	let result: EmptyRunExplicit<'_, usize> = span_handler()
		.dispatch_run_explicit_span_carrier_with_post_action(layer, &HandlersNil, |tag, value| {
			assert_eq!(*tag, "request");
			events.borrow_mut().push("post");
			EmptyRunExplicit::pure(value)
		});

	assert_eq!(result.extract(), label.len());
	assert_eq!(events.into_inner(), vec!["post", "outer"]);
}

#[test]
fn span_boundary_bind_keeps_action_slot_before_dispatch() {
	type SpanScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;

	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let action: RunExplicit<'_, CNilBrand, SpanScopedRow, &str> = RunExplicit::pure(label.as_str());
	let boundary = RunExplicit::span::<&'static str, _>("request", action)
		.bind(|value| RunExplicit::pure(value.len()));

	let final_program: RunExplicit<'_, CNilBrand, SpanScopedRow, usize> = span_handler()
		.dispatch_run_explicit_span_boundary_with_post_action(
			boundary,
			&HandlersNil,
			|tag, value| {
				assert_eq!(*tag, "request");
				events.borrow_mut().push("post");
				RunExplicit::pure(value)
			},
		);

	let final_step = final_program.peel();
	assert!(final_step.is_ok());
	let Ok(final_value) = final_step else {
		return;
	};
	assert_eq!(final_value, label.len());
	assert_eq!(*events.borrow(), vec!["post"]);
}

#[test]
fn span_boundary_map_keeps_action_slot_before_dispatch() {
	type SpanScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;

	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let action: RunExplicit<'_, CNilBrand, SpanScopedRow, &str> = RunExplicit::pure(label.as_str());
	let boundary = RunExplicit::span::<&'static str, _>("request", action).map(|value| value.len());

	let final_program: RunExplicit<'_, CNilBrand, SpanScopedRow, usize> = span_handler()
		.dispatch_run_explicit_span_boundary_with_post_action(
			boundary,
			&HandlersNil,
			|tag, value| {
				assert_eq!(*tag, "request");
				events.borrow_mut().push("post");
				RunExplicit::pure(value)
			},
		);

	let final_step = final_program.peel();
	assert!(final_step.is_ok());
	let Ok(final_value) = final_step else {
		return;
	};
	assert_eq!(final_value, label.len());
	assert_eq!(*events.borrow(), vec!["post"]);
}

#[test]
fn local_boundary_dispatcher_interposes_reader_before_outer_continuation() {
	const LABEL: &str = "borrowed-value";
	let action: LocalBoundaryRunExplicit<'static, &'static str> =
		RunExplicit::<BoxReaderRow, LocalBoundaryScopedRow, i32>::ask::<_>().bind(|env| {
			assert_eq!(env, 11);
			RunExplicit::pure(LABEL)
		});
	let layer: LocalBoundaryLayer<'static, &'static str> = Coproduct::Inl(BoxLocal::Local {
		modify: <BoxBrand as ToDynFnOnce>::new(|env| env + 1),
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
	});
	let boundary: RunExplicitBoundary<
		'static,
		BoxReaderRow,
		LocalBoundaryScopedRow,
		LocalBoundarySBrand,
		Here,
		&'static str,
		usize,
		_,
	> = RunExplicitBoundary::new(layer, |value: &str| LocalBoundaryRunExplicit::pure(value.len()));

	let program: LocalBoundaryRunExplicit<'static, usize> =
		local_handler::<_, BoxReaderRowMinusReader, _>()
			.dispatch_run_explicit_local_boundary(boundary, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, LocalBoundaryRunExplicit<'static, usize>>| match op {
					BoxReader::Ask(k) => k(10),
				},
			},
			crate::scoped_handlers! {
				BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxReaderRowMinusReader, _>(),
			},
		);

	assert_eq!(result, LABEL.len());
}

#[test]
fn ref_local_boundary_dispatcher_borrows_reader_before_outer_continuation() {
	let action: RefLocalBoundaryRunExplicit<'static, i32> =
		RunExplicit::<BoxReaderRow, RefLocalBoundaryScopedRow, i32>::ask::<_>()
			.bind(|env| RunExplicit::pure(env * 2));
	let layer: RefLocalBoundaryLayer<'static, i32> = Coproduct::Inl(BoxRefLocal::Local {
		modify: <BoxBrand as ToDynFnOnce>::ref_new(|env: &i32| *env + 5),
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
	});
	let boundary: RunExplicitBoundary<
		'static,
		BoxReaderRow,
		RefLocalBoundaryScopedRow,
		RefLocalBoundarySBrand,
		Here,
		i32,
		i32,
		_,
	> = RunExplicitBoundary::new(layer, |value| RefLocalBoundaryRunExplicit::pure(value + 1));

	let program: RefLocalBoundaryRunExplicit<'static, i32> =
		ref_local_handler::<_, BoxReaderRowMinusReader, _>()
			.dispatch_run_explicit_ref_local_boundary(boundary, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RefLocalBoundaryRunExplicit<'static, i32>>| match op {
					BoxReader::Ask(k) => k(10),
				},
			},
			crate::scoped_handlers! {
				BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxReaderRowMinusReader, _>(),
			},
		);

	assert_eq!(result, 31);
}

#[test]
fn catch_boundary_dispatcher_recovers_before_outer_continuation() {
	let action: CatchBoundaryRunExplicit<'static, i32> =
		RunExplicit::throw::<&'static str, _>("from-action");
	let layer: CatchBoundaryLayer<'static, i32> = Coproduct::Inl(BoxCatch::Catch {
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
		handler: <BoxBrand as ToDynFnOnce>::new(|err| {
			assert_eq!(err, "from-action");
			CatchBoundaryRunExplicit::pure(41)
		}),
	});
	let boundary: RunExplicitBoundary<
		'static,
		BoxExceptRow,
		CatchBoundaryScopedRow,
		CatchBoundarySBrand,
		Here,
		i32,
		i32,
		_,
	> = RunExplicitBoundary::new(layer, |value| CatchBoundaryRunExplicit::pure(value + 1));

	let program: CatchBoundaryRunExplicit<'static, i32> =
		catch_handler::<_, BoxExceptRowMinusExcept, _>()
			.dispatch_run_explicit_catch_boundary(boundary, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				ExceptBrand<&'static str>: |_op: Except<'_, &'static str, CatchBoundaryRunExplicit<'static, i32>>| {
					CatchBoundaryRunExplicit::pure(-1)
				},
			},
			crate::scoped_handlers! {
				BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxExceptRowMinusExcept, _>(),
			},
		);

	assert_eq!(result, 42);
}

#[test]
fn catch_boundary_dispatcher_preserves_recovery_rethrow() {
	let action: CatchBoundaryRunExplicit<'static, i32> =
		RunExplicit::throw::<&'static str, _>("from-action");
	let layer: CatchBoundaryLayer<'static, i32> = Coproduct::Inl(BoxCatch::Catch {
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
		handler: <BoxBrand as ToDynFnOnce>::new(|err| {
			assert_eq!(err, "from-action");
			CatchBoundaryRunExplicit::throw::<&'static str, _>("from-recovery")
		}),
	});
	let boundary: RunExplicitBoundary<
		'static,
		BoxExceptRow,
		CatchBoundaryScopedRow,
		CatchBoundarySBrand,
		Here,
		i32,
		i32,
		_,
	> = RunExplicitBoundary::new(layer, |value| CatchBoundaryRunExplicit::pure(value + 100));

	let program: CatchBoundaryRunExplicit<'static, i32> =
		catch_handler::<_, BoxExceptRowMinusExcept, _>()
			.dispatch_run_explicit_catch_boundary(boundary, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				ExceptBrand<&'static str>: |op: Except<'_, &'static str, CatchBoundaryRunExplicit<'static, i32>>| match op {
					Except::Throw(err, _) => {
						assert_eq!(err, "from-recovery");
						CatchBoundaryRunExplicit::pure(42)
					},
				},
			},
			crate::scoped_handlers! {
				BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxExceptRowMinusExcept, _>(),
			},
		);

	assert_eq!(result, 42);
}

#[test]
fn bracket_boundary_dispatcher_runs_lifecycle_before_outer_continuation() {
	let events = Rc::new(RefCell::new(Vec::new()));
	let acquire_events = Rc::clone(&events);
	let body_events = Rc::clone(&events);
	let release_events = Rc::clone(&events);
	let outer_events = Rc::clone(&events);
	let bracket: BoxBracketExplicit<
		'_,
		BoxBrand,
		NodeBrand<CNilBrand, BoundaryBracketRow>,
		i32,
		i32,
	> = BoxBracketExplicit::Bracket {
		acquire: <BoxBrand as ToDynFnOnce>::new(move |_: ()| {
			acquire_events.borrow_mut().push("acquire");
			Box::new(BoundaryBracketRunExplicit::pure(7).into_free_explicit())
		}),
		body: <BoxBrand as ToDynFnOnce>::new(move |resource: Box<i32>| {
			body_events.borrow_mut().push("body");
			assert_eq!(*resource, 7);
			Box::new(BoundaryBracketRunExplicit::pure((*resource, 41)).into_free_explicit())
		}),
		release: <BoxBrand as ToDynFnOnce>::new(move |resource: Box<i32>| {
			release_events.borrow_mut().push("release");
			assert_eq!(*resource, 7);
			Box::new(BoundaryBracketRunExplicit::pure(()).into_free_explicit())
		}),
	};
	let layer: BoundaryBracketLayer<'_> = Coproduct::Inl(bracket);
	let boundary: RunExplicitBoundary<
		'_,
		CNilBrand,
		BoundaryBracketRow,
		BoundaryBracketSBrand,
		Here,
		i32,
		i32,
		_,
	> = RunExplicitBoundary::new(layer, move |value| {
		outer_events.borrow_mut().push("outer");
		BoundaryBracketRunExplicit::pure(value + 1)
	});

	let program: BoundaryBracketRunExplicit<'_, i32> =
		bracket_handler().dispatch_run_explicit_bracket_boundary(boundary, &HandlersNil);

	assert!(matches!(program.peel(), Ok(42)));
	assert_eq!(events.borrow().as_slice(), ["acquire", "body", "release", "outer"]);
}

#[test]
fn map_transforms_pure_value() {
	let run: RunAlias<'_, i32> = RunExplicit::pure(7).map(|x| x * 3);
	assert_eq!(run.into_free_explicit().evaluate(), 21);
}
