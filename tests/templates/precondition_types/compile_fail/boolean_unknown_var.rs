use pre::pre;

#[pre(value > 0)]
fn foo(valu: i32) {}

#[pre]
fn main() {
    #[assure(value > 0, reason = "42 > 0")]
    foo(42)
}
