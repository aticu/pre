//! Handles impl blocks in `extern_crate` modules.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    ForeignItemFn, Generics, Ident, Path, PathArguments, PathSegment, Token, Type,
};

use crate::{
    documentation::{generate_docs, ImplBlockContext},
    helpers::visit_matching_attrs_parsed,
    pre_attr::PreAttr,
    precondition::CfgPrecondition,
};

/// An impl block in a `extern_crate` module.
pub(crate) struct ImplBlock {
    /// The impl keyword.
    impl_keyword: Token![impl],
    /// The generics for the impl block.
    pub(crate) generics: Generics,
    /// The type which the impl block is for.
    pub(crate) self_ty: Box<Type>,
    /// The brace of the block.
    brace: Brace,
    /// The functions which the block applies to.
    pub(crate) items: Vec<ForeignItemFn>,
}

impl Parse for ImplBlock {
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

        Ok(ImplBlock {
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
        let self_ty = &self.self_ty;
        tokens.append_all(quote! { #self_ty });
        let where_clause = &generics.where_clause;
        tokens.append_all(quote! { #where_clause });

        let mut items = TokenStream::new();
        items.append_all(&self.items);
        tokens.append_all(quote! { { #items } });

        tokens
    }

    /// Returns the type that this impl block is for.
    pub(crate) fn ty(&self) -> Option<&PathSegment> {
        if let Type::Path(path) = &*self.self_ty {
            if path.path.segments.len() != 1 {
                let mut path_str = String::new();
                for i in 0..(path.path.segments.len() - 1) {
                    let segment = &path.path.segments[i];
                    path_str.push_str(&quote! { #segment }.to_string());

                    if i != path.path.segments.len() - 2 {
                        path_str.push_str("::");
                    }
                }

                let plural = if path.path.segments.len() > 2 {
                    "submodules"
                } else {
                    "a submodule"
                };

                emit_error!(
                    path,
                    "only paths of length 1 are supported here";
                    help = "try adding `{}` as {} and put the `impl` block there", path_str, plural
                );
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
                self.self_ty.span(),
                "`impl` block are only supported for structs, enums and unions in this context"
            );

            None
        }
    }

    /// Generates the code for an impl block inside a `extern_crate` module.
    pub(crate) fn render(
        &self,
        tokens: &mut TokenStream,
        path: &Path,
        visibility: &TokenStream,
        top_level_module: &Ident,
    ) {
        let ty = if let Some(ty) = self.ty() {
            ty
        } else {
            return;
        };

        for function in &self.items {
            let docs = {
                let mut render_docs = true;
                let mut preconditions = Vec::new();

                visit_matching_attrs_parsed(&function.attrs, "pre", |attr| {
                    match attr.into_content() {
                        (PreAttr::NoDoc(_), _, _) => render_docs = false,
                        (PreAttr::Precondition(precondition), cfg, span) => {
                            preconditions.push(CfgPrecondition {
                                precondition,
                                cfg,
                                span,
                            })
                        }
                        _ => (),
                    }
                });

                if render_docs {
                    Some(generate_docs(
                        &function.sig,
                        &preconditions,
                        Some(ImplBlockContext {
                            impl_block: self,
                            path,
                            top_level_module,
                        }),
                    ))
                } else {
                    None
                }
            };

            let name = impl_block_stub_name(ty, &function.sig.ident, function.span());
            tokens.append_all(quote! { #docs });
            tokens.append_all(&function.attrs);
            tokens.append_all(quote_spanned! { function.sig.span()=>
                // The documentation for `impl` blocks is generated here instead of in the `pre`
                // attribute, to allow access to information about the `impl` block.
                // In order to prevent it from being generated twice, `pre(no_doc)` is applied
                // here.
                #[pre(no_doc)]
                // The debug assertions for the original method likely won't make sense here, since
                // they probably depend on local parameters, which aren't present in this empty
                // function. To prevent errors, we remove the debug assertions here.
                #[pre(no_debug_assert)]
                #[inline(always)]
                #[allow(non_snake_case)]
                #visibility fn #name() {}
            });
        }
    }
}

/// Generates a name to use for an impl block stub function.
pub(crate) fn impl_block_stub_name(ty: &PathSegment, fn_name: &Ident, span: Span) -> Ident {
    // Ideally this would start with `_` to reduce the chance for naming collisions with actual
    // functions. However this would silence any `dead_code` warnings, which the user may want to
    // be aware of. Instead this ends with `__` to reduce the chance for naming collisions.
    //
    // Note that hygiene would not help in reducing naming collisions, because the function needs
    // to be callable from an `assure` attribute that could possibly reside in a different hygenic
    // context.
    let mut ident = format_ident!("{}__impl__{}__", ty.ident, fn_name);
    ident.set_span(span);

    ident
}
