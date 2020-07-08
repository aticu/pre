use pre::pre;

struct X;

impl X {
    #[pre("precondition on method")]
    fn foo(&self) {}
}

#[pre]
fn main() {
    #[assure("precondition on method", reason = "it is on a method")]
    X.foo();
}
