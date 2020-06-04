//! This crate defines most tests for the `pre` crate.
//!
//! These are defined in a different crate, because otherwise `proc-macro-crate` does not work
//! properly.

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("ui/*.rs");
    }
}
