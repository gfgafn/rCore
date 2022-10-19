//! Constants used in rCore

pub use crate::board::{CLOCK_FREQ, MMIO};

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
/// 物理内存的终止物理地址
pub const MEMORY_END: usize = 0x80800000;
/// 每物理个页面的大小
pub const PAGE_SIZE: usize = 0x1000;
/// 每物理个页页内偏移的位宽
pub const PAGE_SIZE_BITS: usize = 0xc;

/// 内核和应用地址空间共享的跳板页面的起始地址
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
///  Trap 上下文在应用地址空间中的虚拟地址
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
