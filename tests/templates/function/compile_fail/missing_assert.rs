use pre::pre;

#[pre("must be bar")]
fn foo() {}

#[pre]
fn main() {
    foo()
}
