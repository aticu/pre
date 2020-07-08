use pre::pre;

struct X;

impl X {
    fn foo(&self) {}

    fn bar(&self) {}
}

#[pre]
fn main() {
    #[forward(foo -> bar)]
    X.foo();
}
