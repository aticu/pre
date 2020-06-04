//! Defines a list with multiple preconditions.

use std::fmt;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

/// A list of preconditions for a function call.
pub(crate) struct PreconditionList<T> {
    /// The actual list of preconditions.
    preconditions: Punctuated<T, Token![,]>,
}

impl<T: fmt::Debug> fmt::Debug for PreconditionList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for precondition in self.preconditions.iter() {
            if first {
                write!(f, "{:?}", precondition)?;

                first = false;
            } else {
                write!(f, ", {:?}", precondition)?;
            }
        }

        Ok(())
    }
}

impl<T: Parse> Parse for PreconditionList<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PreconditionList {
            preconditions: Punctuated::parse_terminated(input)?,
        })
    }
}

impl<T> PreconditionList<T> {
    /// Provides an iterator over the preconditions.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.preconditions.iter()
    }
}

impl<T: Ord> PreconditionList<T> {
    /// Provides an iterator with a deterministic ordering over the preconditions.
    pub(crate) fn sorted_iter(&self) -> impl Iterator<Item = &T> {
        let mut index_vec: Vec<_> = (0..self.preconditions.len()).collect();

        // Reverse the order here, so we can simply pop in the iterator
        index_vec.sort_unstable_by_key(|&index| std::cmp::Reverse(&self.preconditions[index]));

        SortedIterator {
            list: &self,
            indices: index_vec,
        }
    }
}

/// An iterator over a sorted precondition list.
struct SortedIterator<'a, T> {
    /// The list of preconditions being iterated over.
    list: &'a PreconditionList<T>,
    /// The already sorted vector of indices into the list.
    indices: Vec<usize>,
}

impl<'a, T> Iterator for SortedIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices
            .pop()
            .map(|index| &self.list.preconditions[index])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.indices.len(), Some(self.indices.len()))
    }
}
