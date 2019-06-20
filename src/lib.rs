#![no_std]

//! qimalloc aka. Quick Incremental (Wasteful) Memory Allocator
//!
//! This memory allocator will not release any memory.
//! Its main use case is in short-lived environment, such as [WebAssembly](https://github.com/webassembly) binaries.
//!
//! **Experimental: do not use this in any real world use case!**
//!
//! If you are looking for a small, yet powerful allocator, we suggest to check out [wee_alloc](https://github.com/rustwasm/wee_alloc).
//!
//! To use qimalloc, simply include it as a dependency in your project.
//!
//! # Examples
//! ```ignore
//! extern crate qimalloc;
//!
//! #[global_allocator]
//! static ALLOC: qimalloc::QIMalloc = qimalloc::QIMalloc::INIT;
//!
//! pub fn main() {
//! }
//! ```
//!
//! Please note this example will only work on wasm32 targets as support for anything else is not implemented at this time.
//!

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32;

#[cfg(feature = "nightly")]
use core::alloc::AllocErr;

#[cfg(not(feature = "nightly"))]
pub struct AllocErr;

pub struct QIMalloc {
    // Pointer to last allocated byte
    ptr: UnsafeCell<*mut u8>,
}

const PAGE_SIZE: usize = 65536;

fn round_to_align(size: usize, align: usize) -> usize {
    if size % align == 0 {
        size
    } else {
        size + (align - (size % align))
    }
}

unsafe impl Sync for QIMalloc {}

#[cfg(target_arch = "wasm32")]
impl QIMalloc {
    pub const INIT: Self = QIMalloc {
        ptr: UnsafeCell::new(0 as *mut u8),
    };

    unsafe fn alloc_impl(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if layout.size() == 0 || layout.align() == 0 {
            return Err(AllocErr);
        }

        let size = round_to_align(layout.size(), layout.align());

        let ptr = self.ptr.get();
        let cur_pages = wasm32::memory_size(0);
        let end = (cur_pages * PAGE_SIZE) as *mut u8;

        // If first time, start at end of initial allocated memory
        if *ptr == 0 as *mut u8 {
            *ptr = end;
        }

        // Translated to rust from:
        // https://github.com/poemm/C_ewasm_contracts/blob/a3276b1242c22f275862869572e77104f1895974/src/ewasm.h#L128
        let total_bytes_needed = (*ptr as usize) + size;
        // Allocate more memory if necessary
        if total_bytes_needed > end as usize {
            let total_pages_needed = round_to_align(total_bytes_needed, PAGE_SIZE) / PAGE_SIZE;
            let pages = total_pages_needed - cur_pages;
            self.alloc_pages(pages)?;
        }

        *ptr = total_bytes_needed as *mut u8;

        Ok((total_bytes_needed - size) as *mut u8)
    }

    fn alloc_pages(&self, pages: usize) -> Result<*mut u8, AllocErr> {
        let ptr = wasm32::memory_grow(0, pages);
        if ptr != usize::max_value() {
            let ptr = (ptr * PAGE_SIZE) as *mut u8;
            Ok(ptr as *mut u8)
        } else {
            Err(AllocErr)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl QIMalloc {
    pub const INIT: Self = QIMalloc {
        ptr: UnsafeCell::new(0 as *mut u8),
    };

    unsafe fn alloc_impl(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
        unimplemented!()
    }
}

unsafe impl GlobalAlloc for QIMalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.alloc_impl(layout) {
            Ok(ptr) => ptr,
            Err(AllocErr) => null_mut(),
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
