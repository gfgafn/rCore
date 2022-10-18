use lazy_static::*;

use core::fmt::{self, Debug, Formatter};

use ::alloc::vec::Vec;

use crate::{config, sync};

use super::{PhysAddr, PhysPageNum};

/// 物理页帧管理器 Trait
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// 实现了 `FrameAllocator` Trait 的物理页帧管理器
struct FrameAllocatorImpl<T: FrameAllocator>(T);

impl<T: FrameAllocator> FrameAllocatorImpl<T> {
    fn monomorphize() -> T {
        T::new()
    }
}

/// an implementation for frame allocator
// 栈式物理页帧管理策略
pub struct StackFrameAllocator {
    /// 空闲内存的起始物理页号
    current: usize,
    /// 空闲内存的结束物理页号
    end: usize,
    /// 保存被回收的物理页号
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.into();
        self.end = r.into();
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn: usize = ppn.into();
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|v| *v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: sync::UPSafeCell<StackFrameAllocator> =
        unsafe { sync::UPSafeCell::new(FrameAllocatorImpl::monomorphize()) };
}

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(config::MEMORY_END).floor(),
    );
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

/// deallocate a frame
fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

/// manage a frame which has the same lifecycle as the tracker
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // FIXME: 删除
        // let bytes_array = ppn.get_bytes_array();
        // for i in bytes_array {
        //     *i = 0;
        // }
        // page cleaning
        ppn.get_bytes_array().fill(0);
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        self::frame_dealloc(self.ppn);
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "FrameTracker:PPN={:#x}",
            <PhysPageNum as Into<usize>>::into(self.ppn)
        ))
    }
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame: FrameTracker = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame: FrameTracker = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
