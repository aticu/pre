use pre::pre;

#[pre(no_debug_assert)]
#[pre(abc > foo)]
#[pre(27 + 25)]
fn foo() {}

#[pre]
fn main() {
    #[assure(abc > foo, reason = "unknown idents are not a problem, because of `no_debug_assert`")]
    #[assure(27 + 25, reason = "non-bool expr is not a problem, because of `no_debug_assert`")]
    foo();
}
