//! Provides preconditions for the standard library.

macro_rules! define_libs {
    (
        $core_name:ident {
            $($core_item:item)*
        }

        $std_name:ident {
            $($std_item:item)*
        }
    ) => {
        /// Precondition definitions for `unsafe` functions in the [`core` library](https://doc.rust-lang.org/core/index.html).
        ///
        /// It can be used as a drop-in replacement for it. For more information about it, you can
        /// read [its documentation](https://doc.rust-lang.org/core/index.html).
        ///
        /// # Is this complete?
        ///
        /// No, currently only a subset of `unsafe` functions actually have preconditions defined
        /// here. More may be added in the future. If you're missing something, please file an
        /// issue.
        ///
        /// **Nevertheless, all of the `core` library is still usable through this module**,
        /// but not all of the `unsafe` functions will have preconditions added to them.
        ///
        /// As a workaround, you can add the preconditions locally in your own crate using the
        /// [`extern_crate` attribute](../attr.extern_crate.html).
        #[cfg(feature = "core")]
        #[pre::extern_crate(core)]
        #[pre::pre(no_doc)]
        pub mod $core_name {
            $($core_item)*
        }

        /// Precondition definitions for `unsafe` functions in the [`std` library](https://doc.rust-lang.org/std/index.html).
        ///
        /// It can be used as a drop-in replacement for it. For more information about it, you can
        /// read [its documentation](https://doc.rust-lang.org/std/index.html).
        ///
        /// # Is this complete?
        ///
        /// No, currently only a subset of `unsafe` functions actually have preconditions defined
        /// here. More may be added in the future. If you're missing something, please file an
        /// issue.
        ///
        /// **Nevertheless, all of the `std` library is still usable through this module**,
        /// but not all of the `unsafe` functions will have preconditions added to them.
        ///
        /// As a workaround, you can add the preconditions locally in your own crate using the
        /// [`extern_crate` attribute](../attr.extern_crate.html).
        #[cfg(feature = "std")]
        #[pre::extern_crate(std)]
        #[pre::pre(no_doc)]
        pub mod $std_name {
            $($core_item)*

            $($std_item)*
        }
    };
}

define_libs! {
    core {
        mod ptr {
            impl<T: ?Sized> NonNull<T> {
                #[pre(!ptr.is_null())]
                const unsafe fn new_unchecked(ptr: *mut T) -> Self;
            }

            #[pre(valid_ptr(src, r))]
            #[pre(valid_ptr(dst, w))]
            #[pre("`src` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dst` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`src` is properly aligned")]
            #[pre("`dst` is properly aligned")]
            unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize);

            #[pre(valid_ptr(src, r))]
            #[pre(valid_ptr(dst, w))]
            #[pre("`src` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dst` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`src` is properly aligned")]
            #[pre("`dst` is properly aligned")]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `src` and `dst` do not overlap")]
            unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);

            #[pre(valid_ptr(to_drop, r+w))]
            #[pre("`to_drop` is properly aligned")]
            #[pre("`to_drop` points to a value that is valid for dropping")]
            unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T);

            #[pre(valid_ptr(src, r))]
            #[pre("`src` is properly aligned")]
            #[pre("`src` points to a properly initialized value of type `T`")]
            unsafe fn read<T>(src: *const T) -> T;

            #[pre(valid_ptr(src, r))]
            #[pre("`src` points to a properly initialized value of type `T`")]
            unsafe fn read_unaligned<T>(src: *const T) -> T;

            #[pre(valid_ptr(src, r))]
            #[pre("`src` is properly aligned")]
            #[pre("`src` points to a properly initialized value of type `T`")]
            unsafe fn read_volatile<T>(src: *const T) -> T;

            #[pre(valid_ptr(dst, r+w))]
            #[pre("`dst` is properly aligned")]
            #[pre("`dst` points to a properly initialized value of type `T`")]
            unsafe fn replace<T>(dst: *mut T, src: T) -> T;

            #[pre(valid_ptr(x, r+w))]
            #[pre(valid_ptr(y, r+w))]
            #[pre("`x` is properly aligned")]
            #[pre("`y` is properly aligned")]
            unsafe fn swap<T>(x: *mut T, y: *mut T);

            #[pre(valid_ptr(x, r+w))]
            #[pre(valid_ptr(y, r+w))]
            #[pre("`x` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`y` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`x` is properly aligned")]
            #[pre("`y` is properly aligned")]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `x` and `y` do not overlap")]
            unsafe fn swap_nonoverlapping<T>(x: *mut T, y: *mut T, count: usize);

            #[pre(valid_ptr(dst, w))]
            #[pre("`dst` is properly aligned")]
            unsafe fn write<T>(dst: *mut T, src: T);

            #[pre(valid_ptr(dst, w))]
            #[pre("`dst` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dst` is properly aligned")]
            unsafe fn write_bytes<T>(dst: *mut T, val: u8, count: usize);

            #[pre(valid_ptr(dst, w))]
            unsafe fn write_unaligned<T>(dst: *mut T, src: T);

            #[pre(valid_ptr(dst, w))]
            #[pre("`dst` is properly aligned")]
            unsafe fn write_volatile<T>(dst: *mut T, src: T);
        }
    }

    std {
    }
}
