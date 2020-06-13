#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    #[cfg(not(nightly))]
    #[test]
    fn stable_tests() {
        let test_cases = TestCases::new();
        test_cases.pass("stable/function/pass/*.rs");
        test_cases.compile_fail("stable/function/compile_fail/*.rs");
    }

    #[cfg(nightly)]
    #[test]
    fn nightly_tests() {
        let test_cases = TestCases::new();
        test_cases.pass("nightly/function/pass/*.rs");
        test_cases.compile_fail("nightly/function/compile_fail/*.rs");
    }
}
