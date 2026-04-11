//! Parsing for the `m_do!` macro input.
//!
//! Defines `DoInput` and `DoStatement` types and their `Parse` implementations.

use {
	proc_macro2::TokenTree,
	syn::{
		Expr,
		Pat,
		Token,
		Type,
		braced,
		parse::{
			Parse,
			ParseStream,
			discouraged::Speculative,
		},
	},
};

/// The parsed input to the `m_do!` macro.
///
/// Represents `[ref] Brand { statements... final_expr }`.
pub struct DoInput {
	/// Whether the `ref` qualifier is present, selecting by-reference dispatch.
	pub ref_mode: bool,
	/// The brand type (e.g., `OptionBrand`).
	pub brand: Type,
	/// The statements preceding the final expression.
	pub statements: Vec<DoStatement>,
	/// The final expression (returned as-is, no trailing `;`).
	pub final_expr: Expr,
}

/// A single statement in a `m_do!` block.
pub enum DoStatement {
	/// `pat <- expr;` or `pat: Type <- expr;` -- monadic bind.
	Bind {
		/// The binding pattern.
		pattern: Pat,
		/// Optional type annotation.
		ty: Option<Type>,
		/// The monadic expression to bind.
		expr: Expr,
	},
	/// `let pat = expr;` or `let pat: Type = expr;` -- pure let binding.
	Let {
		/// The binding pattern.
		pattern: Pat,
		/// Optional type annotation.
		ty: Option<Type>,
		/// The expression to bind.
		expr: Expr,
	},
	/// `expr;` -- monadic sequence (result discarded).
	Sequence {
		/// The monadic expression to sequence.
		expr: Expr,
	},
}

impl Parse for DoInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let ref_mode = input.peek(Token![ref]);
		if ref_mode {
			input.parse::<Token![ref]>()?;
		}
		let brand: Type = input.parse()?;

		let content;
		braced!(content in input);

		let mut statements = Vec::new();

		loop {
			if content.is_empty() {
				return Err(content.error("m_do! block must contain at least one expression"));
			}

			// `let` binding
			if content.peek(Token![let]) {
				statements.push(parse_let_statement(&content)?);
				continue;
			}

			// Try bind: `pat: Type <- expr;` then `pat <- expr;`
			if let Some(bind) = try_parse_bind(&content)? {
				statements.push(bind);
				continue;
			}

			// Expression (sequence or final)
			let expr: Expr = content.parse()?;

			if content.peek(Token![;]) {
				content.parse::<Token![;]>()?;
				statements.push(DoStatement::Sequence {
					expr,
				});
				continue;
			}

			// Final expression
			if !content.is_empty() {
				return Err(syn::Error::new_spanned(
					&expr,
					"expected `;` after statement or end of block",
				));
			}

			return Ok(DoInput {
				ref_mode,
				brand,
				statements,
				final_expr: expr,
			});
		}
	}
}

/// Parses `let pat = expr;` or `let pat: Type = expr;`.
fn parse_let_statement(input: ParseStream) -> syn::Result<DoStatement> {
	input.parse::<Token![let]>()?;
	let pattern = Pat::parse_single(input)?;

	// Type annotation is part of let syntax, not the pattern
	let ty = if input.peek(Token![:]) {
		input.parse::<Token![:]>()?;
		Some(input.parse::<Type>()?)
	} else {
		None
	};

	input.parse::<Token![=]>()?;
	let expr: Expr = input.parse()?;
	input.parse::<Token![;]>()?;

	Ok(DoStatement::Let {
		pattern,
		ty,
		expr,
	})
}

/// Speculatively tries to parse `pat <- expr;` or `pat: Type <- expr;`.
///
/// Returns `Ok(None)` if the current position doesn't match bind syntax.
/// Tries typed bind first, then falls back to simple bind.
fn try_parse_bind(input: ParseStream) -> syn::Result<Option<DoStatement>> {
	// Try typed bind: `pat: Type <- expr;`
	if let Some(bind) = try_parse_typed_bind(input)? {
		return Ok(Some(bind));
	}

	// Try simple bind: `pat <- expr;`
	try_parse_simple_bind(input)
}

/// Tries to parse `pat: Type <- expr;`.
///
/// Uses token-level scanning to find `<-` because `Type::parse` would greedily
/// consume the `<` as the start of generic arguments for types like `i32`.
/// The key insight is that `<-` never appears in valid Rust type syntax.
fn try_parse_typed_bind(input: ParseStream) -> syn::Result<Option<DoStatement>> {
	let fork = input.fork();

	let pattern = match fork.call(Pat::parse_single) {
		Ok(pat) => pat,
		Err(_) => return Ok(None),
	};

	if !fork.peek(Token![:]) || fork.peek(Token![::]) {
		return Ok(None);
	}
	fork.parse::<Token![:]>()?;

	// Collect type tokens until we find `<-`.
	// `<-` (less-than immediately followed by minus) never appears in valid Rust
	// type syntax, so the first occurrence marks the bind arrow boundary.
	let ty = match collect_type_before_arrow(&fork) {
		Some(ty) => ty,
		None => return Ok(None),
	};

	// Parse `<-`
	if fork.parse::<Token![<]>().is_err() || fork.parse::<Token![-]>().is_err() {
		return Ok(None);
	}

	let expr: Expr = fork.parse()?;
	fork.parse::<Token![;]>()?;
	input.advance_to(&fork);

	Ok(Some(DoStatement::Bind {
		pattern,
		ty: Some(ty),
		expr,
	}))
}

/// Tries to parse `pat <- expr;`.
fn try_parse_simple_bind(input: ParseStream) -> syn::Result<Option<DoStatement>> {
	let fork = input.fork();

	let pattern = match fork.call(Pat::parse_single) {
		Ok(pat) => pat,
		Err(_) => return Ok(None),
	};

	// Check for `<-` (two separate punct tokens)
	if !fork.peek(Token![<]) {
		return Ok(None);
	}
	fork.parse::<Token![<]>()?;

	if !fork.peek(Token![-]) {
		return Ok(None);
	}
	fork.parse::<Token![-]>()?;

	// Committed to bind
	let expr: Expr = fork.parse()?;
	fork.parse::<Token![;]>()?;
	input.advance_to(&fork);

	Ok(Some(DoStatement::Bind {
		pattern,
		ty: None,
		expr,
	}))
}

/// Collects tokens from the stream until `<-` is found, then parses them as a `Type`.
///
/// Returns `None` if no `<-` is found or the collected tokens don't form a valid type.
/// Advances the input past the collected type tokens on success.
fn collect_type_before_arrow(input: ParseStream) -> Option<Type> {
	let fork = input.fork();
	let mut tokens = proc_macro2::TokenStream::new();

	while !fork.is_empty() {
		// `<-` never appears in valid Rust type syntax, so the first
		// `<` immediately followed by `-` is always the bind arrow.
		if fork.peek(Token![<]) {
			let check = fork.fork();
			let _ = check.parse::<Token![<]>().ok()?;
			if check.peek(Token![-]) {
				break;
			}
		}

		let tt: TokenTree = fork.parse().ok()?;
		tokens.extend(std::iter::once(tt));
	}

	if tokens.is_empty() {
		return None;
	}

	let ty: Type = syn::parse2(tokens).ok()?;
	input.advance_to(&fork);
	Some(ty)
}

#[cfg(test)]
#[expect(
	clippy::indexing_slicing,
	clippy::expect_used,
	clippy::panic,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use {
		super::*,
		syn::parse_str,
	};

	#[test]
	fn parse_basic_bind() {
		let input: DoInput = parse_str(
			"OptionBrand {
				x <- Some(5);
				pure(x)
			}",
		)
		.expect("failed to parse");

		assert!(matches!(input.statements[0], DoStatement::Bind { .. }));
		assert_eq!(input.statements.len(), 1);
	}

	#[test]
	fn parse_let_binding() {
		let input: DoInput = parse_str(
			"OptionBrand {
				let z = 42;
				pure(z)
			}",
		)
		.expect("failed to parse");

		assert!(matches!(input.statements[0], DoStatement::Let { .. }));
	}

	#[test]
	fn parse_sequence() {
		let input: DoInput = parse_str(
			"OptionBrand {
				Some(());
				pure(5)
			}",
		)
		.expect("failed to parse");

		assert!(matches!(input.statements[0], DoStatement::Sequence { .. }));
	}

	#[test]
	fn parse_typed_bind() {
		let input: DoInput = parse_str(
			"OptionBrand {
				x: i32 <- Some(5);
				pure(x)
			}",
		)
		.expect("failed to parse");

		if let DoStatement::Bind {
			ty, ..
		} = &input.statements[0]
		{
			assert!(ty.is_some());
		} else {
			panic!("expected Bind");
		}
	}

	#[test]
	fn parse_typed_bind_generic() {
		let input: DoInput = parse_str(
			"OptionBrand {
				x: Vec<i32> <- Some(vec![1]);
				pure(x)
			}",
		)
		.expect("failed to parse");

		if let DoStatement::Bind {
			ty, ..
		} = &input.statements[0]
		{
			assert!(ty.is_some());
		} else {
			panic!("expected Bind");
		}
	}

	#[test]
	fn parse_discard_bind() {
		let input: DoInput = parse_str(
			"OptionBrand {
				_ <- Some(());
				pure(5)
			}",
		)
		.expect("failed to parse");

		assert!(matches!(input.statements[0], DoStatement::Bind { .. }));
	}

	#[test]
	fn parse_only_final_expr() {
		let input: DoInput = parse_str(
			"OptionBrand {
				pure(42)
			}",
		)
		.expect("failed to parse");

		assert!(input.statements.is_empty());
	}

	#[test]
	fn parse_multiple_statements() {
		let input: DoInput = parse_str(
			"OptionBrand {
				x <- Some(5);
				y <- Some(x + 1);
				let z = x * y;
				Some(());
				pure(z)
			}",
		)
		.expect("failed to parse");

		assert_eq!(input.statements.len(), 4);
		assert!(matches!(input.statements[0], DoStatement::Bind { .. }));
		assert!(matches!(input.statements[1], DoStatement::Bind { .. }));
		assert!(matches!(input.statements[2], DoStatement::Let { .. }));
		assert!(matches!(input.statements[3], DoStatement::Sequence { .. }));
	}

	#[test]
	fn parse_empty_block_fails() {
		let result = parse_str::<DoInput>("OptionBrand {}");
		assert!(result.is_err());
	}

	#[test]
	fn parse_ref_mode() {
		let input: DoInput = parse_str(
			"ref OptionBrand {
				x <- Some(5);
				pure(x)
			}",
		)
		.expect("failed to parse");

		assert!(input.ref_mode);
		assert!(matches!(input.statements[0], DoStatement::Bind { .. }));
	}

	#[test]
	fn parse_non_ref_mode() {
		let input: DoInput = parse_str(
			"OptionBrand {
				x <- Some(5);
				pure(x)
			}",
		)
		.expect("failed to parse");

		assert!(!input.ref_mode);
	}
}
