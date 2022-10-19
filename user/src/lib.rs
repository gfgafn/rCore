#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

pub use syscall::{SYSCALL_EXIT, SYSCALL_TASK_INFO, SYSCALL_WRITE, SYSCALL_YIELD};

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

const MAX_SYSCALL_NUM: usize = 500;

#[derive(Debug)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

impl TaskInfo {
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::UnInit,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0,
        }
    }
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    crate::syscall::sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    crate::syscall::sys_exit(exit_code)
}

pub fn yield_() -> isize {
    crate::syscall::sys_yield()
}

pub fn get_time() -> isize {
    syscall::sys_get_time()
}

/// 查询当前正在执行的任务信息，将其写入入参 `info`, 成功执行返回 `0`
pub fn task_info(info: &mut TaskInfo) -> isize {
    crate::syscall::sys_task_info(info)
}
