use pre::check_pre;

fn foo() {}

#[check_pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    foo()
}
