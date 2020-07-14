use pre::pre;

#[pre("baz")]
fn foo() -> [u8; 8] {
    [0; 8]
}

#[pre("baz")]
fn bar() -> usize {
    0
}

#[pre]
fn main() {
    #[assure("baz", reason = "is baz")]
    foo()[bar()];
}
