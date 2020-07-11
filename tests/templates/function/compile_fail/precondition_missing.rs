use pre::pre;

#[pre("is bar")]
#[pre("is baz")]
fn foo() {}

#[pre]
fn main() {
    #[assure("is bar", reason = "foo is bar")]
    foo()
}
