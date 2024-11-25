#![no_std]

use core::{alloc::Layout, ptr::NonNull};

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_next: usize,
    b_alloc: usize,
    b_end: usize,
    p_alloc: usize,
    p_next: usize,
    p_end: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        EarlyAllocator {
            start: 0,
            end: 0,
            b_next: 0,
            b_alloc: 0,
            b_end: 0,
            p_alloc: 0,
            p_next: 0,
            p_end: 0,
        }
    }

    fn increase_bytes(&mut self) -> AllocResult {
        let end = self.b_end + Self::PAGE_SIZE;
        if end > self.p_end {
            Err(AllocError::NoMemory)
        } else {
            self.b_end = end;
            Ok(())
        }
    }

    fn increase_pages(&mut self) -> AllocResult {
        let end = self.p_end - Self::PAGE_SIZE * self.total_pages();
        if end < self.b_end {
            Err(AllocError::NoMemory)
        } else {
            self.p_end = end;
            Ok(())
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    /// Initialize the allocator with a free memory region.
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_next = start;
        self.p_next = start + size;
        self.b_end = start;
        self.p_end = start + size;
        self.b_alloc = 0;
        self.p_alloc = 0;
    }

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Ok(())
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    /// Allocate memory with the given size (in bytes) and alignment.
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let alloc_start = align_up(self.b_next, layout.align());
        let alloc_end = alloc_start + layout.size();
        if alloc_end >= self.b_end && self.increase_bytes().is_err() {
            return Err(AllocError::NoMemory);
        }
        self.b_alloc += 1;
        self.b_next = alloc_end;
        Ok(NonNull::new(alloc_start as *mut u8).unwrap())
    }

    /// Deallocate memory at the given position, size, and alignment.
    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        self.b_alloc -= 1;
        if self.b_alloc == 0 {
            self.b_next = self.start;
        }
    }

    /// Returns total memory size in bytes.
    fn total_bytes(&self) -> usize {
        self.b_end - self.start
    }

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize {
        self.b_next - self.start
    }

    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize {
        self.b_end - self.b_next
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    /// The size of a memory page.
    const PAGE_SIZE: usize = PAGE_SIZE;

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % Self::PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / Self::PAGE_SIZE;
        let alloc_start = align_down(self.p_next - num_pages * Self::PAGE_SIZE, align_pow2);
        if alloc_start <= self.p_end && self.increase_pages().is_err() {
            return Err(AllocError::NoMemory);
        }
        self.p_alloc += 1;
        self.p_next = alloc_start;
        Ok(alloc_start)
    }

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        self.p_alloc -= 1;
        if self.p_alloc == 0 {
            self.p_next = self.end;
        }
    }

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize {
        (self.end - self.p_end) / Self::PAGE_SIZE
    }

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize {
        (self.end - self.p_next) / Self::PAGE_SIZE
    }

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize {
        (self.p_next - self.p_end) / Self::PAGE_SIZE
    }
}

const fn align_down(pos: usize, align: usize) -> usize {
    pos & !(align - 1)
}

const fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}
