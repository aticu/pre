use pre::pre;

mod a {
    pub(crate) mod d {
        pub(crate) fn foo() {}
    }
}

mod c {
    pub(crate) mod d {
        use pre::pre;

        #[pre("is foo")]
        pub(crate) fn foo() {}
    }
}

#[pre]
fn main() {
    #[forward(b -> c)]
    #[assure("is foo", reason = "foo is always foo")]
    a::d::foo();
}
