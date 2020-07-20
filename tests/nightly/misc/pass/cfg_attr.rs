use pre::pre;

#[cfg_attr(any(target_endian = "little", target_endian = "big"), pre("foo"))]
fn foo() {}

#[pre]
fn main() {
    #[cfg_attr(
        any(target_endian = "little", target_endian = "big"),
        assure("foo", reason = "is foo")
    )]
    foo();
}
