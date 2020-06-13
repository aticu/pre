//! Contains the types needed for the const generics implementation.

/// A declaration that a custom condition holds.
pub struct CustomConditionHolds<const CONDITION: &'static str>;

/// A declaration that the pointer of name `PTR` is valid.
pub struct ValidPtrConditionHolds<const PTR: &'static str, const ACCESS_TYPE: &'static str>;
