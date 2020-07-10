use pre::pre;

#[pre::extern_crate(std)]
mod pre_std {
    mod ptr {
        #[pre(valid_ptr(src, r))]
        #[pre("`src` must point to a properly initialized value of type `T`")]
        unsafe fn read_unaligned<T>(src: *const T) -> T;

        #[pre(valid_ptr(dst, w))]
        unsafe fn write_unaligned<T>(dst: *mut T, src: T);

        impl<T> NonNull<T> {
            #[pre(!ptr.is_null())]
            const unsafe fn new_unchecked(ptr: *mut T) -> NonNull<T>;
        }
    }
}

#[pre]
fn main() {
    let mut val = 0;

    #[assure(valid_ptr(dst, w), reason = "`dst` is a reference")]
    unsafe {
        pre_std::ptr::write_unaligned(&mut val, 42)
    };
    assert_eq!(val, 42);

    {
        use std::ptr::read_unaligned;

        #[forward(pre_std::ptr)]
        #[assure(valid_ptr(src, r), reason = "`src` is a reference")]
        #[assure(
            "`src` must point to a properly initialized value of type `T`",
            reason = "`src` is a reference"
        )]
        let result = unsafe { read_unaligned(&mut val) };

        assert_eq!(result, 42);
    }

    {
        #[forward(std -> pre_std)]
        #[assure(valid_ptr(src, r), reason = "`src` is a reference")]
        #[assure(
            "`src` must point to a properly initialized value of type `T`",
            reason = "`src` is a reference"
        )]
        let result = unsafe { std::ptr::read_unaligned(&mut val) };

        assert_eq!(result, 42);
    }

    #[forward(impl pre_std::ptr::NonNull)]
    #[assure(!ptr.is_null(), reason = "`ptr` is a reference")]
    let non_null = unsafe { pre_std::ptr::NonNull::new_unchecked(&mut val) };

    let std_non_null = unsafe { std::ptr::NonNull::new_unchecked(&mut val) };

    assert_eq!(non_null, std_non_null);
}
