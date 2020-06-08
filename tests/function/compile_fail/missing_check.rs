use pre::pre;

#[pre(condition("must be bar"))]
fn foo() {}

fn main() {
    foo()
}
