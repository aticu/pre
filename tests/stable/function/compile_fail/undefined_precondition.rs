use pre::pre;

fn foo() {}

#[pre]
fn main() {
    #[assure("must be bar", reason = "is bar")]
    foo()
}
