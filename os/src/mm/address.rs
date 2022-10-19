use core::fmt::{self, Debug, Formatter};
use core::ops::{Add, AddAssign};

use crate::config;

use super::page_table::PageTableEntry;

/// physical address
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - config::PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - config::PAGE_SIZE_BITS;

/// physical address
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct PhysAddr(usize);

/// virtual address
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct VirtAddr(usize);

/// physical page number
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct PhysPageNum(usize);

/// virtual page number
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct VirtPageNum(usize);

impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / config::PAGE_SIZE)
    }

    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 - 1 + config::PAGE_SIZE) / config::PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (config::PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / config::PAGE_SIZE)
    }

    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + config::PAGE_SIZE) / config::PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (config::PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

/// 每个页表都用 9 位索引的，因此有 `1 << 9 = 512` 个页表项
const NUM_PTE_PER_VIRT_PAGE: usize = 1 << 9;

impl PhysPageNum {
    /// 返回一个页表项定长数组的可变引用，代表多级页表中的一个节点
    pub fn as_mut_slice(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        // `PageTableEntry` 只是对 `usize` 的包装，实际上两者具有相同的内存布局，在 64 位的机器上即为 8 Byte
        // NUM_PTE_PER_VIRT_PAGE * core::mem::size_of::<PageTableEntry>() = 4KB = config::PAGE_SIZE
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut PageTableEntry,
                self::NUM_PTE_PER_VIRT_PAGE,
            )
        }
    }

    /// 返回一个字节数组的可变引用，可以以字节为粒度对物理页帧上的数据进行访问
    pub fn as_bytes_mut(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, config::PAGE_SIZE) }
    }

    /// 获取一个恰好放在一个物理页帧开头的类型为 `T` 的数据的可变引用
    pub fn as_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtPageNum {
    #[inline]
    pub fn one() -> Self {
        Self(1usize)
    }

    /// 取出虚拟页号的三级页索引，并按照从高到低的顺序返回 `[VPN2, VPN1, VPN0]`
    ///
    /// 在 SV39 模式中采用三级页表，即将 27 位的虚拟页号分为三个等长的部分，
    /// 第 26-18 位为三级索引 VPN2 ，第17-9 位为二级索引 VPN1 ，第 8-0 位为一级索引 VPN0
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn: usize = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & (self::NUM_PTE_PER_VIRT_PAGE - 1);
            vpn >>= 9;
        }
        idx
    }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << config::PAGE_SIZE_BITS)
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << config::PAGE_SIZE_BITS)
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl Add for VirtPageNum {
    type Output = VirtPageNum;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(
            self.0 + rhs.0 <= usize::MAX,
            "add overflow: left + right > VirtPageNum:MAX"
        );
        (self.0 + rhs.0).into()
    }
}

impl AddAssign for VirtPageNum {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

pub trait StepByOne {
    fn forward_one(start: Self) -> Self;
}

impl StepByOne for VirtPageNum {
    fn forward_one(start: Self) -> Self {
        start + Self(1usize)
    }
}

/// a simple interval structure for type T
///
/// 区间
#[derive(Copy, Clone)]
pub struct SimpleInterval<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    start: T,
    end: T,
}

impl<T> SimpleInterval<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { start, end }
    }

    pub fn start(&self) -> T {
        self.start
    }

    pub fn end(&self) -> T {
        self.end
    }
}

impl<T> IntoIterator for SimpleInterval<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRange<T>;

    fn into_iter(self) -> Self::IntoIter {
        SimpleRange::new(self.start, self.end)
    }
}

/// iterator for the simple interval structure
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    start: T,
    end: T,
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl<T> Iterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            let cur = self.start;
            Some(core::mem::replace(
                &mut self.start,
                <T as StepByOne>::forward_one(cur),
            ))
        }
    }
}

/// a simple interval structure for virtual page number
///
/// 一段虚拟页号的连续区间
pub type VPNInterval = SimpleInterval<VirtPageNum>;
