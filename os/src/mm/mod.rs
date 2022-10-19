//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

pub(crate) use address::{PhysPageNum, VirtAddr};
pub(crate) use memory_set::remap_test;
pub(crate) use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub(crate) use page_table::translated_byte_buffer;

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

/// initiate heap allocator, frame allocator and kernel space
pub(crate) fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    memory_set::KERNEL_SPACE.exclusive_access().activate();
}
