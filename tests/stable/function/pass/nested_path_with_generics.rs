use pre::pre;

mod a {
    pub(crate) mod b {
        pub(crate) mod c {
            use pre::pre;

            #[pre("is foo")]
            pub(crate) fn foo<T>() {}
        }
    }
}

#[pre]
fn main() {
    #[assure("is foo", reason = "foo is always foo")]
    a::b::c::foo::<f64>();
}
