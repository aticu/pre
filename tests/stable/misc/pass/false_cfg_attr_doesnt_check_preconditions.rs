use pre::pre;

#[cfg_attr(all(target_endian = "little", target_endian = "big"), pre("foo"))]
fn foo() {}

#[pre]
fn main() {
    #[cfg_attr(
        all(target_endian = "little", target_endian = "big"),
        assure("fuu", reason = "is fuu")
    )]
    foo();
}
