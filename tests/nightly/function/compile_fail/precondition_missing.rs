use pre::pre;

#[pre("must be bar")]
#[pre("must be baz")]
fn foo() {}

#[pre]
fn main() {
    #[assure("must be bar", reason = "is bar")]
    foo()
}
