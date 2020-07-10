#[pre::extern_crate(std)]
mod pre_std {
    impl<T> ptr::NonNull<T> {
        #[pre(!ptr.is_null())]
        const unsafe fn new_unchecked(ptr: *mut T) -> NonNull<T>;
    }

    impl foo::bar::Baz {}
}

fn main() {}
