//! Defines the `pre` attribute and how it is handled.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{
        visit_expr_mut, visit_file_mut, visit_item_fn_mut, visit_item_mut, visit_local_mut,
        VisitMut,
    },
    Expr, File, Item, ItemFn, Local,
};

use self::expr_handling::render_expr;
use crate::{
    call_handling::remove_call_attributes,
    documentation::generate_docs,
    helpers::{attributes_of_expression, is_attr, visit_matching_attrs_parsed, Parenthesized},
    precondition::Precondition,
    render_pre,
};

mod expr_handling;

/// The custom keywords used for `pre` attributes.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(no_doc);
    custom_keyword!(no_debug_assert);
}

/// A `pre` attribute.
pub(crate) enum PreAttr {
    /// An empty attribute to trigger checking for contained attributes.
    Empty,
    /// A request not to generate `pre`-related documentation for the contained item.
    NoDoc(custom_keywords::no_doc),
    /// A request not to generate `debug_assert` statements for boolean expressions.
    NoDebugAssert(custom_keywords::no_debug_assert),
    /// A precondition that needs to hold for the contained item.
    Precondition(Precondition),
}

impl Parse for PreAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(PreAttr::Empty)
        } else if input.peek(custom_keywords::no_doc) {
            Ok(PreAttr::NoDoc(input.parse()?))
        } else if input.peek(custom_keywords::no_debug_assert) {
            Ok(PreAttr::NoDebugAssert(input.parse()?))
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
}

impl VisitMut for PreAttrVisitor {
    fn visit_file_mut(&mut self, file: &mut File) {
        let original_attr = self.original_attr.take();

        if let [Item::Fn(function)] = &mut file.items[..] {
            // Use `visit_item_fn_mut ` here, so that the function remains an `ItemFn` that can be
            // passed to `render_function`. Using `visit_item_mut` here would result in an
            // `Item::Verbatim` instead.
            visit_item_fn_mut(self, function);

            file.items[0] = Item::Verbatim(render_function(function, original_attr));
        } else {
            visit_file_mut(self, file);

            if let Some(original_attr) = original_attr {
                if let Some(span) = match original_attr {
                    PreAttr::Empty => None,
                    PreAttr::NoDoc(no_doc) => Some(no_doc.span()),
                    PreAttr::NoDebugAssert(no_debug_assert) => Some(no_debug_assert.span()),
                    PreAttr::Precondition(precondition) => Some(precondition.span()),
                } {
                    emit_warning!(span, "this is ignored in this context")
                }
            }
        }
    }

    fn visit_item_mut(&mut self, item: &mut Item) {
        visit_item_mut(self, item);

        if let Item::Fn(function) = item {
            let rendered_function = render_function(function, None);
            *item = Item::Verbatim(rendered_function);
        }
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_expr_mut(self, expr);

        if let Some(attrs) = attributes_of_expression(expr) {
            if let Some(call_attrs) = remove_call_attributes(attrs) {
                render_expr(expr, call_attrs);
            }
        }
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        visit_local_mut(self, local);

        if let Some((_, expr)) = &mut local.init {
            if let Some(call_attrs) = remove_call_attributes(&mut local.attrs) {
                render_expr(expr, call_attrs);
            }
        }
    }
}

/// Renders the given function and applies all `pre` attributes to it.
fn render_function(function: &mut ItemFn, first_attr: Option<PreAttr>) -> TokenStream {
    let first_attr_span = first_attr.as_ref().and_then(|attr| match attr {
        PreAttr::Empty => None,
        PreAttr::NoDoc(no_doc) => Some(no_doc.span()),
        PreAttr::NoDebugAssert(no_debug_assert) => Some(no_debug_assert.span()),
        PreAttr::Precondition(precondition) => Some(precondition.span()),
    });

    let mut preconditions = Vec::new();

    let mut render_docs = true;
    let mut debug_assert = true;

    let mut handle_attr = |attr| match attr {
        PreAttr::Empty => (),
        PreAttr::NoDoc(_) => render_docs = false,
        PreAttr::NoDebugAssert(_) => debug_assert = false,
        PreAttr::Precondition(precondition) => {
            if let Precondition::Boolean(boolean_expr) = &precondition {
                if let Expr::Path(p) = &**boolean_expr {
                    if let (None, Some(ident)) = (&p.qself, p.path.get_ident()) {
                        emit_error!(
                            ident.span(),
                            "keyword `{}` is not recognized by pre", ident;
                            help = "if you wanted to use a boolean expression, try `{} == true`",
                            ident
                        );
                    }
                }
            }
            preconditions.push(precondition)
        }
    };

    if let Some(first_attr) = first_attr {
        handle_attr(first_attr);
    }

    let attr_span = visit_matching_attrs_parsed(
        &mut function.attrs,
        |attr| is_attr("pre", attr),
        |parsed_attr: Parenthesized<PreAttr>, _span| handle_attr(parsed_attr.content),
    );

    let span = match (attr_span, first_attr_span) {
        (Some(attr_span), Some(first_attr_span)) => {
            attr_span.join(first_attr_span).unwrap_or_else(|| attr_span)
        }
        (Some(span), None) => span,
        (None, Some(span)) => span,
        (None, None) => Span::call_site(), // Should never be the case for non-empty preconditions
    };

    if !preconditions.is_empty() {
        if render_docs {
            function
                .attrs
                .push(generate_docs(&function.sig, &preconditions, None));
        }

        if debug_assert {
            for condition in preconditions.iter() {
                if let Precondition::Boolean(expr) = condition {
                    function.block.stmts.insert(
                        0,
                        parse2(quote_spanned! { expr.span()=>
                            ::core::debug_assert!(
                                #expr,
                                "boolean precondition was wrongly assured: `{}`",
                                ::core::stringify!(#expr)
                            );
                        })
                        .expect("valid statement"),
                    );
                }
            }
        }

        render_pre(preconditions, function, span)
    } else {
        quote! { #function }
    }
}
