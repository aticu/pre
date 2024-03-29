//! Provides handling of `extern_crate` attributes.
//!
//! # What the generated code looks like
//!
//! ```rust,ignore
//! #[pre::extern_crate(std)]
//! mod pre_std {
//!     mod ptr {
//!         #[pre(valid_ptr(src, r))]
//!         unsafe fn read<T>(src: *const T) -> T;
//!
//!         impl<T> NonNull<T> {
//!             #[pre(!ptr.is_null())]
//!             const unsafe fn new_unchecked(ptr: *mut T) -> NonNull<T>;
//!         }
//!     }
//! }
//! ```
//!
//! turns into
//!
//! ```rust,ignore
//! #[doc = "..."]
//! mod pre_std {
//!     #[allow(unused_imports)]
//!     use pre::pre;
//!     #[allow(unused_imports)]
//!     #[doc(no_inline)]
//!     pub(crate) use std::*;
//!
//!     #[doc = "..."]
//!     pub(crate) mod ptr {
//!         #[allow(unused_imports)]
//!         use pre::pre;
//!         #[allow(unused_imports)]
//!         #[doc(no_inline)]
//!         pub(crate) use std::ptr::*;
//!
//!         #[doc = "..."]
//!         #[pre(!ptr.is_null())]
//!         #[pre(no_doc)]
//!         #[pre(no_debug_assert)]
//!         #[inline(always)]
//!         #[allow(non_snake_case)]
//!         pub(crate) fn NonNull__impl__new_unchecked__() {}
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
use quote::{quote, quote_spanned, TokenStreamExt};
use std::fmt;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    Attribute, FnArg, ForeignItemFn, Ident, ItemUse, Path, PathArguments, PathSegment, Token,
    Visibility,
};

use crate::{
    documentation::{generate_extern_crate_fn_docs, generate_module_docs},
    helpers::{visit_matching_attrs_parsed_mut, AttributeAction, CRATE_NAME},
    pre_attr::PreAttr,
};

pub(crate) use impl_block::{impl_block_stub_name, ImplBlock};

mod impl_block;

/// The parsed version of the `extern_crate` attribute content.
pub(crate) struct ExternCrateAttr {
    /// The path of the crate/module to which function calls will be forwarded.
    path: Path,
}

impl fmt::Display for ExternCrateAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#[extern_crate(")?;

        if self.path.leading_colon.is_some() {
            write!(f, "::")?;
        }

        for segment in &self.path.segments {
            write!(f, "{}", segment.ident)?;
        }

        write!(f, ")]")
    }
}

impl Parse for ExternCrateAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ExternCrateAttr {
            path: input.call(Path::parse_mod_style)?,
        })
    }
}

/// A parsed `extern_crate` annotated module.
pub(crate) struct Module {
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
    impl_blocks: Vec<ImplBlock>,
    /// The imports contained in the module.
    imports: Vec<ItemUse>,
    /// The functions contained in the module.
    functions: Vec<ForeignItemFn>,
    /// The submodules contained in the module.
    modules: Vec<Module>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.original_token_stream())
    }
}

impl Spanned for Module {
    fn span(&self) -> Span {
        self.visibility
            .span()
            .join(self.braces.span)
            .unwrap_or(self.braces.span)
    }
}

impl Parse for Module {
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

        Ok(Module {
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

impl Module {
    /// Renders this `extern_crate` annotated module to its final result.
    pub(crate) fn render(&self, attr: ExternCrateAttr) -> TokenStream {
        let mut tokens = TokenStream::new();

        self.render_inner(attr.path, &mut tokens, None, &self.ident);

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
        top_level_module: &Ident,
    ) {
        if visibility.is_some() {
            // Update the path only in recursive calls.
            path.segments.push(PathSegment {
                ident: self.ident.clone(),
                arguments: PathArguments::None,
            });
        }

        let mut attrs = self.attrs.clone();
        let mut render_docs = true;
        visit_matching_attrs_parsed_mut(&mut attrs, "pre", |attr| match attr.content() {
            PreAttr::NoDoc(_) => {
                render_docs = false;

                AttributeAction::Remove
            }
            _ => AttributeAction::Keep,
        });

        if render_docs {
            let docs = generate_module_docs(self, &path);
            tokens.append_all(quote! { #docs });
        }
        tokens.append_all(attrs);

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

        let crate_name = Ident::new(&CRATE_NAME, Span::call_site());
        brace_content.append_all(quote! {
            #[allow(unused_imports)]
            #[doc(no_inline)]
            #visibility use #path::*;

            #[allow(unused_imports)]
            use #crate_name::pre;
        });

        for impl_block in &self.impl_blocks {
            impl_block.render(&mut brace_content, &path, &visibility, top_level_module);
        }

        for import in &self.imports {
            brace_content.append_all(quote! { #import });
        }

        for function in &self.functions {
            render_function(function, &mut brace_content, &path, &visibility);
        }

        for module in &self.modules {
            module.render_inner(
                path.clone(),
                &mut brace_content,
                Some(&visibility),
                top_level_module,
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

/// Generates the code for a function inside a `extern_crate` module.
fn render_function(
    function: &ForeignItemFn,
    tokens: &mut TokenStream,
    path: &Path,
    visibility: &TokenStream,
) {
    tokens.append_all(&function.attrs);
    let doc_header = generate_extern_crate_fn_docs(path, &function.sig, function.span());
    tokens.append_all(quote! { #doc_header });
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
    for punct in path
        .segments
        .pairs_mut()
        .map(|p| p.into_tuple().1)
        .flatten()
    {
        punct.spans = [function.span(); 2];
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
