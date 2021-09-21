//! Allows treating function and method call expressions the same.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::convert::TryFrom;
use syn::{punctuated::Punctuated, token::Comma, Expr, ExprCall, ExprMethodCall, ExprPath};

/// A call expression.
#[derive(Clone)]
pub(crate) enum Call {
    /// The call expression is a function call.
    Function(ExprCall),
    /// The call expression is a method call.
    Method(ExprMethodCall),
}

impl Call {
    /// Grants mutable access to the arguments of the call.
    pub(crate) fn args_mut(&mut self) -> &mut Punctuated<Expr, Comma> {
        match self {
            Call::Function(call) => &mut call.args,
            Call::Method(call) => &mut call.args,
        }
    }

    /// The path to the function being called, if present.
    ///
    /// For non-function calls, this returns `None`.
    /// If the expression is a function call expression, but the expression that resolves to the
    /// function is not a path expression, this also returns `None`.
    #[allow(dead_code)]
    pub(crate) fn path(&self) -> Option<ExprPath> {
        match self {
            Call::Function(call) => match &*call.func {
                Expr::Path(path) => Some(path.clone()),
                _ => None,
            },
            Call::Method(_) => None,
        }
    }

    /// Checks if the call expression is a function call.
    #[allow(dead_code)]
    pub(crate) fn is_function(&self) -> bool {
        matches!(self, Call::Function(_))
    }
}

impl From<ExprCall> for Call {
    fn from(call: ExprCall) -> Self {
        Call::Function(call)
    }
}

impl From<ExprMethodCall> for Call {
    fn from(call: ExprMethodCall) -> Self {
        Call::Method(call)
    }
}

impl TryFrom<Expr> for Call {
    type Error = Expr;

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Call(call) => Ok(call.into()),
            Expr::MethodCall(call) => Ok(call.into()),
            _ => Err(value),
        }
    }
}

impl From<Call> for Expr {
    fn from(call: Call) -> Self {
        match call {
            Call::Function(call) => Expr::Call(call),
            Call::Method(call) => Expr::MethodCall(call),
        }
    }
}

impl ToTokens for Call {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Call::Function(call) => tokens.append_all(quote! { #call }),
            Call::Method(call) => tokens.append_all(quote! { #call }),
        }
    }
}
