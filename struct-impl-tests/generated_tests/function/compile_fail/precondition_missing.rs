use pre::{check_pre, pre};

#[pre(condition(valid_ptr(ptr)), condition("must point to `42`"))]
unsafe fn read_twice<T: Copy>(ptr: *const T) -> (T, T) {
    (std::ptr::read(ptr), std::ptr::read(ptr))
}

#[check_pre]
fn main() {
    let ptr: *const i32 = &42;

    let (_, _) = unsafe {
        #[assert_pre(condition(valid_ptr(ptr), reason = "`ptr` comes from a reference"))]
        read_twice(ptr)
    };
}
