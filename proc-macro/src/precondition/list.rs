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

impl<T: fmt::Display> fmt::Display for PreconditionList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for precondition in self.preconditions.iter() {
            if first {
                write!(f, "{}", precondition)?;

                first = false;
            } else {
                write!(f, ", {}", precondition)?;
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
    ///
    /// The order of the preconditions is the order in which they were specified
    #[allow(dead_code)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.preconditions.iter()
    }
}

impl<T: Ord> PreconditionList<T> {
    /// Provides an iterator with a deterministic ordering over the preconditions.
    ///
    /// The same preconditions in a different order will result in the same order using this
    /// iterator.
    #[allow(dead_code)]
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
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::*;
    use crate::precondition::Precondition;

    #[test]
    fn parse_correct() {
        let result: Result<PreconditionList<Precondition>, _> = parse2(quote! {
            condition("foo"), condition("bar")
        });

        let result = result.expect("parsing should work");

        assert_eq!(result.iter().count(), 2);
    }

    #[test]
    fn parse_correct_trailing_comma() {
        let result: Result<PreconditionList<Precondition>, _> = parse2(quote! {
            condition("foo"), condition("bar"),
        });

        let result = result.expect("parsing should work");

        assert_eq!(result.iter().count(), 2);
    }

    #[test]
    fn iter_order_correct() {
        let result: Result<PreconditionList<Precondition>, _> = parse2(quote! {
            condition("5"), condition("4"), condition("3"), condition("2"), condition("1")
        });

        let result = result.expect("parsing should work");

        assert_eq!(
            result.iter().map(|c| format!("{}", c)).collect::<Vec<_>>(),
            (1..=5)
                .rev()
                .map(|num| format!("condition(\"{}\")", num))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn iter_sorted_order_correct() {
        let result1: Result<PreconditionList<Precondition>, _> = parse2(quote! {
            condition("1"), condition("2"), condition(valid_ptr(three, r+w)), condition("4"), condition("5")
        });

        let result1 = result1.expect("parsing should work");

        let result2: Result<PreconditionList<Precondition>, _> = parse2(quote! {
            condition("4"), condition("1"), condition("5"), condition(valid_ptr(three, r+w)), condition("2")
        });

        let result2 = result2.expect("parsing should work");

        assert_eq!(
            result1
                .sorted_iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<_>>(),
            result2
                .sorted_iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<_>>(),
        );
    }
}
