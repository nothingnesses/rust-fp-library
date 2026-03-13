//! Code generation for the `m!` macro.
//!
//! Transforms parsed `DoInput` into nested `bind` calls with `pure` auto-rewriting.

use {
	super::input::{
		DoInput,
		DoStatement,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Expr,
		ExprCall,
		ExprPath,
		Type,
		visit_mut::VisitMut,
	},
};

/// Generates the expanded token stream for a `m!` invocation.
pub fn m_worker(input: DoInput) -> syn::Result<TokenStream> {
	let brand = &input.brand;
	let mut result = rewrite_pure(brand, &input.final_expr);

	for stmt in input.statements.iter().rev() {
		result = match stmt {
			DoStatement::Bind {
				pattern,
				ty,
				expr,
			} => {
				let expr = rewrite_pure(brand, expr);
				let closure_param = match ty {
					Some(ty) => quote! { #pattern: #ty },
					None => quote! { #pattern },
				};
				quote! { bind::<#brand, _, _>(#expr, move |#closure_param| { #result }) }
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
				quote! { { #binding #result } }
			}
			DoStatement::Sequence {
				expr,
			} => {
				let expr = rewrite_pure(brand, expr);
				quote! { bind::<#brand, _, _>(#expr, move |_| { #result }) }
			}
		};
	}

	Ok(result)
}

/// Rewrites bare `pure(args)` calls to `pure::<Brand, _>(args)` within an expression.
pub(crate) fn rewrite_pure(
	brand: &Type,
	expr: &Expr,
) -> TokenStream {
	let mut expr = expr.clone();
	let mut rewriter = PureRewriter {
		brand,
	};
	rewriter.visit_expr_mut(&mut expr);
	quote! { #expr }
}

/// AST visitor that rewrites bare `pure(...)` calls to `pure::<Brand, _>(...)`.
struct PureRewriter<'a> {
	brand: &'a Type,
}

impl VisitMut for PureRewriter<'_> {
	fn visit_expr_mut(
		&mut self,
		expr: &mut Expr,
	) {
		// Visit children first
		syn::visit_mut::visit_expr_mut(self, expr);

		// Rewrite bare `pure(args)` → `pure::<Brand, _>(args)`
		if let Expr::Call(call) = expr
			&& is_bare_pure_call(call)
		{
			let brand = self.brand;
			let args = &call.args;
			*expr = syn::parse_quote! { pure::<#brand, _>(#args) };
		}
	}
}

/// Returns `true` if this is a call to a bare `pure` path (not `Foo::pure`, not `::pure`,
/// not `pure::<T>(...)` with existing turbofish).
fn is_bare_pure_call(call: &ExprCall) -> bool {
	if let Expr::Path(ExprPath {
		qself: None,
		path,
		..
	}) = call.func.as_ref()
	{
		path.leading_colon.is_none()
			&& path.segments.len() == 1
			&& path.segments.first().is_some_and(|s| s.ident == "pure" && s.arguments.is_none())
	} else {
		false
	}
}
