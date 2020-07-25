//! Provides preconditions for the standard library.

macro_rules! define_libs {
    (
        $core_name:ident {
            $($core_item:item)*
        }

        $alloc_name:ident {
            $($alloc_item:item)*
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
        ///
        /// # What about primitives?
        ///
        /// You can find the preconditions for methods of the primitive types in `impl` blocks in
        /// the root `core` module. Currently preconditions are provided for the following
        /// primitives:
        ///
        /// - `*const T`: in `const_pointer` (`#[forward(impl pre::core::const_pointer)]`)
        /// - `*mut T`: in `mut_pointer` (`#[forward(impl pre::core::mut_pointer)]`)
        ///
        /// For more information on how to have these preconditions checked, have a look at the
        /// [documentation of the forward attribute](../attr.forward.html#impl-call).
        #[cfg(feature = "core")]
        #[pre::extern_crate(core)]
        #[pre::pre(no_doc)]
        pub mod $core_name {
            $($core_item)*
        }

        /// Precondition definitions for `unsafe` functions in the [`alloc` library](https://doc.rust-lang.org/alloc/index.html).
        ///
        /// It can be used as a drop-in replacement for it. For more information about it, you can
        /// read [its documentation](https://doc.rust-lang.org/alloc/index.html).
        ///
        /// # Is this complete?
        ///
        /// No, currently only a subset of `unsafe` functions actually have preconditions defined
        /// here. More may be added in the future. If you're missing something, please file an
        /// issue.
        ///
        /// **Nevertheless, all of the `alloc` library is still usable through this module**,
        /// but not all of the `unsafe` functions will have preconditions added to them.
        ///
        /// As a workaround, you can add the preconditions locally in your own crate using the
        /// [`extern_crate` attribute](../attr.extern_crate.html).
        ///
        /// # Why is it named `alloc_lib` in the documentation?
        ///
        /// If it were simply named `alloc` there would be a naming conflict with this module, so
        /// either of them had to have a different name.
        #[cfg(feature = "alloc")]
        #[pre::extern_crate(alloc_lib)]
        #[pre::pre(no_doc)]
        pub mod $alloc_name {
            $($alloc_item)*
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
        ///
        /// # What about primitives?
        ///
        /// You can find the preconditions for methods of the primitive types in `impl` blocks in
        /// the root `std` module. Currently preconditions are provided for the following
        /// primitives:
        ///
        /// - `*const T`: in `const_pointer` (`#[forward(impl pre::std::const_pointer)]`)
        /// - `*mut T`: in `mut_pointer` (`#[forward(impl pre::std::mut_pointer)]`)
        ///
        /// For more information on how to have these preconditions checked, have a look at the
        /// [documentation of the forward attribute](../attr.forward.html#impl-call).
        #[cfg(feature = "std")]
        #[pre::extern_crate(std)]
        #[pre::pre(no_doc)]
        pub mod $std_name {
            $($core_item)*

            $($alloc_item)*

            $($std_item)*
        }
    };
}

define_libs! {
    core {
        mod mem {
            impl<T> ManuallyDrop<T> {
                #[pre("this `ManuallyDrop` is not used again after this call")]
                unsafe fn take(slot: &mut ManuallyDrop<T>) -> T;
            }

            impl<T: ?Sized> ManuallyDrop<T> {
                #[pre("this `ManuallyDrop` is not used again after this call")]
                unsafe fn drop(slot: &mut ManuallyDrop<T>);
            }

            #[pre("I have read and understood https://doc.rust-lang.org/nightly/nomicon/transmutes.html")]
            unsafe fn transmute_copy<T, U>(src: &T) -> U;

            #[pre("an all-zero byte-pattern is a valid value of `T`")]
            unsafe fn zeroed<T>() -> T;

            impl<T> MaybeUninit<T> {
                #[pre("the `MaybeUninit` contains a fully initialized, valid value of `T`")]
                unsafe fn assume_init(self) -> T;
            }
        }

        impl<T> const_pointer<T> where T: ?Sized {
            #[pre("the starting and the resulting pointer are in bounds of the same allocated object")]
            #[pre("the computed offset, in bytes, does not overflow an `isize`")]
            #[pre("performing the addition does not result in overflow")]
            unsafe fn add(self, count: usize) -> *const T;

            #[pre(proper_align(self))]
            #[pre("`self` is null or `self` is a valid for reads")]
            #[pre("`self` is null or `self` points to an initialized value of type `T`")]
            #[pre("the memory referenced by the returned reference is not mutated by any pointer for the duration of `'a`, except inside a contained `UnsafeCell`")]
            unsafe fn as_ref<'a>(self) -> Option<&'a T>;

            #[pre(valid_ptr(self, r))]
            #[pre(valid_ptr(dest, w))]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dest` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(self))]
            #[pre(proper_align(dest))]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_to(self, dest: *mut T, count: usize);

            #[pre(valid_ptr(self, r))]
            #[pre(valid_ptr(dest, w))]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dest` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(self))]
            #[pre(proper_align(dest))]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `self` and `dest` do not overlap")]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_to_nonoverlapping(self, dest: *mut T, count: usize);

            #[pre("the starting and the resulting pointer are in bounds of the same allocated object")]
            #[pre("the computed offset, in bytes, does not overflow an `isize`")]
            #[pre("performing the offset does not result in overflow")]
            unsafe fn offset(self, count: isize) -> *const T;

            #[pre(valid_ptr(self, r))]
            #[pre(proper_align(self))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn read(self) -> T;

            #[pre(valid_ptr(self, r))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn read_unaligned(self) -> T;

            #[pre(valid_ptr(self, r))]
            #[pre(proper_align(self))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn read_volatile(self) -> T;

            #[pre("the starting and the resulting pointer are in bounds of the same allocated object")]
            #[pre("the computed offset, in bytes, does not overflow an `isize`")]
            #[pre("performing the subtraction does not result in overflow")]
            unsafe fn sub(self, count: usize) -> *const T;
        }

        impl<T> mut_pointer<T> where T: ?Sized {
            #[pre("the starting and the resulting pointer are in bounds of the same allocated object")]
            #[pre("the computed offset, in bytes, does not overflow an `isize`")]
            #[pre("performing the addition does not result in overflow")]
            unsafe fn add(self, count: usize) -> *mut T;

            #[pre(proper_align(self))]
            #[pre("`self` is null or `self` is a valid for both reads and writes")]
            #[pre("`self` is null or `self` points to an initialized value of type `T`")]
            #[pre("the memory referenced by the returned reference is not accessed by any pointer other than the returned reference for the duration of `'a`")]
            unsafe fn as_mut<'a>(self) -> Option<&'a mut T>;

            #[pre(proper_align(self))]
            #[pre("`self` is null or `self` is a valid for reads")]
            #[pre("`self` is null or `self` points to an initialized value of type `T`")]
            #[pre("the memory referenced by the returned reference is not mutated by any pointer for the duration of `'a`, except inside a contained `UnsafeCell`")]
            unsafe fn as_ref<'a>(self) -> Option<&'a T>;

            #[pre(valid_ptr(src, r))]
            #[pre(valid_ptr(self, w))]
            #[pre("`src` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(src))]
            #[pre(proper_align(self))]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_from(self, src: *const T, count: usize);

            #[pre(valid_ptr(src, r))]
            #[pre(valid_ptr(self, w))]
            #[pre("`src` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(src))]
            #[pre(proper_align(self))]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `src` and `self` do not overlap")]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_from_nonoverlapping(self, src: *const T, count: usize);

            #[pre(valid_ptr(self, r))]
            #[pre(valid_ptr(dest, w))]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dest` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(self))]
            #[pre(proper_align(dest))]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_to(self, dest: *mut T, count: usize);

            #[pre(valid_ptr(self, r))]
            #[pre(valid_ptr(dest, w))]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dest` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(self))]
            #[pre(proper_align(dest))]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `self` and `dest` do not overlap")]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_to_nonoverlapping(self, dest: *mut T, count: usize);

            #[pre(valid_ptr(self, r+w))]
            #[pre(proper_align(self))]
            #[pre("`self` points to a value that is valid for dropping")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn drop_in_place(self);

            #[pre("the starting and the resulting pointer are in bounds of the same allocated object")]
            #[pre("the computed offset, in bytes, does not overflow an `isize`")]
            #[pre("performing the offset does not result in overflow")]
            unsafe fn offset(self, count: isize) -> *const T;

            #[pre(valid_ptr(self, r))]
            #[pre(proper_align(self))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn read(self) -> T;

            #[pre(valid_ptr(self, r))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn read_unaligned(self) -> T;

            #[pre(valid_ptr(self, r))]
            #[pre(proper_align(self))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*self` isn't used after this call")]
            unsafe fn read_volatile(self) -> T;

            #[pre(valid_ptr(self, r+w))]
            #[pre(proper_align(self))]
            #[pre("`self` points to a properly initialized value of type `T`")]
            unsafe fn replace(self, src: T) -> T;

            #[pre("the starting and the resulting pointer are in bounds of the same allocated object")]
            #[pre("the computed offset, in bytes, does not overflow an `isize`")]
            #[pre("performing the subtraction does not result in overflow")]
            unsafe fn sub(self, count: usize) -> *const T;

            #[pre(valid_ptr(self, r+w))]
            #[pre(valid_ptr(with, r+w))]
            #[pre(proper_align(self))]
            #[pre(proper_align(with))]
            unsafe fn swap(self, with: *mut T);

            #[pre(valid_ptr(self, w))]
            #[pre("`self` is properly aligned")]
            unsafe fn write(self, val: T);

            #[pre(valid_ptr(self, w))]
            #[pre("`self` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(self))]
            #[pre("a valid value of `T` is written to `*self` or `*self` is never used")]
            unsafe fn write_bytes(self, val: u8, count: usize);

            #[pre(valid_ptr(self, w))]
            unsafe fn write_unaligned(self, val: T);

            #[pre(valid_ptr(self, w))]
            #[pre(proper_align(self))]
            unsafe fn write_volatile(self, val: T);
        }

        mod ptr {
            impl<T: ?Sized> NonNull<T> {
                #[pre(!ptr.is_null())]
                const unsafe fn new_unchecked(ptr: *mut T) -> Self;
            }

            #[pre(valid_ptr(src, r))]
            #[pre(valid_ptr(dst, w))]
            #[pre("`src` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dst` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(src))]
            #[pre(proper_align(dst))]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize);

            #[pre(valid_ptr(src, r))]
            #[pre(valid_ptr(dst, w))]
            #[pre("`src` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`dst` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(src))]
            #[pre(proper_align(dst))]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `src` and `dst` do not overlap")]
            #[pre("`T` is `Copy` or only the values in one of the regions are used after this call")]
            unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);

            #[pre(valid_ptr(to_drop, r+w))]
            #[pre(proper_align(to_drop))]
            #[pre("`to_drop` points to a value that is valid for dropping")]
            #[pre("`T` is `Copy` or the value at `*to_drop` isn't used after this call")]
            unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T);

            #[pre(valid_ptr(src, r))]
            #[pre(proper_align(src))]
            #[pre("`src` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*src` isn't used after this call")]
            unsafe fn read<T>(src: *const T) -> T;

            #[pre(valid_ptr(src, r))]
            #[pre("`src` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*src` isn't used after this call")]
            unsafe fn read_unaligned<T>(src: *const T) -> T;

            #[pre(valid_ptr(src, r))]
            #[pre(proper_align(src))]
            #[pre("`src` points to a properly initialized value of type `T`")]
            #[pre("`T` is `Copy` or the value at `*src` isn't used after this call")]
            unsafe fn read_volatile<T>(src: *const T) -> T;

            #[pre(valid_ptr(dst, r+w))]
            #[pre(proper_align(dst))]
            #[pre("`dst` points to a properly initialized value of type `T`")]
            unsafe fn replace<T>(dst: *mut T, src: T) -> T;

            #[pre(valid_ptr(x, r+w))]
            #[pre(valid_ptr(y, r+w))]
            #[pre(proper_align(x))]
            #[pre(proper_align(y))]
            unsafe fn swap<T>(x: *mut T, y: *mut T);

            #[pre(valid_ptr(x, r+w))]
            #[pre(valid_ptr(y, r+w))]
            #[pre("`x` is valid for `count * size_of::<T>()` bytes")]
            #[pre("`y` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(x))]
            #[pre(proper_align(y))]
            #[pre("the memory regions of size `count * size_of::<T>` pointed to by `x` and `y` do not overlap")]
            unsafe fn swap_nonoverlapping<T>(x: *mut T, y: *mut T, count: usize);

            #[pre(valid_ptr(dst, w))]
            #[pre("`dst` is properly aligned")]
            unsafe fn write<T>(dst: *mut T, src: T);

            #[pre(valid_ptr(dst, w))]
            #[pre("`dst` is valid for `count * size_of::<T>()` bytes")]
            #[pre(proper_align(dst))]
            #[pre("a valid value of `T` is written to `*dst` or `*dst` is never used")]
            unsafe fn write_bytes<T>(dst: *mut T, val: u8, count: usize);

            #[pre(valid_ptr(dst, w))]
            unsafe fn write_unaligned<T>(dst: *mut T, src: T);

            #[pre(valid_ptr(dst, w))]
            #[pre(proper_align(dst))]
            unsafe fn write_volatile<T>(dst: *mut T, src: T);
        }

        mod slice {
            #[pre(valid_ptr(data, r))]
            #[pre(proper_align(data))]
            #[pre("the allocated object at `data` is valid for `len * mem::size_of::<T>()` bytes")]
            #[pre("the memory referenced by the returned slice is not mutated by any pointer for the duration of `'a`, except inside a contained `UnsafeCell`")]
            #[pre(len * ::core::mem::size_of::<T>() <= isize::MAX as usize)]
            unsafe fn from_raw_parts<'a, T>(data: *const T, len: usize) -> &'a [T];

            #[pre(valid_ptr(data, r+w))]
            #[pre(proper_align(data))]
            #[pre("the allocated object at `data` is valid for `len * mem::size_of::<T>()` bytes")]
            #[pre("the memory referenced by the returned slice is not accessed by any pointer other than the returned slice for the duration of `'a`")]
            #[pre(len * ::core::mem::size_of::<T>() <= isize::MAX as usize)]
            unsafe fn from_raw_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T];
        }
    }

    alloc {
        mod string {
            impl String {
                #[pre("the content of the `Vec` is valid UTF-8 at the time the reference is dropped")]
                unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8>;

                #[pre("the memory at `buf` was allocated with the standard library allocator with an alignment of exactly 1")]
                #[pre(length <= capacity)]
                #[pre("`capacity` is the capacity that `buf` was allocated with")]
                #[pre("`buf` is not used after this call")]
                #[pre("the first `length` bytes at `buf` are valid UTF-8")]
                unsafe fn from_raw_parts(buf: *mut u8, length: usize, capacity: usize) -> String;

                #[pre("the content of `bytes` is valid UTF-8")]
                unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> String;
            }
        }

        mod vec {
            impl<T> Vec<T> {
                #[pre("`ptr` has been previously allocated via `String` or `Vec<T>`")]
                #[pre("`T` has the same size and alignment as what `ptr` was allocated with")]
                #[pre(length <= capacity)]
                #[pre("`capacity` is the capacity that `ptr` was allocated with")]
                #[pre("`ptr` is not used after this call")]
                unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Vec<T>;

                #[pre(new_len <= self.capacity())]
                #[pre("the elements at `old_len..new_len` are initialized")]
                unsafe fn set_len(&mut self, new_len: usize);
            }
        }
    }

    std {
    }
}
