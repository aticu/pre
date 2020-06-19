//! Defines the `pre` attribute and how it is handled.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_warning;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{visit_file_mut, VisitMut},
    File, Item, ItemFn, Path,
};

use crate::{
    helpers::{crate_name, visit_matching_attrs_parsed, Parenthesized},
    precondition::Precondition,
    render_pre,
};

/// A `pre` attribute.
pub(crate) enum PreAttr {
    /// An empty attribute to trigger checking for contained attributes.
    Empty,
    /// A precondition that needs to hold for the contained item.
    Precondition(Parenthesized<Precondition>),
}

impl Parse for PreAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(PreAttr::Empty)
        } else {
            Ok(PreAttr::Precondition(input.parse()?))
        }
    }
}

/// Applies and removes all visited pre attributes.
pub(crate) struct PreAttrVisitor {
    /// The original attribute that started the visitor.
    original_attr: Option<Precondition>,
    /// All paths that signify a valid `pre` attribute.
    valid_pre_attrs_paths: Vec<Path>,
}

impl PreAttrVisitor {
    /// Creates a new visitor for the syntax tree that `original_attr` was attached to.
    pub(crate) fn new(original_attr: TokenStream) -> PreAttrVisitor {
        let crate_name = crate_name();
        let direct_path = syn::parse2(quote! { pre }).expect("valid path");
        let crate_path = syn::parse2(quote! { #crate_name::pre }).expect("valid path");
        let colon_crate_path = syn::parse2(quote! { ::#crate_name::pre }).expect("valid path");

        PreAttrVisitor {
            original_attr: parse2(original_attr).ok(),
            valid_pre_attrs_paths: vec![direct_path, crate_path, colon_crate_path],
        }
    }

    /// Renders the given function and applies all `pre` attributes to it.
    fn render_function(
        &mut self,
        function: &mut ItemFn,
        additional_precondition: Option<Precondition>,
    ) -> TokenStream {
        let mut preconditions: Vec<_> = additional_precondition.into_iter().collect();

        let attr_span = visit_matching_attrs_parsed(
            &mut function.attrs,
            |attr| {
                self.valid_pre_attrs_paths
                    .iter()
                    .any(|path| path == &attr.path)
            },
            |parsed_attr| match parsed_attr {
                PreAttr::Empty => (),
                PreAttr::Precondition(precondition) => preconditions.push(precondition.content),
            },
        );

        if preconditions.len() > 0 {
            let output = render_pre(
                preconditions,
                function,
                attr_span.unwrap_or_else(|| Span::call_site()),
            );

            output
        } else {
            quote! { #function }
        }
    }
}

impl VisitMut for PreAttrVisitor {
    fn visit_file_mut(&mut self, file: &mut File) {
        if file.items.len() == 1 {
            let new_item = match &mut file.items[0] {
                Item::Fn(function) => {
                    let original_attr = self.original_attr.take();
                    self.render_function(function, original_attr)
                }
                other_item => quote! { #other_item },
            };

            file.items[0] = Item::Verbatim(new_item);
        } else {
            if let Some(attr) = self.original_attr.as_ref() {
                emit_warning!(attr.span(), "this is ignored at this scope");
            }

            visit_file_mut(self, file)
        }
    }
}
