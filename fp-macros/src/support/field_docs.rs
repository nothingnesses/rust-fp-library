/// Field documentation generation utilities.
///
/// This module provides a unified interface for documenting struct and enum variant fields,
/// handling both named and unnamed (tuple) fields.
use crate::{
	core::{Error as CoreError, Result},
	support::{
		parsing,
		syntax::{format_parameter_doc, insert_doc_comment},
	},
};
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Ident, LitStr, Token};

/// Represents a field documentation entry.
///
/// For named fields: `field_name: "description"`
/// For tuple fields: just `"description"`
pub enum FieldDocArg {
	/// Named field: `field_name: "description"`
	Named(Ident, LitStr),
	/// Unnamed field (tuple): `"description"`
	Unnamed(LitStr),
}

impl Parse for FieldDocArg {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// Try to parse as named field first: ident : "string"
		if input.peek(Ident) && input.peek2(Token![:]) {
			let ident: Ident = input.parse()?;
			let _: Token![:] = input.parse()?;
			let lit: LitStr = input.parse()?;
			Ok(FieldDocArg::Named(ident, lit))
		} else {
			// Otherwise, parse as unnamed field: "string"
			let lit: LitStr = input.parse()?;
			Ok(FieldDocArg::Unnamed(lit))
		}
	}
}

pub struct FieldDocArgs {
	pub entries: Punctuated<FieldDocArg, Token![,]>,
}

impl Parse for FieldDocArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(FieldDocArgs { entries: Punctuated::parse_terminated(input)? })
	}
}

/// Information about the fields in a struct or variant.
pub enum FieldInfo {
	/// Named fields with their identifiers
	Named(Vec<Ident>),
	/// Unnamed fields (tuple) with the count
	Unnamed(usize),
}

impl FieldInfo {
	/// Extract field information from a Fields struct.
	///
	/// # Returns
	/// - `Ok(FieldInfo)` if the fields are valid
	/// - `Err` if the fields are unit (no fields)
	pub fn from_fields(
		fields: &Fields,
		span: proc_macro2::Span,
		context: &str,
		attribute_name: &str,
	) -> Result<Self> {
		match fields {
			Fields::Named(fields_named) => {
				let field_names: Vec<_> =
					fields_named.named.iter().map(|f| f.ident.clone().unwrap()).collect();

				let _ = parsing::parse_not_zero_sized(
					field_names.len(),
					span,
					context,
					attribute_name,
				)?;

				Ok(FieldInfo::Named(field_names))
			}
			Fields::Unnamed(fields_unnamed) => {
				let field_count = fields_unnamed.unnamed.len();

				let _ = parsing::parse_not_zero_sized(field_count, span, context, attribute_name)?;

				Ok(FieldInfo::Unnamed(field_count))
			}
			Fields::Unit => Err(CoreError::Parse(syn::Error::new(
				span,
				format!("{attribute_name} cannot be used on unit {context}s"),
			))),
		}
	}
}

/// A helper for generating field documentation.
///
/// This struct encapsulates the logic for validating and generating documentation
/// for both named and unnamed fields.
pub struct FieldDocumenter {
	field_info: FieldInfo,
	attr_span: proc_macro2::Span,
	context: &'static str,
}

impl FieldDocumenter {
	/// Create a new FieldDocumenter.
	///
	/// # Parameters
	/// - `field_info`: Information about the fields to document
	/// - `attr_span`: The span of the attribute for error reporting
	/// - `context`: A description of what's being documented (e.g., "struct", "variant")
	pub fn new(
		field_info: FieldInfo,
		attr_span: proc_macro2::Span,
		context: &'static str,
	) -> Self {
		Self { field_info, attr_span, context }
	}

	/// Validate and generate documentation for fields.
	///
	/// This method validates the provided documentation arguments against the field info,
	/// then generates and inserts the appropriate doc comments.
	///
	/// # Parameters
	/// - `args`: The parsed field documentation arguments
	/// - `attrs`: The attribute list to insert documentation into
	///
	/// # Returns
	/// - `Ok(())` if validation and generation succeeded
	/// - `Err` if validation failed
	pub fn validate_and_generate(
		&self,
		args: FieldDocArgs,
		attrs: &mut Vec<Attribute>,
	) -> Result<()> {
		match &self.field_info {
			FieldInfo::Named(expected_fields) => {
				self.process_named_fields(args, expected_fields, attrs)
			}
			FieldInfo::Unnamed(expected_count) => {
				self.process_unnamed_fields(args, *expected_count, attrs)
			}
		}
	}

	/// Process named fields.
	fn process_named_fields(
		&self,
		args: FieldDocArgs,
		expected_fields: &[Ident],
		attrs: &mut Vec<Attribute>,
	) -> Result<()> {
		// Collect all named entries
		let mut provided_fields = HashMap::new();

		for entry in &args.entries {
			match entry {
				FieldDocArg::Named(ident, desc) => {
					let existing = provided_fields.insert(ident.clone(), desc.clone());
					let _ =
						parsing::parse_no_duplicate(ident, desc.clone(), existing, self.context)?;
				}
				FieldDocArg::Unnamed(_) => {
					return Err(CoreError::Parse(syn::Error::new(
						self.attr_span,
						format!(
							r#"Expected named field documentation (e.g., `field_name: "description"`), found unnamed description. Use named syntax for {}s with named fields."#,
							self.context
						),
					)));
				}
			}
		}

		// Validate completeness
		let (_expected, provided_fields) = parsing::parse_named_entries(
			expected_fields,
			provided_fields,
			self.attr_span,
			"field",
		)?;

		// Generate documentation in the order of field declaration
		for field_name in expected_fields {
			if let Some(desc) = provided_fields.get(field_name) {
				let doc_comment = format_parameter_doc(&field_name.to_string(), &desc.value());
				insert_doc_comment(attrs, doc_comment, proc_macro2::Span::call_site());
			}
		}

		Ok(())
	}

	/// Process unnamed (tuple) fields.
	fn process_unnamed_fields(
		&self,
		args: FieldDocArgs,
		expected_count: usize,
		attrs: &mut Vec<Attribute>,
	) -> Result<()> {
		// Collect all unnamed entries
		let mut descriptions = Vec::new();

		for entry in &args.entries {
			match entry {
				FieldDocArg::Unnamed(desc) => {
					descriptions.push(desc.clone());
				}
				FieldDocArg::Named(ident, _) => {
					return Err(CoreError::Parse(syn::Error::new(
						ident.span(),
						format!(
							r#"Expected unnamed field documentation (e.g., just `"description"`), found named syntax. Use comma-separated descriptions for tuple {}s."#,
							self.context
						),
					)));
				}
			}
		}

		// Validate count
		let _ = parsing::parse_entry_count(
			expected_count,
			descriptions.len(),
			self.attr_span,
			"field",
		)?;

		// Generate documentation for tuple fields
		for (idx, desc) in descriptions.iter().enumerate() {
			let field_name = format!("{idx}");
			let doc_comment = format_parameter_doc(&field_name, &desc.value());
			insert_doc_comment(attrs, doc_comment, proc_macro2::Span::call_site());
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use quote::quote;
	use syn::parse_quote;

	#[test]
	fn test_field_doc_arg_named() {
		let tokens = quote! { x: "description" };
		let arg: FieldDocArg = syn::parse2(tokens).unwrap();
		match arg {
			FieldDocArg::Named(ident, desc) => {
				assert_eq!(ident.to_string(), "x");
				assert_eq!(desc.value(), "description");
			}
			_ => panic!("Expected Named variant"),
		}
	}

	#[test]
	fn test_field_doc_arg_unnamed() {
		let tokens = quote! { "description" };
		let arg: FieldDocArg = syn::parse2(tokens).unwrap();
		match arg {
			FieldDocArg::Unnamed(desc) => {
				assert_eq!(desc.value(), "description");
			}
			_ => panic!("Expected Unnamed variant"),
		}
	}

	#[test]
	fn test_field_info_from_named_fields() {
		let item_struct: syn::ItemStruct = parse_quote! {
			struct Test { x: i32, y: i32 }
		};
		let info = FieldInfo::from_fields(
			&item_struct.fields,
			proc_macro2::Span::call_site(),
			"struct",
			"#[test]",
		)
		.unwrap();

		match info {
			FieldInfo::Named(names) => {
				assert_eq!(names.len(), 2);
				assert_eq!(names[0].to_string(), "x");
				assert_eq!(names[1].to_string(), "y");
			}
			_ => panic!("Expected Named variant"),
		}
	}

	#[test]
	fn test_field_info_from_unnamed_fields() {
		let item_struct: syn::ItemStruct = parse_quote! {
			struct Test(i32, String);
		};
		let info = FieldInfo::from_fields(
			&item_struct.fields,
			proc_macro2::Span::call_site(),
			"struct",
			"#[test]",
		)
		.unwrap();

		match info {
			FieldInfo::Unnamed(count) => {
				assert_eq!(count, 2);
			}
			_ => panic!("Expected Unnamed variant"),
		}
	}

	#[test]
	fn test_field_info_from_unit_fails() {
		let item_struct: syn::ItemStruct = parse_quote! {
			struct Test;
		};
		let result = FieldInfo::from_fields(
			&item_struct.fields,
			proc_macro2::Span::call_site(),
			"struct",
			"#[test]",
		);
		assert!(result.is_err());
	}
}
