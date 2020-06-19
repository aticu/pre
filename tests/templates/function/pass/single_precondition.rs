use pre::pre;

#[pre("must be bar")]
fn foo() {}

#[pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    foo()
}
