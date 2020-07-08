#![deny(warnings)]

use pre::pre;

fn foo() {}

mod nested {
    use pre::pre;

    #[pre("nested foo")]
    pub(super) fn foo() {}
}

mod nested_no_pre {
    pub(super) fn foo() {}
}

#[pre]
fn main() {
    foo();

    #[forward(nested)]
    #[assure("nested foo", reason = "corresponding forward present")]
    foo();

    #[forward(nested_no_pre -> nested)]
    #[assure("nested foo", reason = "corresponding forward present")]
    nested_no_pre::foo();

    use nested_no_pre::foo as bar;

    #[forward(bar -> nested::foo)]
    #[assure("nested foo", reason = "corresponding forward present")]
    bar();
}
