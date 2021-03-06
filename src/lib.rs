//! The primary purpose of this crate is that it serves as the default memory
//! allocator for the `wasm32-unknown-unknown` target in the standard library.
//! Support for other platforms is largely untested and unused, but is used when
//! testing this crate.

#![cfg_attr(feature = "allocator-api", feature(allocator_api))]
#![cfg_attr(not(feature = "allocator-api"), allow(dead_code))]
#![no_std]
//#![deny(missing_docs)]

#[cfg(feature = "allocator-api")]
use core::alloc::{Alloc, AllocErr, Layout};
use core::cmp;
use core::ptr;

#[cfg(all(feature = "global", not(test)))]
pub use self::global::GlobalMinmalloc;

mod minmalloc;
#[cfg(all(feature = "global", not(test)))]
mod global;

pub struct Minmalloc(minmalloc::Minmalloc);

pub const MINMALLOC_INIT: Minmalloc = Minmalloc(minmalloc::MINMALLOC_INIT);

#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
mod sys;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod sys;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod sys;

impl Minmalloc {
    /// Creates a new instance of an allocator, same as `MINMALLOC_INIT`.
    pub fn new() -> Minmalloc {
        MINMALLOC_INIT
    }

    /// Allocates `size` bytes with `align` align.
    ///
    /// Returns a null pointer if allocation fails. Returns a valid pointer
    /// otherwise.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::alloc`
    /// method contracts.
    #[inline]
    pub unsafe fn malloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if align <= self.0.malloc_alignment() {
            self.0.malloc(size)
        } else {
            self.0.memalign(align, size)
        }
    }

    /// Same as `malloc`, except if the allocation succeeds it's guaranteed to
    /// point to `size` bytes of zeros.
    #[inline]
    pub unsafe fn calloc(&mut self, size: usize, align: usize) -> *mut u8 {
        let ptr = self.malloc(size, align);
        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    /// Deallocates a `ptr` with `size` and `align` as the previous request used
    /// to allocate it.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::dealloc`
    /// method contracts.
    #[inline]
    pub unsafe fn free(&mut self, ptr: *mut u8, size: usize, align: usize) {
        drop((size, align));
        self.0.free(ptr)
    }

    /// Reallocates `ptr`, a previous allocation with `old_size` and
    /// `old_align`, to have `new_size` and the same alignment as before.
    ///
    /// Returns a null pointer if the memory couldn't be reallocated, but `ptr`
    /// is still valid. Returns a valid pointer and frees `ptr` if the request
    /// is satisfied.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::realloc`
    /// method contracts.
    #[inline]
    pub unsafe fn realloc(
        &mut self,
        ptr: *mut u8,
        old_size: usize,
        old_align: usize,
        new_size: usize,
    ) -> *mut u8 {
        if old_align <= self.0.malloc_alignment() {
            self.0.realloc(ptr, new_size)
        } else {
            let res = self.malloc(new_size, old_align);
            if !res.is_null() {
                let size = cmp::min(old_size, new_size);
                ptr::copy_nonoverlapping(ptr, res, size);
                self.free(ptr, old_size, old_align);
            }
            res
        }
    }
}

#[cfg(feature = "allocator-api")]
unsafe impl Alloc for Minmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<ptr::NonNull<u8>, AllocErr> {
        let ptr = <Minmalloc>::malloc(self, layout.size(), layout.align());
        ptr::NonNull::new(ptr).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: ptr::NonNull<u8>, layout: Layout) {
        <Minmalloc>::free(self, ptr.as_ptr(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: ptr::NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<ptr::NonNull<u8>, AllocErr> {
        let ptr = <Minmalloc>::realloc(self, ptr.as_ptr(), layout.size(), layout.align(), new_size);
        ptr::NonNull::new(ptr).ok_or(AllocErr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<ptr::NonNull<u8>, AllocErr> {
        let ptr = <Minmalloc>::calloc(self, layout.size(), layout.align());
        ptr::NonNull::new(ptr).ok_or(AllocErr)
    }
}
