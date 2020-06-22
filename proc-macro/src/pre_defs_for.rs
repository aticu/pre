//! Provides handling of `pre_defs_for` attributes.
//!
//! # What the generated code looks like
//!
//! ```rust,ignore
//! #[pre::pre_defs_for(std)]
//! mod pre_std {
//!     mod ptr {
//!         #[pre(valid_ptr(src, r))]
//!         unsafe fn read<T>(src: *const T) -> T;
//!     }
//! }
//! ```
//!
//! turns into
//!
//! ```rust,ignore
//! mod pre_std {
//!     #[allow(unused_imports)]
//!     use pre::pre;
//!     #[allow(unused_imports)]
//!     use std::*;
//!
//!     pub(crate) mod ptr {
//!         #[allow(unused_imports)]
//!         use pre::pre;
//!         #[allow(unused_imports)]
//!         use std::ptr::*;
//!
//!         #[pre(valid_ptr(src, r))]
//!         #[inline(always)]
//!         pub(crate) unsafe fn read<T>(src: *const T) -> T {
//!             std::ptr::read(src)
//!         }
//!     }
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, TokenStreamExt};
use std::fmt;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    Attribute, FnArg, ForeignItemFn, GenericArgument, GenericParam, Generics, Ident, ItemUse,
    LifetimeDef, Path, PathArguments, PathSegment, Token, Type, TypeParam, Visibility,
};

use crate::helpers::crate_name;

/// The parsed version of the `pre_defs_for` attribute content.
pub(crate) struct DefinitionsForAttr {
    /// The path of the crate/module to which function calls will be forwarded.
    path: Path,
}

impl fmt::Display for DefinitionsForAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#[pre_defs_for(")?;

        if self.path.leading_colon.is_some() {
            write!(f, "::")?;
        }

        for segment in &self.path.segments {
            write!(f, "{}", segment.ident)?;
        }

        write!(f, ")]")
    }
}

impl Parse for DefinitionsForAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DefinitionsForAttr {
            path: input.call(Path::parse_mod_style)?,
        })
    }
}

/// An impl block in a `pre_defs_for` module.
pub(crate) struct DefinitionsForImplBlock {
    /// The impl keyword.
    impl_keyword: Token![impl],
    /// The generics for the impl block.
    generics: Generics,
    /// The type which the impl block is for.
    self_ty: Box<Type>,
    /// The brace of the block.
    brace: Brace,
    /// The functions which the block applies to.
    items: Vec<ForeignItemFn>,
}

impl Parse for DefinitionsForImplBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let impl_keyword = input.parse()?;
        let generics = input.parse()?;
        let self_ty = input.parse()?;
        let where_clause = input.parse()?;
        let content;
        let brace = braced!(content in input);

        let mut items = Vec::new();

        while !content.is_empty() {
            items.push(content.parse()?);
        }

        Ok(DefinitionsForImplBlock {
            impl_keyword,
            generics: Generics {
                where_clause,
                ..generics
            },
            self_ty,
            brace,
            items,
        })
    }
}

impl Spanned for DefinitionsForImplBlock {
    fn span(&self) -> Span {
        self.impl_keyword
            .span()
            .join(self.brace.span)
            .unwrap_or_else(|| self.impl_keyword.span())
    }
}

impl DefinitionsForImplBlock {
    /// Generates a token stream that is semantically equivalent to the original token stream.
    ///
    /// This should only be used for debug purposes.
    fn original_token_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        let impl_keyword = &self.impl_keyword;
        tokens.append_all(quote! { #impl_keyword });
        let generics = &self.generics;
        tokens.append_all(quote! { #generics });
        let self_ty = &self.self_ty;
        tokens.append_all(quote! { #self_ty });
        let where_clause = &generics.where_clause;
        tokens.append_all(quote! { #where_clause });

        let mut items = TokenStream::new();
        items.append_all(&self.items);
        tokens.append_all(quote! { { #items } });

        tokens
    }
}

/// A parsed `pre_defs_for` annotated module.
pub(crate) struct DefinitionsForModule {
    /// The attributes on the module.
    attrs: Vec<Attribute>,
    /// The visibility on the module.
    visibility: Visibility,
    /// The `mod` token.
    mod_token: Token![mod],
    /// The name of the module.
    ident: Ident,
    /// The braces surrounding the content.
    braces: Brace,
    /// The impl blocks contained in the module.
    impl_blocks: Vec<DefinitionsForImplBlock>,
    /// The imports contained in the module.
    imports: Vec<ItemUse>,
    /// The functions contained in the module.
    functions: Vec<ForeignItemFn>,
    /// The submodules contained in the module.
    modules: Vec<DefinitionsForModule>,
}

impl fmt::Display for DefinitionsForModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.original_token_stream())
    }
}

impl Parse for DefinitionsForModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let mod_token = input.parse()?;
        let ident = input.parse()?;

        let content;
        let braces = braced!(content in input);
        let mut impl_blocks = Vec::new();
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut modules = Vec::new();

        while !content.is_empty() {
            if content.peek(Token![impl]) {
                impl_blocks.push(content.parse()?);
            } else if <ItemUse as Parse>::parse(&content.fork()).is_ok() {
                imports.push(content.parse()?);
            } else if <ForeignItemFn as Parse>::parse(&content.fork()).is_ok() {
                functions.push(content.parse()?);
            } else {
                modules.push(content.parse().map_err(|err| {
                    syn::Error::new(
                        err.span(),
                        "expected a module, a function signature, an impl block or a use statement",
                    )
                })?);
            }
        }

        Ok(DefinitionsForModule {
            attrs,
            visibility,
            mod_token,
            ident,
            braces,
            impl_blocks,
            imports,
            functions,
            modules,
        })
    }
}

impl DefinitionsForModule {
    /// Renders this `pre_defs_for` annotated module to its final result.
    pub(crate) fn render(&self, attr: DefinitionsForAttr) -> TokenStream {
        let mut tokens = TokenStream::new();
        let crate_name = crate_name();

        self.render_inner(attr.path, &mut tokens, None, &crate_name);

        tokens
    }

    /// A helper function to generate the final token stream.
    ///
    /// This allows passing the top level visibility and the updated path into recursive calls.
    fn render_inner(
        &self,
        mut path: Path,
        tokens: &mut TokenStream,
        visibility: Option<&TokenStream>,
        crate_name: &Ident,
    ) {
        tokens.append_all(&self.attrs);

        if visibility.is_some() {
            // Update the path only in recursive calls.
            path.segments.push(PathSegment {
                ident: self.ident.clone(),
                arguments: PathArguments::None,
            });
        }

        let visibility = if let Some(visibility) = visibility {
            // We're in a recursive call.
            // Use the visibility passed to us.
            tokens.append_all(quote! { #visibility });

            visibility.clone()
        } else {
            // We're in the outermost call.
            // Use the original visibility and decide which visibility to use in recursive calls.
            let local_vis = &self.visibility;
            tokens.append_all(quote! { #local_vis });

            if let Visibility::Public(pub_keyword) = local_vis {
                quote! { #pub_keyword }
            } else {
                let span = match local_vis {
                    Visibility::Inherited => self.mod_token.span(),
                    _ => local_vis.span(),
                };
                quote_spanned! { span=> pub(crate) }
            }
        };

        let mod_token = self.mod_token;
        tokens.append_all(quote! { #mod_token });

        tokens.append(self.ident.clone());

        let mut brace_content = TokenStream::new();

        brace_content.append_all(quote! {
            #[allow(unused_imports)]
            use #path::*;

            #[allow(unused_imports)]
            use #crate_name::pre;
        });

        for impl_block in &self.impl_blocks {
            render_impl_block(&path, impl_block, &mut brace_content, &visibility);
        }

        for import in &self.imports {
            brace_content.append_all(quote! { #import });
        }

        for function in &self.functions {
            render_function(&path, function, &mut brace_content, &visibility);
        }

        for module in &self.modules {
            module.render_inner(
                path.clone(),
                &mut brace_content,
                Some(&visibility),
                crate_name,
            );
        }

        tokens.append_all(quote_spanned! { self.braces.span=> { #brace_content } });
    }

    /// Generates a token stream that is semantically equivalent to the original token stream.
    ///
    /// This should only be used for debug purposes.
    fn original_token_stream(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        stream.append_all(&self.attrs);
        let vis = &self.visibility;
        stream.append_all(quote! { #vis });
        stream.append_all(quote! { mod });
        stream.append(self.ident.clone());

        let mut content = TokenStream::new();
        content.append_all(
            self.impl_blocks
                .iter()
                .map(|impl_block| impl_block.original_token_stream()),
        );
        content.append_all(&self.imports);
        content.append_all(&self.functions);
        content.append_all(self.modules.iter().map(|m| m.original_token_stream()));

        stream.append_all(quote! { { #content } });

        stream
    }
}

/// Generates the code for an impl block inside a `pre_defs_for` module.
fn render_impl_block(
    path: &Path,
    impl_block: &DefinitionsForImplBlock,
    tokens: &mut TokenStream,
    visibility: &TokenStream,
) {
    let ty = if let Type::Path(path) = &*impl_block.self_ty {
        if path.path.segments.len() != 1 {
            emit_error!(path, "only paths of length 1 are supported here");
            return;
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
            return;
        }

        let ty = &path.path.segments[0];

        if matches!(ty.arguments, PathArguments::Parenthesized(_)) {
            emit_error!(
                ty.arguments.span(),
                "parenthesized type arguments are not supported here"
            );
            return;
        }

        ty
    } else {
        emit_error!(
            impl_block.self_ty.span(),
            "`impl` block are only supported for structs, enums and unions in this context"
        );
        return;
    };

    let struct_arguments =
        if let PathArguments::AngleBracketed(mut arguments) = ty.arguments.clone() {
            let new_arguments: Punctuated<_, Token![,]> = arguments
                .args
                .iter_mut()
                .enumerate()
                .filter_map(|(i, arg)| -> Option<GenericParam> {
                    match arg.clone() {
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
                                ident: Ident::new(&format!("T{}", i), ty.span()),
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
                                ident: Ident::new(&format!("T{}", i), ty.span()),
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
                })
                .collect();

            Some(new_arguments)
        } else {
            None
        };

    // First generate a dummy struct that the impl block will be attached to.
    // Care must be taken that all generic parameters are used in the struct definition.
    tokens.append_all(quote_spanned! { impl_block.span()=> #[allow(dead_code)] });
    tokens.append_all(visibility.clone().into_iter().map(|mut token| {
        token.set_span(impl_block.span());
        token
    }));

    let name = &ty.ident;
    if let Some(struct_arguments) = struct_arguments {
        let mut struct_contents: Punctuated<TokenStream, Token![,]> = struct_arguments
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

        tokens.append_all(quote_spanned! { impl_block.span()=>
            struct #name <#struct_arguments>(::core::marker::PhantomData<(#struct_contents)>);
        });
    } else {
        tokens.append_all(quote_spanned! { impl_block.span()=>
            struct #name;
        });
    }
}

/// Generates the code for a function inside a `pre_defs_for` module.
fn render_function(
    path: &Path,
    function: &ForeignItemFn,
    tokens: &mut TokenStream,
    visibility: &TokenStream,
) {
    tokens.append_all(&function.attrs);
    tokens.append_all(quote_spanned! { function.span()=> #[inline(always)] });
    tokens.append_all(visibility.clone().into_iter().map(|mut token| {
        token.set_span(function.span());
        token
    }));
    let signature = &function.sig;
    tokens.append_all(quote! { #signature });

    let mut path = path.clone();

    path.segments.push(PathSegment {
        ident: function.sig.ident.clone(),
        arguments: PathArguments::None,
    });

    // Update the spans of the `::` tokens to lie in the function
    for punct in path.segments.pairs_mut().map(|p| p.into_tuple().1) {
        if let Some(punct) = punct {
            punct.spans = [function.span(); 2];
        }
    }

    let mut args_list = TokenStream::new();
    args_list.append_separated(
        function.sig.inputs.iter().map(|arg| match arg {
            FnArg::Receiver(_) => unreachable!("receiver is not valid in a function argument list"),
            FnArg::Typed(pat) => &pat.pat,
        }),
        quote_spanned! { function.span()=> , },
    );
    tokens.append_all(quote_spanned! { function.span()=> { #path(#args_list) } });
}
