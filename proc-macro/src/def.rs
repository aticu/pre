//! Provides handling of `def_pre` attributes.

use std::fmt;
use syn::{
    parse::{Parse, ParseStream},
    Path,
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
