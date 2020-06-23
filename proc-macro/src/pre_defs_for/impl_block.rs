//! Deals with parsing and rendering the `impl` blocks in `pre_defs_for` modules.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, TokenStreamExt};
use std::mem;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    AngleBracketedGenericArguments, FnArg, ForeignItemFn, GenericArgument, GenericParam, Generics,
    Ident, LifetimeDef, PatIdent, PatType, Path, PathArguments, PathSegment, Signature, Token,
    Type, TypeParam, TypePath, TypeReference,
};

use super::helpers::{find_new_ident, replace_idents, replace_types};

/// An impl block in a `pre_defs_for` module.
pub(crate) struct ImplBlock {
    /// The impl keyword.
    impl_keyword: Token![impl],
    /// The generics for the impl block.
    generics: Generics,
    /// The type which the impl block is for.
    self_type: Box<Type>,
    /// The brace of the block.
    brace: Brace,
    /// The functions which the block applies to.
    functions: Vec<ForeignItemFn>,
}

impl Parse for ImplBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let impl_keyword = input.parse()?;
        let generics = input.parse()?;
        let self_type = input.parse()?;
        let where_clause = input.parse()?;
        let content;
        let brace = braced!(content in input);

        let mut functions = Vec::new();

        while !content.is_empty() {
            functions.push(content.parse()?);
        }

        Ok(ImplBlock {
            impl_keyword,
            generics: Generics {
                where_clause,
                ..generics
            },
            self_type,
            brace,
            functions,
        })
    }
}

impl Spanned for ImplBlock {
    fn span(&self) -> Span {
        self.impl_keyword
            .span()
            .join(self.brace.span)
            .unwrap_or_else(|| self.impl_keyword.span())
    }
}

impl ImplBlock {
    /// Generates a token stream that is semantically equivalent to the original token stream.
    ///
    /// This should only be used for debug purposes.
    pub(crate) fn original_token_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        let impl_keyword = &self.impl_keyword;
        tokens.append_all(quote! { #impl_keyword });
        let generics = &self.generics;
        tokens.append_all(quote! { #generics });
        let self_type = &self.self_type;
        tokens.append_all(quote! { #self_type });
        let where_clause = &generics.where_clause;
        tokens.append_all(quote! { #where_clause });

        let mut items = TokenStream::new();
        items.append_all(&self.functions);
        tokens.append_all(quote! { { #items } });

        tokens
    }

    /// Returns the type that this impl block is for.
    pub(crate) fn ty(&self) -> Option<&PathSegment> {
        if let Type::Path(path) = &*self.self_type {
            if path.path.segments.len() != 1 {
                emit_error!(path, "only paths of length 1 are supported here");
                return None;
            }

            if let Some(qself) = &path.qself {
                emit_error!(
                    qself
                        .lt_token
                        .span()
                        .join(qself.gt_token.span())
                        .unwrap_or_else(|| qself.ty.span()),
                    "qualified paths are not supported here"
                );
                return None;
            }

            let ty = &path.path.segments[0];

            if matches!(ty.arguments, PathArguments::Parenthesized(_)) {
                emit_error!(
                    ty.arguments.span(),
                    "parenthesized type arguments are not supported here"
                );

                None
            } else {
                Some(ty)
            }
        } else {
            emit_error!(
                self.self_type.span(),
                "`impl` block are only supported for structs, enums and unions in this context"
            );

            None
        }
    }

    /// Generates the code for an impl block inside a `pre_defs_for` module.
    pub(crate) fn render(&self, tokens: &mut TokenStream, path: &Path, visibility: &TokenStream) {
        let self_type = if let Some(self_type) = self.ty() {
            self_type
        } else {
            return;
        };

        // First generate a dummy struct that the impl block will be attached to.
        self.render_struct(tokens, self_type, visibility);

        let mut impl_functions = TokenStream::new();
        for function in self.functions.iter() {
            render_impl_fn(function, &mut impl_functions, &self_type, path, visibility);
        }

        let impl_keyword = &self.impl_keyword;
        let generic_params = &self.generics.params;
        let self_type = &self.self_type;
        let where_clause = &self.generics.where_clause;

        tokens.append_all(quote! {
            #impl_keyword <#generic_params> #self_type #where_clause {
                #impl_functions
            }
        });
    }

    /// Renders the struct that the impl block will be for.
    fn render_struct(
        &self,
        tokens: &mut TokenStream,
        self_type: &PathSegment,
        visibility: &TokenStream,
    ) {
        let struct_params =
            if let PathArguments::AngleBracketed(arguments) = self_type.arguments.clone() {
                let params: Punctuated<_, Token![,]> = arguments
                    .args
                    .into_iter()
                    .enumerate()
                    .filter_map(to_generic_param)
                    .collect();

                Some(params)
            } else {
                None
            };

        tokens.append_all(quote_spanned! { self.span()=> #[allow(dead_code)] });
        tokens.append_all(visibility.clone().into_iter().map(|mut token| {
            token.set_span(self.span());
            token
        }));

        let name = &self_type.ident;
        if let Some(struct_params) = struct_params {
            // Ensure that all generic parameters are used in the struct definition.
            let mut struct_contents: Punctuated<TokenStream, Token![,]> = struct_params
                .iter()
                .filter_map(|arg| match arg {
                    GenericParam::Lifetime(lifetime) => {
                        Some(quote_spanned! { lifetime.span()=> &#lifetime () })
                    }
                    GenericParam::Type(ty) => Some(quote_spanned! { ty.span()=> #ty }),
                    GenericParam::Const(_) => None,
                })
                .collect();

            // Ensure that it's always a tuple type, even if there is only one element.
            if !struct_contents.empty_or_trailing() {
                struct_contents.push_punct(Default::default());
            }

            tokens.append_all(quote_spanned! { self.span()=>
                struct #name <#struct_params>(::core::marker::PhantomData<(#struct_contents)>);
            });
        } else {
            tokens.append_all(quote_spanned! { self.span()=>
                struct #name;
            });
        }
    }
}

/// Converts a generic argument to a generic parameter if possible.
fn to_generic_param((arg_index, argument): (usize, GenericArgument)) -> Option<GenericParam> {
    match argument.clone() {
        GenericArgument::Lifetime(lifetime) => Some(
            LifetimeDef {
                attrs: Vec::new(),
                lifetime,
                colon_token: None,
                bounds: Punctuated::new(),
            }
            .into(),
        ),
        GenericArgument::Type(ty) => Some(
            TypeParam {
                attrs: Vec::new(),
                ident: Ident::new(&format!("T{}", arg_index), ty.span()),
                colon_token: None,
                bounds: Punctuated::new(),
                eq_token: None,
                default: None,
            }
            .into(),
        ),
        GenericArgument::Constraint(ty) => Some(
            TypeParam {
                attrs: Vec::new(),
                ident: Ident::new(&format!("T{}", arg_index), ty.span()),
                colon_token: None,
                bounds: Punctuated::new(),
                eq_token: None,
                default: None,
            }
            .into(),
        ),
        GenericArgument::Const(expr) => {
            emit_error!(
                expr,
                "const generics are currently not supported in this context"
            );
            None
        }
        GenericArgument::Binding(binding) => {
            emit_error!(binding, "type bindings are not supported in this context");
            None
        }
    }
}

/// Renders a single function inside the impl block.
fn render_impl_fn(
    function: &ForeignItemFn,
    tokens: &mut TokenStream,
    self_type: &PathSegment,
    path: &Path,
    visibility: &TokenStream,
) {
    let mut function = function.clone();

    to_function(&mut function.sig);

    let mut self_type = self_type.clone();
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
        ref mut colon2_token,
        ..
    }) = &mut self_type.arguments
    {
        colon2_token.get_or_insert(Default::default());
    }

    let mut path = path.clone();
    path.segments.push(self_type.into());

    replace_types(&mut function.sig, |path| path.path.is_ident("Self"), &path);

    super::render_function(&function, tokens, &path, visibility);
}

/// Turns methods into equivalent functions, that can only be called using function call syntax.
///
/// # Example
///
/// ```rust,ignore
/// unsafe fn do_stuff(&mut self, complex_self_type: Box<(Self, Self)>, _self: Self);
/// ```
/// turns into
///
/// ```rust,ignore
/// unsafe fn do_stuff(__self: &mut Self, complex_self_type: Box<(Self, Self)>, _self: Self);
/// ```
fn to_function(signature: &mut Signature) {
    if let Some(receiver) = signature.receiver() {
        let new_ident = find_new_ident(signature, Some(receiver.span()));

        if let Some(first_arg) = signature.inputs.first_mut() {
            if let FnArg::Receiver(receiver) = first_arg.clone() {
                let span = receiver.span();

                #[allow(non_snake_case)]
                let Self_type = TypePath {
                    path: Ident::new("Self", span).into(),
                    qself: None,
                }
                .into();

                let ty = if let Some((and_token, lifetime)) = receiver.reference {
                    TypeReference {
                        and_token,
                        lifetime,
                        mutability: receiver.mutability,
                        elem: Box::new(Self_type),
                    }
                    .into()
                } else {
                    Self_type
                };

                let pat_type = PatType {
                    attrs: receiver.attrs,
                    pat: Box::new(
                        PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("self", span),
                            subpat: None,
                        }
                        .into(),
                    ),
                    colon_token: Default::default(),
                    ty: Box::new(ty),
                };

                mem::swap(first_arg, &mut FnArg::Typed(pat_type));
            }

            match first_arg {
                FnArg::Typed(arg) => {
                    replace_idents(&mut arg.pat, |ident| ident == "self", &new_ident)
                }
                FnArg::Receiver(_) => unreachable!("receiver arguments were replaced above"),
            }
        }
    }
}
