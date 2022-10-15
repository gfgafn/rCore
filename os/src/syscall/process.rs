//! Process management syscalls

use crate::{config, task, timer};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    crate::task::exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    crate::task::suspend_current_and_run_next();
    0
}

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// 功能：获取当前的时间，保存到 `TimeVal` 结构体 `ts` 中，`_tz` 在我们的实现中忽略
///
/// 返回值：返回是否执行成功，成功则返回 `0`
///
/// syscall ID：`169`
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    unsafe {
        if let Some(ts) = ts.as_mut() {
            let us = timer::get_time_us();
            ts.sec = us / timer::MICRO_PER_SEC;
            ts.usec = us % timer::MICRO_PER_SEC;
            return 0;
        }
        -1
    }
}

/// 任务信息。包括任务状态、任务使用的系统调用及调用次数、任务总运行时长（单位`ms`）
pub struct TaskInfo {
    /// 任务状态
    status: task::TaskStatus,
    /// 任务使用的系统调用及调用次数
    syscall_times: [u32; config::MAX_SYSCALL_NUM],
    /// 任务总运行时长（单位`ms`）
    time: usize,
}

/// 查询当前正在执行的任务信息
///
/// 返回值：返回是否执行成功，成功则返回 `0`
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    unsafe {
        if let Some(ti) = ti.as_mut() {
            let task_info: task::TaskControlBlock = task::get_current_control_block();
            ti.status = task_info.task_status;
            ti.syscall_times = task_info.syscall_times;
            ti.time = timer::get_time_ms() - task_info.lifecycle.first_run_time_ms;
            return 0;
        }
    }
    -1
}
