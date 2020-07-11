use pre::pre;

#[pre("is foo")]
fn foo() {}

#[pre]
fn main() {
    #[assure("is foo", reason = "bar is always foo")]
    foo()
}
