//! Centralized attribute utilities.
//!
//! This module provides utilities for filtering and working with attributes,
//! particularly documentation-specific attributes like `doc_default` and `doc_use`.

use syn::Attribute;

/// Utility for filtering documentation-specific attributes.
///
/// This centralizes the logic for identifying and filtering out attributes
/// that are specific to the documentation generation system, preventing
/// duplication across the codebase.
pub struct DocAttributeFilter;

impl DocAttributeFilter {
    /// Returns true if the attribute should be kept in generated code.
    ///
    /// This filters out documentation-specific attributes like `doc_default`
    /// and `doc_use` which are processed by the macro system but should not
    /// appear in the final generated code.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use syn::parse_quote;
    ///
    /// let attr: Attribute = parse_quote!(#[doc_default]);
    /// assert!(!DocAttributeFilter::should_keep(&attr));
    ///
    /// let attr: Attribute = parse_quote!(#[derive(Debug)]);
    /// assert!(DocAttributeFilter::should_keep(&attr));
    /// ```
    pub fn should_keep(attr: &Attribute) -> bool {
        !Self::is_doc_specific(attr)
    }

    /// Returns true if the attribute is documentation-specific.
    ///
    /// Documentation-specific attributes include:
    /// - `doc_default`: Marks an associated type as the default for resolution
    /// - `doc_use`: Specifies which associated type to use for documentation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use syn::parse_quote;
    ///
    /// let attr: Attribute = parse_quote!(#[doc_default]);
    /// assert!(DocAttributeFilter::is_doc_specific(&attr));
    ///
    /// let attr: Attribute = parse_quote!(#[doc_use = "Of"]);
    /// assert!(DocAttributeFilter::is_doc_specific(&attr));
    /// ```
    pub fn is_doc_specific(attr: &Attribute) -> bool {
        attr.path().is_ident("doc_default") || attr.path().is_ident("doc_use")
    }

    /// Filters out documentation-specific attributes from a slice.
    ///
    /// This is a convenience method that returns an iterator over attributes
    /// that should be kept (i.e., are not documentation-specific).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use syn::{Attribute, parse_quote};
    ///
    /// let attrs: Vec<Attribute> = vec![
    ///     parse_quote!(#[doc_default]),
    ///     parse_quote!(#[derive(Debug)]),
    ///     parse_quote!(#[doc_use = "Of"]),
    /// ];
    ///
    /// let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
    /// assert_eq!(filtered.len(), 1); // Only #[derive(Debug)] remains
    /// ```
    pub fn filter_doc_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
        attrs.iter().filter(|attr| Self::should_keep(attr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_filter_doc_default() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[doc_default]),
            parse_quote!(#[derive(Debug)]),
        ];

        let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].path().is_ident("derive"));
    }

    #[test]
    fn test_filter_doc_use() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[doc_use = "Of"]),
            parse_quote!(#[inline]),
        ];

        let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].path().is_ident("inline"));
    }

    #[test]
    fn test_is_doc_specific() {
        let doc_default: Attribute = parse_quote!(#[doc_default]);
        let doc_use: Attribute = parse_quote!(#[doc_use = "Of"]);
        let derive: Attribute = parse_quote!(#[derive(Debug)]);

        assert!(DocAttributeFilter::is_doc_specific(&doc_default));
        assert!(DocAttributeFilter::is_doc_specific(&doc_use));
        assert!(!DocAttributeFilter::is_doc_specific(&derive));
    }

    #[test]
    fn test_should_keep() {
        let doc_default: Attribute = parse_quote!(#[doc_default]);
        let derive: Attribute = parse_quote!(#[derive(Debug)]);

        assert!(!DocAttributeFilter::should_keep(&doc_default));
        assert!(DocAttributeFilter::should_keep(&derive));
    }

    #[test]
    fn test_filter_empty() {
        let attrs: Vec<Attribute> = vec![];
        let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_all_doc_attrs() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[doc_default]),
            parse_quote!(#[doc_use = "Of"]),
        ];

        let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_no_doc_attrs() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[derive(Debug)]),
            parse_quote!(#[inline]),
        ];

        let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs).collect();
        assert_eq!(filtered.len(), 2);
    }
}
