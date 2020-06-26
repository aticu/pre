use pre::pre;

#[pre("must be bar")]
#[pre("must also be baz")]
fn foo() {}

#[pre]
fn main() {
    #[assure("must be bar", reason = "is bar")]
    #[assure("must also be baz", reason = "is also baz")]
    foo()
}
