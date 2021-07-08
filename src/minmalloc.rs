use core::mem;
use core::ptr;

use sys;

pub struct Minmalloc {
    total_allocated: usize,
}

unsafe impl Send for Minmalloc {}

pub const MINMALLOC_INIT: Minmalloc = Minmalloc {
    total_allocated: 0,
};

#[repr(C)]
struct Chunk {
    asize: usize, // size of system allocation
    start: isize, // start of system allocation relative to data
    data: *mut u8,
}

fn align_up(a: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (a + (alignment - 1)) & !(alignment - 1)
}

impl Minmalloc {
    pub fn malloc_alignment(&self) -> usize {
        2 * mem::size_of::<usize>()
    }

    pub unsafe fn malloc(&mut self, size: usize) -> *mut u8 {
        let asize = align_up(size + 2 * mem::size_of::<usize>(), sys::page_size());
        let (tbase, _tsize, _flags) = sys::alloc(asize);
        if tbase.is_null() {
            return tbase;
        }

        let allocated = tbase as *mut Chunk;
        (*allocated).asize = asize;
        (*allocated).start = -2 * (mem::size_of::<usize>() as isize);
        self.total_allocated = self.total_allocated + asize;
        return Chunk::to_mem(allocated);
    }

    pub unsafe fn realloc(&mut self, oldmem: *mut u8, bytes: usize) -> *mut u8 {
        let chunk = Chunk::from_mem(oldmem);
        let asize = (*chunk).asize;
        let start = (*chunk).start;

        let old_size = asize - (-start as usize);
        if bytes <= old_size {
            return oldmem;
        }

        let newmem = self.malloc(bytes);
        ptr::copy_nonoverlapping(oldmem, newmem, old_size);
        self.free(oldmem);
        return newmem;
    }

    pub unsafe fn free(&mut self, mem: *mut u8) {
        let chunk = Chunk::from_mem(mem);
        let asize = (*chunk).asize;
        let start = (*chunk).start;
        self.total_allocated = self.total_allocated - asize;
        sys::free((chunk as *mut u8).offset(start), asize);
    }

    // Only call this with power-of-two alignment and alignment >
    // `self.malloc_alignment()`
    pub unsafe fn memalign(&mut self, alignment: usize, bytes: usize) -> *mut u8 {
        let asize = align_up(bytes + alignment + 2 * mem::size_of::<usize>(), sys::page_size());
        let (tbase, _tsize, _flags) = sys::alloc(asize);
        if tbase.is_null() {
            return tbase;
        }
        let tbase_num = tbase as *const u8 as usize;
        let tbase_num_aligned = align_up(tbase_num + 2 * mem::size_of::<usize>(), alignment);
        let tbase_aligned = tbase.offset((tbase_num_aligned - tbase_num) as isize);

        let chunk = Chunk::from_mem(tbase_aligned);
        (*chunk).asize = asize;
        (*chunk).start = -((tbase_num_aligned - tbase_num) as isize);
        self.total_allocated = self.total_allocated + asize;
        return tbase_aligned;
    }
}

impl Chunk {
    unsafe fn to_mem(me: *mut Chunk) -> *mut u8 {
        (me as *mut u8).offset(2 * (mem::size_of::<usize>() as isize))
    }

    unsafe fn from_mem(mem: *mut u8) -> *mut Chunk {
        mem.offset(-2 * (mem::size_of::<usize>() as isize)) as *mut Chunk
    }
}
