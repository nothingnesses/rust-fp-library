//! Code generation for the `a_do!` macro.
//!
//! Transforms parsed `DoInput` into `pure` / `map` / `liftN` calls.

use {
	crate::{
		m_do::codegen::{
			format_bind_param,
			format_discard_param,
			rewrite_pure,
			wrap_container_ref,
		},
		support::do_input::{
			DoInput,
			DoStatement,
		},
	},
	proc_macro2::TokenStream,
	quote::{
		format_ident,
		quote,
	},
};

/// Generates the expanded token stream for an `a_do!` invocation.
///
/// Separates statements into binds (applicative computations) and let bindings,
/// then desugars into the appropriate combinator:
///
/// - 0 binds -> `pure::<Brand, _>(final_expr)` (explicit) or `compile_error!` (inferred)
/// - 1 bind  -> `explicit::map::<Brand, ...>(...)` (explicit) or `map(...)` (inferred)
/// - N binds -> `explicit::liftN::<Brand, ...>(...)` (explicit) or `liftN(...)` (inferred)
pub fn a_do_worker(input: DoInput) -> syn::Result<TokenStream> {
	let brand = input.brand.as_ref();
	let ref_mode = input.ref_mode;

	let mut leading_lets: Vec<TokenStream> = vec![];
	let mut inner_lets: Vec<TokenStream> = vec![];
	let mut bind_params: Vec<TokenStream> = vec![];
	let mut bind_exprs: Vec<TokenStream> = vec![];
	let mut seen_bind = false;

	for stmt in &input.statements {
		match stmt {
			DoStatement::Bind {
				pattern,
				ty,
				expr,
			} => {
				seen_bind = true;
				let expr = rewrite_pure(brand, expr, ref_mode);
				bind_params.push(format_bind_param(pattern, ty.as_ref(), ref_mode));
				bind_exprs.push(wrap_container_ref(expr, ref_mode));
			}
			DoStatement::Let {
				pattern,
				ty,
				expr,
			} => {
				let binding = match ty {
					Some(ty) => quote! { let #pattern: #ty = #expr; },
					None => quote! { let #pattern = #expr; },
				};
				if seen_bind {
					inner_lets.push(binding);
				} else {
					leading_lets.push(binding);
				}
			}
			DoStatement::Sequence {
				expr,
			} => {
				seen_bind = true;
				let expr = rewrite_pure(brand, expr, ref_mode);
				bind_params.push(format_discard_param(ref_mode));
				bind_exprs.push(wrap_container_ref(expr, ref_mode));
			}
		}
	}

	let final_expr = &input.final_expr;
	let n = bind_params.len();
	let result = match (bind_params.as_slice(), bind_exprs.as_slice()) {
		// 0 binds: pure(final_expr)
		([], _) => {
			if let Some(brand) = brand {
				if ref_mode {
					quote! { ref_pure::<#brand, _>(&(#final_expr)) }
				} else {
					quote! { pure::<#brand, _>(#final_expr) }
				}
			} else {
				// Inferred mode with 0 binds: pure needs a brand
				quote! {
					compile_error!("a_do! with no bindings generates pure(), which requires an explicit brand; use a_do!(Brand { ... }) or write the concrete constructor directly")
				}
			}
		}
		// 1 bind: map
		([param], [expr]) =>
			if let Some(brand) = brand {
				quote! {
					explicit::map::<#brand, _, _, _, _>(|#param| { #(#inner_lets)* #final_expr }, #expr)
				}
			} else {
				quote! {
					map(|#param| { #(#inner_lets)* #final_expr }, #expr)
				}
			},
		// 2-5 binds: liftN
		_ if n <= 5 => {
			if let Some(brand) = brand {
				let fn_name = format_ident!("lift{}", n);
				// Dispatched liftN functions have type params:
				// Brand + N value types + result type + N container types (FA, FB, ...) + Marker
				// Total underscores: n + 1 + n + 1 = 2n + 2
				let underscores: Vec<TokenStream> =
					(0 .. 2 * n + 2).map(|_| quote! { _ }).collect();
				quote! {
					explicit::#fn_name::<#brand, #(#underscores),*>(
						|#(#bind_params),*| { #(#inner_lets)* #final_expr },
						#(#bind_exprs),*
					)
				}
			} else {
				let fn_name = format_ident!("lift{}", n);
				quote! {
					#fn_name(
						|#(#bind_params),*| { #(#inner_lets)* #final_expr },
						#(#bind_exprs),*
					)
				}
			}
		}
		_ => {
			return Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				"a_do! supports at most 5 bindings; use m_do! for more complex cases",
			));
		}
	};

	if leading_lets.is_empty() { Ok(result) } else { Ok(quote! { { #(#leading_lets)* #result } }) }
}
