use pre::pre;

#[pre("must be bar")]
fn foo() {}

#[pre]
fn main() {
    #[assure("must be bar", reason = "is bar")]
    #[assure("must be baz", reason = "is baz")]
    foo()
}
