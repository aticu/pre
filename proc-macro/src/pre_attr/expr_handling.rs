//! Handles rendering of expressions and descending into nested expressions.

use proc_macro_error::emit_warning;
use std::convert::TryInto;
use syn::{spanned::Spanned, Block, Expr, Local, Stmt};

use crate::call_handling::{render_call, CallAttributes};

/// Renders the contained call in the given expression.
///
/// This only works, if the call can be unambiguosly determined.
/// Otherwise warnings are printed.
pub(crate) fn render_expr(expr: &mut Expr, attrs: CallAttributes) {
    if let Some(expr) = extract_call_expr(expr) {
        let call = expr
            .clone()
            .try_into()
            .expect("`extract_call_expr` should only return call expressions");

        *expr = render_call(attrs, call);
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
