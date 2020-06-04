#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    #[test]
    fn const_generics_impl_ui_tests() {
        let test_cases = TestCases::new();

        test_cases.compile_fail("tests/compile_fail/*.rs");
    }
}
