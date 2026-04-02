//! ```cargo
//! [dependencies]
//! fp-library = { path = "../fp-library" }
//! ```
//!
//! Probes the stack overflow threshold for recursive types.
//!
//! Runs each type's recursive operation at increasing depths using subprocess
//! isolation, so a stack overflow in one probe does not crash the rest.
//! Reports the depth at which each type overflows (or "OK" if it survives
//! all tested depths).
//!
//! Usage:
//!   rust-script scripts/stack_probe.rs
//!   rust-script scripts/stack_probe.rs -- <probe_name> <depth>
//!
//! Probes: rc_lazy, arc_lazy, coyoneda, rc_coyoneda, arc_coyoneda,
//! thunk_map, thunk_bind.

use {
	fp_library::{
		brands::VecBrand,
		types::{
			ArcCoyoneda,
			ArcLazy,
			ArcLazyConfig,
			Coyoneda,
			Lazy,
			RcCoyoneda,
			RcLazyConfig,
			Thunk,
		},
	},
	std::process::Command,
};

fn run_single(
	name: &str,
	depth: usize,
) {
	match name {
		"rc_lazy" => {
			let mut lazy = Lazy::<_, RcLazyConfig>::new(|| 0i64);
			for _ in 0 .. depth {
				lazy = lazy.ref_map(|x| *x + 1);
			}
			let _ = *lazy.evaluate();
		}
		"arc_lazy" => {
			let mut lazy: Lazy<'_, i64, ArcLazyConfig> = ArcLazy::new(|| 0i64);
			for _ in 0 .. depth {
				lazy = lazy.ref_map(|x| *x + 1);
			}
			let _ = *lazy.evaluate();
		}
		"coyoneda" => {
			let v: Vec<i32> = (0 .. 100).collect();
			let mut coyo = Coyoneda::<VecBrand, _>::lift(v);
			for _ in 0 .. depth {
				coyo = coyo.map(|x: i32| x + 1);
			}
			let _ = coyo.lower();
		}
		"rc_coyoneda" => {
			let v: Vec<i32> = (0 .. 100).collect();
			let mut coyo = RcCoyoneda::<VecBrand, _>::lift(v);
			for _ in 0 .. depth {
				coyo = coyo.map(|x: i32| x + 1);
			}
			let _ = coyo.lower_ref();
		}
		"arc_coyoneda" => {
			let v: Vec<i32> = (0 .. 100).collect();
			let mut coyo = ArcCoyoneda::<VecBrand, _>::lift(v);
			for _ in 0 .. depth {
				coyo = coyo.map(|x: i32| x + 1);
			}
			let _ = coyo.lower_ref();
		}
		"thunk_map" => {
			let mut thunk = Thunk::new(|| 0i64);
			for _ in 0 .. depth {
				thunk = thunk.map(|x| x + 1);
			}
			let _ = thunk.evaluate();
		}
		"thunk_bind" => {
			let mut thunk = Thunk::new(|| 0i64);
			for _ in 0 .. depth {
				thunk = thunk.bind(|x| Thunk::pure(x + 1));
			}
			let _ = thunk.evaluate();
		}
		_ => {
			eprintln!("Unknown probe: {name}");
			std::process::exit(2);
		}
	}
}

fn probe_via_subprocess(
	script_path: &str,
	probe_name: &str,
	display_name: &str,
	depths: &[usize],
) {
	for &depth in depths {
		let output = Command::new("rust-script")
			.args([script_path, "--", probe_name, &depth.to_string()])
			.output()
			.expect("Failed to spawn rust-script subprocess");

		if output.status.success() {
			println!("{display_name} depth {depth}: OK");
		} else {
			println!("{display_name} depth {depth}: CRASHED (stack overflow)");
			return;
		}
	}
}

fn main() {
	let args: Vec<String> = std::env::args().collect();

	// Subprocess mode: run a single probe and exit.
	// When invoked by rust-script with "-- probe_name depth", args look like:
	//   [binary_path, probe_name, depth]
	// Detect this by checking if arg[1] matches a known probe name.
	if args.len() >= 3 {
		if let Ok(depth) = args[args.len() - 1].parse::<usize>() {
			let name = &args[args.len() - 2];
			run_single(name, depth);
			return;
		}
	}

	// Main mode: run all probes via subprocesses.
	// Determine the script path from the RUST_SCRIPT_PATH env var or fall back
	// to a default relative to the repo root.
	let script_path =
		std::env::var("RUST_SCRIPT_PATH").unwrap_or_else(|_| "scripts/stack_probe.rs".to_string());

	let depths: &[usize] =
		&[100, 500, 1000, 2000, 5000, 10000, 15000, 20000, 25000, 30000, 35000, 40000, 50000];

	let probes = [
		("rc_lazy", "RcLazy ref_map"),
		("arc_lazy", "ArcLazy ref_map"),
		("coyoneda", "Coyoneda lower"),
		("rc_coyoneda", "RcCoyoneda lower_ref"),
		("arc_coyoneda", "ArcCoyoneda lower_ref"),
		("thunk_map", "Thunk map chain"),
		("thunk_bind", "Thunk bind chain"),
	];

	for (probe_name, display_name) in probes {
		probe_via_subprocess(&script_path, probe_name, display_name, depths);
		println!();
	}
}
