use pre::pre;

#[pre("some condition")]
fn foo() {}

#[pre]
fn main() {
    #[assure(
        "some condition",
        reason = "<specify the reason why you can assure this here>"
    )]
    foo()
}
