#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    macro_rules! add_category {
        ($test_cases:expr, $scenario:literal, $category:literal) => {{
            $test_cases.pass(concat!($scenario, "/", $category, "/pass/*.rs"));
            $test_cases.compile_fail(concat!($scenario, "/", $category, "/compile_fail/*.rs"));
        }};
    }

    macro_rules! add_testcases {
        ($test_cases:expr, $scenario:literal) => {{
            add_category!($test_cases, $scenario, "function");
            add_category!($test_cases, $scenario, "precondition_types");
            add_category!($test_cases, $scenario, "extern_crate");
            add_category!($test_cases, $scenario, "misc");
        }};
    }

    #[cfg(not(nightly))]
    #[test]
    fn stable_tests() {
        let test_cases = TestCases::new();

        add_testcases!(test_cases, "stable");

        add_category!(test_cases, "stable", "stable-only");
    }

    #[cfg(nightly)]
    #[test]
    fn nightly_tests() {
        let test_cases = TestCases::new();

        add_testcases!(test_cases, "nightly");

        add_category!(test_cases, "nightly", "nightly-only");
    }
}
