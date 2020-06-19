use pre::pre;

fn foo() {}

#[pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    foo()
}
