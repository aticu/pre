use pre::pre;

mod a {
    pub(crate) mod b {
        pub(crate) mod c {
            use pre::pre;

            #[pre("must be foo")]
            pub(crate) fn foo<T>() {}
        }
    }
}

#[pre]
fn main() {
    #[assure("must be foo", reason = "is foo")]
    a::b::c::foo::<f64>();
}
