use crate::{
	core::{Error as CoreError, Result, constants::attributes::DOCUMENT_FIELDS},
	support::{
		attributes::AttributeExt,
		document_field::{DocumentFieldParameters, FieldDocumenter, FieldInfo},
	},
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ItemEnum, ItemStruct, Variant, spanned::Spanned};

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
	// Find, remove, and parse the attribute in one operation
	let Some(args) = variant.attrs.find_and_remove::<DocumentFieldParameters>(DOCUMENT_FIELDS)?
	else {
		// No attribute on this variant, skip it
		return Ok(());
	};

	// Get the span for error messages (we need to reconstruct since we already consumed the attr)
	let attr_span = variant.span();

	// Extract field information from the variant
	let field_info =
		FieldInfo::from_fields(&variant.fields, variant.span(), "variant", DOCUMENT_FIELDS)?;

	// Use the documenter to validate and generate docs
	let documenter = FieldDocumenter::new(field_info, attr_span, "variant");
	documenter.validate_and_generate(args, &mut variant.attrs)?;

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
	let args = syn::parse2::<DocumentFieldParameters>(attr.clone())?;

	// Extract field information
	let field_info =
		FieldInfo::from_fields(&item_struct.fields, item_struct.span(), "struct", DOCUMENT_FIELDS)?;

	// Use the documenter to validate and generate docs
	let documenter = FieldDocumenter::new(field_info, attr.span(), "struct");
	documenter.validate_and_generate(args, &mut item_struct.attrs)?;

	Ok(item_struct.to_token_stream())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::support::generate_documentation::get_doc;
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
		eprintln!("Actual error: {}", error);
		assert!(error.contains("Field `z` does not exist"));
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
		assert!(error.contains("Duplicate documentation for"));
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
		assert!(error.contains("cannot be used on unit struct"));
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
		assert!(error.contains("Use comma-separated descriptions for tuple"));
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
		assert!(error.contains("Use named syntax for"));
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
