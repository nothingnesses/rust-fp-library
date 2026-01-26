use criterion::{criterion_group, criterion_main};

#[path = "benchmarks/cat_list.rs"]
mod cat_list;
#[path = "benchmarks/cat_queue.rs"]
mod cat_queue;
#[path = "benchmarks/functions.rs"]
mod functions;
#[path = "benchmarks/once_cell.rs"]
mod once_cell;
#[path = "benchmarks/once_lock.rs"]
mod once_lock;
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

use cat_list::bench_cat_list;
use cat_queue::bench_cat_queue;
use functions::bench_functions;
use once_cell::bench_once_cell;
use once_lock::bench_once_lock;
use option::bench_option;
use pair::bench_pair;
use result::bench_result;
use string::bench_string;
use vec::bench_vec;

criterion_group!(
	benches,
	bench_vec,
	bench_option,
	bench_result,
	bench_pair,
	bench_string,
	bench_functions,
	bench_once_cell,
	bench_once_lock,
	bench_cat_list,
	bench_cat_queue
);
criterion_main!(benches);
