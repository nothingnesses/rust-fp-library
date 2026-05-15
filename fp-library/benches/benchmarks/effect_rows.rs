// Criterion benches for effect-row canonicalisation and handler-list
// composition. The macro canonicalisation path has no runtime sorting step:
// the macro emits a canonical row at expansion time, so the corresponding
// runtime benchmark is direct construction of a value already in canonical
// shape. The fallback path benchmarks the runtime `CoproductSubsetter`
// permutation proof used when callers hand-write a non-canonical coproduct.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		handlers,
		types::effects::{
			coproduct::{
				CNil,
				Coproduct,
				CoproductSubsetter,
			},
			nt,
		},
	},
	std::hint::black_box,
};

#[derive(Clone, Copy)]
struct AlphaBrand;

#[derive(Clone, Copy)]
struct BetaBrand;

#[derive(Clone, Copy)]
struct GammaBrand;

#[derive(Clone, Copy)]
struct DeltaBrand;

#[derive(Clone, Copy)]
struct EpsilonBrand;

type Canonical3 = Coproduct<i32, Coproduct<bool, Coproduct<&'static str, CNil>>>;
type Reverse3 = Coproduct<&'static str, Coproduct<bool, Coproduct<i32, CNil>>>;

type Canonical5 =
	Coproduct<i32, Coproduct<bool, Coproduct<&'static str, Coproduct<u8, Coproduct<f32, CNil>>>>>;
type Reverse5 =
	Coproduct<f32, Coproduct<u8, Coproduct<&'static str, Coproduct<bool, Coproduct<i32, CNil>>>>>;

fn canonical3_value(value: i32) -> Canonical3 {
	Coproduct::Inl(value)
}

fn reverse3_value(value: i32) -> Reverse3 {
	Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(value)))
}

fn subset_reverse3(value: Reverse3) -> Canonical3 {
	<Reverse3 as CoproductSubsetter<Canonical3, _>>::subset(value).expect("3-effect permutation")
}

fn canonical5_value(value: i32) -> Canonical5 {
	Coproduct::Inl(value)
}

fn reverse5_value(value: i32) -> Reverse5 {
	Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(value)))))
}

fn subset_reverse5(value: Reverse5) -> Canonical5 {
	<Reverse5 as CoproductSubsetter<Canonical5, _>>::subset(value).expect("5-effect permutation")
}

fn macro_handlers3(seed: i32) -> impl Sized {
	let alpha = seed;
	let beta = seed + 1;
	let gamma = seed + 2;
	handlers! {
		GammaBrand: move |value: i32| value + gamma,
		AlphaBrand: move |value: i32| value + alpha,
		BetaBrand: move |value: i32| value + beta,
	}
}

fn builder_handlers3(seed: i32) -> impl Sized {
	let alpha = seed;
	let beta = seed + 1;
	let gamma = seed + 2;
	nt().on::<GammaBrand, _>(move |value: i32| value + gamma)
		.on::<BetaBrand, _>(move |value: i32| value + beta)
		.on::<AlphaBrand, _>(move |value: i32| value + alpha)
}

fn macro_handlers5(seed: i32) -> impl Sized {
	let alpha = seed;
	let beta = seed + 1;
	let gamma = seed + 2;
	let delta = seed + 3;
	let epsilon = seed + 4;
	handlers! {
		EpsilonBrand: move |value: i32| value + epsilon,
		GammaBrand: move |value: i32| value + gamma,
		AlphaBrand: move |value: i32| value + alpha,
		DeltaBrand: move |value: i32| value + delta,
		BetaBrand: move |value: i32| value + beta,
	}
}

fn builder_handlers5(seed: i32) -> impl Sized {
	let alpha = seed;
	let beta = seed + 1;
	let gamma = seed + 2;
	let delta = seed + 3;
	let epsilon = seed + 4;
	nt().on::<GammaBrand, _>(move |value: i32| value + gamma)
		.on::<EpsilonBrand, _>(move |value: i32| value + epsilon)
		.on::<DeltaBrand, _>(move |value: i32| value + delta)
		.on::<BetaBrand, _>(move |value: i32| value + beta)
		.on::<AlphaBrand, _>(move |value: i32| value + alpha)
}

fn bench_row_canonicalisation(c: &mut Criterion) {
	let mut group = c.benchmark_group("Effect Rows Row Canonicalisation");

	group.bench_function(BenchmarkId::new("direct canonical", "3 effects"), |b| {
		b.iter(|| black_box(canonical3_value(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("subset fallback", "3 effects"), |b| {
		b.iter_batched(
			|| reverse3_value(black_box(7)),
			|value| black_box(subset_reverse3(value)),
			BatchSize::SmallInput,
		)
	});
	group.bench_function(BenchmarkId::new("direct canonical", "5 effects"), |b| {
		b.iter(|| black_box(canonical5_value(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("subset fallback", "5 effects"), |b| {
		b.iter_batched(
			|| reverse5_value(black_box(7)),
			|value| black_box(subset_reverse5(value)),
			BatchSize::SmallInput,
		)
	});

	group.finish();
}

fn bench_handler_composition(c: &mut Criterion) {
	let mut group = c.benchmark_group("Effect Rows Handler Composition");

	group.bench_function(BenchmarkId::new("handlers macro", "3 handlers"), |b| {
		b.iter(|| black_box(macro_handlers3(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("builder chain", "3 handlers"), |b| {
		b.iter(|| black_box(builder_handlers3(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("handlers macro", "5 handlers"), |b| {
		b.iter(|| black_box(macro_handlers5(black_box(7))))
	});
	group.bench_function(BenchmarkId::new("builder chain", "5 handlers"), |b| {
		b.iter(|| black_box(builder_handlers5(black_box(7))))
	});

	group.finish();
}

pub fn bench_effect_rows(c: &mut Criterion) {
	bench_row_canonicalisation(c);
	bench_handler_composition(c);
}
