//! Handles impl blocks in `pre_defs_for` modules.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    ForeignItemFn, Generics, Ident, Path, PathArguments, PathSegment, Token, Type,
};

/// An impl block in a `pre_defs_for` module.
pub(crate) struct ImplBlock {
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
                self.self_ty.span(),
                "`impl` block are only supported for structs, enums and unions in this context"
            );

            None
        }
    }

    /// Generates the code for an impl block inside a `pre_defs_for` module.
    pub(crate) fn render(&self, tokens: &mut TokenStream, _path: &Path, visibility: &TokenStream) {
        let ty = if let Some(ty) = self.ty() {
            ty
        } else {
            return;
        };

        for function in &self.items {
            tokens.append_all(&function.attrs);

            let name = impl_block_stub_name(ty, &function.sig.ident, function.span());
            tokens.append_all(quote_spanned! { function.sig.span()=>
                #[inline(always)]
                #[allow(non_snake_case)]
                #visibility fn #name() {}
            });
        }
    }
}

/// Generates a name to use for an impl block stub function.
pub(crate) fn impl_block_stub_name(
    ty: &PathSegment,
    fn_name: &impl std::fmt::Display,
    span: Span,
) -> Ident {
    // Ideally this would start with `_` to reduce the chance for naming collisions with actual
    // functions. However this would silence any `dead_code` warnings, which the user may want to
    // be aware of. Instead this ends with `__` to reduce the chance for naming collisions.
    //
    // Note that hygiene would not help in reducing naming collisions, because the function needs
    // to be callable from an `assure` attribute that could possibly reside in a different hygenic
    // context.
    Ident::new(&format!("{}__impl__{}__", ty.ident, fn_name), span)
}
