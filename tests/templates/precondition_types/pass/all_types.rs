use pre::pre;

#[pre("`some_ptr` is from a reference")]
#[pre(valid_ptr(some_ptr, r))]
#[pre(proper_align(some_ptr))]
#[pre(!some_ptr.is_null())]
fn foo<T>(some_ptr: *const T) {}

#[pre]
fn main() {
    #[assure(valid_ptr(some_ptr, r), reason = "it is from a reference")]
    #[assure(!some_ptr.is_null(), reason = "it is from a reference")]
    #[assure("`some_ptr` is from a reference", reason = "it is")]
    #[assure(proper_align(some_ptr), reason = "it is from a reference")]
    foo(&42)
}
