use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Type, GenericParam, TypeParamBound, TraitBound, PathArguments, GenericArgument, ReturnType, WherePredicate, parse_quote};
use syn::spanned::Spanned;
use std::collections::HashMap;

pub fn hm_signature_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = syn::parse2::<ItemFn>(item).expect("Failed to parse function");
    let trait_name = if attr.is_empty() {
        None
    } else {
        Some(attr.to_string())
    };
    
    let signature = generate_signature(&input_fn, trait_name.as_deref());
    let doc_comment = format!("`{}`", signature);
    
    insert_doc_comment(&mut input_fn, doc_comment, proc_macro2::Span::call_site());

    quote! {
        #input_fn
    }.into()
}

fn insert_doc_comment(input_fn: &mut ItemFn, doc_comment: String, macro_span: proc_macro2::Span) {
    let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);

    // Find insertion point based on macro invocation position
    let mut insert_idx = input_fn.attrs.len();
    
    for (i, attr) in input_fn.attrs.iter().enumerate() {
        // If the attribute is after the macro invocation, insert before it
        if attr.span().start().line > macro_span.start().line {
            insert_idx = i;
            break;
        }
    }

    input_fn.attrs.insert(insert_idx, doc_attr);
}

fn generate_signature(input: &ItemFn, trait_context: Option<&str>) -> String {
    let mut fn_bounds = HashMap::new();
    
    // Collect Fn bounds from generics
    for param in &input.sig.generics.params {
        if let GenericParam::Type(type_param) = param {
            let name = type_param.ident.to_string();
            for bound in &type_param.bounds {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    if let Some(sig) = get_fn_signature(trait_bound, &fn_bounds) {
                        fn_bounds.insert(name.clone(), sig);
                    }
                }
            }
        }
    }

    // Collect Fn bounds from where clause
    if let Some(where_clause) = &input.sig.generics.where_clause {
        for predicate in &where_clause.predicates {
            if let WherePredicate::Type(predicate_type) = predicate {
                if let Type::Path(type_path) = &predicate_type.bounded_ty {
                    if type_path.path.segments.len() == 1 {
                        let name = type_path.path.segments[0].ident.to_string();
                        for bound in &predicate_type.bounds {
                            if let TypeParamBound::Trait(trait_bound) = bound {
                                if let Some(sig) = get_fn_signature(trait_bound, &fn_bounds) {
                                    fn_bounds.insert(name.clone(), sig);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let (mut forall, mut constraints) = format_generics(&input.sig.generics, &fn_bounds);
    let params = format_parameters(input, &fn_bounds);
    let ret = format_return_type(&input.sig.output, &fn_bounds);

    // Check if "self" is used in params or return type
    let uses_self = params.contains("self") || ret.contains("self");
    
    if uses_self {
        // Add "self" to forall
        if forall.is_empty() {
            forall = "forall self.".to_string();
        } else {
            // Insert "self" after "forall "
            forall.insert_str(7, "self ");
        }

        if let Some(trait_name) = trait_context {
             let constraint = format!("{} self", trait_name);
             if constraints.is_empty() {
                 constraints = constraint;
             } else {
                 // Prepend constraint
                 if constraints.starts_with('(') && constraints.ends_with(')') {
                     constraints.insert_str(1, &format!("{}, ", constraint));
                 } else {
                     constraints = format!("({}, {})", constraint, constraints);
                 }
             }
        }
    }

    let mut parts = Vec::new();
    if !forall.is_empty() {
        parts.push(forall);
    }
    if !constraints.is_empty() {
        parts.push(format!("{} =>", constraints));
    }
    
    let func_sig = if params.is_empty() {
        ret
    } else {
        format!("{} -> {}", params, ret)
    };
    parts.push(func_sig);

    parts.join(" ")
}

fn get_fn_signature(trait_bound: &TraitBound, fn_bounds: &HashMap<String, String>) -> Option<String> {
    let name = trait_bound.path.segments.last().unwrap().ident.to_string();
    if name == "Fn" || name == "FnMut" || name == "FnOnce" {
        Some(format_fn_trait(trait_bound, fn_bounds))
    } else if name == "SendCloneableFn" || name == "CloneableFn" || name == "Function" {
        Some("fn_brand_marker".to_string())
    } else {
        None
    }
}

fn format_generics(generics: &syn::Generics, fn_bounds: &HashMap<String, String>) -> (String, String) {
    let mut type_vars = Vec::new();
    let mut constraints = Vec::new();

    for param in &generics.params {
        if let GenericParam::Type(type_param) = param {
            let name = type_param.ident.to_string();
            
            // Only include in forall if it's not a function type variable that we are expanding
            if !fn_bounds.contains_key(&name) {
                type_vars.push(name.to_lowercase());
            }

            for bound in &type_param.bounds {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    if let Some(constraint) = format_trait_bound(trait_bound, &name.to_lowercase()) {
                        constraints.push(constraint);
                    }
                }
            }
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for predicate in &where_clause.predicates {
            if let WherePredicate::Type(predicate_type) = predicate {
                let type_name = format_type(&predicate_type.bounded_ty, fn_bounds);
                for bound in &predicate_type.bounds {
                    if let TypeParamBound::Trait(trait_bound) = bound {
                        if let Some(constraint) = format_trait_bound(trait_bound, &type_name) {
                            constraints.push(constraint);
                        }
                    }
                }
            }
        }
    }

    let forall = if type_vars.is_empty() {
        String::new()
    } else {
        format!("forall {}.", type_vars.join(" "))
    };

    let constraints_str = if constraints.is_empty() {
        String::new()
    } else if constraints.len() == 1 {
        constraints[0].clone()
    } else {
        format!("({})", constraints.join(", "))
    };

    (forall, constraints_str)
}

fn format_trait_bound(bound: &TraitBound, type_var: &str) -> Option<String> {
    let trait_name = bound.path.segments.last().unwrap().ident.to_string();
    
    // Filter out implementation details
    match trait_name.as_str() {
        "Clone" | "Copy" | "Debug" | "Display" | "PartialEq" | "Eq" | "PartialOrd" | "Ord" | "Hash" | "Default" | "Send" | "Sync" | "Sized" | "Unpin" => None,
        // Also filter out function traits used in bounds (e.g. FnBrand: SendCloneableFn)
        "Fn" | "FnMut" | "FnOnce" | "CloneableFn" | "SendCloneableFn" | "Function" => None,
        _ => Some(format!("{} {}", trait_name, type_var)),
    }
}

fn format_parameters(input: &ItemFn, fn_bounds: &HashMap<String, String>) -> String {
    let mut params = Vec::new();
    for input in &input.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            params.push(format_type(&pat_type.ty, fn_bounds));
        }
    }

    if params.is_empty() {
        String::new()
    } else if params.len() == 1 {
        params[0].clone()
    } else {
        format!("({})", params.join(", "))
    }
}

fn format_return_type(output: &ReturnType, fn_bounds: &HashMap<String, String>) -> String {
    match output {
        ReturnType::Default => "()".to_string(), // Unit type
        ReturnType::Type(_, ty) => format_type(ty, fn_bounds),
    }
}

fn format_brand_name(name: &str) -> String {
    if name.ends_with("Brand") {
        name[..name.len() - 5].to_string()
    } else {
        name.to_string()
    }
}

fn format_type_arg(ty: &Type, fn_bounds: &HashMap<String, String>) -> String {
    let s = format_type(ty, fn_bounds);
    // If the string contains spaces and isn't already wrapped in parens, wrap it.
    // Simple heuristic: if it contains space and doesn't start with '(', wrap it.
    // Or if it starts with '(' but the matching ')' is not at the end (e.g. "(a -> b) -> c").
    // A robust check would count parens.
    
    if !s.contains(' ') {
        return s;
    }
    
    // Check if it's fully parenthesized
    if s.starts_with('(') && s.ends_with(')') {
        // Check if the outer parens are matching
        let mut depth = 0;
        let mut fully_wrapped = true;
        for (i, c) in s.chars().enumerate() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 && i < s.len() - 1 {
                    fully_wrapped = false;
                    break;
                }
            }
        }
        if fully_wrapped {
            return s;
        }
    }
    
    format!("({})", s)
}

fn format_type(ty: &Type, fn_bounds: &HashMap<String, String>) -> String {
    match ty {
        Type::Path(type_path) => {
            // Handle <F as Kind...>::Of<'a, A>
            if let Some(qself) = &type_path.qself {
                // Check for function traits
                if type_path.path.segments.len() >= 2 {
                    let trait_name = type_path.path.segments[0].ident.to_string();
                    if trait_name == "SendCloneableFn" || trait_name == "CloneableFn" || trait_name == "Function" {
                        let last_segment = type_path.path.segments.last().unwrap();
                        if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            let mut type_args = Vec::new();
                            for arg in &args.args {
                                if let GenericArgument::Type(inner_ty) = arg {
                                    type_args.push(format_type(inner_ty, fn_bounds));
                                }
                            }
                            // Skip lifetime if present (usually first arg)
                            // But we don't know if first arg is lifetime from here easily,
                            // as we only collected types.
                            // However, `args.args` contains GenericArgument which can be Lifetime or Type.
                            // We only pushed Types.
                            // So `type_args` contains only types.
                            // Assuming structure <...>::Assoc<'a, Input, Output>
                            // type_args should have 2 elements: Input and Output.
                            
                            if type_args.len() >= 2 {
                                let input = &type_args[type_args.len() - 2];
                                let output = &type_args[type_args.len() - 1];
                                
                                // If input is a tuple string "(a, b)", we keep it.
                                // If input is "a", we keep it.
                                // We want "input -> output".
                                // But we need to wrap input in parens if it's a function type?
                                // format_type returns string.
                                
                                // Wait, format_type returns "(a, b)" for tuple.
                                // So we just join them with " -> ".
                                return format!("{} -> {}", input, output);
                            }
                        }
                    }
                }

                let constructor = format_type(&qself.ty, fn_bounds);
                let last_segment = type_path.path.segments.last().unwrap();
                
                if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                     let mut type_args = Vec::new();
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            type_args.push(format_type_arg(inner_ty, fn_bounds));
                        }
                    }
                    if !type_args.is_empty() {
                        return format!("{} {}", constructor, type_args.join(" "));
                    }
                }
                return constructor;
            }

            // Handle associated types: F::Of<A> -> f a
            // Or Self::Of<A> -> self a
            if type_path.path.segments.len() >= 2 {
                let first = &type_path.path.segments[0];
                let last = type_path.path.segments.last().unwrap();
                
                let constructor = first.ident.to_string().to_lowercase();
                
                if let PathArguments::AngleBracketed(args) = &last.arguments {
                     let mut type_args = Vec::new();
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            type_args.push(format_type_arg(inner_ty, fn_bounds));
                        }
                    }
                    if !type_args.is_empty() {
                        return format!("{} {}", constructor, type_args.join(" "));
                    }
                }
            }

            let segment = type_path.path.segments.last().unwrap();
            let name = segment.ident.to_string();
            
            // Check if this type is a function type variable
            if let Some(sig) = fn_bounds.get(&name) {
                if sig == "fn_brand_marker" {
                    return name.to_lowercase();
                }
                // If it's a function type, we should wrap it in parens if it's used as an argument
                // But format_type returns a string, so we can't easily know context.
                // However, arrow syntax usually needs parens when nested on the left.
                // For now, let's return it as is, and let the caller handle parens if needed?
                // Or better, always wrap in parens if it contains "->"?
                if sig.contains("->") {
                    return format!("({})", sig);
                }
                return sig.clone();
            }
            
            if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                return name.to_lowercase();
            }
            
            if name == "Self" {
                return "self".to_string();
            }

            let name = format_brand_name(&name);

            // Handle generics
            match &segment.arguments {
                PathArguments::AngleBracketed(args) => {
                    let mut type_args = Vec::new();
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            type_args.push(format_type_arg(inner_ty, fn_bounds));
                        }
                    }
                    if type_args.is_empty() {
                        name
                    } else {
                        format!("{} {}", name, type_args.join(" "))
                    }
                },
                _ => name,
            }
        },
        Type::ImplTrait(impl_trait) => {
            // Handle impl Fn(A) -> B
            for bound in &impl_trait.bounds {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    let name = trait_bound.path.segments.last().unwrap().ident.to_string();
                    if name == "Fn" || name == "FnMut" || name == "FnOnce" {
                        return format_fn_trait(trait_bound, fn_bounds);
                    }
                }
            }
            "impl_trait".to_string() // Fallback
        },
        Type::Reference(type_ref) => format_type(&type_ref.elem, fn_bounds), // Ignore reference
        Type::Tuple(tuple) => {
            let types: Vec<String> = tuple.elems.iter().map(|t| format_type(t, fn_bounds)).collect();
            format!("({})", types.join(", "))
        },
        Type::Macro(type_macro) => {
            if type_macro.mac.path.is_ident("Apply") {
                match syn::parse2::<crate::apply::ApplyInput>(type_macro.mac.tokens.clone()) {
                    Ok(apply_input) => {
                        let constructor = format_type(&apply_input.brand, fn_bounds);
                        let mut type_args = Vec::new();
                        for arg in &apply_input.args.args {
                            if let GenericArgument::Type(inner_ty) = arg {
                                type_args.push(format_type_arg(inner_ty, fn_bounds));
                            }
                        }
                        if !type_args.is_empty() {
                            return format!("{} {}", constructor, type_args.join(" "));
                        }
                        return constructor;
                    }
                    Err(e) => return format!("macro_error: {}", e),
                }
            }
            "macro".to_string()
        }
        _ => "_".to_string(), // Fallback
    }
}

fn format_fn_trait(trait_bound: &TraitBound, fn_bounds: &HashMap<String, String>) -> String {
    let segment = trait_bound.path.segments.last().unwrap();
    if let PathArguments::Parenthesized(args) = &segment.arguments {
        let inputs: Vec<String> = args.inputs.iter().map(|t| format_type(t, fn_bounds)).collect();
        let output = match &args.output {
            ReturnType::Default => "()".to_string(),
            ReturnType::Type(_, ty) => format_type(ty, fn_bounds),
        };
        
        let input_str = if inputs.len() == 1 {
            inputs[0].clone()
        } else {
            format!("({})", inputs.join(", "))
        };
        
        format!("{} -> {}", input_str, output)
    } else {
        "fn".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_simple_signature() {
        let input: ItemFn = parse_quote! {
            fn identity<A>(x: A) -> A { x }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall a. a -> a");
    }

    #[test]
    fn test_impl_fn() {
        let input: ItemFn = parse_quote! {
            fn map<A, B>(f: impl Fn(A) -> B, x: A) -> B { todo!() }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall a b. (a -> b, a) -> b");
    }
    
    #[test]
    fn test_associated_type() {
        let input: ItemFn = parse_quote! {
            fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B> { todo!() }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall f a b. Functor f => (a -> b, f a) -> f b");
    }

    #[test]
    fn test_apply_macro() {
        let input: ItemFn = parse_quote! {
            fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: Apply!(<F as Kind!(type Of<'a, T>: 'a;)>::Of<'a, A>)) -> Apply!(<F as Kind!(type Of<'a, T>: 'a;)>::Of<'a, B>) { todo!() }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall f a b. Functor f => (a -> b, f a) -> f b");
    }

    #[test]
    fn test_brand_name() {
        let input: ItemFn = parse_quote! {
            fn map<A, B>(x: OptionBrand<A>) -> OptionBrand<B> { todo!() }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall a b. Option a -> Option b");
    }

    #[test]
    fn test_where_clause() {
        let input: ItemFn = parse_quote! {
            fn map<F, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>
            where F: Functor
            { todo!() }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall f a b. Functor f => (a -> b, f a) -> f b");
    }

    #[test]
    fn test_fn_bound_in_where() {
        let input: ItemFn = parse_quote! {
            fn map<Func, A, B>(f: Func, x: A) -> B
            where Func: Fn(A) -> B
            { todo!() }
        };
        let sig = generate_signature(&input, None);
        assert_eq!(sig, "forall a b. ((a -> b), a) -> b");
    }

    #[test]
    fn test_wilt_signature() {
        let input: ItemFn = parse_quote! {
            fn wilt<'a, M: Applicative, A: 'a + Clone, O: 'a + Clone, E: 'a + Clone, Func>(
                func: Func,
                ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
            ) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
                'a,
                Pair<
                    Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
                    Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
                >,
            >)
            where
                Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
                Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
                Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
            {
                todo!()
            }
        };
        let sig = generate_signature(&input, Some("Witherable"));
        assert_eq!(sig, "forall self m a o e. (Witherable self, Applicative m) => ((a -> m (Result o e)), self a) -> m (Pair (self o) (self e))");
    }

    #[test]
    fn test_placement_logic() {
        // 1: empty
        // 2: First
        // 3: empty (where macro was)
        // 4: Second
        let code = "\n#[doc = \"First\"]\n\n#[doc = \"Second\"]\nfn foo() {}";
        let mut input_fn: ItemFn = syn::parse_str(code).unwrap();
        
        // Create span at line 3
        let span_source = "\n\nstruct S;";
        let ts: proc_macro2::TokenStream = span_source.parse().unwrap();
        let macro_span = ts.into_iter().next().unwrap().span();
        
        insert_doc_comment(&mut input_fn, "Signature".to_string(), macro_span);
        
        assert_eq!(input_fn.attrs.len(), 3);
        
        let get_doc = |attr: &syn::Attribute| -> String {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(lit) = &nv.value {
                    if let syn::Lit::Str(s) = &lit.lit {
                        return s.value();
                    }
                }
            }
            panic!("Not a doc comment");
        };

        assert_eq!(get_doc(&input_fn.attrs[0]), "First");
        assert_eq!(get_doc(&input_fn.attrs[1]), "Signature");
        assert_eq!(get_doc(&input_fn.attrs[2]), "Second");
    }

    #[test]
    fn test_par_fold_right() {
        let input: ItemFn = parse_quote! {
            fn par_fold_right<'a, FnBrand, A, B>(
                func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
                init: B,
                fa: <Self as Kind_cdc7cd43dac7585f>::Of<'a, A>,
            ) -> B
            where
                A: 'a + Clone + Send + Sync,
                B: Send + Sync + 'a,
                FnBrand: 'a + SendCloneableFn,
            { todo!() }
        };
        let sig = generate_signature(&input, Some("ParFoldable"));
        // Expected: forall self a b. ParFoldable self => ((a, b) -> b, b, self a) -> b
        assert_eq!(sig, "forall self a b. ParFoldable self => ((a, b) -> b, b, self a) -> b");
    }
}
