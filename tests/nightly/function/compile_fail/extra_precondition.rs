use pre::pre;

#[pre("is bar")]
fn foo() {}

#[pre]
fn main() {
    #[assure("is bar", reason = "foo is bar")]
    #[assure("is baz", reason = "foo is baz")]
    foo()
}
