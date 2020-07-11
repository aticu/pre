use pre::pre;

#[pre("is foo")]
fn foo() {}

#[pre]
fn main() {
    #[assure("is foo")]
    foo()
}
