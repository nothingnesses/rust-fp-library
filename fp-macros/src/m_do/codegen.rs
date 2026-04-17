//! Code generation for the `m_do!` macro.
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

/// Generates the expanded token stream for a `m_do!` invocation.
pub fn m_do_worker(input: DoInput) -> syn::Result<TokenStream> {
	let brand = input.brand.as_ref();
	let ref_mode = input.ref_mode;
	let mut result = rewrite_pure(brand, &input.final_expr, ref_mode);

	for stmt in input.statements.iter().rev() {
		result = match stmt {
			DoStatement::Bind {
				pattern,
				ty,
				expr,
			} => {
				let expr = rewrite_pure(brand, expr, ref_mode);
				let closure_param = format_bind_param(pattern, ty.as_ref(), ref_mode);
				let container = wrap_container_ref(quote! { #expr }, ref_mode);
				if let Some(brand) = brand {
					quote! { explicit::bind::<#brand, _, _, _, _>(#container, move |#closure_param| { #result }) }
				} else {
					quote! { bind(#container, move |#closure_param| { #result }) }
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
				quote! { { #binding #result } }
			}
			DoStatement::Sequence {
				expr,
			} => {
				let expr = rewrite_pure(brand, expr, ref_mode);
				let discard = format_discard_param(ref_mode);
				let container = wrap_container_ref(quote! { #expr }, ref_mode);
				if let Some(brand) = brand {
					quote! { explicit::bind::<#brand, _, _, _, _>(#container, move |#discard| { #result }) }
				} else {
					quote! { bind(#container, move |#discard| { #result }) }
				}
			}
		};
	}

	Ok(result)
}

/// Format a bind closure parameter with optional type annotation and ref mode.
pub(crate) fn format_bind_param(
	pattern: &syn::Pat,
	ty: Option<&Type>,
	ref_mode: bool,
) -> TokenStream {
	match (ref_mode, ty) {
		// Ref mode, untyped: add &_ for dispatch inference
		(true, None) => quote! { #pattern: &_ },
		// Ref mode, typed: user wrote the full type (including &)
		(true, Some(ty)) => quote! { #pattern: #ty },
		// Val mode, typed
		(false, Some(ty)) => quote! { #pattern: #ty },
		// Val mode, untyped
		(false, None) => quote! { #pattern },
	}
}

/// Wrap an expression in a reference if in ref mode.
pub(crate) fn wrap_container_ref(
	expr: TokenStream,
	ref_mode: bool,
) -> TokenStream {
	if ref_mode {
		quote! { &(#expr) }
	} else {
		expr
	}
}

/// Format a discard pattern for sequence statements.
pub(crate) fn format_discard_param(ref_mode: bool) -> TokenStream {
	if ref_mode {
		quote! { _: &_ }
	} else {
		quote! { _ }
	}
}

/// Rewrites bare `pure(args)` calls within an expression.
///
/// In explicit mode:
/// - Normal: `pure(args)` -> `pure::<Brand, _>(args)`.
/// - Ref: `pure(args)` -> `ref_pure::<Brand, _>(&(args))`.
///
/// In inferred mode (brand is `None`):
/// - Emits `compile_error!` because `pure` has no container argument
///   and cannot infer the brand.
pub(crate) fn rewrite_pure(
	brand: Option<&Type>,
	expr: &Expr,
	ref_mode: bool,
) -> TokenStream {
	let mut expr = expr.clone();
	let mut rewriter = PureRewriter {
		brand,
		ref_mode,
	};
	rewriter.visit_expr_mut(&mut expr);
	quote! { #expr }
}

/// AST visitor that rewrites bare `pure(...)` calls.
struct PureRewriter<'a> {
	brand: Option<&'a Type>,
	ref_mode: bool,
}

impl VisitMut for PureRewriter<'_> {
	fn visit_expr_mut(
		&mut self,
		expr: &mut Expr,
	) {
		// Visit children first
		syn::visit_mut::visit_expr_mut(self, expr);

		if let Expr::Call(call) = expr
			&& is_bare_pure_call(call)
		{
			if let Some(brand) = self.brand {
				let args = &call.args;
				if self.ref_mode {
					*expr = syn::parse_quote! { ref_pure::<#brand, _>(&(#args)) };
				} else {
					*expr = syn::parse_quote! { pure::<#brand, _>(#args) };
				}
			} else {
				// Inferred mode: pure cannot infer the brand
				*expr = syn::parse_quote! {
					compile_error!("pure() requires an explicit brand; use m_do!(Brand { ... }) or write the concrete constructor (e.g., Some(x) instead of pure(x))")
				};
			}
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
