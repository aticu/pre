//! Handles impl blocks in `pre_defs_for` modules.

use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    ForeignItemFn, Generics, Path, Token, Type,
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

    /// Generates the code for an impl block inside a `pre_defs_for` module.
    pub(crate) fn render(&self, tokens: &mut TokenStream, path: &Path, visibility: &TokenStream) {
        todo!();
    }
}
