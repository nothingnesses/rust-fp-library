# Clean-Room Design for fp-macros

**Date**: 2026-02-08
**Author**: Architectural Analysis
**Status**: Proposal

This document outlines an ideal clean-room design for the `fp-macros` crate, addressing the architectural issues identified in the [architectural analysis](./architectural-analysis.md) while preserving the strengths of the current implementation.

---

## Table of Contents

- [High-Level Architecture](#high-level-architecture)
- [Key Design Decisions](#key-design-decisions)
- [Core Principles](#core-principles)
- [Comparison with Current Design](#comparison-with-current-design)
- [Migration Strategy](#migration-strategy)
- [What to Preserve](#what-to-preserve)

---

## High-Level Architecture

```
fp-macros/
├── src/
│   ├── lib.rs                    # Public API surface
│   ├── error.rs                  # Unified error handling
│   │
│   ├── hkt/                      # Higher-Kinded Types
│   │   ├── mod.rs
│   │   ├── signature.rs          # Kind signature representation
│   │   ├── canonicalization.rs   # Deterministic signature normalization
│   │   ├── naming.rs             # Hash-based name generation
│   │   ├── generation.rs         # Trait & impl generation
│   │   └── application.rs        # Apply! macro logic
│   │
│   ├── type_analysis/            # Type system analysis
│   │   ├── mod.rs
│   │   ├── ast.rs                # Normalized type AST
│   │   ├── visitor.rs            # Type traversal framework
│   │   ├── patterns.rs           # Pattern matching (FnBrand, Apply!, etc.)
│   │   ├── generics.rs           # Generic parameter analysis
│   │   └── bounds.rs             # Trait bound analysis
│   │
│   ├── hm_conversion/            # Hindley-Milner conversion
│   │   ├── mod.rs
│   │   ├── types.rs              # HM type representation
│   │   ├── converter.rs          # Rust → HM conversion
│   │   ├── formatter.rs          # HM signature formatting
│   │   └── constraints.rs        # Constraint extraction
│   │
│   ├── documentation/            # Documentation generation
│   │   ├── mod.rs
│   │   ├── context.rs            # Documentation context extraction
│   │   ├── resolution.rs         # Self/AssocType resolution
│   │   ├── signature.rs          # #[hm_signature] implementation
│   │   ├── params.rs             # Parameter documentation
│   │   ├── orchestration.rs      # Module-level orchestration
│   │   └── templates.rs          # Documentation templates
│   │
│   ├── codegen/                  # Code generation utilities
│   │   ├── mod.rs
│   │   ├── scanner.rs            # File/directory scanning
│   │   ├── reexports.rs          # Re-export generation
│   │   └── builders.rs           # TokenStream builders
│   │
│   ├── config/                   # Configuration management
│   │   ├── mod.rs
│   │   ├── schema.rs             # Config structure
│   │   ├── loader.rs             # Cargo.toml parsing
│   │   ├── cache.rs              # Config caching
│   │   └── validation.rs         # Config validation
│   │
│   └── support/                  # Supporting utilities
│       ├── mod.rs
│       ├── attributes.rs         # Attribute parsing
│       ├── syntax.rs             # Syn helpers
│       └── spans.rs              # Span management
│
└── tests/
    ├── unit/                     # Unit tests per module
    ├── integration/              # Integration tests
    ├── ui/                       # Error message tests
    └── property/                 # Property-based tests
```

---

## Key Design Decisions

### 1. Unified Error Handling

**Problem**: Current design mixes `panic!()` and `Result`, making errors inconsistent.

**Solution**: Result-based error handling throughout.

```rust
// error.rs
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for fp-macros
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(#[from] syn::Error),
    
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        span: Span,
    },
    
    #[error("Resolution error: {message}")]
    Resolution {
        message: String,
        span: Span,
        available_types: Vec<String>,
    },
    
    #[error("Unsupported feature: {0}")]
    Unsupported(#[from] UnsupportedFeature),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Specific unsupported feature variants
#[derive(Debug, Error)]
pub enum UnsupportedFeature {
    #[error("Const generic parameters are not supported in Kind definitions")]
    ConstGenerics { span: Span },
    
    #[error("Verbatim bounds are not supported")]
    VerbatimBounds { span: Span },
    
    #[error("Complex type not supported: {description}")]
    ComplexTypes {
        description: String,
        span: Span
    },
}

impl Error {
    /// Create a validation error
    pub fn validation(span: Span, message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            span,
        }
    }
    
    /// Create a resolution error with available types for helpful messages
    pub fn resolution(
        span: Span,
        message: impl Into<String>,
        available_types: Vec<String>
    ) -> Self {
        Self::Resolution {
            message: message.into(),
            span,
            available_types,
        }
    }
    
    /// Create an internal error (for "should never happen" cases)
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
    
    /// Get the span for this error
    pub fn span(&self) -> Span {
        match self {
            Error::Parse(e) => e.span(),
            Error::Validation { span, .. } => *span,
            Error::Resolution { span, .. } => *span,
            Error::Unsupported(u) => u.span(),
            Error::Internal(_) => Span::call_site(),
        }
    }
}

impl UnsupportedFeature {
    /// Get the span for this unsupported feature
    pub fn span(&self) -> Span {
        match self {
            UnsupportedFeature::ConstGenerics { span } => *span,
            UnsupportedFeature::VerbatimBounds { span } => *span,
            UnsupportedFeature::ComplexTypes { span, .. } => *span,
        }
    }
}

/// Convert our error to syn::Error for proc macro output
impl From<Error> for syn::Error {
    fn from(err: Error) -> Self {
        let span = err.span();
        let message = err.to_string();
        
        let mut syn_err = syn::Error::new(span, message);
        
        // Add additional context for resolution errors
        if let Error::Resolution { available_types, .. } = &err {
            if !available_types.is_empty() {
                let note = format!(
                    "note: Available types: {}",
                    available_types.join(", ")
                );
                syn_err.combine(syn::Error::new(span, note));
            }
        }
        
        syn_err
    }
}
```

**Benefits**:
- No panics at compile time
- Rich error context
- Explicit unsupported feature handling
- Easy error composition

---

### 2. Type-Safe Kind Signatures

**Problem**: Empty associated type lists are accepted; const generics are silently ignored.

**Solution**: Strong typing with validation at parse boundaries.

```rust
// hkt/signature.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindSignature {
    associated_types: Vec<AssociatedType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedType {
    name: Ident,
    parameters: GenericParameters,
    output_bounds: Vec<Bound>,
    attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParameters {
    lifetimes: Vec<Lifetime>,
    types: Vec<TypeParameter>,
    consts: Vec<ConstParameter>,  // Explicitly tracked
}

impl KindSignature {
    /// Parse from token stream with validation
    pub fn parse(input: ParseStream) -> Result<Self> {
        let mut assoc_types = Vec::new();
        
        while !input.is_empty() {
            assoc_types.push(AssociatedType::parse(input)?);
        }
        
        // Validation: non-empty
        if assoc_types.is_empty() {
            return Err(Error::validation(
                Span::call_site(),
                "Kind definition must have at least one associated type"
            ));
        }
        
        // Validation: unsupported features
        for assoc in &assoc_types {
            if !assoc.parameters.consts.is_empty() {
                return Err(Error::unsupported(
                    UnsupportedFeature::ConstGenerics {
                        span: assoc.parameters.consts[0].span()
                    }
                ));
            }
        }
        
        Ok(Self { assoc_types })
    }
    
    /// Get canonical representation
    pub fn canonicalize(&self) -> Result<CanonicalSignature> {
        CanonicalSignature::from_signature(self)
    }
}
```

**Benefits**:
- Explicit modeling of all parameter types
- Early validation catches errors
- Clear error messages
- Type safety prevents invalid states

---

### 3. Result-Based Canonicalization

**Problem**: Canonicalization uses `panic!()` for unsupported types.

**Solution**: Return errors for unsupported cases.

```rust
// hkt/canonicalization.rs
pub struct Canonicalizer {
    lifetime_map: BTreeMap<String, usize>,
    type_map: BTreeMap<String, usize>,
    const_map: BTreeMap<String, usize>,
}

impl Canonicalizer {
    pub fn new(params: &GenericParameters) -> Self {
        let mut lifetime_map = BTreeMap::new();
        let mut type_map = BTreeMap::new();
        let mut const_map = BTreeMap::new();
        
        for (idx, lt) in params.lifetimes.iter().enumerate() {
            lifetime_map.insert(lt.ident.to_string(), idx);
        }
        
        for (idx, ty) in params.types.iter().enumerate() {
            type_map.insert(ty.ident.to_string(), idx);
        }
        
        for (idx, c) in params.consts.iter().enumerate() {
            const_map.insert(c.ident.to_string(), idx);
        }
        
        Self { lifetime_map, type_map, const_map }
    }
    
    pub fn canonicalize_bound(&self, bound: &Bound) -> Result<String> {
        match bound {
            Bound::Lifetime(lt) => self.canonicalize_lifetime(lt),
            Bound::Trait(tr) => self.canonicalize_trait(tr),
            Bound::Verbatim(tokens) => {
                Err(Error::unsupported(UnsupportedFeature::VerbatimBounds {
                    span: tokens.span()
                }))
            }
        }
    }
    
    fn canonicalize_type(&self, ty: &Type) -> Result<String> {
        match ty {
            Type::Path(p) => self.canonicalize_path(p),
            Type::Reference(r) => self.canonicalize_reference(r),
            Type::Tuple(t) => self.canonicalize_tuple(t),
            Type::Slice(s) => self.canonicalize_slice(s),
            Type::Array(a) => self.canonicalize_array(a),
            Type::Never(_) => Ok("!".to_string()),
            Type::Infer(_) => Ok("_".to_string()),
            
            // Explicit handling of complex types
            Type::BareFn(_) | Type::ImplTrait(_) | Type::TraitObject(_) => {
                Err(Error::unsupported(UnsupportedFeature::ComplexTypes {
                    description: format!("Type {} in Kind signature", quote!(#ty)),
                    span: ty.span(),
                }))
            }
            
            // Catch-all for future syn versions
            _ => Err(Error::unsupported(UnsupportedFeature::ComplexTypes {
                description: "Unknown type variant".to_string(),
                span: ty.span(),
            }))
        }
    }
    
    fn canonicalize_reference(&self, ref_: &TypeReference) -> Result<String> {
        let lt = if let Some(lt) = &ref_.lifetime {
            if let Some(idx) = self.lifetime_map.get(&lt.ident.to_string()) {
                format!("l{} ", idx)
            } else {
                format!("{} ", lt.ident)
            }
        } else {
            String::new()
        };
        
        let mutability = if ref_.mutability.is_some() { "mut " } else { "" };
        let elem = self.canonicalize_type(&ref_.elem)?;
        
        Ok(format!("&{}{}{}", lt, mutability, elem))
    }
}
```

**Benefits**:
- No panics - all unsupported cases return errors
- Clear error messages guide users
- Exhaustive matching catches new variants
- Easy to extend support

---

### 4. Builder-Based Code Generation

**Problem**: Large documentation strings are built with format! and string manipulation.

**Solution**: Builder pattern with testable components.

```rust
// hkt/generation.rs
pub struct KindTraitBuilder {
    signature: CanonicalSignature,
    name: Ident,
}

impl KindTraitBuilder {
    pub fn new(signature: KindSignature) -> Result<Self> {
        let canonical = signature.canonicalize()?;
        let name = canonical.generate_name();
        Ok(Self { signature: canonical, name })
    }
    
    pub fn build_definition(self) -> TokenStream {
        let name = &self.name;
        let doc = self.build_documentation();
        let assoc_types = self.build_associated_types();
        
        quote! {
            #doc
            #[allow(non_camel_case_types)]
            pub trait #name {
                #(#assoc_types)*
            }
        }
    }
    
    fn build_documentation(&self) -> TokenStream {
        DocumentationBuilder::new(&self.signature, &self.name)
            .build()
    }
    
    fn build_associated_types(&self) -> Vec<TokenStream> {
        self.signature
            .associated_types()
            .iter()
            .map(|assoc| {
                let ident = &assoc.name;
                let generics = &assoc.parameters;
                let bounds = &assoc.output_bounds;
                let attrs = &assoc.attributes;
                
                let bounds_tokens = if bounds.is_empty() {
                    quote!()
                } else {
                    quote!(: #(#bounds)+*)
                };
                
                quote! {
                    #(#attrs)*
                    type #ident #generics #bounds_tokens;
                }
            })
            .collect()
    }
}

// documentation/templates.rs
pub struct DocumentationBuilder<'a> {
    signature: &'a CanonicalSignature,
    name: &'a Ident,
}

impl<'a> DocumentationBuilder<'a> {
    pub fn new(signature: &'a CanonicalSignature, name: &'a Ident) -> Self {
        Self { signature, name }
    }
    
    pub fn build(self) -> TokenStream {
        let sections = vec![
            self.build_summary(),
            self.build_overview(),
            self.build_associated_types_section(),
            self.build_implementation_section(),
            self.build_naming_section(),
            self.build_see_also_section(),
        ];
        
        let doc = sections.join("\n\n");
        quote! { #[doc = #doc] }
    }
    
    fn build_summary(&self) -> String {
        let assoc_types = self.signature.associated_types();
        let summary = assoc_types
            .iter()
            .map(|a| format!("`{}`", a.display()))
            .collect::<Vec<_>>()
            .join("; ");
        
        match assoc_types.len() {
            1 => format!("`Kind` with associated type: {}.", summary),
            _ => format!("`Kind` with associated types: {}.", summary),
        }
    }
    
    fn build_overview(&self) -> String {
        concat!(
            "Higher-Kinded Type (HKT) trait auto-generated by ",
            "[`def_kind!`](crate::def_kind!), representing type constructors ",
            "that can be applied to generic parameters to produce concrete types."
        ).to_string()
    }
    
    fn build_associated_types_section(&self) -> String {
        let mut section = String::from("# Associated Types\n\n");
        
        for assoc in self.signature.associated_types() {
            section.push_str(&format!("### `type {}`\n\n", assoc.name));
            section.push_str(&self.format_assoc_type_details(assoc));
            section.push_str("\n\n");
        }
        
        section
    }
    
    fn format_assoc_type_details(&self, assoc: &AssociatedType) -> String {
        let params = &assoc.parameters;
        
        let lifetimes = if params.lifetimes.is_empty() {
            "None".to_string()
        } else {
            params.lifetimes
                .iter()
                .map(|lt| format!("`{}`", lt))
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        let types = if params.types.is_empty() {
            "None".to_string()
        } else {
            params.types
                .iter()
                .map(|ty| format!("`{}`", ty.display_with_bounds()))
                .collect::<Vec<_>>()
                .join(", ")
        };
        
        let bounds = if assoc.output_bounds.is_empty() {
            "None".to_string()
        } else {
            format!("`{}`", assoc.display_output_bounds())
        };
        
        format!(
            "* **Lifetimes** ({}): {}\n* **Type parameters** ({}): {}\n* **Output bounds**: {}",
            params.lifetimes.len(), lifetimes,
            params.types.len(), types,
            bounds
        )
    }
    
    // Other sections...
}
```

**Benefits**:
- No string manipulation of quote! output
- Testable documentation components
- Easy to modify documentation format
- Clear separation of concerns

---

### 5. Dependency Injection for File I/O

**Problem**: Direct file I/O without caching or proper dependency tracking.

**Solution**: Abstract file system with caching and tracking.

```rust
// codegen/scanner.rs
pub trait FileSystem {
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn read_file(&self, path: &Path) -> io::Result<String>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(path)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }
    
    fn read_file(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }
}

pub struct CachedFileSystem {
    cache: DashMap<PathBuf, Arc<String>>,
    inner: Box<dyn FileSystem + Send + Sync>,
}

impl CachedFileSystem {
    pub fn new(inner: impl FileSystem + Send + Sync + 'static) -> Self {
        Self {
            cache: DashMap::new(),
            inner: Box::new(inner),
        }
    }
}

impl FileSystem for CachedFileSystem {
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.inner.read_dir(path)
    }
    
    fn read_file(&self, path: &Path) -> io::Result<String> {
        if let Some(cached) = self.cache.get(path) {
            return Ok((**cached).clone());
        }
        
        let content = self.inner.read_file(path)?;
        self.cache.insert(path.to_path_buf(), Arc::new(content.clone()));
        Ok(content)
    }
}

// codegen/reexports.rs
pub struct ReexportGenerator<F: FileSystem> {
    fs: F,
    config: ReexportConfig,
}

impl<F: FileSystem> ReexportGenerator<F> {
    pub fn new(fs: F, config: ReexportConfig) -> Self {
        Self { fs, config }
    }
    
    pub fn scan_directory(&self) -> Result<Vec<ReexportItem>> {
        let path = &self.config.source_dir;
        let files = self.fs.read_dir(path)
            .map_err(|e| Error::internal(format!("Failed to read directory: {}", e)))?;
        
        // Track dependencies for incremental compilation
        for file in &files {
            if let Some(path_str) = file.to_str() {
                proc_macro::tracked_path::path(path_str);
            }
        }
        
        files.into_iter()
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("rs"))
            .filter(|p| p.file_stem().and_then(|s| s.to_str()) != Some("mod"))
            .filter_map(|path| self.process_file(&path).transpose())
            .collect()
    }
    
    fn process_file(&self, path: &Path) -> Result<Option<ReexportItem>> {
        let content = self.fs.read_file(path)
            .map_err(|e| Error::internal(format!("Failed to read file: {}", e)))?;
        
        let file = syn::parse_file(&content)
            .map_err(|e| Error::parse(e))?;
        
        self.extract_items(&file, path)
    }
}

// In tests:
pub struct MockFileSystem {
    files: HashMap<PathBuf, String>,
}

impl FileSystem for MockFileSystem {
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(self.files.keys().cloned().collect())
    }
    
    fn read_file(&self, path: &Path) -> io::Result<String> {
        self.files.get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reexport_generation() {
        let mut fs = MockFileSystem::new();
        fs.add_file("src/foo.rs", "pub fn bar() {}");
        
        let generator = ReexportGenerator::new(fs, config);
        let items = generator.scan_directory().unwrap();
        
        assert_eq!(items.len(), 1);
    }
}
```

**Benefits**:
- Testable without real file I/O
- Explicit caching strategy
- Proper dependency tracking
- Easy to mock for tests

---

### 6. Type-Safe Projection Resolution

**Problem**: Projection keys use tuples, making order errors easy.

**Solution**: Newtype with clear API.

```rust
// documentation/resolution.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectionKey {
    type_path: String,
    trait_path: Option<String>,
    assoc_name: String,
}

impl ProjectionKey {
    pub fn new(
        type_path: impl Into<String>,
        assoc_name: impl Into<String>
    ) -> Self {
        Self {
            type_path: type_path.into(),
            trait_path: None,
            assoc_name: assoc_name.into(),
        }
    }
    
    pub fn with_trait(mut self, trait_path: impl Into<String>) -> Self {
        self.trait_path = Some(trait_path.into());
        self
    }
    
    pub fn module_level(mut self) -> Self {
        self.trait_path = None;
        self
    }
    
    pub fn scoped(
        type_path: impl Into<String>,
        trait_path: impl Into<String>,
        assoc_name: impl Into<String>
    ) -> Self {
        Self {
            type_path: type_path.into(),
            trait_path: Some(trait_path.into()),
            assoc_name: assoc_name.into(),
        }
    }
}

pub struct ProjectionResolver<'a> {
    config: &'a Config,
}

impl<'a> ProjectionResolver<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    
    pub fn resolve(&self, context: &ResolutionContext) -> Result<ResolvedType> {
        // Explicit hierarchy with clear precedence
        self.try_method_override(context)
            .or_else(|_| self.try_impl_override(context))
            .or_else(|_| self.try_scoped_default(context))
            .or_else(|_| self.try_module_default(context))
            .map_err(|_| self.create_resolution_error(context))
    }
    
    fn try_method_override(&self, ctx: &ResolutionContext) -> Result<ResolvedType> {
        let doc_use = ctx.method_doc_use.ok_or_else(|| Error::not_found())?;
        
        let key = ProjectionKey::scoped(
            &ctx.self_type,
            &ctx.trait_name,
            doc_use
        );
        
        self.lookup(&key)
    }
    
    fn try_impl_override(&self, ctx: &ResolutionContext) -> Result<ResolvedType> {
        let doc_use = ctx.impl_doc_use.ok_or_else(|| Error::not_found())?;
        
        let key = ProjectionKey::scoped(
            &ctx.self_type,
            &ctx.trait_name,
            doc_use
        );
        
        self.lookup(&key)
    }
    
    fn try_scoped_default(&self, ctx: &ResolutionContext) -> Result<ResolvedType> {
        let key = ProjectionKey::scoped(
            &ctx.self_type,
            &ctx.trait_name,
            &ctx.assoc_name
        );
        
        self.lookup(&key)
    }
    
    fn try_module_default(&self, ctx: &ResolutionContext) -> Result<ResolvedType> {
        let key = ProjectionKey::new(&ctx.self_type, &ctx.assoc_name);
        self.lookup(&key)
    }
    
    fn lookup(&self, key: &ProjectionKey) -> Result<ResolvedType> {
        self.config
            .projections
            .get(key)
            .cloned()
            .ok_or_else(|| Error::not_found())
    }
    
    fn create_resolution_error(&self, ctx: &ResolutionContext) -> Error {
        ResolutionErrorBuilder::new(ctx, self.config)
            .build()
    }
}

#[derive(Debug)]
pub struct ResolutionContext<'a> {
    pub self_type: &'a str,
    pub trait_name: &'a str,
    pub assoc_name: &'a str,
    pub method_doc_use: Option<&'a str>,
    pub impl_doc_use: Option<&'a str>,
    pub span: Span,
}

impl<'a> ResolutionContext<'a> {
    pub fn new(
        self_type: &'a str,
        trait_name: &'a str,
        assoc_name: &'a str,
        span: Span
    ) -> Self {
        Self {
            self_type,
            trait_name,
            assoc_name,
            method_doc_use: None,
            impl_doc_use: None,
            span,
        }
    }
    
    pub fn with_method_doc_use(mut self, doc_use: &'a str) -> Self {
        self.method_doc_use = Some(doc_use);
        self
    }
    
    pub fn with_impl_doc_use(mut self, doc_use: &'a str) -> Self {
        self.impl_doc_use = Some(doc_use);
        self
    }
}
```

**Benefits**:
- Type-safe keys prevent ordering errors
- Explicit hierarchy is self-documenting
- Builder pattern for context construction
- Easy to test each resolution level

---

### 7. Parameterized Re-exports

**Problem**: Hardcoded `crate::classes` path.

**Solution**: Accept base module as parameter.

```rust
// codegen/reexports.rs
pub struct ReexportConfig {
    pub source_dir: PathBuf,
    pub base_module: syn::Path,
    pub aliases: HashMap<String, Ident>,
    pub item_kind: ItemKind,
}

impl ReexportConfig {
    pub fn functions(
        source_dir: impl Into<PathBuf>,
        base_module: syn::Path
    ) -> Self {
        Self {
            source_dir: source_dir.into(),
            base_module,
            aliases: HashMap::new(),
            item_kind: ItemKind::Functions,
        }
    }
    
    pub fn traits(
        source_dir: impl Into<PathBuf>,
        base_module: syn::Path
    ) -> Self {
        Self {
            source_dir: source_dir.into(),
            base_module,
            aliases: HashMap::new(),
            item_kind: ItemKind::Traits,
        }
    }
    
    pub fn with_aliases(mut self, aliases: HashMap<String, Ident>) -> Self {
        self.aliases = aliases;
        self
    }
}

pub enum ItemKind {
    Functions,
    Traits,
}

pub fn generate_reexports(config: ReexportConfig) -> Result<TokenStream> {
    let fs = CachedFileSystem::new(RealFileSystem);
    let generator = ReexportGenerator::new(fs, config);
    
    let items = generator.scan_directory()?;
    let base_module = &generator.config.base_module;
    
    match generator.config.item_kind {
        ItemKind::Functions => {
            quote! {
                pub use #base_module::{
                    #(#items),*
                };
            }
        }
        ItemKind::Traits => {
            quote! {
                #(
                    pub use #base_module::#items;
                )*
            }
        }
    }
}

// Usage in macro:
#[proc_macro]
pub fn generate_function_re_exports(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ReexportMacroInput);
    
    let config = ReexportConfig::functions(
        input.source_dir.value(),
        input.base_module
    )
    .with_aliases(input.aliases);
    
    match generate_reexports(config) {
        Ok(tokens) => tokens.into(),
        Err(e) => syn::Error::from(e).to_compile_error().into(),
    }
}

// Macro input parsing
struct ReexportMacroInput {
    source_dir: LitStr,
    base_module: syn::Path,
    aliases: HashMap<String, Ident>,
}

impl Parse for ReexportMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source_dir: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let base_module: syn::Path = input.parse()?;
        
        let aliases = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            parse_aliases(input)?
        } else {
            HashMap::new()
        };
        
        Ok(Self { source_dir, base_module, aliases })
    }
}
```

**Benefits**:
- No hardcoded paths
- Reusable across projects
- Clear configuration structure
- Type-safe module paths

---

## Core Principles

### 1. Railway-Oriented Programming

All operations return `Result`, enabling composition and error propagation:

```rust
pub fn process_macro(input: TokenStream) -> Result<TokenStream> {
    let parsed = parse_input(input)?;
    let validated = validate_input(parsed)?;
    let transformed = transform(validated)?;
    let generated = generate(transformed)?;
    Ok(generated)
}
```

### 2. Explicit Over Implicit

```rust
// ❌ Bad: Silent fallback
let result = operation().unwrap_or_else(|| default_value);

// ✅ Good: Explicit error with context
let result = operation()
    .map_err(|e| e.with_context("Failed to perform operation"))?;
```

### 3. Validation at Boundaries

```rust
impl Parse for KindSignature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sig = Self::parse_unchecked(input)?;
        sig.validate()?;  // Validate immediately after parsing
        Ok(sig)
    }
}
```

### 4. Composition Over Inheritance

```rust
// Use traits for behavior, structs for data
pub trait TypeVisitor {
    fn visit_path(&mut self, path: &TypePath) -> Result<HMType>;
    fn visit_reference(&mut self, ref_: &TypeReference) -> Result<HMType>;
}

pub struct HMConverter<'a> {
    config: &'a Config,
    analyzer: GenericAnalyzer,
}

impl<'a> TypeVisitor for HMConverter<'a> {
    // Implementation
}
```

### 5. Dependency Injection for Testability

```rust
// Accept abstractions, not concrete types
pub struct ReexportGenerator<F: FileSystem> {
    fs: F,
    config: ReexportConfig,
}

// Easy to test with mocks
#[test]
fn test_generation() {
    let fs = MockFileSystem::new();
    let generator = ReexportGenerator::new(fs, config);
    // ...
}
```

---

## Comparison with Current Design

| Aspect | Current Design | Clean-Room Design |
|--------|---------------|-------------------|
| Error Handling | Mixed (panic + Result) | 100% Result-based |
| Const Generics | Silent ignore | Explicit unsupported error |
| Input Validation | Partial | Complete at boundaries |
| File I/O | Direct fs calls | Abstracted + cached + tracked |
| Module Paths | Hardcoded | Parameterized |
| String Manipulation | Post-process quote! | Native formatting |
| Projection Keys | Tuples | Type-safe newtype |
| Documentation | String building | Builder pattern |
| Testing | Good | Excellent (DI + mocks) |
| Type Safety | Good | Excellent (newtypes) |
| Canonicalization | panic!() on error | Result-based |

---

## Migration Steps

This section provides concrete steps to migrate from the current implementation to the clean-room design. API breaking changes are acceptable as long as expected behavior tests pass.

### Step 1: Create Unified Error System

**Goal**: Establish Result-based error handling foundation

**Actions**:

1. Create `src/error.rs`:
   ```rust
   pub type Result<T> = std::result::Result<T, Error>;
   
   #[derive(Debug)]
   pub struct Error {
       kind: ErrorKind,
       span: Span,
       context: Vec<String>,
   }
   
   #[derive(Debug)]
   pub enum ErrorKind {
       Parse(syn::Error),
       Validation(ValidationError),
       Resolution(ResolutionError),
       Unsupported(UnsupportedFeature),
       Internal(String),
   }
   
   pub enum UnsupportedFeature {
       ConstGenerics { span: Span },
       VerbatimBounds { span: Span },
       ComplexTypes { description: String, span: Span },
   }
   ```

2. Add `From<Error>` for `syn::Error` conversion

3. Update all public macro entry points to use new error system

**Testing**:
- All existing tests must pass
- Error messages should be equal or better quality

---

### Step 2: Add Input Validation

**Goal**: Validate inputs at parse boundaries

**Actions**:

1. Update `hm_conversion/patterns.rs`:
   ```rust
   impl Parse for KindInput {
       fn parse(input: ParseStream) -> syn::Result<Self> {
           let mut assoc_types = Vec::new();
           while !input.is_empty() {
               assoc_types.push(input.parse()?);
           }
           
           // NEW: Validate non-empty
           if assoc_types.is_empty() {
               return Err(Error::validation(
                   Span::call_site(),
                   "Kind definition must have at least one associated type"
               ));
           }
           
           Ok(KindInput { assoc_types })
       }
   }
   ```

2. Add validation for const generics in `KindInput::parse()`

3. Add UI tests for invalid inputs

**Testing**:
- Add new tests for empty input
- Add tests for const generics error messages
- Existing behavior tests must pass

---

### Step 3: Replace panic!() with Result

**Goal**: Eliminate all panic!() calls in production code

**Actions**:

1. Update `hm_conversion/transformations.rs`:
   - Change `canonicalize_bound` signature to return `Result<String>`
   - Replace `panic!("Unsupported bound type")` with proper error
   - Change `canonicalize_type` signature to return `Result<String>`
   - Replace all `panic!()` calls with `Err(Error::unsupported(...))`

2. Update all callers to propagate errors with `?` operator

3. Update trait definitions:
   ```rust
   pub trait Canonicalizer {
       fn canonicalize_bound(&self, bound: &Bound) -> Result<String>;
       fn canonicalize_type(&self, ty: &Type) -> Result<String>;
   }
   ```

**Testing**:
- All existing tests must pass
- Add tests for previously-panic cases
- Verify error messages are helpful

---

### Step 4: Refactor Documentation String Building

**Goal**: Replace string manipulation with builder pattern

**Actions**:

1. Create `documentation/templates.rs`:
   ```rust
   pub struct DocumentationBuilder<'a> {
       signature: &'a CanonicalSignature,
       name: &'a Ident,
   }
   
   impl DocumentationBuilder<'_> {
       pub fn build(self) -> TokenStream { ... }
       fn build_summary(&self) -> String { ... }
       fn build_overview(&self) -> String { ... }
       fn build_associated_types_section(&self) -> String { ... }
   }
   ```

2. Update `hkt/kind.rs` to use builder:
   ```rust
   let doc = DocumentationBuilder::new(&signature, &name).build();
   ```

3. Remove all string `.replace()` calls on `quote!()` output

**Testing**:
- Documentation output must be functionally equivalent
- Generated docs must be valid Markdown
- Verify with doc tests

---

### Step 5: Add Type-Safe ProjectionKey

**Goal**: Replace tuple-based keys with newtype

**Actions**:

1. Create `ProjectionKey` in `documentation/resolution.rs`:
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct ProjectionKey {
       type_path: String,
       trait_path: Option<String>,
       assoc_name: String,
   }
   ```

2. Update `Config` type to use `ProjectionKey`:
   ```rust
   pub struct Config {
       pub projections: HashMap<ProjectionKey, (Generics, Type)>,
       // ... other fields
   }
   ```

3. Update all code that constructs projection keys

4. Update all code that looks up projections

**Testing**:
- All resolution tests must pass
- Verify no regressions in Self resolution
- Test with complex projection hierarchies

---

### Step 6: Abstract File I/O

**Goal**: Add dependency injection for file operations

**Actions**:

1. Create `codegen/scanner.rs`:
   ```rust
   pub trait FileSystem {
       fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
       fn read_file(&self, path: &Path) -> io::Result<String>;
   }
   
   pub struct RealFileSystem;
   pub struct CachedFileSystem { ... }
   pub struct MockFileSystem { ... }
   ```

2. Update `re_export.rs` to accept `FileSystem` trait:
   ```rust
   pub struct ReexportGenerator<F: FileSystem> {
       fs: F,
       config: ReexportConfig,
   }
   ```

3. Add `proc_macro::tracked_path::path()` calls for dependency tracking

4. Create mock filesystem for tests

**Testing**:
- All re-export tests must pass
- Add tests with MockFileSystem
- Verify incremental compilation triggers correctly

---

### Step 7: Parameterize Module Paths

**Goal**: Remove hardcoded module paths

**Actions**:

1. Update `ReexportConfig` structure:
   ```rust
   pub struct ReexportConfig {
       pub source_dir: PathBuf,
       pub base_module: syn::Path,  // NEW: Not hardcoded
       pub aliases: HashMap<String, Ident>,
       pub item_kind: ItemKind,
   }
   ```

2. Update macro parsing to accept base module:
   ```rust
   #[proc_macro]
   pub fn generate_function_re_exports(input: TokenStream) -> TokenStream {
       // Parse: "src/classes", crate::classes, { aliases }
   }
   ```

3. Update all usages in `fp-library` to pass `crate::classes`

**Testing**:
- All existing re-exports must still work
- Test with different module paths
- Verify generated code is correct

---

### Step 8: Create ResolutionContext Builder

**Goal**: Replace 7-parameter constructor with builder

**Actions**:

1. Create `ResolutionContext` in `documentation/resolution.rs`:
   ```rust
   pub struct ResolutionContext<'a> {
       pub self_type: &'a str,
       pub trait_name: &'a str,
       pub assoc_name: &'a str,
       pub method_doc_use: Option<&'a str>,
       pub impl_doc_use: Option<&'a str>,
       pub span: Span,
   }
   
   impl<'a> ResolutionContext<'a> {
       pub fn new(...) -> Self { ... }
       pub fn with_method_doc_use(mut self, doc_use: &'a str) -> Self { ... }
       pub fn with_impl_doc_use(mut self, doc_use: &'a str) -> Self { ... }
   }
   ```

2. Update `SelfSubstitutor` to accept `ResolutionContext`

3. Update all call sites to use builder pattern

**Testing**:
- All resolution tests must pass
- Verify no behavior changes
- Test builder with various combinations

---

### Step 9: Extract ProjectionResolver

**Goal**: Make resolution hierarchy explicit

**Actions**:

1. Create `ProjectionResolver` in `documentation/resolution.rs`:
   ```rust
   pub struct ProjectionResolver<'a> {
       config: &'a Config,
   }
   
   impl ProjectionResolver<'_> {
       pub fn resolve(&self, context: &ResolutionContext) -> Result<ResolvedType> {
           self.try_method_override(context)
               .or_else(|_| self.try_impl_override(context))
               .or_else(|_| self.try_scoped_default(context))
               .or_else(|_| self.try_module_default(context))
               .map_err(|_| self.create_resolution_error(context))
       }
       
       fn try_method_override(&self, ctx: &ResolutionContext) -> Result<ResolvedType> { ... }
       fn try_impl_override(&self, ctx: &ResolutionContext) -> Result<ResolvedType> { ... }
       fn try_scoped_default(&self, ctx: &ResolutionContext) -> Result<ResolvedType> { ... }
       fn try_module_default(&self, ctx: &ResolutionContext) -> Result<ResolvedType> { ... }
   }
   ```

2. Extract resolution logic from existing scattered locations

3. Update documentation to reference new clear hierarchy

**Testing**:
- All resolution tests must pass
- Test each hierarchy level independently
- Verify error messages maintain quality

---

### Step 10: Refactor Code Duplication in Re-exports

**Goal**: Unify function and trait re-export generation

**Actions**:

1. Create unified implementation:
   ```rust
   enum ItemKind {
       Function,
       Trait,
   }
   
   fn generate_re_exports_impl(
       input: ReexportInput,
       kind: ItemKind
   ) -> Result<TokenStream> {
       // Unified logic
   }
   ```

2. Update `generate_function_re_exports_impl` to call unified version

3. Update `generate_trait_re_exports_impl` to call unified version

4. Remove duplicate code

**Testing**:
- All re-export tests must pass
- Verify both functions and traits still work
- Check generated output is identical

---

### Migration Testing Strategy

**For each step above:**

1. **Before making changes:**
   - Run full test suite: `cargo test --all-features`
   - Document current behavior with snapshot tests if needed

2. **During migration:**
   - Make changes incrementally
   - Run tests after each sub-change
   - Use `#[allow(deprecated)]` for compatibility shims if needed

3. **After completing step:**
   - Run full test suite again
   - Run integration tests in `fp-library`
   - Verify all doctests pass
   - Check UI tests for error message quality

4. **Cross-step validation:**
   - After every 2-3 steps, run comprehensive tests
   - Verify no performance regressions with benchmarks
   - Test incremental compilation behavior

**Rollback strategy:**
- Each step should be a separate commit
- Tag stable points: `migration-step-N-stable`
- Keep backward compatibility shims for 1-2 steps if needed

**Success criteria:**
- All existing tests pass
- No panics in production code
- Error messages are equal or better
- Documentation quality maintained
- Performance within 10% of current

---

## What to Preserve

The current design has many strengths worth keeping:

### 1. ✅ Modular Structure
The current module organization is excellent:
- `hkt/` - HKT macros
- `hm_conversion/` - Type conversion
- `documentation/` - Doc generation
- `analysis/` - Type analysis
- `resolution/` - Self resolution
- `common/` - Shared utilities
- `config/` - Configuration

**Keep this structure**, just refine internal implementations.

### 2. ✅ Deterministic Naming
The hash-based Kind trait naming system is solid:
- Canonical signatures ensure consistency
- Deterministic hashing works well
- Good separation of concerns

**Keep the algorithm**, just make error handling robust.

### 3. ✅ Visitor Pattern
The `TypeVisitor` trait for type traversal is a good abstraction:
- Extensible
- Composable
- Well-tested

**Keep the pattern**, just return Results instead of panicking.

### 4. ✅ Config Caching
Performance optimization with `LazyLock` is smart:
- Loads config once per compilation
- Efficient caching strategy

**Keep the caching**, just abstract file system for testing.

### 5. ✅ Comprehensive Tests
Good test coverage throughout:
- Unit tests in modules
- Integration tests
- Property-based tests
- UI tests for errors

**Keep and expand** the test suite.

### 6. ✅ Public API Documentation
Excellent documentation in `lib.rs`:
- Clear examples
- Comprehensive explanations
- Usage patterns

**Keep and maintain** this documentation standard.

---

## Conclusion

The clean-room design focuses on:

1. **Eliminating panics** - Everything returns Result
2. **Type safety** - Newtypes for all key concepts
3. **Testability** - Dependency injection everywhere
4. **Clarity** - Explicit over implicit
5. **Flexibility** - Parameterized, not hardcoded
6. **Validation** - Check at boundaries, fail fast

**Key Insight**: The current implementation is **good and production-ready**. The clean-room design represents an ideal architecture that addresses identified issues while preserving the system's strengths.

**Recommendation**: Rather than a complete rewrite, pursue **incremental refactoring** starting with high-priority issues:
1. Replace panics with Results (Phase 1, Issue #1)
2. Add const generic handling (Phase 1, Issue #2)
3. Add input validation (Phase 1, Issue #3)

This approach delivers value quickly while moving toward the ideal design over time.
