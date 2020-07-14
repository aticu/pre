use pre::pre;

#[pre(unknown_keyword)]
fn foo(unknown_keyword: bool) {}

#[pre]
fn main() {
    #[assure(unknown_keyword, reason = "`unknown_keyword` is `true`")]
    foo(true);
}
