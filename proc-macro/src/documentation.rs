//! Provides functions to generate documentation about the preconditions.

use proc_macro2::Span;
use quote::{quote, quote_spanned};
use std::{env, fmt::Write};
use syn::{
    spanned::Spanned,
    token::{Bracket, Pound},
    AttrStyle, Attribute, Ident, LitStr, Path, PathArguments, Signature,
};

use crate::{
    extern_crate::{ImplBlock, Module},
    helpers::HINT_REASON,
    precondition::{CfgPrecondition, Precondition},
};

/// Evaluates to the base URL of the documentation for the `pre` crate.
macro_rules! docs_url {
    () => {
        concat!("https://docs.rs/pre/", env!("CARGO_PKG_VERSION"), "/pre",)
    };
}

/// A link to the documentation of the `pre` attribute.
const PRE_LINK: &str = concat!(docs_url!(), "/attr.pre.html");

/// A link to the documentation of the `assure` attribute.
const ASSURE_LINK: &str = concat!(docs_url!(), "/attr.assure.html");

/// A link to the documentation of the `extern_crate` attribute.
const EXTERN_CRATE_LINK: &str = concat!(docs_url!(), "/attr.extern_crate.html");

/// The required context for generating `impl` block documentation.
pub(crate) struct ImplBlockContext<'a> {
    /// The `impl` block that the item belongs to.
    pub(crate) impl_block: &'a ImplBlock,
    /// The path to the `impl` block.
    pub(crate) path: &'a Path,
    /// The name of the top level module that the `impl` block is contained in.
    pub(crate) top_level_module: &'a Ident,
}

macro_rules! doc_inline {
    ($docs:expr) => {
        write!($docs).expect("string writes don't fail")
    };
    ($docs:expr, $format_str:literal) => {
        write!($docs, $format_str).expect("string writes don't fail")
    };
    ($docs:expr, $format_str:literal, $($args:expr),*) => {
        write!($docs, $format_str, $($args,)*).expect("string writes don't fail")
    };
}

macro_rules! doc {
    ($docs:expr) => {
        writeln!($docs).expect("string writes don't fail")
    };
    ($docs:expr, $format_str:literal) => {
        writeln!($docs, $format_str).expect("string writes don't fail")
    };
    ($docs:expr, $format_str:literal, $($args:expr),*) => {
        writeln!($docs, $format_str, $($args,)*).expect("string writes don't fail")
    };
}

/// Generates documentation of the preconditions for a function or method.
pub(crate) fn generate_docs(
    function: &Signature,
    preconditions: &[CfgPrecondition],
    impl_block_context: Option<ImplBlockContext>,
) -> Attribute {
    let span = function.span();
    let mut docs = String::new();
    let plural = preconditions.len() != 1;

    if let Some(ctx) = &impl_block_context {
        let (path_str, path_str_no_generics) = if let Some(ty) = &ctx.impl_block.ty() {
            let mut path_str = String::new();
            for segment in ctx.path.segments.iter() {
                doc_inline!(path_str, "{}::", segment.ident);
            }

            doc_inline!(path_str, "{}", ty.ident);

            let mut path_str_no_generics = path_str.clone();

            match &ty.arguments {
                PathArguments::None => (),
                PathArguments::AngleBracketed(args) => {
                    doc_inline!(path_str, "<");
                    for arg in &args.args {
                        doc_inline!(path_str, "{}", quote! { #arg });
                    }
                    doc_inline!(path_str, ">");
                }
                PathArguments::Parenthesized(_) => unreachable!(),
            }

            let name = &function.ident;
            doc_inline!(path_str, "::{}", quote! { #name });
            doc_inline!(path_str_no_generics, "::{}", quote! { #name });

            (path_str, Some(path_str_no_generics))
        } else {
            let path = &ctx.path;
            let ty = &ctx.impl_block.self_ty;
            let name = &function.ident;

            (
                format!(
                    "{}::{}::{}",
                    quote! { #path },
                    quote! { #ty },
                    quote! { #name }
                ),
                None,
            )
        };

        // TODO: remove the nightly condition here, once rust paths are supported for documentation
        // links on stable
        match (cfg!(nightly), path_str_no_generics) {
            (true, Some(no_generics)) => doc!(
                docs,
                "A stub for the preconditions of the [`{}`](value@{}) function.",
                path_str,
                no_generics
            ),
            _ => doc!(
                docs,
                "A stub for the preconditions of the `{}` function.",
                path_str
            ),
        }

        doc!(docs);

        doc!(docs, "# What is this function?");
        doc!(docs);

        doc!(
            docs,
            "This function was generated by an `impl` block inside a [`extern_crate` attribute]({}) that looked like this:",
            EXTERN_CRATE_LINK
        );
        doc!(docs);

        doc!(docs, "```rust,ignore");
        let ty = &ctx.impl_block.self_ty;
        let where_clause = &ctx.impl_block.generics.where_clause;
        let generics = if !ctx.impl_block.generics.params.is_empty() {
            Some(&ctx.impl_block.generics)
        } else {
            None
        };

        doc!(
            docs,
            "impl{} {} {} {{",
            quote! { #generics },
            quote! { #ty },
            quote! { #where_clause }
        );

        doc!(docs, "    {};", quote! { #function });
        if ctx.impl_block.items.len() > 1 {
            doc!(docs, "    /* other items omitted */");
        }

        doc!(docs, "}}");
        doc!(docs, "```");

        doc!(docs);
        doc!(docs, "Preconditions on external functions inside of an `impl` block are attached to empty functions like this one.");
        doc!(docs, "When the preconditions should be checked, a call to this function is inserted, which triggers checking the preconditions.");
        doc!(docs);
    }

    if !preconditions.is_empty() {
        doc!(docs, "# This function has preconditions");
        doc!(docs);

        if plural {
            doc!(docs, "This function has the following preconditions generated by [`pre` attributes]({}):", PRE_LINK);
        } else {
            doc!(docs, "This function has the following precondition generated by the [`pre` attribute]({}):", PRE_LINK);
        }
        doc!(docs);

        for precondition in preconditions {
            match precondition.precondition() {
                Precondition::ValidPtr {
                    ident, read_write, ..
                } => doc!(
                    docs,
                    "- the pointer `{}` must be valid for {}",
                    ident.to_string(),
                    read_write.doc_description()
                ),
                Precondition::ProperAlign { ident, .. } => doc!(
                    docs,
                    "- the pointer `{}` must have a proper alignment for its type",
                    ident.to_string()
                ),
                Precondition::Boolean(expr) => doc!(docs, "- `{}`", quote! { #expr }),
                Precondition::Custom(text) => doc!(docs, "- {}", text.value()),
            }
        }

        doc!(docs);
        if plural {
            doc!(
                docs,
                "To call the function you need to [`assure`]({}) that the preconditions hold:",
                ASSURE_LINK
            );
        } else {
            doc!(
                docs,
                "To call the function you need to [`assure`]({}) that the precondition holds:",
                ASSURE_LINK
            );
        }
        doc!(docs);
        doc!(docs, "```rust,ignore");

        if let Some(ctx) = &impl_block_context {
            let mut path_str = format!("{}", ctx.top_level_module);
            for segment in ctx.path.segments.iter().skip(1).chain(ctx.impl_block.ty()) {
                write!(path_str, "::{}", segment.ident).expect("string writes don't fail");
            }

            if let Ok(name) = env::var("CARGO_PKG_NAME") {
                let mut name = name.replace('-', "_");
                name.push_str("::");
                path_str.insert_str(0, &name);
            }

            doc!(docs, "#[forward(impl {})]", path_str);
        }

        for precondition in preconditions {
            doc!(docs, "#[assure(",);
            doc!(docs, "    {},", precondition.precondition());
            doc!(docs, "    reason = {:?}", HINT_REASON);
            doc!(docs, ")]");
        }

        let receiver = if function.receiver().is_some() {
            "x."
        } else {
            ""
        };
        let parameters = if function.inputs.is_empty() {
            ""
        } else {
            "/* parameters omitted */"
        };
        doc!(docs, "{}{}({});", receiver, function.ident, parameters);

        doc!(docs, "```");
    }

    let docs = LitStr::new(&docs, span);
    Attribute {
        pound_token: Pound { spans: [span] },
        style: AttrStyle::Outer,
        bracket_token: Bracket { span },
        path: Ident::new("doc", span).into(),
        tokens: quote_spanned! { span=>
            = #docs
        },
    }
}

/// Generates documentation of the preconditions for a `extern_crate` module.
pub(crate) fn generate_module_docs(module: &Module, path: &Path) -> Attribute {
    let span = module.span();
    let mut docs = String::new();

    let mut path_str = String::new();
    for segment in path.segments.iter() {
        if !path_str.is_empty() {
            path_str.push_str("::");
        }
        doc_inline!(path_str, "{}", segment.ident);
    }

    let item_name = if path.segments.len() == 1 {
        "crate"
    } else {
        "module"
    };

    if cfg!(nightly) {
        doc!(
            docs,
            "[`pre` definitions]({}) for the [`{}`](module@{}) {}.",
            PRE_LINK,
            path_str,
            path_str,
            item_name
        );
    } else {
        doc!(
            docs,
            "[`pre` definitions]({}) for the `{}` {}.",
            PRE_LINK,
            path_str,
            item_name
        );
    }

    doc!(docs);
    doc!(
        docs,
        "This module was generated by a [`extern_crate` attribute]({}).",
        EXTERN_CRATE_LINK
    );
    doc!(
        docs,
        "It acts as a drop-in replacement for the `{}` module.",
        path_str
    );

    let docs = LitStr::new(&docs, span);
    Attribute {
        pound_token: Pound { spans: [span] },
        style: AttrStyle::Outer,
        bracket_token: Bracket { span },
        path: Ident::new("doc", span).into(),
        tokens: quote_spanned! { span=>
            = #docs
        },
    }
}

/// Generates the start of the documentation for `extern_crate`-defined functions.
pub(crate) fn generate_extern_crate_fn_docs(
    path: &Path,
    function: &Signature,
    span: Span,
) -> Attribute {
    let mut docs = String::new();

    let mut path_str = String::new();
    for segment in path.segments.iter() {
        doc_inline!(path_str, "{}::", segment.ident);
    }
    doc_inline!(path_str, "{}", function.ident);

    if cfg!(nightly) {
        doc!(
            docs,
            "[`{}`](value@{}) with preconditions.",
            path_str,
            path_str
        );
    } else {
        doc!(docs, "`{}` with preconditions.", path_str);
    }
    doc!(docs);
    doc!(
        docs,
        "This function behaves exactly like `{}`, but also has preconditions checked by `pre`.",
        path_str
    );
    if function.unsafety.is_some() {
        doc!(docs);
        if cfg!(nightly) {
            doc!(
                docs,
                "**You should also read the [Safety section on the documentation of `{}`](value@{}#safety).**",
                path_str,
                path_str
            );
        } else {
            doc!(
                docs,
                "**You should also read the Safety section on the documentation of `{}`.**",
                path_str
            );
        }
    }
    doc!(docs);

    let docs = LitStr::new(&docs, span);
    Attribute {
        pound_token: Pound { spans: [span] },
        style: AttrStyle::Outer,
        bracket_token: Bracket { span },
        path: Ident::new("doc", span).into(),
        tokens: quote_spanned! { span=>
            = #docs
        },
    }
}
