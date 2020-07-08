use pre::pre;

fn foo() {}

mod nested {
    use pre::pre;

    #[pre("nested foo")]
    pub(super) fn foo() {}
}

mod other_nested {
    use pre::pre;

    #[pre("nested foo")]
    pub(super) fn foo() {}
}

#[pre]
fn main() {
    #[forward(nested)]
    #[forward(other_nested)]
    #[assure("nested foo", reason = "corresponding forward present")]
    foo();
}
