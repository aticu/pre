#![cfg_attr(feature = "const-generics-impl", feature(const_generics))]
#![cfg_attr(feature = "const-generics-impl", allow(incomplete_features))]

pub use pre_proc_macro::{assert_precondition, pre};

#[cfg(feature = "const-generics-impl")]
mod const_generics_types;

#[cfg(feature = "const-generics-impl")]
#[doc(hidden)]
pub use const_generics_types::*;
