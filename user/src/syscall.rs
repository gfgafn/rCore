use core::arch::asm;

use crate::{TaskInfo, TimeVal};

pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_GETTIMEOFDAY: usize = 169;
pub const SYSCALL_TASK_INFO: usize = 410;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

/// 功能：将内存中缓冲区中的数据写入文件。
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buffer` 表示内存中缓冲区的起始地址；
/// 返回值：返回成功写入的长度。
/// syscall ID：64
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`exit_code` 表示应用程序的返回值。
/// 返回值：
/// syscall ID：93
pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

/// 功能：获取当前的时间，保存到 `TimeVal` 结构体 `ts` 中，`_tz` 在我们的实现中忽略
///
/// 返回值：返回是否执行成功，成功则返回 `0`
///
/// syscall ID：`169`
pub fn sys_get_time(time: &mut TimeVal, tz: usize) -> isize {
    syscall(SYSCALL_GETTIMEOFDAY, [time as *mut _ as usize, tz, 0])
}

/// 查询当前正在执行的任务信息。
pub fn sys_task_info(info: &mut TaskInfo) -> isize {
    syscall(SYSCALL_TASK_INFO, [info as *mut _ as usize, 0, 0])
}
