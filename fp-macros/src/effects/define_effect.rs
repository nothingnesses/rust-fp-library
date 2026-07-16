//! Parsing and emission for the `define_effect!` macro.
//!
//! The input is a block of smart-constructor signatures grouped under an
//! effect header. Every emitted item derives from those signatures:
//!
//! - Each `fn name(payloads...) -> Resume;` line is one operation. The return
//!   type is the continuation position (the value the handler resumes the
//!   continuation with); `-> !` declares a no-resume (aborting) operation
//!   whose variant stores `PhantomData` instead of a continuation.
//! - A payload written `Program<T>` is a sub-program over the ambient effect
//!   row with result `T`; its presence (directly or as a callable's return)
//!   classifies the effect higher-order and adds a row type parameter `R` to
//!   the emitted brand and operations enum. Effects without one are
//!   first-order.
//! - A payload written `impl FnOnce(Args...) -> Ret` is a callable stored as
//!   `Box<dyn FnOnce(Args...) -> Ret + 'a>`; `Ret` may itself be
//!   `Program<T>`. `impl Fn` payloads are reserved for the multi-shot stores
//!   and rejected until an emission for them exists, rather than emitting
//!   single-shot storage with the wrong resume semantics.
//! - The `#[multi_shot]` operation attribute emits the operation's
//!   continuation in the `Rc` store's re-callable form (`Rc<dyn Fn>` instead
//!   of `Box<dyn FnOnce>`), so a forking runner may invoke one captured
//!   continuation once per branch; the `Functor` arm composes by re-wrapping.
//!   It requires a resume type (a `-> !` operation has no continuation to
//!   re-call) and is rejected on higher-order operations (prompt finalization
//!   and multi-shot resumption conflict, so sub-program-owning operations
//!   stay single-shot). An effect containing a `#[multi_shot]` operation
//!   emits no handler pieces: the one-pass `#[handlers]` surface is
//!   single-shot by construction (an arm resumes exactly once), so such
//!   effects are interpreted by the narrowing runners instead, and placing
//!   one in a `#[handlers]` row fails with a missing-`HandlerPieces` bound.
//!   The `Arc` re-callable form is deliberately not emitted: `Functor::map`'s
//!   function carries no `Send` bound and cannot be re-wrapped into
//!   `Arc<dyn Fn + Send + Sync>`; that form needs the `SendFunctor` route and
//!   waits for a `Send` consumer.
//!
//! Names are used verbatim (the constructor keeps the spec name; the variant
//! is the spec name's UpperCamelCase form) and collisions are expansion
//! errors, never silently suffixed. The generic parameter names `R`, `I`,
//! `A`, `B`, `Label`, and `Store` and the payload name `k` are reserved by
//! the emission.
//!
//! A first-order operation's constructors are store-generic: an inferred
//! `Store` parameter (last) selects the `Free` spine's closure store, with
//! the `Box` default serving single-shot call sites unchanged. Higher-order
//! constructors keep the `Box`-store return, because a higher-order cell
//! pins `Box`-store types in the operations enum itself (`Free<R, _>`
//! sub-program payloads and the `Box<dyn FnOnce>` continuation), so a
//! store-generic return would admit a program no interpreter serves.
//!
//! Alongside each smart constructor the emission carries its labelled
//! variant `<name>_at<Label, ...>`: the same signature with a leading label
//! parameter, injecting at `TaggedBrand<Label, Brand>` rather than the bare
//! brand, so a row holding the effect under several labels addresses one
//! cell specifically. An explicit operation named `<name>_at` therefore
//! collides with the labelled constructor emitted for an operation named
//! `<name>`, and the pair is rejected at parse time.
//!
//! Alongside the cell, the emission carries the effect's handler pieces for
//! the row-level handler surface (`define_row!`'s `#[handlers]` extension
//! reaches them by path-resolved projection): the `<Name>Arms` bundle (one
//! boxed closure field per resumptive operation; payloads-to-resume-value
//! for first-order operations, sub-programs plus per-pin re-entry handles
//! returning `Result` for higher-order ones), the `<Name>Abort` type (one
//! variant per no-resume operation, uninhabited when every operation
//! resumes), and the `EffectAbort`/`HandlerPieces` impls on the brand whose
//! dispatch interprets one lowered operation against the arms.

use {
	crate::hkt::{
		AssociatedTypes,
		ImplKindInput,
		generate_name,
		impl_kind_worker,
	},
	proc_macro2::{
		TokenStream,
		TokenTree,
	},
	quote::{
		format_ident,
		quote,
	},
	syn::{
		Attribute,
		GenericParam,
		Generics,
		Ident,
		Path,
		ReturnType,
		Token,
		Type,
		TypeParamBound,
		Visibility,
		braced,
		parenthesized,
		parse::{
			Parse,
			ParseStream,
		},
		spanned::Spanned,
	},
};

syn::custom_keyword!(effect);

/// The handler-state taxonomy classes an effect must declare via
/// `#[handler_state(...)]`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HandlerState {
	/// The handler holds no state for this effect.
	None,
	/// A by-value handler field, scoped by derivation at recursive
	/// interpretation.
	ScopedByValue,
	/// A shared cell surviving recursive interpretation.
	SharedByReference,
	/// An accumulator threaded through the interpretation loop; reserved
	/// until the threaded runners exist.
	ThreadedByValue,
}

impl HandlerState {
	fn parse_ident(ident: &Ident) -> Option<Self> {
		match ident.to_string().as_str() {
			"none" => Some(Self::None),
			"scoped_by_value" => Some(Self::ScopedByValue),
			"shared_by_reference" => Some(Self::SharedByReference),
			"threaded_by_value" => Some(Self::ThreadedByValue),
			_ => Option::None,
		}
	}

	fn prose(self) -> &'static str {
		match self {
			Self::None => "none (the handler holds no state for this effect)",
			Self::ScopedByValue =>
				"scoped by value (a by-value handler field, scoped by derivation at recursive interpretation)",
			Self::SharedByReference =>
				"shared by reference (a shared cell surviving recursive interpretation)",
			Self::ThreadedByValue =>
				"threaded by value (an accumulator threaded through the interpretation loop)",
		}
	}
}

/// A payload parameter's classification.
enum PayloadKind {
	/// A plain by-value payload field.
	Value(Type),
	/// A sub-program over the ambient row with the carried result type.
	Program(Type),
	/// A one-shot callable payload.
	Callable { inputs: Vec<Type>, output: CallableRet },
}

/// A callable payload's return classification.
enum CallableRet {
	/// A plain value (unit when the callable declares no return type).
	Value(Type),
	/// A sub-program over the ambient row with the carried result type.
	Program(Type),
}

impl PayloadKind {
	fn is_higher_order(&self) -> bool {
		match self {
			Self::Program(_) => true,
			Self::Callable {
				output: CallableRet::Program(_), ..
			} => true,
			Self::Value(_)
			| Self::Callable {
				..
			} => false,
		}
	}
}

/// One parsed operation.
struct Operation {
	docs: Vec<Attribute>,
	name: Ident,
	variant: Ident,
	payloads: Vec<(Ident, PayloadKind)>,
	/// `Some(resume type)` for a resuming operation; `None` for `-> !`.
	resume: Option<Type>,
	/// `true` when the continuation is emitted in the `Rc` store's
	/// re-callable form (`#[multi_shot]`).
	multi_shot: bool,
}

impl Operation {
	fn is_higher_order(&self) -> bool {
		self.payloads.iter().any(|(_, kind)| kind.is_higher_order())
	}
}

/// The parsed `define_effect!` input.
pub struct EffectSpec {
	docs: Vec<Attribute>,
	handler_state: HandlerState,
	crate_path: Path,
	vis: Visibility,
	name: Ident,
	generics: Generics,
	operations: Vec<Operation>,
}

/// Splits a snake_case operation name into its UpperCamelCase variant name.
fn variant_ident(name: &Ident) -> syn::Result<Ident> {
	let text = name.to_string();
	let mut variant = String::new();
	for segment in text.split('_') {
		let mut chars = segment.chars();
		match chars.next() {
			Some(first) => {
				variant.extend(first.to_uppercase());
				variant.push_str(chars.as_str());
			}
			None => {
				return Err(syn::Error::new(
					name.span(),
					"operation names must be snake_case with non-empty segments (no leading, trailing, or doubled underscores)",
				));
			}
		}
	}
	Ok(format_ident!("{}", variant, span = name.span()))
}

/// Classifies a payload type written `Program<T>`.
fn as_program(ty: &Type) -> Option<Type> {
	let Type::Path(type_path) = ty else {
		return None;
	};
	if type_path.qself.is_some() || type_path.path.segments.len() != 1 {
		return None;
	}
	let segment = type_path.path.segments.first()?;
	if segment.ident != "Program" {
		return None;
	}
	let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
		return None;
	};
	if arguments.args.len() != 1 {
		return None;
	}
	match arguments.args.first()? {
		syn::GenericArgument::Type(inner) => Some(inner.clone()),
		_ => None,
	}
}

/// Classifies a payload type written `impl FnOnce(Args...) -> Ret`.
fn as_callable(ty: &Type) -> syn::Result<Option<(Vec<Type>, CallableRet)>> {
	let Type::ImplTrait(impl_trait) = ty else {
		return Ok(None);
	};
	if impl_trait.bounds.len() != 1 {
		return Err(syn::Error::new(
			ty.span(),
			"callable payloads are written as exactly `impl FnOnce(Args...) -> Ret` with no further bounds (lifetime and auto-trait bounds are added by the emission)",
		));
	}
	let Some(TypeParamBound::Trait(bound)) = impl_trait.bounds.first() else {
		return Err(syn::Error::new(
			ty.span(),
			"callable payloads are written as exactly `impl FnOnce(Args...) -> Ret`",
		));
	};
	let Some(segment) = bound.path.segments.last() else {
		return Err(syn::Error::new(ty.span(), "callable payloads name `FnOnce` directly"));
	};
	match segment.ident.to_string().as_str() {
		"FnOnce" => {}
		"Fn" => {
			return Err(syn::Error::new(
				ty.span(),
				"`impl Fn` payloads (re-callable) are reserved for the multi-shot stores and are not yet emitted",
			));
		}
		"FnMut" => {
			return Err(syn::Error::new(ty.span(), "`impl FnMut` payloads are not supported"));
		}
		_ => return Ok(None),
	}
	let syn::PathArguments::Parenthesized(arguments) = &segment.arguments else {
		return Err(syn::Error::new(
			ty.span(),
			"callable payloads use parenthesized argument syntax: `impl FnOnce(Args...) -> Ret`",
		));
	};
	let inputs: Vec<Type> = arguments.inputs.iter().cloned().collect();
	let output = match &arguments.output {
		ReturnType::Default => CallableRet::Value(syn::parse_quote!(())),
		ReturnType::Type(_, output_ty) => match as_program(output_ty) {
			Some(inner) => CallableRet::Program(inner),
			None => CallableRet::Value((**output_ty).clone()),
		},
	};
	Ok(Some((inputs, output)))
}

/// The outer attributes of an effect or operation, split into doc comments
/// and the recognised markers.
struct SplitAttributes {
	docs: Vec<Attribute>,
	handler_state: Option<HandlerState>,
	crate_path: Option<Path>,
	multi_shot: bool,
}

/// Splits the outer attributes into doc comments and the recognised markers.
fn split_attributes(
	attrs: Vec<Attribute>,
	allow_effect_markers: bool,
) -> syn::Result<SplitAttributes> {
	let mut docs = Vec::new();
	let mut handler_state = None;
	let mut crate_path = None;
	let mut multi_shot = false;
	for attr in attrs {
		if attr.path().is_ident("doc") {
			docs.push(attr);
		} else if allow_effect_markers && attr.path().is_ident("handler_state") {
			if handler_state.is_some() {
				return Err(syn::Error::new(attr.span(), "duplicate `#[handler_state(...)]`"));
			}
			let ident: Ident = attr.parse_args()?;
			handler_state = Some(HandlerState::parse_ident(&ident).ok_or_else(|| {
				syn::Error::new(
					ident.span(),
					"`#[handler_state(...)]` takes one of: `none`, `scoped_by_value`, `shared_by_reference`, `threaded_by_value`",
				)
			})?);
		} else if allow_effect_markers && attr.path().is_ident("crate_path") {
			if crate_path.is_some() {
				return Err(syn::Error::new(attr.span(), "duplicate `#[crate_path(...)]`"));
			}
			crate_path = Some(attr.parse_args()?);
		} else if !allow_effect_markers && attr.path().is_ident("multi_shot") {
			multi_shot = true;
		} else {
			return Err(syn::Error::new(attr.span(), "unrecognised attribute in `define_effect!`"));
		}
	}
	Ok(SplitAttributes {
		docs,
		handler_state,
		crate_path,
		multi_shot,
	})
}

impl Parse for EffectSpec {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let SplitAttributes {
			docs,
			handler_state,
			crate_path,
			..
		} = split_attributes(attrs, true)?;
		let handler_state = handler_state.ok_or_else(|| {
			syn::Error::new(
				input.span(),
				"an effect must declare its handler-state class with `#[handler_state(...)]`",
			)
		})?;
		let crate_path = match crate_path {
			Some(path) => path,
			None => syn::parse_quote!(::fp_library),
		};
		let vis: Visibility = input.parse()?;
		input.parse::<effect>()?;
		let name: Ident = input.parse()?;
		let generics: Generics = input.parse()?;
		if let Some(where_clause) = &generics.where_clause {
			return Err(syn::Error::new(
				where_clause.span(),
				"effect generics do not take a `where` clause; write the bounds inline",
			));
		}
		for param in &generics.params {
			let GenericParam::Type(type_param) = param else {
				return Err(syn::Error::new(
					param.span(),
					"effect generics are type parameters only (no lifetimes or const parameters)",
				));
			};
			if ["R", "I", "A", "B", "Label", "Store"]
				.contains(&type_param.ident.to_string().as_str())
			{
				return Err(syn::Error::new(
					type_param.ident.span(),
					"the generic parameter names `R`, `I`, `A`, `B`, `Label`, and `Store` are reserved by the emission",
				));
			}
			let has_static = type_param.bounds.iter().any(
				|bound| matches!(bound, TypeParamBound::Lifetime(lifetime) if lifetime.ident == "static"),
			);
			if !has_static {
				return Err(syn::Error::new(
					type_param.ident.span(),
					"every effect type parameter must be bounded `: 'static` (the erased substrate the emission targets)",
				));
			}
		}
		if docs.is_empty() {
			return Err(syn::Error::new(
				name.span(),
				"every effect must carry a doc comment; it is emitted onto the brand",
			));
		}

		let body;
		braced!(body in input);
		let mut operations = Vec::new();
		while !body.is_empty() {
			let attrs = body.call(Attribute::parse_outer)?;
			let SplitAttributes {
				docs: op_docs,
				multi_shot,
				..
			} = split_attributes(attrs, false)?;
			body.parse::<Token![fn]>()?;
			let op_name: Ident = body.parse()?;
			if op_docs.is_empty() {
				return Err(syn::Error::new(
					op_name.span(),
					"every operation must carry a doc comment; it is emitted onto the variant and the constructor",
				));
			}
			let arguments;
			parenthesized!(arguments in body);
			let mut payloads = Vec::new();
			let parsed_arguments = arguments.parse_terminated(parse_payload_argument, Token![,])?;
			for (arg_name, arg_ty) in parsed_arguments {
				if arg_name == "k" {
					return Err(syn::Error::new(
						arg_name.span(),
						"the payload name `k` is reserved for the emitted continuation field",
					));
				}
				let kind = match as_program(&arg_ty) {
					Some(inner) => PayloadKind::Program(inner),
					None => match as_callable(&arg_ty)? {
						Some((inputs, output)) => PayloadKind::Callable {
							inputs,
							output,
						},
						None => PayloadKind::Value(arg_ty),
					},
				};
				payloads.push((arg_name, kind));
			}
			let resume = match body.parse::<ReturnType>()? {
				ReturnType::Default => Some(syn::parse_quote!(())),
				ReturnType::Type(_, ty) => match *ty {
					Type::Never(_) => None,
					other => Some(other),
				},
			};
			body.parse::<Token![;]>()?;
			let variant = variant_ident(&op_name)?;
			if multi_shot && resume.is_none() {
				return Err(syn::Error::new(
					op_name.span(),
					"`#[multi_shot]` requires a resume type; a `-> !` operation has no continuation to re-call",
				));
			}
			operations.push(Operation {
				docs: op_docs,
				name: op_name,
				variant,
				payloads,
				resume,
				multi_shot,
			});
		}
		if operations.is_empty() {
			return Err(syn::Error::new(
				name.span(),
				"an effect must declare at least one operation",
			));
		}
		let mut seen_variants: Vec<&Ident> = Vec::new();
		for operation in &operations {
			if seen_variants.iter().any(|seen| **seen == operation.variant) {
				return Err(syn::Error::new(
					operation.name.span(),
					"two operations derive the same variant name; rename one (names are never suffixed implicitly)",
				));
			}
			seen_variants.push(&operation.variant);
		}
		for operation in &operations {
			let labelled_name = format!("{}_at", operation.name);
			if let Some(other) = operations.iter().find(|other| other.name == labelled_name) {
				return Err(syn::Error::new(
					other.name.span(),
					format!(
						"the operation name `{}` collides with the labelled constructor emitted for `{}`; rename one (names are never suffixed implicitly)",
						other.name, operation.name,
					),
				));
			}
		}
		for operation in &operations {
			if operation.resume.is_none() && operation.is_higher_order() {
				return Err(syn::Error::new(
					operation.name.span(),
					"no-resume (`-> !`) operations with sub-program payloads are not supported",
				));
			}
		}
		for operation in &operations {
			if operation.multi_shot && operation.is_higher_order() {
				return Err(syn::Error::new(
					operation.name.span(),
					"`#[multi_shot]` is rejected on higher-order operations: prompt finalization and multi-shot resumption conflict, so sub-program-owning operations stay single-shot",
				));
			}
		}

		Ok(EffectSpec {
			docs,
			handler_state,
			crate_path,
			vis,
			name,
			generics,
			operations,
		})
	}
}

/// Parses one `name: Type` payload argument.
fn parse_payload_argument(input: ParseStream) -> syn::Result<(Ident, Type)> {
	let name: Ident = input.parse()?;
	input.parse::<Token![:]>()?;
	let ty: Type = input.parse()?;
	Ok((name, ty))
}

/// Reports whether `stream` contains `target` as a standalone identifier
/// token, recursing into groups; the occurs-check that decides which generic
/// parameters an emitted handler-pieces item actually needs.
fn tokens_contain_ident(
	stream: &TokenStream,
	target: &Ident,
) -> bool {
	stream.clone().into_iter().any(|tree| match tree {
		TokenTree::Ident(ident) => ident == *target,
		TokenTree::Group(group) => tokens_contain_ident(&group.stream(), target),
		_ => false,
	})
}

/// Emits the effect definition for a parsed spec.
pub fn define_effect_worker(spec: EffectSpec) -> syn::Result<TokenStream> {
	let EffectSpec {
		docs,
		handler_state,
		crate_path: cp,
		vis,
		name,
		generics,
		operations,
	} = spec;

	let brand = format_ident!("{}Brand", name);
	let ops_enum = format_ident!("{}F", name);
	let higher_order = operations.iter().any(Operation::is_higher_order);
	let has_lifetime = operations.iter().any(|op| {
		op.resume.is_some()
			|| op.payloads.iter().any(|(_, kind)| matches!(kind, PayloadKind::Callable { .. }))
	});
	let uses_map_function = operations.iter().any(|op| op.resume.is_some());
	// An effect with a re-callable operation emits no handler pieces: the
	// one-pass `#[handlers]` surface is single-shot by construction (an arm
	// resumes exactly once), so these effects go to the narrowing runners,
	// and a `#[handlers]` row holding one fails on the missing
	// `HandlerPieces` bound.
	let has_multi_shot = operations.iter().any(|op| op.multi_shot);

	// The kind trait every projection references, named through the same
	// generator that produces it, so the emission cannot drift from the
	// trait `trait_kind!` defines.
	let kind_signature: AssociatedTypes = syn::parse_quote!(
		type Of<'a, T: 'a>: 'a;
	);
	let kind_trait = generate_name(&kind_signature)?;

	// Generic parameter fragments. `user_params` carries the spec bounds;
	// `user_idents` is the bare argument list.
	let user_params: Vec<&syn::TypeParam> = generics
		.params
		.iter()
		.filter_map(|param| match param {
			GenericParam::Type(type_param) => Some(type_param),
			_ => None,
		})
		.collect();
	let user_idents: Vec<&Ident> = user_params.iter().map(|param| &param.ident).collect();
	let row_param: Option<TokenStream> =
		higher_order.then(|| quote!(R: #cp::classes::WrapDrop + 'static));
	let row_ident: Option<TokenStream> = higher_order.then(|| quote!(R));

	// `<brand generic args>` and the brand type itself.
	let brand_args: Vec<TokenStream> =
		row_ident.iter().cloned().chain(user_idents.iter().map(|ident| quote!(#ident))).collect();
	let brand_ty =
		if brand_args.is_empty() { quote!(#brand) } else { quote!(#brand<#(#brand_args),*>) };
	let phantom_ty = match brand_args.as_slice() {
		[] => None,
		[only] => Some(quote!(#only)),
		_ => Some(quote!((#(#brand_args),*))),
	};
	let brand_struct = match &phantom_ty {
		Some(phantom) => {
			let bare_params: Vec<TokenStream> = row_ident
				.iter()
				.cloned()
				.chain(user_idents.iter().map(|ident| quote!(#ident)))
				.collect();
			quote!(#vis struct #brand<#(#bare_params),*>(::core::marker::PhantomData<#phantom>);)
		}
		None => quote!(#vis struct #brand;),
	};

	// Impl-position generics: the row bound plus the user bounds.
	let impl_params: Vec<TokenStream> =
		row_param.iter().cloned().chain(user_params.iter().map(|param| quote!(#param))).collect();
	let impl_generics =
		if impl_params.is_empty() { None } else { Some(quote!(<#(#impl_params),*>)) };

	// The operations enum.
	let lifetime: Option<TokenStream> = has_lifetime.then(|| quote!('a));
	let enum_params: Vec<TokenStream> = lifetime
		.iter()
		.cloned()
		.chain(impl_params.iter().cloned())
		.chain(std::iter::once(quote!(A)))
		.collect();
	let enum_args: Vec<TokenStream> = lifetime
		.iter()
		.cloned()
		.chain(brand_args.iter().cloned())
		.chain(std::iter::once(quote!(A)))
		.collect();
	let payload_field_ty = |kind: &PayloadKind| -> TokenStream {
		match kind {
			PayloadKind::Value(ty) => quote!(#ty),
			PayloadKind::Program(result) => quote!(#cp::types::Free<R, #result>),
			PayloadKind::Callable {
				inputs,
				output,
			} => {
				let output_ty = match output {
					CallableRet::Value(ty) => quote!(#ty),
					CallableRet::Program(result) => quote!(#cp::types::Free<R, #result>),
				};
				quote!(::std::boxed::Box<dyn FnOnce(#(#inputs),*) -> #output_ty + 'a>)
			}
		}
	};
	let variants: Vec<TokenStream> = operations
		.iter()
		.map(|op| {
			let op_docs = &op.docs;
			let variant = &op.variant;
			// Parsing rejects no-resume higher-order operations, so the
			// higher-order arm always sees a resume type; the tuple arm is
			// the graceful shape for everything else.
			match (op.is_higher_order(), &op.resume) {
				(true, Some(resume)) => {
					let fields: Vec<TokenStream> = op
						.payloads
						.iter()
						.map(|(field, kind)| {
							let field_ty = payload_field_ty(kind);
							// Named fields need docs (a public effect is under
							// `missing_docs`); the payload kind decides the noun.
							let field_doc = match kind {
								PayloadKind::Value(_) => format!(" The `{field}` payload."),
								PayloadKind::Program(_) => {
									format!(" The owned `{field}` sub-program.")
								}
								PayloadKind::Callable {
									..
								} => format!(" The `{field}` callable."),
							};
							quote! {
								#[doc = #field_doc]
								#field: #field_ty
							}
						})
						.collect();
					quote! {
						#(#op_docs)*
						#variant {
							#(#fields,)*
							#[doc = " The continuation, invoked with the operation's result."]
							k: ::std::boxed::Box<dyn FnOnce(#resume) -> A + 'a>,
						}
					}
				}
				(_, resume) => {
					let field_types: Vec<TokenStream> =
						op.payloads.iter().map(|(_, kind)| payload_field_ty(kind)).collect();
					let tail = match resume {
						Some(resume) if op.multi_shot => {
							quote!(::std::rc::Rc<dyn Fn(#resume) -> A + 'a>)
						}
						Some(resume) => quote!(::std::boxed::Box<dyn FnOnce(#resume) -> A + 'a>),
						None => quote!(::core::marker::PhantomData<A>),
					};
					quote! {
						#(#op_docs)*
						#variant(#(#field_types,)* #tail)
					}
				}
			}
		})
		.collect();
	let ops_enum_docs = format!(
		" The operations of [`{brand}`], one variant per smart constructor, with the continuation hole `A`.",
	);
	let ops_enum_def = quote! {
		#[doc = #ops_enum_docs]
		#vis enum #ops_enum<#(#enum_params),*> {
			#(#variants,)*
		}
	};

	// The kind projection, emitted through the same worker `impl_kind!` uses.
	let cell_ty = quote!(#ops_enum<#(#enum_args),*>);
	let cell_ty_static: TokenStream = if has_lifetime {
		let static_args: Vec<TokenStream> = std::iter::once(quote!('static))
			.chain(brand_args.iter().cloned())
			.chain(std::iter::once(quote!(A)))
			.collect();
		quote!(#ops_enum<#(#static_args),*>)
	} else {
		cell_ty.clone()
	};
	let impl_kind_input: ImplKindInput = syn::parse2(match &impl_generics {
		Some(impl_generics) => quote! {
			impl #impl_generics for #brand_ty {
				type Of<'a, A: 'a>: 'a = #cell_ty;
			}
		},
		None => quote! {
			impl for #brand_ty {
				type Of<'a, A: 'a>: 'a = #cell_ty;
			}
		},
	})?;
	// The worker emits the kind traits unqualified (its hand-written call
	// sites glob-import `kinds`); scoping the glob inside an anonymous const
	// keeps the emission self-contained at any invocation site.
	let kind_impl = impl_kind_worker(impl_kind_input)?;
	let kind_impl = quote! {
		const _: () = {
			use #cp::kinds::*;
			#kind_impl
		};
	};

	// The Functor instance: compose the mapped function into each
	// continuation; rebuild the phantom for no-resume variants.
	let map_function = if uses_map_function { format_ident!("f") } else { format_ident!("_f") };
	let functor_arms: Vec<TokenStream> = operations
		.iter()
		.map(|op| {
			let variant = &op.variant;
			if op.is_higher_order() {
				// Rebind each payload field to a positional binder, exactly as the
				// first-order arm below does, so no payload binding can be named `f`
				// and shadow the map-function parameter inside the arm (a payload
				// literally named `f`, as the built-in `Censor`'s transform is, would
				// otherwise capture `#map_function`'s name and be called in its place).
				let fields: Vec<&Ident> = op.payloads.iter().map(|(field, _)| field).collect();
				let binders: Vec<Ident> = op
					.payloads
					.iter()
					.enumerate()
					.map(|(index, _)| format_ident!("payload_{}", index))
					.collect();
				quote! {
					#ops_enum::#variant { #(#fields: #binders,)* k } => #ops_enum::#variant {
						#(#fields: #binders,)*
						k: ::std::boxed::Box::new(move |x| #map_function(k(x))),
					}
				}
			} else {
				let binders: Vec<Ident> = op
					.payloads
					.iter()
					.enumerate()
					.map(|(index, _)| format_ident!("payload_{}", index))
					.collect();
				match &op.resume {
					// The re-callable continuation composes by re-wrapping:
					// the mapped function must survive one call per branch,
					// so it moves into a fresh `Rc<dyn Fn>` around the old.
					Some(_) if op.multi_shot => quote! {
						#ops_enum::#variant(#(#binders,)* k) => #ops_enum::#variant(
							#(#binders,)*
							::std::rc::Rc::new(move |x| #map_function(k(x))),
						)
					},
					Some(_) => quote! {
						#ops_enum::#variant(#(#binders,)* k) => #ops_enum::#variant(
							#(#binders,)*
							::std::boxed::Box::new(move |x| #map_function(k(x))),
						)
					},
					None => quote! {
						#ops_enum::#variant(#(#binders,)* _) => #ops_enum::#variant(
							#(#binders,)*
							::core::marker::PhantomData,
						)
					},
				}
			}
		})
		.collect();
	let functor_impl = quote! {
		impl #impl_generics #cp::classes::Functor for #brand_ty {
			fn map<'a, A: 'a, B: 'a>(
				#map_function: impl Fn(A) -> B + 'a,
				fa: <Self as #cp::kinds::#kind_trait>::Of<'a, A>,
			) -> <Self as #cp::kinds::#kind_trait>::Of<'a, B> {
				match fa {
					#(#functor_arms,)*
				}
			}
		}
	};

	// The order marker, computed from the operation list.
	let order_marker = if higher_order {
		quote!(#cp::types::effects::order::HigherOrder)
	} else {
		quote!(#cp::types::effects::order::FirstOrder)
	};
	let order_impl = quote! {
		impl #impl_generics #cp::types::effects::order::OrderOf for #brand_ty {
			type Order = #order_marker;
		}
	};

	// Derived documentation appended to the brand.
	let order_doc = if higher_order {
		" Order: higher-order (an operation owns a sub-program; interpreters elaborate it, and native stack use grows with the nesting depth of higher-order cells, not with program length)."
	} else {
		" Order: first-order (no operation owns a sub-program)."
	};
	let handler_state_doc = format!(" Handler state: {}.", handler_state.prose());

	// The smart constructors, each with its labelled `<name>_at` variant: the
	// same cell construction annotated at `TaggedBrand<Label, Brand>` instead
	// of the bare brand (the tagged projection reuses the operations enum, so
	// only the `Coyoneda` pin and the injection target differ).
	let constructors: Vec<TokenStream> = operations
		.iter()
		.map(|op| {
			let op_docs = &op.docs;
			let constructor = &op.name;
			let variant = &op.variant;
			// First-order constructors are store-generic (an inferred `Store`
			// parameter, last); higher-order constructors keep the `Box`-store
			// default, because a higher-order cell pins `Box`-store types in
			// the operations enum itself (`Free<R, _>` sub-program payloads
			// and the `Box<dyn FnOnce>` continuation), so a store-generic
			// return would admit a program no interpreter serves: the
			// elaborating handler surface is `Box`-tier, and `#[multi_shot]`
			// is rejected on higher-order operations.
			let store_generic = !op.is_higher_order();
			let store_param: Option<TokenStream> = store_generic.then(|| quote!(Store));
			let value_for_bound = quote!(#cp::types::closure_storage::ValueFor<Store>);
			// The continuation hole: the resume type, or a fresh `T` for a
			// no-resume operation (the program never continues, so the
			// constructor is polymorphic in its result). A no-resume operation
			// is first-order by parsing, so its fresh `T` is always on a
			// store-generic constructor and carries the store's value bound
			// inline.
			let (hole, hole_param): (TokenStream, Option<TokenStream>) = match &op.resume {
				Some(resume) => (quote!(#resume), None),
				None => (quote!(T), Some(quote!(T: 'static + #value_for_bound))),
			};
			// The hole's `ValueFor<Store>` bound sits inline when the hole is
			// itself one of the constructor's generic parameters (the fresh
			// `T` above, or a user parameter named as the resume type): those
			// parameters already carry inline bounds, and re-bounding one in
			// the where clause trips `clippy::multiple_bound_locations`.
			// Compound and concrete resume types take the where clause, which
			// re-bounds no parameter.
			let hole_user_ident: Option<&Ident> = match &op.resume {
				Some(Type::Path(type_path)) if type_path.qself.is_none() => {
					type_path.path.get_ident().filter(|ident| user_idents.contains(ident))
				}
				_ => None,
			};
			let (return_ty, store_bounds): (TokenStream, Option<TokenStream>) = if store_generic {
				let hole_where_bound = (op.resume.is_some() && hole_user_ident.is_none())
					.then(|| quote!(#hole: #value_for_bound,));
				(
					quote!(#cp::types::Free<R, #hole, Store>),
					Some(quote! {
						Store: #cp::types::closure_storage::ClosureStorage,
						#hole_where_bound
					}),
				)
			} else {
				(quote!(#cp::types::Free<R, #hole>), None)
			};
			let constructor_params: Vec<TokenStream> = user_params
				.iter()
				.map(|param| {
					if store_generic
						&& hole_user_ident.is_some_and(|hole_ident| *hole_ident == param.ident)
					{
						let mut bounded = (*param).clone();
						bounded.bounds.push(syn::parse_quote!(#value_for_bound));
						quote!(#bounded)
					} else {
						quote!(#param)
					}
				})
				.chain(hole_param)
				.chain([quote!(R), quote!(I)])
				.chain(store_param)
				.collect();
			let arguments: Vec<TokenStream> = op
				.payloads
				.iter()
				.map(|(field, kind)| match kind {
					PayloadKind::Value(ty) => quote!(#field: #ty),
					PayloadKind::Program(result) => quote!(#field: #cp::types::Free<R, #result>),
					PayloadKind::Callable {
						inputs,
						output,
					} => {
						let output_ty = match output {
							CallableRet::Value(ty) => quote!(#ty),
							CallableRet::Program(result) => quote!(#cp::types::Free<R, #result>),
						};
						quote!(#field: impl FnOnce(#(#inputs),*) -> #output_ty + 'static)
					}
				})
				.collect();
			let cell_expr = if op.is_higher_order() {
				let fields: Vec<TokenStream> = op
					.payloads
					.iter()
					.map(|(field, kind)| match kind {
						PayloadKind::Callable {
							..
						} => quote!(#field: ::std::boxed::Box::new(#field)),
						_ => quote!(#field),
					})
					.collect();
				quote! {
					#ops_enum::#variant {
						#(#fields,)*
						k: ::std::boxed::Box::new(|x| x),
					}
				}
			} else {
				let field_exprs: Vec<TokenStream> = op
					.payloads
					.iter()
					.map(|(field, kind)| match kind {
						PayloadKind::Callable {
							..
						} => quote!(::std::boxed::Box::new(#field)),
						_ => quote!(#field),
					})
					.collect();
				let tail = match &op.resume {
					Some(_) if op.multi_shot => quote!(::std::rc::Rc::new(|x| x)),
					Some(_) => quote!(::std::boxed::Box::new(|x| x)),
					None => quote!(::core::marker::PhantomData),
				};
				quote!(#ops_enum::#variant(#(#field_exprs,)* #tail))
			};
			let cell_annotation: TokenStream = if has_lifetime {
				let static_args: Vec<TokenStream> = std::iter::once(quote!('static))
					.chain(brand_args.iter().cloned())
					.chain(std::iter::once(hole.clone()))
					.collect();
				quote!(#ops_enum<#(#static_args),*>)
			} else {
				let args: Vec<TokenStream> =
					brand_args.iter().cloned().chain(std::iter::once(hole.clone())).collect();
				quote!(#ops_enum<#(#args),*>)
			};
			let bare = quote! {
				#(#op_docs)*
				#vis fn #constructor<#(#constructor_params),*>(
					#(#arguments),*
				) -> #return_ty
				where
					R: #cp::classes::Functor + #cp::classes::WrapDrop + 'static,
					<R as #cp::kinds::#kind_trait>::Of<'static, #hole>:
						#cp::types::effects::coproduct::CoprodInjector<
							#cp::types::Coyoneda<'static, #brand_ty, #hole>,
							I,
						>,
					#store_bounds
				{
					let cell: #cell_annotation = #cell_expr;
					let coyo: #cp::types::Coyoneda<'static, #brand_ty, #hole> =
						#cp::types::Coyoneda::lift(cell);
					let node: <R as #cp::kinds::#kind_trait>::Of<'static, #hole> =
						#cp::types::effects::coproduct::CoprodInjector::inject(coyo);
					#cp::types::Free::lift_f(node)
				}
			};
			let at_constructor = format_ident!("{}_at", constructor, span = constructor.span());
			let at_doc = format!(
				" Labelled variant of [`{constructor}`]: injects at the cell tagged `Label`, so a row holding the effect under several labels addresses this one specifically.",
			);
			let at_params: Vec<TokenStream> = std::iter::once(quote!(Label: 'static))
				.chain(constructor_params.iter().cloned())
				.collect();
			let tagged_brand_ty =
				quote!(#cp::types::effects::tagged::TaggedBrand<Label, #brand_ty>);
			let labelled = quote! {
				#(#op_docs)*
				#[doc = ""]
				#[doc = #at_doc]
				#vis fn #at_constructor<#(#at_params),*>(
					#(#arguments),*
				) -> #return_ty
				where
					R: #cp::classes::Functor + #cp::classes::WrapDrop + 'static,
					<R as #cp::kinds::#kind_trait>::Of<'static, #hole>:
						#cp::types::effects::coproduct::CoprodInjector<
							#cp::types::Coyoneda<'static, #tagged_brand_ty, #hole>,
							I,
						>,
					#store_bounds
				{
					let cell: #cell_annotation = #cell_expr;
					let coyo: #cp::types::Coyoneda<'static, #tagged_brand_ty, #hole> =
						#cp::types::Coyoneda::lift(cell);
					let node: <R as #cp::kinds::#kind_trait>::Of<'static, #hole> =
						#cp::types::effects::coproduct::CoprodInjector::inject(coyo);
					#cp::types::Free::lift_f(node)
				}
			};
			quote! {
				#bare
				#labelled
			}
		})
		.collect();

	// ---- The handler pieces (the arm grammar over the trait seam) ----
	// Every arm's type mentions only the effect's own pinned types, never the
	// program result: first-order resumptive operations get
	// payloads-to-resume-value closures (dispatch applies the continuation),
	// no-resume operations get no arm and reify into the per-effect abort
	// type, and higher-order operations get elaboration arms receiving their
	// owned sub-programs plus re-entry handles built over the row handler.
	// The reserved parameter names `R` (the row) and `B` (the row abort)
	// carry the row-level types through the emitted impls.
	let arms_name = format_ident!("{}Arms", name);
	let abort_name = format_ident!("{}Abort", name);
	let handle_path = quote!(#cp::types::effects::handle);
	// The payload as dispatch hands it to an arm: the cell's stored shape at
	// the `'static` instantiation.
	let payload_arm_ty = |kind: &PayloadKind| -> TokenStream {
		match kind {
			PayloadKind::Value(ty) => quote!(#ty),
			PayloadKind::Program(result) => quote!(#cp::types::Free<R, #result>),
			PayloadKind::Callable {
				inputs,
				output,
			} => {
				let output_ty = match output {
					CallableRet::Value(ty) => quote!(#ty),
					CallableRet::Program(result) => quote!(#cp::types::Free<R, #result>),
				};
				quote!(::std::boxed::Box<dyn FnOnce(#(#inputs),*) -> #output_ty + 'static>)
			}
		}
	};
	// The distinct sub-program result types an operation owns, in first
	// appearance order: one re-entry handle per pin.
	let op_pins = |op: &Operation| -> Vec<TokenStream> {
		let mut pins: Vec<TokenStream> = Vec::new();
		for (_, kind) in &op.payloads {
			let pin = match kind {
				PayloadKind::Program(result) => Some(quote!(#result)),
				PayloadKind::Callable {
					output: CallableRet::Program(result), ..
				} => Some(quote!(#result)),
				_ => None,
			};
			let Some(pin) = pin else {
				continue;
			};
			if !pins.iter().any(|existing| existing.to_string() == pin.to_string()) {
				pins.push(pin);
			}
		}
		pins
	};
	let resumptive: Vec<&Operation> = operations.iter().filter(|op| op.resume.is_some()).collect();
	let aborting: Vec<&Operation> = operations.iter().filter(|op| op.resume.is_none()).collect();
	let arm_field_tys: Vec<TokenStream> = resumptive
		.iter()
		.map(|op| {
			let resume = &op.resume;
			let payload_tys: Vec<TokenStream> =
				op.payloads.iter().map(|(_, kind)| payload_arm_ty(kind)).collect();
			if op.is_higher_order() {
				let retries: Vec<TokenStream> = op_pins(op)
					.iter()
					.map(
						|pin| quote!(&dyn Fn(#cp::types::Free<R, #pin>) -> ::core::result::Result<#pin, B>),
					)
					.collect();
				quote! {
					::std::boxed::Box<
						dyn Fn(#(#payload_tys,)* #(#retries,)*) -> ::core::result::Result<#resume, B> + 'h,
					>
				}
			} else {
				quote!(::std::boxed::Box<dyn Fn(#(#payload_tys),*) -> #resume + 'h>)
			}
		})
		.collect();
	// The generic parameters each emitted item actually needs, by occurrence.
	let r_ident = format_ident!("R");
	let b_ident = format_ident!("B");
	let arms_fields_stream: TokenStream = arm_field_tys.iter().cloned().collect();
	let arms_has_fields = !arm_field_tys.is_empty();
	let arms_uses_r = tokens_contain_ident(&arms_fields_stream, &r_ident);
	let arms_uses_b = tokens_contain_ident(&arms_fields_stream, &b_ident);
	let arms_user: Vec<&&syn::TypeParam> = user_params
		.iter()
		.filter(|param| tokens_contain_ident(&arms_fields_stream, &param.ident))
		.collect();
	let arms_params: Vec<TokenStream> = arms_has_fields
		.then(|| quote!('h))
		.into_iter()
		.chain(arms_uses_r.then(|| quote!(R: #cp::classes::WrapDrop + 'static)))
		.chain(arms_user.iter().map(|param| quote!(#param)))
		.chain(arms_uses_b.then(|| quote!(B)))
		.collect();
	let arms_args: Vec<TokenStream> = arms_has_fields
		.then(|| quote!('h))
		.into_iter()
		.chain(arms_uses_r.then(|| quote!(R)))
		.chain(arms_user.iter().map(|param| {
			let ident = &param.ident;
			quote!(#ident)
		}))
		.chain(arms_uses_b.then(|| quote!(B)))
		.collect();
	let arms_ty_gat =
		if arms_args.is_empty() { quote!(#arms_name) } else { quote!(#arms_name<#(#arms_args),*>) };
	let arms_def = if arms_has_fields {
		let arms_doc = format!(
			" The arm bundle for [`{brand}`]: one field per resumptive operation, consumed by the emitted dispatch; `'h` is the lifetime of any handler state the arms borrow.",
		);
		let arms_fields: Vec<TokenStream> = resumptive
			.iter()
			.zip(&arm_field_tys)
			.map(|(op, ty)| {
				let op_docs = &op.docs;
				let field = &op.name;
				quote! {
					#(#op_docs)*
					#vis #field: #ty
				}
			})
			.collect();
		quote! {
			#[doc = #arms_doc]
			#vis struct #arms_name<#(#arms_params),*> {
				#(#arms_fields,)*
			}
		}
	} else {
		let arms_doc = format!(
			" The arm bundle for [`{brand}`]: no operation resumes, so there is nothing to store and the bundle is empty.",
		);
		quote! {
			#[doc = #arms_doc]
			#vis struct #arms_name;
		}
	};
	let abort_payload_stream: TokenStream = aborting
		.iter()
		.flat_map(|op| op.payloads.iter().map(|(_, kind)| payload_arm_ty(kind)))
		.collect();
	let abort_user: Vec<&&syn::TypeParam> = user_params
		.iter()
		.filter(|param| tokens_contain_ident(&abort_payload_stream, &param.ident))
		.collect();
	let abort_args: Vec<TokenStream> = abort_user
		.iter()
		.map(|param| {
			let ident = &param.ident;
			quote!(#ident)
		})
		.collect();
	let abort_ty = if abort_args.is_empty() {
		quote!(#abort_name)
	} else {
		quote!(#abort_name<#(#abort_args),*>)
	};
	let abort_def = {
		let abort_doc = if aborting.is_empty() {
			format!(
				" The abort type for [`{brand}`]: every operation resumes, so this is uninhabited and the effect contributes nothing to a row's abort union.",
			)
		} else {
			format!(
				" The abort type for [`{brand}`]: one variant per no-resume operation, carrying its payloads.",
			)
		};
		let abort_variants: Vec<TokenStream> = aborting
			.iter()
			.map(|op| {
				let op_docs = &op.docs;
				let variant = &op.variant;
				let payload_tys: Vec<TokenStream> =
					op.payloads.iter().map(|(_, kind)| payload_arm_ty(kind)).collect();
				if payload_tys.is_empty() {
					quote! {
						#(#op_docs)*
						#variant
					}
				} else {
					quote! {
						#(#op_docs)*
						#variant(#(#payload_tys),*)
					}
				}
			})
			.collect();
		let abort_params: Vec<TokenStream> =
			abort_user.iter().map(|param| quote!(#param)).collect();
		if abort_params.is_empty() {
			quote! {
				#[doc = #abort_doc]
				#vis enum #abort_name {
					#(#abort_variants,)*
				}
			}
		} else {
			quote! {
				#[doc = #abort_doc]
				#vis enum #abort_name<#(#abort_params),*> {
					#(#abort_variants,)*
				}
			}
		}
	};
	let effect_abort_impl = quote! {
		impl #impl_generics #handle_path::EffectAbort for #brand_ty {
			type Abort = #abort_ty;
		}
	};
	let pieces_params: Vec<TokenStream> =
		std::iter::once(quote!(R: #cp::classes::WrapDrop + 'static))
			.chain(user_params.iter().map(|param| quote!(#param)))
			.chain(std::iter::once(quote!(B)))
			.collect();
	let dispatch_arms: Vec<TokenStream> = operations
		.iter()
		.map(|op| {
			let variant = &op.variant;
			let binders: Vec<Ident> =
				(0 .. op.payloads.len()).map(|index| format_ident!("payload_{}", index)).collect();
			if op.is_higher_order() {
				let fields: Vec<&Ident> = op.payloads.iter().map(|(field, _)| field).collect();
				let op_name = &op.name;
				let pins = op_pins(op);
				let retry_names: Vec<Ident> =
					(0 .. pins.len()).map(|index| format_ident!("reenter_{}", index)).collect();
				quote! {
					#ops_enum::#variant { #(#fields: #binders,)* k } => {
						#(let #retry_names = |sub: #cp::types::Free<R, #pins>|
							#handle_path::RowHandler::handle(handler, sub);)*
						let value = (arms.#op_name)(#(#binders,)* #(&#retry_names,)*)?;
						::core::result::Result::Ok(k(value))
					}
				}
			} else {
				match &op.resume {
					Some(_) => {
						let op_name = &op.name;
						quote! {
							#ops_enum::#variant(#(#binders,)* k) =>
								::core::result::Result::Ok(k((arms.#op_name)(#(#binders),*)))
						}
					}
					None => {
						let abort_ctor = if abort_args.is_empty() {
							quote!(#abort_name::#variant)
						} else {
							quote!(#abort_name::<#(#abort_args),*>::#variant)
						};
						let ctor_expr = if binders.is_empty() {
							quote!(#abort_ctor)
						} else {
							quote!(#abort_ctor(#(#binders),*))
						};
						quote! {
							#ops_enum::#variant(#(#binders,)* _) =>
								::core::result::Result::Err(inject_abort(#ctor_expr))
						}
					}
				}
			}
		})
		.collect();
	let arms_param_name =
		if resumptive.is_empty() { format_ident!("_arms") } else { format_ident!("arms") };
	let handler_param_name = if operations.iter().any(Operation::is_higher_order) {
		format_ident!("handler")
	} else {
		format_ident!("_handler")
	};
	let inject_param_name = if aborting.is_empty() {
		format_ident!("_inject_abort")
	} else {
		format_ident!("inject_abort")
	};
	let pieces_impl = quote! {
		impl<#(#pieces_params),*> #handle_path::HandlerPieces<R, B> for #brand_ty {
			type Arms<'h> = #arms_ty_gat;

			fn dispatch<A: 'static>(
				op: <Self as #cp::kinds::#kind_trait>::Of<'static, #cp::types::Free<R, A>>,
				#arms_param_name: &Self::Arms<'_>,
				#handler_param_name: &impl #handle_path::RowHandler<R, B>,
				#inject_param_name: impl Fn(<Self as #handle_path::EffectAbort>::Abort) -> B,
			) -> ::core::result::Result<#cp::types::Free<R, A>, B> {
				match op {
					#(#dispatch_arms,)*
				}
			}
		}
	};

	// The handler surface is withheld for effects with a re-callable
	// operation (see `has_multi_shot` above); everything else emits it.
	let handler_surface = if has_multi_shot {
		quote!()
	} else {
		quote! {
			#arms_def

			#abort_def

			#effect_abort_impl

			#pieces_impl
		}
	};

	// `cell_ty_static` documents the emitted continuation-hole shape in the
	// brand docs so readers see the concrete cell a row stores.
	let cell_doc = format!(" Row cell: `Coyoneda` over `{}`.", quote!(#cell_ty_static));

	Ok(quote! {
		#(#docs)*
		#[doc = ""]
		#[doc = #order_doc]
		#[doc = #handler_state_doc]
		#[doc = #cell_doc]
		#brand_struct

		#ops_enum_def

		#kind_impl

		#functor_impl

		#order_impl

		#handler_surface

		#(#constructors)*
	})
}
