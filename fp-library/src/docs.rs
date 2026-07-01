#![allow(rustdoc::invalid_rust_codeblocks)]
//! Design documentation for fp-library.
//!
//! Each submodule contains one design document from the `docs/` directory,
//! with cross-document links rewritten as rustdoc intra-doc links. This
//! makes links between documents work in rendered documentation (docs.rs
//! and local `cargo doc` builds).

pub mod architecture;
pub mod benchmarking;
pub mod brand_inference;
pub mod coyoneda;
pub mod dispatch;
pub mod features;
pub mod hkt;
pub mod impl_trait_vs_named_generics;
pub mod lazy_evaluation;
pub mod lifetime_ablation_experiment;
pub mod limitations_and_workarounds;
pub mod optics_analysis;
pub mod parallelism;
pub mod pointer_abstraction;
pub mod profunctor_analysis;
pub mod project_structure;
pub mod references;
pub mod release_process;
pub mod std_coverage_checklist;
pub mod zero_cost;
