use crate::{
	core::{Error as CoreError, Result, constants::attributes::DOCUMENT_FIELDS},
	support::syntax::{format_parameter_doc, insert_doc_comment},
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
	Fields, Ident, ItemEnum, ItemStruct, LitStr, Token, Variant,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	spanned::Spanned,
};

/// Represents a field documentation entry.
///
/// For named structs: `field_name: "description"`
/// For tuple structs: just `"description"`
pub enum FieldDocArg {
	/// Named field: `field_name: "description"`
	Named(Ident, LitStr),
	/// Unnamed field (tuple struct): `"description"`
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

/// Processes an enum with `#[document_fields]` on variants.
///
/// This function looks for `#[document_fields(...)]` attributes on enum variants
/// and processes them to generate field documentation.
fn document_enum_fields(mut item_enum: ItemEnum) -> Result<TokenStream> {
	// Process each variant
	for variant in &mut item_enum.variants {
		process_variant_fields(variant)?;
	}

	Ok(item_enum.to_token_stream())
}

/// Processes a single variant's `#[document_fields(...)]` attribute if present.
fn process_variant_fields(variant: &mut Variant) -> Result<()> {
	// Look for #[document_fields(...)] attribute on this variant
	let mut doc_fields_attr_idx = None;
	let mut attr_tokens = None;

	for (idx, attr) in variant.attrs.iter().enumerate() {
		if attr.path().is_ident(DOCUMENT_FIELDS) {
			doc_fields_attr_idx = Some(idx);
			attr_tokens = Some(attr.meta.require_list()?.tokens.clone());
			break;
		}
	}

	// If no #[document_fields(...)] attribute, skip this variant
	let Some(attr_idx) = doc_fields_attr_idx else {
		return Ok(());
	};

	let attr = attr_tokens.unwrap();

	// Remove the attribute from the variant
	variant.attrs.remove(attr_idx);

	// Parse the field documentation arguments
	let args = syn::parse2::<FieldDocArgs>(attr.clone())?;

	// Extract field information from the variant
	let field_info = match &variant.fields {
		Fields::Named(fields_named) => {
			let field_names: Vec<_> =
				fields_named.named.iter().map(|f| f.ident.clone().unwrap()).collect();

			if field_names.is_empty() {
				return Err(CoreError::Parse(syn::Error::new(
					variant.span(),
					format!("{DOCUMENT_FIELDS} cannot be used on variants with no fields"),
				)));
			}

			FieldInfo::Named(field_names)
		}
		Fields::Unnamed(fields_unnamed) => {
			let field_count = fields_unnamed.unnamed.len();

			if field_count == 0 {
				return Err(CoreError::Parse(syn::Error::new(
					variant.span(),
					format!("{DOCUMENT_FIELDS} cannot be used on variants with no fields"),
				)));
			}

			FieldInfo::Unnamed(field_count)
		}
		Fields::Unit => {
			return Err(CoreError::Parse(syn::Error::new(
				variant.span(),
				format!("{DOCUMENT_FIELDS} cannot be used on unit variants"),
			)));
		}
	};

	// Validate and generate documentation
	match field_info {
		FieldInfo::Named(expected_fields) => {
			// Collect all named entries
			let mut provided_fields = std::collections::HashMap::new();

			for entry in &args.entries {
				match entry {
					FieldDocArg::Named(ident, desc) => {
						if provided_fields.insert(ident.clone(), desc.clone()).is_some() {
							return Err(CoreError::Parse(syn::Error::new(
								ident.span(),
								format!("Duplicate documentation for field `{ident}`"),
							)));
						}
					}
					FieldDocArg::Unnamed(_) => {
						return Err(CoreError::Parse(syn::Error::new(
							attr.span(),
							r#"Expected named field documentation (e.g., `field_name: "description"`), found unnamed description. Use named syntax for variants with named fields."#,
						)));
					}
				}
			}

			// Check that all fields have documentation
			for field_name in &expected_fields {
				if !provided_fields.contains_key(field_name) {
					return Err(CoreError::Parse(syn::Error::new(
						attr.span(),
						format!(
							"Missing documentation for field `{field_name}`. All fields must be documented."
						),
					)));
				}
			}

			// Check that no extra fields are documented
			for provided_field in provided_fields.keys() {
				if !expected_fields.iter().any(|f| f == provided_field) {
					return Err(CoreError::Parse(syn::Error::new(
						provided_field.span(),
						format!(
							"Field `{provided_field}` does not exist in variant. Available fields: {}",
							expected_fields
								.iter()
								.map(|f| format!("`{f}`"))
								.collect::<Vec<_>>()
								.join(", ")
						),
					)));
				}
			}

			// Generate documentation in the order of field declaration
			for field_name in expected_fields {
				if let Some(desc) = provided_fields.get(&field_name) {
					let doc_comment = format_parameter_doc(&field_name.to_string(), &desc.value());
					insert_doc_comment(
						&mut variant.attrs,
						doc_comment,
						proc_macro2::Span::call_site(),
					);
				}
			}
		}
		FieldInfo::Unnamed(expected_count) => {
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
							r#"Expected unnamed field documentation (e.g., just `"description"`), found named syntax. Use comma-separated descriptions for tuple variants."#,
						)));
					}
				}
			}

			// Check count matches
			if descriptions.len() != expected_count {
				return Err(CoreError::Parse(syn::Error::new(
					attr.span(),
					format!(
						"Expected {expected_count} description arguments (one for each field), found {}. All fields must be documented.",
						descriptions.len()
					),
				)));
			}

			// Generate documentation for tuple fields
			for (idx, desc) in descriptions.iter().enumerate() {
				let field_name = format!("{idx}");
				let doc_comment = format_parameter_doc(&field_name, &desc.value());
				insert_doc_comment(&mut variant.attrs, doc_comment, proc_macro2::Span::call_site());
			}
		}
	}

	Ok(())
}

pub fn document_fields_worker(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> Result<TokenStream> {
	// Try to parse as enum first, then struct
	if let Ok(item_enum) = syn::parse2::<ItemEnum>(item_tokens.clone()) {
		// For enums, the attribute should be empty (no arguments)
		// The actual field documentation is on the variants themselves
		if !attr.is_empty() {
			return Err(CoreError::Parse(syn::Error::new(
				attr.span(),
				format!(
					"{DOCUMENT_FIELDS} on enums should not have arguments. Use #[{DOCUMENT_FIELDS}] on the enum, and #[{DOCUMENT_FIELDS}(...)] on individual variants."
				),
			)));
		}

		return document_enum_fields(item_enum);
	}

	// Fall back to struct handling
	let mut item_struct = syn::parse2::<ItemStruct>(item_tokens)?;
	let args = syn::parse2::<FieldDocArgs>(attr.clone())?;

	// Extract field information
	let field_info = match &item_struct.fields {
		Fields::Named(fields_named) => {
			let field_names: Vec<_> =
				fields_named.named.iter().map(|f| f.ident.clone().unwrap()).collect();

			if field_names.is_empty() {
				return Err(CoreError::Parse(syn::Error::new(
					item_struct.span(),
					format!(
						"{DOCUMENT_FIELDS} cannot be used on zero-sized types (structs with no fields)"
					),
				)));
			}

			FieldInfo::Named(field_names)
		}
		Fields::Unnamed(fields_unnamed) => {
			let field_count = fields_unnamed.unnamed.len();

			if field_count == 0 {
				return Err(CoreError::Parse(syn::Error::new(
					item_struct.span(),
					format!(
						"{DOCUMENT_FIELDS} cannot be used on zero-sized types (structs with no fields)"
					),
				)));
			}

			FieldInfo::Unnamed(field_count)
		}
		Fields::Unit => {
			return Err(CoreError::Parse(syn::Error::new(
				item_struct.span(),
				format!("{DOCUMENT_FIELDS} cannot be used on zero-sized types (unit structs)"),
			)));
		}
	};

	// Validate and generate documentation
	match field_info {
		FieldInfo::Named(expected_fields) => {
			// Collect all named entries
			let mut provided_fields = std::collections::HashMap::new();

			for entry in &args.entries {
				match entry {
					FieldDocArg::Named(ident, desc) => {
						if provided_fields.insert(ident.clone(), desc.clone()).is_some() {
							return Err(CoreError::Parse(syn::Error::new(
								ident.span(),
								format!("Duplicate documentation for field `{ident}`"),
							)));
						}
					}
					FieldDocArg::Unnamed(_) => {
						return Err(CoreError::Parse(syn::Error::new(
							attr.span(),
							r#"Expected named field documentation (e.g., `field_name: "description"`), found unnamed description. Use named syntax for structs with named fields."#,
						)));
					}
				}
			}

			// Check that all fields have documentation
			for field_name in &expected_fields {
				if !provided_fields.contains_key(field_name) {
					return Err(CoreError::Parse(syn::Error::new(
						attr.span(),
						format!(
							"Missing documentation for field `{}`. All fields must be documented.",
							field_name
						),
					)));
				}
			}

			// Check that no extra fields are documented
			for provided_field in provided_fields.keys() {
				if !expected_fields.iter().any(|f| f == provided_field) {
					return Err(CoreError::Parse(syn::Error::new(
						provided_field.span(),
						format!(
							"Field `{provided_field}` does not exist in struct. Available fields: {}",
							expected_fields
								.iter()
								.map(|f| format!("`{}`", f))
								.collect::<Vec<_>>()
								.join(", ")
						),
					)));
				}
			}

			// Generate documentation in the order of field declaration
			for field_name in expected_fields {
				if let Some(desc) = provided_fields.get(&field_name) {
					let doc_comment = format_parameter_doc(&field_name.to_string(), &desc.value());
					insert_doc_comment(
						&mut item_struct.attrs,
						doc_comment,
						proc_macro2::Span::call_site(),
					);
				}
			}
		}
		FieldInfo::Unnamed(expected_count) => {
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
							r#"Expected unnamed field documentation (e.g., just `"description"`), found named syntax. Use comma-separated descriptions for tuple structs."#,
						)));
					}
				}
			}

			// Check count matches
			if descriptions.len() != expected_count {
				return Err(CoreError::Parse(syn::Error::new(
					attr.span(),
					format!(
						"Expected {expected_count} description arguments (one for each field), found {}. All fields must be documented.",
						descriptions.len()
					),
				)));
			}

			// Generate documentation for tuple fields
			for (idx, desc) in descriptions.iter().enumerate() {
				let field_name = format!("{idx}");
				let doc_comment = format_parameter_doc(&field_name, &desc.value());
				insert_doc_comment(
					&mut item_struct.attrs,
					doc_comment,
					proc_macro2::Span::call_site(),
				);
			}
		}
	}

	Ok(item_struct.to_token_stream())
}

/// Information about the fields in a struct.
enum FieldInfo {
	/// Named fields with their identifiers
	Named(Vec<Ident>),
	/// Unnamed fields (tuple struct) with the count
	Unnamed(usize),
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::support::syntax::get_doc;
	use quote::quote;

	#[test]
	fn test_document_fields_named_struct() {
		let attr = quote! { x: "The x coordinate", y: "The y coordinate" };
		let item = quote! {
			pub struct Point {
				pub x: i32,
				pub y: i32,
			}
		};

		let output = document_fields_worker(attr, item).unwrap();
		let output_struct: ItemStruct = syn::parse2(output).unwrap();

		assert_eq!(output_struct.attrs.len(), 2);
		assert_eq!(get_doc(&output_struct.attrs[0]), "* `x`: The x coordinate");
		assert_eq!(get_doc(&output_struct.attrs[1]), "* `y`: The y coordinate");
	}

	#[test]
	fn test_document_fields_tuple_struct() {
		let attr = quote! { "The wrapped value", "The secondary value" };
		let item = quote! {
			pub struct Wrapper(pub i32, pub String);
		};

		let output = document_fields_worker(attr, item).unwrap();
		let output_struct: ItemStruct = syn::parse2(output).unwrap();

		assert_eq!(output_struct.attrs.len(), 2);
		assert_eq!(get_doc(&output_struct.attrs[0]), "* `0`: The wrapped value");
		assert_eq!(get_doc(&output_struct.attrs[1]), "* `1`: The secondary value");
	}

	#[test]
	fn test_document_fields_missing_field() {
		let attr = quote! { x: "The x coordinate" };
		let item = quote! {
			pub struct Point {
				pub x: i32,
				pub y: i32,
			}
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Missing documentation for field `y`"));
	}

	#[test]
	fn test_document_fields_extra_field() {
		let attr = quote! { x: "The x coordinate", y: "The y coordinate", z: "Extra" };
		let item = quote! {
			pub struct Point {
				pub x: i32,
				pub y: i32,
			}
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Field `z` does not exist in struct"));
	}

	#[test]
	fn test_document_fields_duplicate_field() {
		let attr = quote! { x: "First", x: "Second", y: "Y coord" };
		let item = quote! {
			pub struct Point {
				pub x: i32,
				pub y: i32,
			}
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Duplicate documentation for field `x`"));
	}

	#[test]
	fn test_document_fields_tuple_wrong_count() {
		let attr = quote! { "Only one description" };
		let item = quote! {
			pub struct Wrapper(pub i32, pub String);
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected 2 description arguments"));
		assert!(error.contains("found 1"));
	}

	#[test]
	fn test_document_fields_unit_struct() {
		let attr = quote! {};
		let item = quote! {
			pub struct Unit;
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("zero-sized types"));
	}

	#[test]
	fn test_document_fields_empty_named_struct() {
		let attr = quote! {};
		let item = quote! {
			pub struct Empty {}
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("zero-sized types"));
	}

	#[test]
	fn test_document_fields_named_on_tuple() {
		let attr = quote! { field: "Description" };
		let item = quote! {
			pub struct Wrapper(pub i32);
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected unnamed field documentation"));
		assert!(error.contains("Use comma-separated descriptions for tuple structs"));
	}

	#[test]
	fn test_document_fields_unnamed_on_named() {
		let attr = quote! { "Description" };
		let item = quote! {
			pub struct Point {
				pub x: i32,
			}
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("Expected named field documentation"));
		assert!(error.contains("Use named syntax for structs with named fields"));
	}

	#[test]
	fn test_document_fields_single_field_tuple() {
		let attr = quote! { "The wrapped value" };
		let item = quote! {
			pub struct Wrapper(pub i32);
		};

		let output = document_fields_worker(attr, item).unwrap();
		let output_struct: ItemStruct = syn::parse2(output).unwrap();

		assert_eq!(output_struct.attrs.len(), 1);
		assert_eq!(get_doc(&output_struct.attrs[0]), "* `0`: The wrapped value");
	}

	#[test]
	fn test_document_fields_enum_with_named_fields() {
		let attr = quote! {};
		let item = quote! {
			#[document_fields]
			pub enum MyEnum {
				#[document_fields(
					x: "The x coordinate",
					y: "The y coordinate"
				)]
				Point {
					x: i32,
					y: i32,
				},
			}
		};

		let output = document_fields_worker(attr, item).unwrap();
		let output_enum: ItemEnum = syn::parse2(output).unwrap();

		assert_eq!(output_enum.variants.len(), 1);
		let variant = &output_enum.variants[0];
		assert_eq!(variant.attrs.len(), 2);
		assert_eq!(get_doc(&variant.attrs[0]), "* `x`: The x coordinate");
		assert_eq!(get_doc(&variant.attrs[1]), "* `y`: The y coordinate");
	}

	#[test]
	fn test_document_fields_enum_with_tuple_fields() {
		let attr = quote! {};
		let item = quote! {
			#[document_fields]
			pub enum MyEnum {
				#[document_fields(
					"The first value",
					"The second value"
				)]
				Tuple(i32, String),
			}
		};

		let output = document_fields_worker(attr, item).unwrap();
		let output_enum: ItemEnum = syn::parse2(output).unwrap();

		assert_eq!(output_enum.variants.len(), 1);
		let variant = &output_enum.variants[0];
		assert_eq!(variant.attrs.len(), 2);
		assert_eq!(get_doc(&variant.attrs[0]), "* `0`: The first value");
		assert_eq!(get_doc(&variant.attrs[1]), "* `1`: The second value");
	}

	#[test]
	fn test_document_fields_enum_multiple_variants() {
		let attr = quote! {};
		let item = quote! {
			#[document_fields]
			pub enum Result<T, E> {
				#[document_fields(value: "The success value")]
				Ok { value: T },
				#[document_fields(error: "The error value")]
				Err { error: E },
			}
		};

		let output = document_fields_worker(attr, item).unwrap();
		let output_enum: ItemEnum = syn::parse2(output).unwrap();

		assert_eq!(output_enum.variants.len(), 2);

		let ok_variant = &output_enum.variants[0];
		assert_eq!(ok_variant.attrs.len(), 1);
		assert_eq!(get_doc(&ok_variant.attrs[0]), "* `value`: The success value");

		let err_variant = &output_enum.variants[1];
		assert_eq!(err_variant.attrs.len(), 1);
		assert_eq!(get_doc(&err_variant.attrs[0]), "* `error`: The error value");
	}

	#[test]
	fn test_document_fields_enum_with_args_fails() {
		let attr = quote! { some_arg: "value" };
		let item = quote! {
			#[document_fields]
			pub enum MyEnum {
				Variant,
			}
		};

		let result = document_fields_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("should not have arguments"));
	}
}
