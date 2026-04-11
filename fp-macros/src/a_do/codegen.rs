//! Code generation for the `a_do!` macro.
//!
//! Transforms parsed `DoInput` into `pure` / `map` / `liftN` calls.

use {
	crate::m_do::{
		codegen::rewrite_pure,
		input::{
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
/// - 0 binds -> `pure::<Brand, _>(final_expr)`
/// - 1 bind  -> `map::<Brand, _, _, _, _>(|pat| body, expr)`
/// - N binds -> `liftN::<Brand, _, ...>(|pats...| body, exprs...)`
pub fn a_do_worker(input: DoInput) -> syn::Result<TokenStream> {
	let brand = &input.brand;
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
				let param = match (ref_mode, ty) {
					// Ref mode, untyped: add &_ for dispatch inference
					(true, None) => quote! { #pattern: &_ },
					// Ref mode, typed: user wrote the full type (including &)
					(true, Some(ty)) => quote! { #pattern: #ty },
					// Val mode, typed
					(false, Some(ty)) => quote! { #pattern: #ty },
					// Val mode, untyped
					(false, None) => quote! { #pattern },
				};
				bind_params.push(param);
				if ref_mode {
					bind_exprs.push(quote! { &(#expr) });
				} else {
					bind_exprs.push(expr);
				}
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
				let discard = if ref_mode {
					quote! { _: &_ }
				} else {
					quote! { _ }
				};
				bind_params.push(discard);
				if ref_mode {
					bind_exprs.push(quote! { &(#expr) });
				} else {
					bind_exprs.push(expr);
				}
			}
		}
	}

	let final_expr = &input.final_expr;
	let n = bind_params.len();
	let result = match (bind_params.as_slice(), bind_exprs.as_slice()) {
		([], _) =>
			if ref_mode {
				quote! { ref_pure::<#brand, _>(&(#final_expr)) }
			} else {
				quote! { pure::<#brand, _>(#final_expr) }
			},
		([param], [expr]) => {
			// map has type params: Brand, A, B, FA, Marker = 5 total, 4 underscores
			quote! {
				map_explicit::<#brand, _, _, _, _>(|#param| { #(#inner_lets)* #final_expr }, #expr)
			}
		}
		_ if n <= 5 => {
			let fn_name = format_ident!("lift{}_explicit", n);
			// Dispatched liftN functions have type params:
			// Brand + N value types + result type + N container types (FA, FB, ...) + Marker
			// Total underscores: n + 1 + n + 1 = 2n + 2
			let underscores: Vec<TokenStream> = (0 .. 2 * n + 2).map(|_| quote! { _ }).collect();
			quote! {
				#fn_name::<#brand, #(#underscores),*>(
					|#(#bind_params),*| { #(#inner_lets)* #final_expr },
					#(#bind_exprs),*
				)
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
