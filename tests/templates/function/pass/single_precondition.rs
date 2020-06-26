use pre::pre;

#[pre("must be bar")]
fn foo() {}

#[pre]
fn main() {
    #[assure("must be bar", reason = "is bar")]
    foo()
}
