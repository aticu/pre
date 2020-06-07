#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    #[test]
    fn struct_impl_ui_tests() {
        let test_cases = TestCases::new();

        test_cases.pass(concat!(env!("OUT_DIR"), "/function/pass/*.rs"));
        test_cases.compile_fail(concat!(env!("OUT_DIR"), "/function/compile_fail/*.rs"));
    }
}
