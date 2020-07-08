use pre::pre;

#[pre(14 + 20 + 8)]
fn foo(valu: i32) {}

#[pre]
fn main() {
    #[assure(14 + 20 + 8, reason = "math")]
    foo(42)
}
