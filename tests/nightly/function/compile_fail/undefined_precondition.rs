use pre::pre;

fn foo() {}

#[pre]
fn main() {
    #[assure("is bar", reason = "foo is bar")]
    foo()
}
