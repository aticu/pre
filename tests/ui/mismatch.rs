#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

use pre::{assert_precondition, pre};

#[pre(condition(valid_ptr(ptr)), condition("must point to `42`"))]
unsafe fn read_twice<T: Copy>(ptr: *const T) -> (T, T) {
    (std::ptr::read(ptr), std::ptr::read(ptr))
}

fn main() {
    let ptr: *const i32 = &42;

    let (_, _) = unsafe {
        #[assert_precondition(holds(valid_ptr(ptr), reason = "`ptr` comes from a reference"))]
        read_twice(ptr)
    };
}
