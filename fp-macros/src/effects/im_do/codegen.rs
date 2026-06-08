//! Code generation for the `im_do!` macro.
//!
//! Transforms parsed [`DoInput`](crate::support::do_input::DoInput) into
//! nested inherent `bind` / `ref_bind` method calls, with bare `pure(x)`
//! calls rewritten to `Wrapper::pure(x)` (or `Wrapper::ref_pure(&(x))`
//! in `ref` mode).

use {
	crate::{
		m_do::codegen::{
			format_bind_param,
			format_discard_param,
		},
		support::do_input::{
			DoInput,
			DoStatement,
		},
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

/// Generates the expanded token stream for an `im_do!` invocation.
pub fn im_do_worker(input: DoInput) -> syn::Result<TokenStream> {
	let wrapper = input.brand.as_ref();
	let ref_mode = input.ref_mode;
	let bind_method = if ref_mode {
		quote! { ref_bind }
	} else {
		quote! { bind }
	};

	let mut result = rewrite_pure_inherent(wrapper, &input.final_expr, ref_mode);

	for stmt in input.statements.iter().rev() {
		result = match stmt {
			DoStatement::Bind {
				pattern,
				ty,
				expr,
			} => {
				let expr = rewrite_pure_inherent(wrapper, expr, ref_mode);
				let closure_param = format_bind_param(pattern, ty.as_ref(), ref_mode);
				quote! { (#expr).#bind_method(move |#closure_param| { #result }) }
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
				let expr = rewrite_pure_inherent(wrapper, expr, ref_mode);
				let discard = format_discard_param(ref_mode);
				quote! { (#expr).#bind_method(move |#discard| { #result }) }
			}
		};
	}

	Ok(result)
}

/// Rewrites bare `pure(args)` calls to inherent `Wrapper::pure` /
/// `Wrapper::ref_pure` associated-function calls.
///
/// In explicit mode:
/// - Normal: `pure(args)` -> `Wrapper::pure(args)`.
/// - Ref: `pure(args)` -> `Wrapper::ref_pure(&(args))`.
///
/// In inferred mode (`wrapper` is `None`):
/// - Emits `compile_error!` because `pure` has no container argument and
///   cannot infer the wrapper type. Mirrors `m_do!`'s rejection.
fn rewrite_pure_inherent(
	wrapper: Option<&Type>,
	expr: &Expr,
	ref_mode: bool,
) -> TokenStream {
	let mut expr = expr.clone();
	let mut rewriter = PureRewriter {
		wrapper,
		ref_mode,
	};
	rewriter.visit_expr_mut(&mut expr);
	quote! { #expr }
}

/// AST visitor that rewrites bare `pure(...)` calls to inherent
/// associated-function calls on the wrapper type.
struct PureRewriter<'a> {
	wrapper: Option<&'a Type>,
	ref_mode: bool,
}

impl VisitMut for PureRewriter<'_> {
	fn visit_expr_mut(
		&mut self,
		expr: &mut Expr,
	) {
		// Visit children first so nested `pure(...)` calls are rewritten too.
		syn::visit_mut::visit_expr_mut(self, expr);

		if let Expr::Call(call) = expr
			&& is_bare_pure_call(call)
		{
			if let Some(wrapper) = self.wrapper {
				let args = &call.args;
				if self.ref_mode {
					*expr = syn::parse_quote! { #wrapper::ref_pure(&(#args)) };
				} else {
					*expr = syn::parse_quote! { #wrapper::pure(#args) };
				}
			} else {
				// Inferred mode: `pure` has no container argument to infer the
				// wrapper from. Mirrors `m_do!`'s rejection.
				*expr = syn::parse_quote! {
					compile_error!("pure() requires an explicit wrapper type; use im_do!(Wrapper { ... }) or write the concrete constructor (e.g., RcRun::pure(x) instead of pure(x))")
				};
			}
		}
	}
}

/// Returns `true` if this is a call to a bare `pure` path (not `Foo::pure`,
/// not `::pure`, not `pure::<T>(...)` with existing turbofish).
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
