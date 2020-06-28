//! Defines the `pre` attribute and how it is handled.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use std::{convert::TryInto, mem};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{visit_expr_mut, visit_file_mut, visit_item_fn_mut, visit_item_mut, VisitMut},
    Expr, File, Item, ItemFn,
};

use crate::{
    call_handling::{remove_call_attributes, render_call},
    helpers::{attributes_of_expression, is_attr, visit_matching_attrs_parsed, Parenthesized},
    precondition::Precondition,
    render_pre,
};

/// A `pre` attribute.
pub(crate) enum PreAttr {
    /// An empty attribute to trigger checking for contained attributes.
    Empty,
    /// A precondition that needs to hold for the contained item.
    Precondition(Precondition),
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
    original_attr: Option<PreAttr>,
}

impl PreAttrVisitor {
    /// Creates a new visitor for the syntax tree that `original_attr` was attached to.
    pub(crate) fn new(original_attr: TokenStream) -> PreAttrVisitor {
        let original_attr = if !original_attr.is_empty() {
            let span = original_attr.span();

            match parse2(original_attr) {
                Ok(attr) => Some(attr),
                Err(err) => {
                    emit_error!(
                        span,
                        "expected either nothing or a valid `pre` attribute here"
                    );
                    emit_error!(err);

                    None
                }
            }
        } else {
            None
        };

        PreAttrVisitor { original_attr }
    }

    /// Renders the given function and applies all `pre` attributes to it.
    fn render_function(
        &mut self,
        function: &mut ItemFn,
        first_attr: Option<PreAttr>,
    ) -> TokenStream {
        let mut preconditions: Vec<_> = first_attr
            .and_then(|attr| match attr {
                PreAttr::Precondition(precondition) => Some(precondition),
                _ => None,
            })
            .into_iter()
            .collect();

        let attr_span = visit_matching_attrs_parsed(
            &mut function.attrs,
            |attr| is_attr("pre", attr),
            |parsed_attr: Parenthesized<PreAttr>| match parsed_attr.content {
                PreAttr::Empty => (),
                PreAttr::Precondition(precondition) => preconditions.push(precondition),
            },
        );

        if !preconditions.is_empty() {
            render_pre(
                preconditions,
                function,
                attr_span.unwrap_or_else(Span::call_site),
            )
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

                    visit_item_fn_mut(self, function);
                    self.render_function(function, original_attr)
                }
                other_item => {
                    visit_item_mut(self, other_item);

                    quote! { #other_item }
                }
            };

            file.items[0] = Item::Verbatim(new_item);
        } else {
            match self.original_attr.take() {
                Some(PreAttr::Empty) => (),
                Some(PreAttr::Precondition(precondition)) => {
                    emit_warning!(precondition.span(), "this does not do anything")
                }
                None => (),
            }

            visit_file_mut(self, file);
        }
    }

    fn visit_item_mut(&mut self, item: &mut Item) {
        visit_item_mut(self, item);

        if let Item::Fn(function) = item {
            let rendered_function = self.render_function(function, None);
            mem::swap(item, &mut Item::Verbatim(rendered_function));
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_expr_mut(self, expr);

        let attrs = if let Some(attrs) = attributes_of_expression(expr) {
            attrs
        } else {
            return;
        };

        if let Some(attrs) = remove_call_attributes(attrs) {
            if let Some(expr) = extract_call_expr(expr) {
                let call = expr
                    .clone()
                    .try_into()
                    .expect("`extract_call_expr` should only return call expressions");

                mem::swap(&mut render_call(attrs, call), expr);
            } else {
                if let Some(forward) = attrs.forward {
                    emit_warning!(forward.span(), "this is ignored for non-call expressions");
                }

                for precondition in attrs.preconditions {
                    emit_warning!(
                        precondition.span(),
                        "this is ignored for non-call expressions"
                    );
                }
            }
        }
    }
}

/// Extracts an expression that is a valid call from the given expression.
///
/// This may descend into nested expressions, if it would be obvious which nested expression is
/// meant.
fn extract_call_expr(expr: &mut Expr) -> Option<&mut Expr> {
    match expr {
        Expr::Call(_) => Some(expr),
        Expr::MethodCall(_) => Some(expr),
        _ => None,
    }
}
