use crate::canonicalize::Canonicalizer;
use crate::generate::generate_name;
use crate::parse::{KindInput, TypeInput};
use syn::punctuated::Punctuated;
use syn::{Lifetime, Token, TypeParamBound, parse::Parse, parse_quote};

fn parse_kind_input(input: &str) -> KindInput {
	syn::parse_str(input).expect("Failed to parse KindInput")
}

#[test]
fn test_canonicalize_simple_bound() {
	let lifetimes = Punctuated::new();
	let types = Punctuated::new();
	let canon = Canonicalizer::new(&lifetimes, &types);

	let bound: TypeParamBound = parse_quote!(Clone);
	assert_eq!(canon.canonicalize_bound(&bound), "tClone");
}

#[test]
fn test_canonicalize_path_bound() {
	let lifetimes = Punctuated::new();
	let types = Punctuated::new();
	let canon = Canonicalizer::new(&lifetimes, &types);

	let bound: TypeParamBound = parse_quote!(std::fmt::Debug);
	assert_eq!(canon.canonicalize_bound(&bound), "tstd::fmt::Debug");
}

#[test]
fn test_canonicalize_generic_bound() {
	let lifetimes = Punctuated::new();
	let types = Punctuated::new();
	let canon = Canonicalizer::new(&lifetimes, &types);

	let bound: TypeParamBound = parse_quote!(Iterator<Item = String>);
	assert_eq!(canon.canonicalize_bound(&bound), "tIterator<Item=String>");
}

#[test]
fn test_canonicalize_fn_bound() {
	let lifetimes = Punctuated::new();
	let types = Punctuated::new();
	let canon = Canonicalizer::new(&lifetimes, &types);

	let bound: TypeParamBound = parse_quote!(Fn(i32) -> bool);
	assert_eq!(canon.canonicalize_bound(&bound), "tFn(i32)->bool");
}

#[test]
fn test_canonicalize_lifetime_bound() {
	let mut lifetimes = Punctuated::new();
	lifetimes.push(parse_quote!('a));
	let types = Punctuated::new();
	let canon = Canonicalizer::new(&lifetimes, &types);

	let bound: TypeParamBound = parse_quote!('a);
	assert_eq!(canon.canonicalize_bound(&bound), "l0");
}

#[test]
fn test_canonicalize_bounds_sorting() {
	let lifetimes = Punctuated::new();
	let types = Punctuated::new();
	let canon = Canonicalizer::new(&lifetimes, &types);

	// These should produce the same result regardless of order
	let bounds1: Punctuated<TypeParamBound, Token![+]> = parse_quote!(Clone + std::fmt::Debug);
	let bounds2: Punctuated<TypeParamBound, Token![+]> = parse_quote!(std::fmt::Debug + Clone);

	assert_eq!(canon.canonicalize_bounds(&bounds1), canon.canonicalize_bounds(&bounds2));
}

#[test]
fn test_generate_name_determinism() {
	let input1 = parse_kind_input("('a), (A: 'a), ('a)");
	let name1 = generate_name(&input1);

	let input2 = parse_kind_input("('a), (A: 'a), ('a)");
	let name2 = generate_name(&input2);

	assert_eq!(name1, name2);
	assert!(name1.to_string().starts_with("Kind_"));
}

#[test]
fn test_generate_name_different_inputs() {
	let input1 = parse_kind_input("('a), (A: 'a), ('a)");
	let name1 = generate_name(&input1);

	let input2 = parse_kind_input("(), (A), ()");
	let name2 = generate_name(&input2);

	assert_ne!(name1, name2);
}
