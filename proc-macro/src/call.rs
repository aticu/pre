//! Allows treating function and method call expressions the same.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, Expr, ExprCall, ExprMethodCall, ExprPath,
};

/// A call expression.
pub(crate) enum Call {
    /// The call expression is a function call.
    Function(ExprCall),
    /// The call expression is a method call.
    Method(ExprMethodCall),
}

impl Call {
    /// Access a mutable reference arguments of the call.
    pub(crate) fn args_mut(&mut self) -> &mut Punctuated<Expr, Comma> {
        match self {
            Call::Function(call) => &mut call.args,
            Call::Method(call) => &mut call.args,
        }
    }

    /// The name of the function or method.
    ///
    /// For a method, the path is created from the single identifier that is the method name.
    #[allow(dead_code)]
    pub(crate) fn path(&self) -> Option<ExprPath> {
        match self {
            Call::Function(call) => match &*call.func {
                Expr::Path(path) => Some(path.clone()),
                _ => None,
            },
            Call::Method(call) => {
                let method = &call.method;
                Some(parse_quote! {
                    #method
                })
            }
        }
    }

    /// Returns true, if a function is called.
    #[allow(dead_code)]
    pub(crate) fn is_function(&self) -> bool {
        match self {
            Call::Function(_) => true,
            _ => false,
        }
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
