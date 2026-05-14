// Criterion benches for scoped-operation overhead. The comparison pairs
// Bracket / RefBracket construction and dispatcher execution against
// equivalent non-scoped bind chains that simulate acquire/body/release through
// ordinary closure capture.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		Apply,
		brands::{
			BoxBracketBrand,
			BoxBrand,
			BracketBrand,
			CNilBrand,
			CoproductBrand,
			NodeBrand,
			RcBrand,
			RefBracketBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		handlers,
		impl_kind,
		kinds::*,
		scoped_handlers,
		types::effects::{
			rc_run::RcRun,
			run::Run,
			scoped_dispatchers::{
				bracket_dispatcher,
				ref_bracket_dispatcher,
			},
		},
	},
	std::hint::black_box,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RunBracketRow;

type RunBracketUnderlyingRow = CoproductBrand<
	BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, RunBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RunBracketUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RunBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RunBracketUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RunBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RunBracketUnderlyingRow as Functor>::map(f, fa)
	}
}

type RunBracketProg = Run<CNilBrand, RunBracketRow, i32>;
type RunManualProg = Run<CNilBrand, CNilBrand, i32>;

fn make_run_bracket(resource: i32) -> RunBracketProg {
	let acquire: RunBracketProg = Run::pure(resource);
	Run::<CNilBrand, RunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: Box<i32>| Run::pure((*resource, *resource + 35)),
		|_resource: Box<i32>| Run::pure(()),
	)
}

fn interpret_run_bracket(program: RunBracketProg) -> i32 {
	program.interpret(
		handlers! {},
		scoped_handlers! {
			BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, RunBracketRow>, i32, i32>: bracket_dispatcher(),
		},
	)
}

fn make_run_manual(resource: i32) -> RunManualProg {
	Run::pure(resource).bind(|resource| {
		let result = resource + 35;
		Run::<CNilBrand, CNilBrand, ()>::pure(()).bind(move |()| Run::pure(result))
	})
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunBracketRow;

type RcRunBracketUnderlyingRow = CoproductBrand<
	BracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunBracketUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunBracketUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunBracketUnderlyingRow as Functor>::map(f, fa)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunRefBracketRow;

type RcRunRefBracketUnderlyingRow = CoproductBrand<
	RefBracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunRefBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunRefBracketUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunRefBracketUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunRefBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunRefBracketUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunBracketProg = RcRun<CNilBrand, RcRunBracketRow, i32>;
type RcRunRefBracketProg = RcRun<CNilBrand, RcRunRefBracketRow, i32>;
type RcRunManualProg = RcRun<CNilBrand, CNilBrand, i32>;

fn make_rc_run_bracket(resource: i32) -> RcRunBracketProg {
	let acquire: RcRunBracketProg = RcRun::pure(resource);
	RcRun::<CNilBrand, RcRunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRun::pure((*resource, *resource + 35)),
		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
	)
}

fn interpret_rc_run_bracket(program: RcRunBracketProg) -> i32 {
	program.interpret(
		handlers! {},
		scoped_handlers! {
			BracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunBracketRow>, i32, i32>: bracket_dispatcher(),
		},
	)
}

fn make_rc_run_ref_bracket(resource: i32) -> RcRunRefBracketProg {
	let acquire: RcRunRefBracketProg = RcRun::pure(resource);
	RcRun::<CNilBrand, RcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRun::pure(*resource + 35),
		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
	)
}

fn interpret_rc_run_ref_bracket(program: RcRunRefBracketProg) -> i32 {
	program.interpret(
		handlers! {},
		scoped_handlers! {
			RefBracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunRefBracketRow>, i32, i32>: ref_bracket_dispatcher(),
		},
	)
}

fn make_rc_run_manual(resource: i32) -> RcRunManualProg {
	RcRun::pure(resource).bind(|resource| {
		let result = resource + 35;
		RcRun::<CNilBrand, CNilBrand, ()>::pure(()).bind(move |()| RcRun::pure(result))
	})
}

fn make_rc_run_ref_manual(resource: i32) -> RcRunManualProg {
	RcRun::pure(resource).bind(|resource| {
		let shared = std::rc::Rc::new(resource);
		let result = *shared + 35;
		let release_resource = std::rc::Rc::clone(&shared);
		RcRun::<CNilBrand, CNilBrand, ()>::pure(()).bind(move |()| {
			let _ = std::rc::Rc::strong_count(&release_resource);
			RcRun::pure(result)
		})
	})
}

fn bench_run_bracket(c: &mut Criterion) {
	let mut group = c.benchmark_group("Scoped Operations Run Bracket");

	group.bench_function(BenchmarkId::new("construct scoped", "BoxBracket"), |b| {
		b.iter(|| black_box(make_run_bracket(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("construct manual", "bind-chain"), |b| {
		b.iter(|| black_box(make_run_manual(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("dispatch scoped", "BoxBracket"), |b| {
		b.iter_batched(
			|| make_run_bracket(black_box(7)),
			|program| black_box(interpret_run_bracket(program)),
			BatchSize::SmallInput,
		)
	});
	group.bench_function(BenchmarkId::new("extract manual", "bind-chain"), |b| {
		b.iter_batched(
			|| make_run_manual(black_box(7)),
			|program| black_box(program.extract()),
			BatchSize::SmallInput,
		)
	});

	group.finish();
}

fn bench_rc_run_bracket(c: &mut Criterion) {
	let mut group = c.benchmark_group("Scoped Operations RcRun Bracket");

	group.bench_function(BenchmarkId::new("construct scoped", "Bracket"), |b| {
		b.iter(|| black_box(make_rc_run_bracket(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("construct manual", "bind-chain"), |b| {
		b.iter(|| black_box(make_rc_run_manual(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("dispatch scoped", "Bracket"), |b| {
		b.iter_batched(
			|| make_rc_run_bracket(black_box(7)),
			|program| black_box(interpret_rc_run_bracket(program)),
			BatchSize::SmallInput,
		)
	});
	group.bench_function(BenchmarkId::new("extract manual", "bind-chain"), |b| {
		b.iter_batched(
			|| make_rc_run_manual(black_box(7)),
			|program| black_box(program.extract()),
			BatchSize::SmallInput,
		)
	});

	group.finish();
}

fn bench_rc_run_ref_bracket(c: &mut Criterion) {
	let mut group = c.benchmark_group("Scoped Operations RcRun RefBracket");

	group.bench_function(BenchmarkId::new("construct scoped", "RefBracket"), |b| {
		b.iter(|| black_box(make_rc_run_ref_bracket(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("construct manual", "Rc bind-chain"), |b| {
		b.iter(|| black_box(make_rc_run_ref_manual(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("dispatch scoped", "RefBracket"), |b| {
		b.iter_batched(
			|| make_rc_run_ref_bracket(black_box(7)),
			|program| black_box(interpret_rc_run_ref_bracket(program)),
			BatchSize::SmallInput,
		)
	});
	group.bench_function(BenchmarkId::new("extract manual", "Rc bind-chain"), |b| {
		b.iter_batched(
			|| make_rc_run_ref_manual(black_box(7)),
			|program| black_box(program.extract()),
			BatchSize::SmallInput,
		)
	});

	group.finish();
}

pub fn bench_scoped_operations(c: &mut Criterion) {
	bench_run_bracket(c);
	bench_rc_run_bracket(c);
	bench_rc_run_ref_bracket(c);
}
