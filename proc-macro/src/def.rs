//! Provides handling of `def_pre` attributes.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, TokenStreamExt};
use std::fmt;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    Attribute, FnArg, ForeignItemFn, Ident, Path, PathArguments, PathSegment, Token, Visibility,
};

/// The parsed version of the `def_pre` attribute content.
pub(crate) struct DefPreAttr {
    /// The path of the crate/module to which function calls will be forwarded.
    path: Path,
}

impl fmt::Display for DefPreAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#[def_pre(")?;

        if self.path.leading_colon.is_some() {
            write!(f, "::")?;
        }

        for segment in &self.path.segments {
            write!(f, "{}", segment.ident)?;
        }

        write!(f, ")]")
    }
}

impl Parse for DefPreAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DefPreAttr {
            path: input.call(Path::parse_mod_style)?,
        })
    }
}

/// A parsed `def_pre` annotated module.
pub(crate) struct DefPreModule {
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
    /// The submodules.
    modules: Vec<DefPreModule>,
    /// The functions contained in the module.
    functions: Vec<ForeignItemFn>,
}

impl fmt::Display for DefPreModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.original_token_stream())
    }
}

impl Parse for DefPreModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let mod_token = input.parse()?;
        let ident = input.parse()?;

        let content;
        let braces = braced!(content in input);
        let mut modules = Vec::new();
        let mut functions = Vec::new();

        loop {
            if content.is_empty() {
                break;
            }

            let is_function = {
                let result: syn::Result<ForeignItemFn> = content.fork().parse();
                result.is_ok()
            };

            if is_function {
                functions.push(content.parse()?);
            } else {
                modules.push(content.parse()?);
            }
        }

        Ok(DefPreModule {
            attrs,
            visibility,
            mod_token,
            ident,
            braces,
            modules,
            functions,
        })
    }
}

impl DefPreModule {
    /// Renders this `def_pre` annotated module to its final result.
    pub(crate) fn render(&self, attr: DefPreAttr) -> TokenStream {
        let mut tokens = TokenStream::new();

        self.render_inner(attr.path, &mut tokens, None);

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

        for function in &self.functions {
            render_function(path.clone(), function, &mut brace_content, &visibility);
        }

        for module in &self.modules {
            module.render_inner(path.clone(), &mut brace_content, Some(&visibility));
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
        content.append_all(&self.functions);
        content.append_all(self.modules.iter().map(|m| m.original_token_stream()));

        stream.append_all(quote! { { #content } });

        stream
    }
}

/// Renders a function inside a `def_pre` attribute to it's final result.
fn render_function(
    mut path: Path,
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

    path.segments.push(PathSegment {
        ident: function.sig.ident.clone(),
        arguments: PathArguments::None,
    });

    // Update the spans of the `::` tokens to lie in the function
    for (_segment, punct) in path.segments.pairs_mut().map(|p| p.into_tuple()) {
        if let Some(punct) = punct {
            punct.spans[0] = function.span();
            punct.spans[1] = function.span();
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
