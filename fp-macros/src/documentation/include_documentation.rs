//! `include_documentation!` macro for including markdown files with link rewriting.
//!
//! Reads a markdown file relative to `CARGO_MANIFEST_DIR` and rewrites
//! relative `.md` links to rustdoc intra-doc links pointing at
//! `crate::docs::module_name` submodules.

use {
	crate::core::constants::configuration,
	proc_macro2::TokenStream,
	quote::quote,
};

/// Worker function for the `include_documentation!` proc macro.
///
/// Parses a string literal file path, reads the file relative to
/// `CARGO_MANIFEST_DIR`, rewrites same-directory `.md` links to
/// intra-doc links, and returns a string literal token.
pub fn include_documentation_worker(input: TokenStream) -> Result<TokenStream, syn::Error> {
	let lit: syn::LitStr = syn::parse2(input)?;
	let rel_path = lit.value();

	#[expect(clippy::expect_used, reason = "CARGO_MANIFEST_DIR is always set by Cargo")]
	let manifest_dir =
		std::env::var(configuration::CARGO_MANIFEST_DIR).expect("CARGO_MANIFEST_DIR not set");

	let full_path = std::path::Path::new(&manifest_dir).join(&rel_path);
	let content = std::fs::read_to_string(&full_path).map_err(|e| {
		syn::Error::new_spanned(&lit, format!("failed to read {}: {e}", full_path.display()))
	})?;

	let rewritten = rewrite_md_links(&content);
	Ok(quote!(#rewritten))
}

/// Rewrite same-directory `.md` links to rustdoc intra-doc links.
///
/// Transforms `[text](./foo-bar.md)` and `[text](foo-bar.md)` into
/// `[text][crate::docs::foo_bar]`. Links with path separators (e.g.,
/// `../` or subdirectories) are left unchanged.
fn rewrite_md_links(content: &str) -> String {
	let mut result = String::with_capacity(content.len());
	let mut rest = content;

	while let Some(open_pos) = rest.find('[') {
		// Push everything before the `[`
		result.push_str(&rest[.. open_pos]);
		let after_open = &rest[open_pos + 1 ..];

		// Find matching `]` (no nesting support needed for doc links)
		if let Some(close_offset) = after_open.find(']') {
			let link_text = &after_open[.. close_offset];
			let after_close = &after_open[close_offset + 1 ..];

			// Check if `(` follows immediately
			if let Some(url_content) = after_close.strip_prefix('(') {
				// Find closing `)` (simple, no nested parens needed for .md URLs)
				if let Some(paren_end) = url_content.find(')') {
					let url = &url_content[.. paren_end];

					if let Some(module_name) = md_link_to_module(url) {
						// Rewrite to intra-doc link
						result.push('[');
						result.push_str(link_text);
						result.push_str("][crate::docs::");
						result.push_str(&module_name);
						result.push(']');
						rest = &url_content[paren_end + 1 ..];
						continue;
					}
				}
			}
		}

		// Not a rewritable link; push the `[` and advance past it
		result.push('[');
		rest = after_open;
	}

	// Push any remaining content after the last `[`
	result.push_str(rest);
	result
}

/// If a URL is a same-directory `.md` link, return the corresponding
/// module name. Returns `None` for non-local links.
///
/// Accepts `./foo-bar.md` or `foo-bar.md` (no path separators beyond
/// the optional `./` prefix). Hyphens are converted to underscores.
fn md_link_to_module(url: &str) -> Option<String> {
	let stripped = url.strip_prefix("./").unwrap_or(url);

	// Must end with .md
	let stem = stripped.strip_suffix(".md")?;

	// Must not contain path separators (no subdirectories or parent refs)
	if stem.contains('/') || stem.contains('\\') {
		return None;
	}

	// Must not be empty
	if stem.is_empty() {
		return None;
	}

	// Convert hyphens to underscores for a valid Rust module name
	Some(stem.replace('-', "_"))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_md_link_to_module_with_dot_slash() {
		assert_eq!(md_link_to_module("./zero-cost.md"), Some("zero_cost".into()));
	}

	#[test]
	fn test_md_link_to_module_without_prefix() {
		assert_eq!(md_link_to_module("hkt.md"), Some("hkt".into()));
	}

	#[test]
	fn test_md_link_to_module_with_hyphens() {
		assert_eq!(md_link_to_module("./brand-inference.md"), Some("brand_inference".into()));
	}

	#[test]
	fn test_md_link_to_module_parent_path_ignored() {
		assert_eq!(md_link_to_module("../src/foo.md"), None);
	}

	#[test]
	fn test_md_link_to_module_subdirectory_ignored() {
		assert_eq!(md_link_to_module("./sub/foo.md"), None);
	}

	#[test]
	fn test_md_link_to_module_non_md_ignored() {
		assert_eq!(md_link_to_module("./foo.rs"), None);
	}

	#[test]
	fn test_rewrite_simple_link() {
		let input = "See [Zero-Cost](./zero-cost.md) for details.";
		let expected = "See [Zero-Cost][crate::docs::zero_cost] for details.";
		assert_eq!(rewrite_md_links(input), expected);
	}

	#[test]
	fn test_rewrite_multiple_links() {
		let input = "See [HKT](./hkt.md) and [Dispatch](./dispatch.md).";
		let expected = "See [HKT][crate::docs::hkt] and [Dispatch][crate::docs::dispatch].";
		assert_eq!(rewrite_md_links(input), expected);
	}

	#[test]
	fn test_rewrite_leaves_parent_links() {
		let input = "See [source](../src/foo.rs) for details.";
		assert_eq!(rewrite_md_links(input), input);
	}

	#[test]
	fn test_rewrite_leaves_non_md_links() {
		let input = "See [docs](https://example.com) for details.";
		assert_eq!(rewrite_md_links(input), input);
	}

	#[test]
	fn test_rewrite_without_dot_slash_prefix() {
		let input = "See [Optics](optics-analysis.md) for details.";
		let expected = "See [Optics][crate::docs::optics_analysis] for details.";
		assert_eq!(rewrite_md_links(input), expected);
	}

	#[test]
	fn test_rewrite_preserves_surrounding_text() {
		let input = "prefix [A](./a.md) middle [B](./b.md) suffix";
		let expected = "prefix [A][crate::docs::a] middle [B][crate::docs::b] suffix";
		assert_eq!(rewrite_md_links(input), expected);
	}

	#[test]
	fn test_rewrite_inline_link_in_bold() {
		let input = "**Config:** see [Pointer Abstraction](./pointer-abstraction.md) for how.";
		let expected =
			"**Config:** see [Pointer Abstraction][crate::docs::pointer_abstraction] for how.";
		assert_eq!(rewrite_md_links(input), expected);
	}
}
