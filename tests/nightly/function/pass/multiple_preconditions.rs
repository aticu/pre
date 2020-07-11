use pre::pre;

#[pre("is bar")]
#[pre("is also baz")]
fn foo() {}

#[pre]
fn main() {
    #[assure("is bar", reason = "foo is bar")]
    #[assure("is also baz", reason = "foo is also baz")]
    foo()
}
