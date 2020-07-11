use pre::pre;

#[pre("is bar")]
fn foo() {}

#[pre]
fn main() {
    foo()
}
