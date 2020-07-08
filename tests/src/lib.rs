#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    macro_rules! add_testcases {
        ($test_cases:expr, $scenario:literal) => {{
            $test_cases.pass(concat!($scenario, "/function/pass/*.rs"));
            $test_cases.compile_fail(concat!($scenario, "/function/compile_fail/*.rs"));

            $test_cases.pass(concat!($scenario, "/precondition_types/pass/*.rs"));
            $test_cases.compile_fail(concat!($scenario, "/precondition_types/compile_fail/*.rs"));
        }};
    }

    #[cfg(not(nightly))]
    #[test]
    fn stable_tests() {
        let test_cases = TestCases::new();

        add_testcases!(test_cases, "stable");
    }

    #[cfg(nightly)]
    #[test]
    fn nightly_tests() {
        let test_cases = TestCases::new();

        add_testcases!(test_cases, "nightly");

        test_cases.pass("nightly/nightly-only/pass/*.rs");
        test_cases.compile_fail("nightly/nightly-only/compile_fail/*.rs");
    }
}
