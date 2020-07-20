use pre::pre;

#[cfg_attr(target_endian = "big", pre("foo_big"))]
#[cfg_attr(target_endian = "little", pre("foo_little"))]
fn foo() {}

#[pre("bar")]
#[cfg_attr(any(target_endian = "big", target_endian = "little"), pre("baz"))]
fn bar() {}

#[pre]
fn main() {
    #[cfg_attr(target_endian = "big", assure("foo_big", reason = "is foo_big"))]
    #[cfg_attr(
        target_endian = "little",
        assure("foo_little", reason = "is foo_little")
    )]
    foo();

    #[assure("bar", reason = "is bar")]
    #[cfg_attr(
        any(target_endian = "big", target_endian = "little"),
        assure("baz", reason = "is baz")
    )]
    bar();
}
