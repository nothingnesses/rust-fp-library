use criterion::{
	criterion_group,
	criterion_main,
};

#[path = "benchmarks/cat_list.rs"]
mod cat_list;
#[path = "benchmarks/coyoneda.rs"]
mod coyoneda;
#[path = "benchmarks/functions.rs"]
mod functions;
#[path = "benchmarks/lazy.rs"]
mod lazy;
#[path = "benchmarks/option.rs"]
#[expect(
	clippy::identity_op,
	clippy::unnecessary_map_or,
	clippy::bind_instead_of_map,
	reason = "Intentional operations for fair std-vs-fp benchmark comparison"
)]
mod option;
#[path = "benchmarks/pair.rs"]
#[expect(
	clippy::identity_op,
	reason = "Intentional identity operations for fair std-vs-fp benchmark comparison"
)]
mod pair;
#[path = "benchmarks/ref_dispatch.rs"]
mod ref_dispatch;
#[path = "benchmarks/result.rs"]
#[expect(
	clippy::identity_op,
	clippy::bind_instead_of_map,
	reason = "Intentional operations for fair std-vs-fp benchmark comparison"
)]
mod result;
#[path = "benchmarks/string.rs"]
mod string;
#[path = "benchmarks/vec.rs"]
#[expect(
	clippy::unnecessary_fold,
	reason = "Intentional fold for fair std-vs-fp benchmark comparison"
)]
mod vec;

use {
	cat_list::bench_cat_list,
	coyoneda::bench_coyoneda,
	functions::bench_functions,
	lazy::bench_lazy,
	option::bench_option,
	pair::bench_pair,
	ref_dispatch::bench_ref_dispatch,
	result::bench_result,
	string::bench_string,
	vec::bench_vec,
};

criterion_group!(
	benches,
	bench_vec,
	bench_option,
	bench_result,
	bench_pair,
	bench_string,
	bench_functions,
	bench_cat_list,
	bench_lazy,
	bench_coyoneda,
	bench_ref_dispatch
);
criterion_main!(benches);
