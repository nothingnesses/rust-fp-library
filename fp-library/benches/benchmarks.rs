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
mod option;
#[path = "benchmarks/pair.rs"]
mod pair;
#[path = "benchmarks/result.rs"]
mod result;
#[path = "benchmarks/string.rs"]
mod string;
#[path = "benchmarks/vec.rs"]
mod vec;

use {
	cat_list::bench_cat_list,
	coyoneda::bench_coyoneda,
	functions::bench_functions,
	lazy::bench_lazy,
	option::bench_option,
	pair::bench_pair,
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
	bench_coyoneda
);
criterion_main!(benches);
