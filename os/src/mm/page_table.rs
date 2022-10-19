use bitflags::*;

use ::alloc::vec;
use ::alloc::vec::Vec;

use super::{frame_alloc, FrameTracker, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

/// page table structure
pub struct PageTable {
    /// 根节点的物理页号
    root_ppn: PhysPageNum,
    /// 页表所有的节点（包括根节点）所在的物理页帧
    frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    pub fn new() -> Self {
        let frame: FrameTracker = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    // TODO: 修改命名 find_pte_or_create
    /// 在多级页表找到一个虚拟页号对应的页表项的可变引用。如果在遍历的过程中发现有节点尚未创建则会新建一个节点
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs: [usize; 3] = vpn.indexes();
        let mut ppn: PhysPageNum = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, &vpn) in idxs.iter().enumerate() {
            let pte: &mut PageTableEntry = &mut ppn.get_pte_array()[vpn];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame: FrameTracker = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    /// 当找不到合法叶子节点的时候不会新建叶子节点而是直接返回 `None` 即查找失败
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs: [usize; 3] = vpn.indexes();
        let mut ppn: PhysPageNum = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        // 这里采用一种最简单的 恒等映射 (Identical Mapping) ，即对于物理内存上的每个物理页帧，
        // 我们都在多级页表中用一个与其物理页号相等的虚拟页号来映射。
        for (i, &vpn) in idxs.iter().enumerate() {
            let pte: &mut PageTableEntry = &mut ppn.get_pte_array()[vpn];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    /// 建立虚实地址映射关系
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte: &mut PageTableEntry = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    /// 拆除虚实地址映射关系
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte: &mut PageTableEntry = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    /// 如果能够找到页表项，那么它会将页表项拷贝一份并返回，否则就返回一个 `None`
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).copied()
    }

    /// 按照 `satp CSR` 格式要求 构造一个无符号 64 位无符号整数，使得其分页模式为 SV39 ，
    /// 且将当前多级页表的根节点所在的物理页号填充进去
    pub fn token(&self) -> usize {
        8usize << 60 | <PhysPageNum as Into<usize>>::into(self.root_ppn)
    }

    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
///
/// SV39 分页模式下的页表项，其中 `[53 : 10]` 这 44 位是物理页号，最低的 8 位 `[7 ：0]` 则是标志位，
/// 控制页表项是否合法、控制索引到这个页表项的对应虚拟页面是否允许读/写/执行等
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: <PhysPageNum as Into<usize>>::into(ppn) << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    /// 物理页号
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }

    /// 标志位，控制页表项是否合法、控制索引到这个页表项的对应虚拟页面是否允许读/写/执行等
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    /// 页表项是否合法
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    /// 索引到这个页表项的对应虚拟页面是否允许读
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    /// 索引到这个页表项的对应虚拟页面是否允许写
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    /// 索引到这个页表项的对应虚拟页面是否允许执行
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// translate a pointer to a mutable u8 Vec through page table
///
/// 将应用地址空间中一个缓冲区转化为在内核空间中能够直接访问的形式
///
/// 参数中的 `token` 是某个应用地址空间的 `token` ， `ptr` 和 `len` 则
/// 分别表示该地址空间中的一段缓冲区的起始地址和长度(注：这个缓冲区的应用虚拟地址范围是连续的)
///
/// 返回一组可以在内核空间中直接访问的字节数组切片（注：这个缓冲区的内核虚拟地址范围有可能是不连续的）
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v: Vec<&mut [u8]> = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn: VirtPageNum = start_va.floor();
        let ppn: PhysPageNum = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}
