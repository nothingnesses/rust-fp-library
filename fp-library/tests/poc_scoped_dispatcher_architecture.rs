#![expect(
	clippy::panic,
	reason = "The POC interpreter uses panic only for impossible row-shape mismatches."
)]
#![expect(
	clippy::type_complexity,
	reason = "The checkpoint intentionally spells dispatcher evidence types inline."
)]

//! Architecture checkpoint for standard scoped dispatcher signatures.
//!
//! This file is intentionally a POC rather than production dispatcher
//! code. It checks whether the standard scoped handlers can be shaped
//! around the wrapper's actual peeled-layer lifetime (`'static` for
//! `RcRun`) and witness-bearing dispatcher values, before step 7
//! commits to a public implementation.
//!
//! The test keeps the explanation self-contained:
//! - `Catch`, `Local`, and `RefLocal` all need first-order row-removal
//!   evidence because their dispatchers use `interpose` to answer
//!   `Except::Throw` or `Reader::Ask` inside the protected action.
//! - `Span` needs no row-removal evidence; it observes a by-value tag
//!   and returns the action program.
//! - `Bracket` and `RefBracket` are result-specific scoped brands, so
//!   they are prototyped in separate rows from the control-flow row.

use {
	core::marker::PhantomData,
	fp_library::{
		Apply,
		brands::{
			BracketBrand,
			CNilBrand,
			CatchBrand,
			CoproductBrand,
			ExceptBrand,
			LocalBrand,
			NodeBrand,
			RcBrand,
			RcCoyonedaBrand,
			ReaderBrand,
			RefBracketBrand,
			RefLocalBrand,
			SpanBrand,
		},
		classes::{
			Functor,
			ToDynCloneFn,
			WrapDrop,
		},
		define_scoped_row,
		handlers,
		kinds::*,
		scoped_handlers,
		types::{
			RcCoyoneda,
			RcFree,
			RcFreeExplicit,
			effects::{
				bracket::Bracket,
				catch::Catch,
				coproduct::CoproductEmbedder,
				except::Except,
				interpreter::{
					DispatchHandlers,
					DispatchScopedHandler,
					DispatchScopedHandlers,
				},
				local::Local,
				member::Member,
				node::Node,
				rc_run::RcRun,
				rc_run_explicit::RcRunExplicit,
				reader::Reader,
				ref_bracket::RefBracket,
				ref_local::RefLocal,
				span::Span,
			},
			rc_free::RcTypeErasedValue,
		},
	},
	std::rc::Rc,
};

type FirstRow = CoproductBrand<
	RcCoyonedaBrand<ExceptBrand<&'static str>>,
	CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>,
>;

type FirstRowMinusExcept = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type FirstRowMinusReader = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;

define_scoped_row! {
	struct ControlScopedRow;
	[
		CatchBrand<RcBrand, &'static str>,
		LocalBrand<RcBrand, i32>,
		RefLocalBrand<RcBrand, i32>,
		SpanBrand<RcBrand, &'static str>,
	]
}

define_scoped_row! {
	struct ExplicitControlScopedRow;
	[
		CatchBrand<RcBrand, &'static str>,
		LocalBrand<RcBrand, i32>,
	]
}

define_scoped_row! {
	struct BracketScopedRow;
	[
		BracketBrand<RcBrand, NodeBrand<FirstRow, Self>, i32, i32>,
	]
}

define_scoped_row! {
	struct RefBracketScopedRow;
	[
		RefBracketBrand<RcBrand, NodeBrand<FirstRow, Self>, i32, i32>,
	]
}

type ControlProg = RcRun<FirstRow, ControlScopedRow, i32>;
type ControlFirstLayer = Apply!(<FirstRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'static,
	ControlProg,
>);
type ControlScopedLayer =
	Apply!(<ControlScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ControlProg>);

type ExplicitControlProg<'a> = RcRunExplicit<'a, FirstRow, ExplicitControlScopedRow, i32>;
type ExplicitControlFirstLayer<'a> =
	Apply!(<FirstRow as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ExplicitControlProg<'a>>);
type ExplicitControlScopedLayer<'a> = Apply!(
	<ExplicitControlScopedRow as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		ExplicitControlProg<'a>,
	>
);

type BracketProg = RcRun<FirstRow, BracketScopedRow, i32>;
type BracketFirstLayer = Apply!(<FirstRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'static,
	BracketProg,
>);
type BracketScopedLayer =
	Apply!(<BracketScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, BracketProg>);

type RefBracketProg = RcRun<FirstRow, RefBracketScopedRow, i32>;
type RefBracketFirstLayer = Apply!(<FirstRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'static,
	RefBracketProg,
>);
type RefBracketScopedLayer =
	Apply!(<RefBracketScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RefBracketProg>);

struct CatchDispatcher<Idx, RMinusE, EmbedIndices>(
	PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
);

impl<Idx, RMinusE, EmbedIndices> CatchDispatcher<Idx, RMinusE, EmbedIndices> {
	const fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'static,
		Catch<'static, RcBrand, &'static str, ControlProg>,
		ControlFirstLayer,
		ControlProg,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	RMinusE: WrapDrop + Functor + 'static,
	ControlFirstLayer: Member<
			RcCoyoneda<'static, ExceptBrand<&'static str>, ControlProg>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ControlProg>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<FirstRow, ControlScopedRow>, i32>,
	>): CoproductEmbedder<
			Apply!(<FirstRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<FirstRow, ControlScopedRow>, i32>,
			>),
			EmbedIndices,
		>,
	Apply!(<NodeBrand<FirstRow, ControlScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<FirstRow, ControlScopedRow>, RcTypeErasedValue>,
	>): Clone,
{
	fn dispatch_scoped_head(
		&self,
		layer: Catch<'static, RcBrand, &'static str, ControlProg>,
		_fo_handlers: &impl DispatchHandlers<'static, ControlFirstLayer, ControlProg>,
	) -> ControlProg {
		match layer {
			Catch::Catch {
				action,
				handler,
			} => action(()).interpose::<ExceptBrand<&'static str>, Idx, RMinusE, EmbedIndices>(
				move |op| match op {
					Except::Throw(e, _) => (*handler)(e),
				},
			),
		}
	}
}

impl<'a, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'a,
		Catch<'a, RcBrand, &'static str, ExplicitControlProg<'a>>,
		ExplicitControlFirstLayer<'a>,
		ExplicitControlProg<'a>,
	> for CatchDispatcher<Idx, RMinusE, EmbedIndices>
where
	RMinusE: WrapDrop + Functor + 'static,
	ExplicitControlFirstLayer<'a>: Member<
			RcCoyoneda<'a, ExceptBrand<&'static str>, ExplicitControlProg<'a>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ExplicitControlProg<'a>>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<FirstRow, ExplicitControlScopedRow>, i32>,
	>): CoproductEmbedder<
			Apply!(<FirstRow as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<FirstRow, ExplicitControlScopedRow>, i32>,
			>),
			EmbedIndices,
		>,
	Apply!(<NodeBrand<FirstRow, ExplicitControlScopedRow> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<FirstRow, ExplicitControlScopedRow>, i32>,
	>): Clone,
{
	fn dispatch_scoped_head(
		&self,
		layer: Catch<'a, RcBrand, &'static str, ExplicitControlProg<'a>>,
		_fo_handlers: &impl DispatchHandlers<'a, ExplicitControlFirstLayer<'a>, ExplicitControlProg<'a>>,
	) -> ExplicitControlProg<'a> {
		match layer {
			Catch::Catch {
				action,
				handler,
			} => action(()).interpose::<ExceptBrand<&'static str>, Idx, RMinusE, EmbedIndices>(
				move |op| match op {
					Except::Throw(e, _) => (*handler)(e),
				},
			),
		}
	}
}

struct LocalDispatcher<Idx, RMinusE, EmbedIndices>(
	PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
);

impl<Idx, RMinusE, EmbedIndices> LocalDispatcher<Idx, RMinusE, EmbedIndices> {
	const fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'static,
		Local<'static, RcBrand, i32, ControlProg>,
		ControlFirstLayer,
		ControlProg,
	> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
where
	RMinusE: WrapDrop + Functor + 'static,
	ControlFirstLayer: Member<
			RcCoyoneda<'static, ReaderBrand<RcBrand, i32>, ControlProg>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ControlProg>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<FirstRow, ControlScopedRow>, i32>,
	>): CoproductEmbedder<
			Apply!(<FirstRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<FirstRow, ControlScopedRow>, i32>,
			>),
			EmbedIndices,
		>,
	Apply!(<NodeBrand<FirstRow, ControlScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<FirstRow, ControlScopedRow>, RcTypeErasedValue>,
	>): Clone,
{
	fn dispatch_scoped_head(
		&self,
		layer: Local<'static, RcBrand, i32, ControlProg>,
		fo_handlers: &impl DispatchHandlers<'static, ControlFirstLayer, ControlProg>,
	) -> ControlProg {
		match layer {
			Local::Local {
				modify,
				action,
			} => {
				let ask: Reader<'static, RcBrand, i32, ControlProg> =
					Reader::Ask(<RcBrand as ToDynCloneFn>::new(RcRun::pure));
				let ask_layer = <ControlFirstLayer as Member<
					RcCoyoneda<'static, ReaderBrand<RcBrand, i32>, ControlProg>,
					Idx,
				>>::inject(RcCoyoneda::lift(ask));

				fo_handlers.dispatch(ask_layer).bind(move |parent_env| {
					let modified_env = (*modify)(parent_env);
					action(()).interpose::<ReaderBrand<RcBrand, i32>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => (*k)(modified_env),
						},
					)
				})
			}
		}
	}
}

impl<'a, Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'a,
		Local<'a, RcBrand, i32, ExplicitControlProg<'a>>,
		ExplicitControlFirstLayer<'a>,
		ExplicitControlProg<'a>,
	> for LocalDispatcher<Idx, RMinusE, EmbedIndices>
where
	RMinusE: WrapDrop + Functor + 'static,
	ExplicitControlFirstLayer<'a>: Member<
			RcCoyoneda<'a, ReaderBrand<RcBrand, i32>, ExplicitControlProg<'a>>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, ExplicitControlProg<'a>>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<FirstRow, ExplicitControlScopedRow>, i32>,
	>): CoproductEmbedder<
			Apply!(<FirstRow as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<FirstRow, ExplicitControlScopedRow>, i32>,
			>),
			EmbedIndices,
		>,
	Apply!(<NodeBrand<FirstRow, ExplicitControlScopedRow> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<FirstRow, ExplicitControlScopedRow>, i32>,
	>): Clone,
{
	fn dispatch_scoped_head(
		&self,
		layer: Local<'a, RcBrand, i32, ExplicitControlProg<'a>>,
		fo_handlers: &impl DispatchHandlers<'a, ExplicitControlFirstLayer<'a>, ExplicitControlProg<'a>>,
	) -> ExplicitControlProg<'a> {
		match layer {
			Local::Local {
				modify,
				action,
			} => {
				let ask: Reader<'a, RcBrand, i32, ExplicitControlProg<'a>> =
					Reader::Ask(<RcBrand as ToDynCloneFn>::new(RcRunExplicit::pure));
				let ask_layer = <ExplicitControlFirstLayer<'a> as Member<
					RcCoyoneda<'a, ReaderBrand<RcBrand, i32>, ExplicitControlProg<'a>>,
					Idx,
				>>::inject(RcCoyoneda::lift(ask));

				fo_handlers.dispatch(ask_layer).bind(move |parent_env| {
					let modified_env = (*modify)(parent_env);
					action(()).interpose::<ReaderBrand<RcBrand, i32>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => (*k)(modified_env),
						},
					)
				})
			}
		}
	}
}

struct RefLocalDispatcher<Idx, RMinusE, EmbedIndices>(
	PhantomData<fn() -> (Idx, RMinusE, EmbedIndices)>,
);

impl<Idx, RMinusE, EmbedIndices> RefLocalDispatcher<Idx, RMinusE, EmbedIndices> {
	const fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Idx, RMinusE, EmbedIndices>
	DispatchScopedHandler<
		'static,
		RefLocal<'static, RcBrand, i32, ControlProg>,
		ControlFirstLayer,
		ControlProg,
	> for RefLocalDispatcher<Idx, RMinusE, EmbedIndices>
where
	RMinusE: WrapDrop + Functor + 'static,
	ControlFirstLayer: Member<
			RcCoyoneda<'static, ReaderBrand<RcBrand, i32>, ControlProg>,
			Idx,
			Remainder = Apply!(
							<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ControlProg>
						),
		>,
	Apply!(<RMinusE as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<FirstRow, ControlScopedRow>, i32>,
	>): CoproductEmbedder<
			Apply!(<FirstRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<FirstRow, ControlScopedRow>, i32>,
			>),
			EmbedIndices,
		>,
	Apply!(<NodeBrand<FirstRow, ControlScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<FirstRow, ControlScopedRow>, RcTypeErasedValue>,
	>): Clone,
{
	fn dispatch_scoped_head(
		&self,
		layer: RefLocal<'static, RcBrand, i32, ControlProg>,
		fo_handlers: &impl DispatchHandlers<'static, ControlFirstLayer, ControlProg>,
	) -> ControlProg {
		match layer {
			RefLocal::Local {
				modify,
				action,
			} => {
				let ask: Reader<'static, RcBrand, i32, ControlProg> =
					Reader::Ask(<RcBrand as ToDynCloneFn>::new(RcRun::pure));
				let ask_layer = <ControlFirstLayer as Member<
					RcCoyoneda<'static, ReaderBrand<RcBrand, i32>, ControlProg>,
					Idx,
				>>::inject(RcCoyoneda::lift(ask));

				fo_handlers.dispatch(ask_layer).bind(move |parent_env| {
					let modified_env = (*modify)(&parent_env);
					action(()).interpose::<ReaderBrand<RcBrand, i32>, Idx, RMinusE, EmbedIndices>(
						move |op| match op {
							Reader::Ask(k) => (*k)(modified_env),
						},
					)
				})
			}
		}
	}
}

struct SpanDispatcher;

impl
	DispatchScopedHandler<
		'static,
		Span<'static, RcBrand, &'static str, ControlProg>,
		ControlFirstLayer,
		ControlProg,
	> for SpanDispatcher
{
	fn dispatch_scoped_head(
		&self,
		layer: Span<'static, RcBrand, &'static str, ControlProg>,
		_fo_handlers: &impl DispatchHandlers<'static, ControlFirstLayer, ControlProg>,
	) -> ControlProg {
		match layer {
			Span::Span {
				tag: _,
				action,
			} => action(()),
		}
	}
}

struct BracketDispatcher;

impl
	DispatchScopedHandler<
		'static,
		Bracket<'static, RcBrand, NodeBrand<FirstRow, BracketScopedRow>, i32, i32>,
		BracketFirstLayer,
		BracketProg,
	> for BracketDispatcher
{
	fn dispatch_scoped_head(
		&self,
		layer: Bracket<'static, RcBrand, NodeBrand<FirstRow, BracketScopedRow>, i32, i32>,
		_fo_handlers: &impl DispatchHandlers<'static, BracketFirstLayer, BracketProg>,
	) -> BracketProg {
		match layer {
			Bracket::Bracket {
				acquire,
				body,
				release,
			} => RcRun::from_rc_free(acquire(())).bind(move |resource| {
				let body_resource = Rc::new(resource);
				let release_for_body = release.clone();
				RcRun::from_rc_free((*body)(body_resource)).bind(move |(resource, body_result)| {
					RcRun::from_rc_free((*release_for_body)(Rc::new(resource)))
						.map(move |()| body_result)
				})
			}),
		}
	}
}

struct RefBracketDispatcher;

impl
	DispatchScopedHandler<
		'static,
		RefBracket<'static, RcBrand, NodeBrand<FirstRow, RefBracketScopedRow>, i32, i32>,
		RefBracketFirstLayer,
		RefBracketProg,
	> for RefBracketDispatcher
{
	fn dispatch_scoped_head(
		&self,
		layer: RefBracket<'static, RcBrand, NodeBrand<FirstRow, RefBracketScopedRow>, i32, i32>,
		_fo_handlers: &impl DispatchHandlers<'static, RefBracketFirstLayer, RefBracketProg>,
	) -> RefBracketProg {
		match layer {
			RefBracket::Bracket {
				acquire,
				body,
				release,
			} => RcRun::from_rc_free(acquire(())).bind(move |resource| {
				let resource = Rc::new(resource);
				let release_resource = resource.clone();
				let release_for_body = release.clone();
				RcRun::from_rc_free((*body)(resource)).bind(move |body_result| {
					RcRun::from_rc_free((*release_for_body)(release_resource.clone()))
						.map(move |()| body_result)
				})
			}),
		}
	}
}

fn interpret_control_static(
	start: ControlProg,
	handlers: impl DispatchHandlers<'static, ControlFirstLayer, ControlProg>,
	scoped_handlers: impl DispatchScopedHandlers<
		'static,
		ControlScopedLayer,
		ControlFirstLayer,
		ControlProg,
	>,
) -> i32 {
	let mut prog = start;
	loop {
		match prog.peel() {
			Ok(a) => return a,
			Err(Node::First(layer)) => prog = handlers.dispatch(layer),
			Err(Node::Scoped(layer)) => prog = scoped_handlers.dispatch_scoped(layer, &handlers),
		}
	}
}

fn interpret_bracket_static(
	start: BracketProg,
	handlers: impl DispatchHandlers<'static, BracketFirstLayer, BracketProg>,
	scoped_handlers: impl DispatchScopedHandlers<
		'static,
		BracketScopedLayer,
		BracketFirstLayer,
		BracketProg,
	>,
) -> i32 {
	let mut prog = start;
	loop {
		match prog.peel() {
			Ok(a) => return a,
			Err(Node::First(layer)) => prog = handlers.dispatch(layer),
			Err(Node::Scoped(layer)) => prog = scoped_handlers.dispatch_scoped(layer, &handlers),
		}
	}
}

fn interpret_ref_bracket_static(
	start: RefBracketProg,
	handlers: impl DispatchHandlers<'static, RefBracketFirstLayer, RefBracketProg>,
	scoped_handlers: impl DispatchScopedHandlers<
		'static,
		RefBracketScopedLayer,
		RefBracketFirstLayer,
		RefBracketProg,
	>,
) -> i32 {
	let mut prog = start;
	loop {
		match prog.peel() {
			Ok(a) => return a,
			Err(Node::First(layer)) => prog = handlers.dispatch(layer),
			Err(Node::Scoped(layer)) => prog = scoped_handlers.dispatch_scoped(layer, &handlers),
		}
	}
}

fn interpret_explicit_control<'a>(
	start: ExplicitControlProg<'a>,
	handlers: impl DispatchHandlers<'a, ExplicitControlFirstLayer<'a>, ExplicitControlProg<'a>>,
	scoped_handlers: impl DispatchScopedHandlers<
		'a,
		ExplicitControlScopedLayer<'a>,
		ExplicitControlFirstLayer<'a>,
		ExplicitControlProg<'a>,
	>,
) -> i32 {
	let mut prog = start;
	loop {
		match prog.peel() {
			Ok(a) => return a,
			Err(Node::First(layer)) => prog = handlers.dispatch(layer),
			Err(Node::Scoped(layer)) => prog = scoped_handlers.dispatch_scoped(layer, &handlers),
		}
	}
}

#[test]
fn rc_static_lifetime_dispatchers_cover_catch_local_ref_local_and_span() {
	let action: ControlProg =
		RcRun::throw::<&'static str, _>("boom").bind(|_: i32| RcRun::ask::<_>());
	let program = RcRun::catch::<&'static str, _>(action, |_e| {
		RcRun::span::<&'static str, _>(
			"recovered",
			RcRun::local::<i32, _>(|env| env + 1, RcRun::ask::<_>()),
		)
	});

	let result = interpret_control_static(
		program,
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ControlProg>| {
				panic!("catch dispatcher should replace throws before the FO handler sees them")
			},
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, ControlProg>| match op {
				Reader::Ask(k) => (*k)(41),
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: CatchDispatcher::<_, FirstRowMinusExcept, _>::new(),
			LocalBrand<RcBrand, i32>: LocalDispatcher::<_, FirstRowMinusReader, _>::new(),
			RefLocalBrand<RcBrand, i32>: RefLocalDispatcher::<_, FirstRowMinusReader, _>::new(),
			SpanBrand<RcBrand, &'static str>: SpanDispatcher,
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_explicit_lifetime_dispatchers_cover_catch_and_local() {
	let action: ExplicitControlProg<'static> =
		RcRunExplicit::throw::<&'static str, _>("boom").bind(|_: i32| RcRunExplicit::ask::<_>());
	let local_boundary = RcRunExplicit::local::<i32, _>(|env| env + 1, RcRunExplicit::ask::<_>());
	let local_program: ExplicitControlProg<'static> =
		fp_library::types::effects::scoped_dispatchers::local_dispatcher::<
			_,
			FirstRowMinusReader,
			_,
		>()
		.dispatch_rc_run_explicit_local_boundary(local_boundary, &handlers! {});
	let catch_boundary =
		RcRunExplicit::catch::<&'static str, _>(action, move |_e| local_program.clone());
	let program: ExplicitControlProg<'static> =
		fp_library::types::effects::scoped_dispatchers::catch_dispatcher::<
			_,
			FirstRowMinusExcept,
			_,
		>()
		.dispatch_rc_run_explicit_catch_boundary(catch_boundary, &handlers! {});

	let result = interpret_explicit_control(
		program,
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ExplicitControlProg<'static>>| {
				panic!("catch dispatcher should replace throws before the FO handler sees them")
			},
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, ExplicitControlProg<'static>>| match op {
				Reader::Ask(k) => (*k)(41),
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: CatchDispatcher::<_, FirstRowMinusExcept, _>::new(),
			LocalBrand<RcBrand, i32>: LocalDispatcher::<_, FirstRowMinusReader, _>::new(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_static_lifetime_dispatcher_covers_ref_local() {
	let program =
		RcRun::ref_local::<i32, _>(|env| env + 2, RcRun::<FirstRow, ControlScopedRow, i32>::ask());

	let result = interpret_control_static(
		program,
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ControlProg>| {
				panic!("test program does not throw")
			},
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, ControlProg>| match op {
				Reader::Ask(k) => (*k)(40),
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: CatchDispatcher::<_, FirstRowMinusExcept, _>::new(),
			LocalBrand<RcBrand, i32>: LocalDispatcher::<_, FirstRowMinusReader, _>::new(),
			RefLocalBrand<RcBrand, i32>: RefLocalDispatcher::<_, FirstRowMinusReader, _>::new(),
			SpanBrand<RcBrand, &'static str>: SpanDispatcher,
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_static_lifetime_dispatcher_covers_bracket() {
	let program = RcRun::<FirstRow, BracketScopedRow, i32>::bracket::<i32, _>(
		RcRun::pure(7),
		|resource| RcRun::pure((*resource, *resource + 35)),
		|_resource| RcRun::pure(()),
	);

	let result = interpret_bracket_static(
		program,
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BracketProg>| {
				panic!("test program does not throw")
			},
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, BracketProg>| match op {
				Reader::Ask(k) => (*k)(0),
			},
		},
		scoped_handlers! {
			BracketBrand<RcBrand, NodeBrand<FirstRow, BracketScopedRow>, i32, i32>: BracketDispatcher,
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_static_lifetime_dispatcher_covers_ref_bracket() {
	let program = RcRun::<FirstRow, RefBracketScopedRow, i32>::ref_bracket::<i32, _>(
		RcRun::pure(7),
		|resource| RcRun::pure(*resource + 35),
		|_resource| RcRun::pure(()),
	);

	let result = interpret_ref_bracket_static(
		program,
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RefBracketProg>| {
				panic!("test program does not throw")
			},
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RefBracketProg>| match op {
				Reader::Ask(k) => (*k)(0),
			},
		},
		scoped_handlers! {
			RefBracketBrand<RcBrand, NodeBrand<FirstRow, RefBracketScopedRow>, i32, i32>: RefBracketDispatcher,
		},
	);

	assert_eq!(result, 42);
}
