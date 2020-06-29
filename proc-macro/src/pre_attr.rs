//! Defines the `pre` attribute and how it is handled.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use std::{convert::TryInto, mem};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{
        visit_expr_mut, visit_file_mut, visit_item_fn_mut, visit_item_mut, visit_local_mut,
        VisitMut,
    },
    Block, Expr, File, Item, ItemFn, Local, Stmt,
};

use crate::{
    call_handling::{remove_call_attributes, render_call, CallAttributes},
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
            |parsed_attr: Parenthesized<PreAttr>, _span| match parsed_attr.content {
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

/// Renders the contained call in the given expression.
///
/// This only works, if the call can be unambiguosly determined.
/// Otherwise warnings are printed.
fn render_expr(expr: &mut Expr, attrs: CallAttributes) {
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

        for assure_attribute in attrs.assure_attributes {
            emit_warning!(
                assure_attribute.span(),
                "this is ignored for non-call expressions"
            );
        }
    }
}

/// Extracts an expression that is a valid call from the given expression.
///
/// This may descend into nested expressions, if it would be obvious which nested expression is
/// meant.
fn extract_call_expr(expr: &mut Expr) -> Option<&mut Expr> {
    fn extract_from_block(block: &mut Block) -> Option<&mut Expr> {
        if block.stmts.len() == 1 {
            match &mut block.stmts[0] {
                Stmt::Local(Local {
                    init: Some((_, expr)),
                    ..
                }) => extract_call_expr(expr),
                Stmt::Local(_) => None,
                Stmt::Item(_) => None,
                Stmt::Expr(expr) => extract_call_expr(expr),
                Stmt::Semi(expr, _) => extract_call_expr(expr),
            }
        } else {
            None
        }
    }

    macro_rules! find_subexpr {
        ($expr:expr;
         direct_return:
             $($direct_return:ident),*;
         subexpressions:
             $($simple_ty:ident . $simple_field:ident),*;
         binary_subexpressions:
             $($binary_ty:ident : $left:ident ^ $right:ident),*;
         optional_subexpressions:
             $($optional_ty:ident . $optional_syn_ty:ident ? $optional_field:ident),*;
         subblocks:
             $($block_ty:ident . $block_name:ident),*;
         manual:
             $($($manual_pat:pat)|+ $(if $manual_guard:expr)? => $manual_expr:expr),* $(;)?
        ) => {
            match $expr {
                // Direct return:
                // We found a call, so return it directly
                $(
                    Expr::$direct_return(_) => Some($expr),
                )*
                // Subexpressions:
                // There is a single unambiguos subexpression that will be searched.
                $(
                    Expr::$simple_ty(expr) => extract_call_expr(&mut expr.$simple_field),
                )*
                // Binary subexpressions:
                // There are always exactly two subexpressions. Search them both and return the
                // call if exactly one of them is an unambiguos call expression.
                $(
                    Expr::$binary_ty(expr) =>
                    extract_call_expr(&mut expr.$left).xor(extract_call_expr(&mut expr.$right)),
                )*
                // Optional subexpressions:
                // There may or may not be a subexpression. If there is one, search it.
                $(
                    Expr::$optional_ty(syn::$optional_syn_ty { expr: Some(expr), .. }) =>
                    extract_call_expr(expr),
                )*
                // Subblocks:
                // Search the contained block using the `extract_from_block`.
                $(
                    Expr::$block_ty(expr) => extract_from_block(&mut expr.$block_name),
                )*
                // Manual:
                // Manually match on an expression pattern and handle it.
                $(
                    $($manual_pat)|+ $(if $manual_guard)? => $manual_expr,
                )*
                // Otherwise:
                // Assume there is no contained call expression otherwise.
                _ => None,
            }
        }
    }

    find_subexpr! { expr;
        direct_return:
            Call,
            MethodCall;
        subexpressions:
            Await.base,
            Box.expr,
            Cast.expr,
            Closure.body,
            Field.base,
            Group.expr,
            Let.expr,
            Paren.expr,
            Reference.expr,
            Try.expr,
            Type.expr,
            Unary.expr;
        binary_subexpressions:
            Assign: left ^ right,
            AssignOp: left ^ right,
            Binary: left ^ right,
            Index: expr ^ index;
        optional_subexpressions:
            Break.ExprBreak ? expr,
            Return.ExprReturn ? expr,
            Yield.ExprYield ? expr;
        subblocks:
            Async.block,
            Block.block,
            Loop.body,
            TryBlock.block,
            Unsafe.block;
        manual:
            Expr::Tuple(expr) if expr.elems.len() == 1 => extract_call_expr(&mut expr.elems[0]);
    }
}
