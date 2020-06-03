//! Contains the types needed for the const generics implementation.

/// A custom condition that must be upheld.
pub trait CustomCondition<const CONDITION: &'static str> {}

/// A declaration that a custom condition holds.
pub struct CustomConditionHolds<const CONDITION: &'static str>;

impl<const CONDITION: &'static str> CustomCondition<CONDITION> for CustomConditionHolds<CONDITION> {}

/// A condition that the pointer of name `PTR` is valid.
pub trait ValidPtrCondition<const PTR: &'static str> {}

/// A declaration that the pointer of name `PTR` is valid.
pub struct ValidPtrConditionHolds<const PTR: &'static str>;

impl<const PTR: &'static str> ValidPtrCondition<PTR> for ValidPtrConditionHolds<PTR> {}
