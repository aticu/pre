use pre::pre;

#[pre("must be bar")]
fn foo() {}

fn main() {
    foo()
}
