#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    #[test]
    fn const_generics_impl_ui_tests() {
        let test_cases = TestCases::new();

        test_cases.pass("generated_tests/function/pass/*.rs");
        test_cases.compile_fail("generated_tests/function/compile_fail/*.rs");
    }
}
