use pre::pre;

#[pre("must be bar")]
#[pre("must be baz")]
fn foo() {}

#[pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    foo()
}
