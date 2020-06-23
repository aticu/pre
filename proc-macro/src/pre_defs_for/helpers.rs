//! Provides helper function for handling `pre_defs_for` modules.

use proc_macro2::Span;
use std::mem;
use syn::{
    spanned::Spanned,
    visit::{visit_pat, Visit},
    visit_mut::{visit_ident_mut, visit_type_path_mut, VisitMut},
    Ident, Pat, Path, Signature, TypePath,
};

/// A struct that replaces matching types with `replace_with` when visiting them.
struct TypeReplacer<'ty, Matches: FnMut(&mut TypePath) -> bool> {
    /// The function that checks if a type matches.
    matches: Matches,
    /// The type to replace the matching types with.
    replace_with: &'ty Path,
}

/// Replaces all matching types in `signature` with `replace_with`.
pub(crate) fn replace_types<Matches: FnMut(&mut TypePath) -> bool>(
    mut signature: &mut Signature,
    matches: Matches,
    replace_with: &Path,
) {
    TypeReplacer {
        matches,
        replace_with,
    }
    .visit_signature_mut(&mut signature);
}

impl<Matches: FnMut(&mut TypePath) -> bool> VisitMut for TypeReplacer<'_, Matches> {
    fn visit_type_path_mut(&mut self, path: &mut TypePath) {
        if (self.matches)(path) {
            mem::swap(&mut path.path, &mut self.replace_with.clone());
        }

        visit_type_path_mut(self, path);
    }
}

/// A struct that replaces matching identifiers with `replace_with` when visiting them.
struct IdentReplacer<'ty, Matches: FnMut(&mut Ident) -> bool> {
    /// The function that checks if an identifier matches.
    matches: Matches,
    /// The identifier to replace the matching identifier with.
    replace_with: &'ty Ident,
}

/// Replaces all matching identifiers in `signature` with `replace_with`.
pub(crate) fn replace_idents<Matches: FnMut(&mut Ident) -> bool>(
    mut pattern: &mut Pat,
    matches: Matches,
    replace_with: &Ident,
) {
    IdentReplacer {
        matches,
        replace_with,
    }
    .visit_pat_mut(&mut pattern);
}

impl<Matches: FnMut(&mut Ident) -> bool> VisitMut for IdentReplacer<'_, Matches> {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if (self.matches)(ident) {
            mem::swap(ident, &mut self.replace_with.clone());
        }

        visit_ident_mut(self, ident);
    }
}

/// Finds a new and unused identifier that can be used when altering the signature.
pub(crate) fn find_new_ident(signature: &Signature, span: Option<Span>) -> Ident {
    let mut ident = String::from("_self");

    loop {
        let mut finder = IdentFinder {
            ident: &ident,
            found: false,
        };

        finder.visit_signature(signature);

        if finder.found {
            ident.insert(0, '_');
        } else {
            break;
        }
    }

    Ident::new(&ident, span.unwrap_or_else(|| signature.span()))
}

/// A struct that can find a given identifier when visiting a syntax tree.
struct IdentFinder<'ident> {
    /// The identifier to look for.
    ident: &'ident str,
    /// Whether the identifier was found.
    found: bool,
}

impl Visit<'_> for IdentFinder<'_> {
    fn visit_ident(&mut self, ident: &Ident) {
        if self.found {
            return;
        }

        self.found = ident == self.ident;
    }

    // This is just implemented as an optimization to prevent descending further into patterns, if
    // the ident was already found.
    fn visit_pat(&mut self, pattern: &Pat) {
        if self.found {
            return;
        }

        visit_pat(self, pattern);
    }
}
