use pre::pre;

#[pre("is bar")]
unsafe fn foo() {}

#[pre]
fn main() {
    unsafe { foo() }
}
