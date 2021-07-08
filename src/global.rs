#[cfg(feature = "allocator-api")]
use core::alloc::{Alloc, AllocErr};
use core::alloc::{GlobalAlloc, Layout};
use core::ops::{Deref, DerefMut};
#[cfg(feature = "allocator-api")]
use core::ptr::NonNull;

use Minmalloc;

/// An instance of a "global allocator" backed by `Minmalloc`
///
/// This API requires the `global` feature is activated, and this type
/// implements the `GlobalAlloc` trait in the standard library.
pub struct GlobalMinmalloc;

unsafe impl GlobalAlloc for GlobalMinmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        <Minmalloc>::malloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        <Minmalloc>::free(&mut get(), ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        <Minmalloc>::calloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        <Minmalloc>::realloc(&mut get(), ptr, layout.size(), layout.align(), new_size)
    }
}

#[cfg(feature = "allocator-api")]
unsafe impl Alloc for GlobalMinmalloc {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        get().alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        get().dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>, AllocErr> {
        Alloc::realloc(&mut *get(), ptr, layout, new_size)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        get().alloc_zeroed(layout)
    }
}

static mut MINMALLOC: Minmalloc = Minmalloc(::minmalloc::MINMALLOC_INIT);

struct Instance;

unsafe fn get() -> Instance {
    ::sys::acquire_global_lock();
    Instance
}

impl Deref for Instance {
    type Target = Minmalloc;
    fn deref(&self) -> &Minmalloc {
        unsafe { &MINMALLOC }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Minmalloc {
        unsafe { &mut MINMALLOC }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        ::sys::release_global_lock()
    }
}
