use pre::pre;

mod a {
    pub(crate) mod d {
        pub(crate) fn foo() {}
    }
}

mod c {
    pub(crate) mod d {
        use pre::pre;

        #[pre("must be foo")]
        pub(crate) fn foo() {}
    }
}

#[pre]
fn main() {
    #[forward(b -> c)]
    #[assure("must be foo", reason = "is foo")]
    a::d::foo();
}
