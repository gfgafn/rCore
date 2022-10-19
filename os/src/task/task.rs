//! Types related to task management

use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use super::TaskContext;
use crate::mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};

// /// ```rust
// /// pub struct TaskControlBlock {
// ///     pub task_status: TaskStatus,
// ///     pub task_cx: TaskContext,
// ///     pub lifecycle: TaskLifecycle,
// ///     pub syscall_times: [u32; config::MAX_SYSCALL_NUM],
// /// }
// /// ```
/// task control block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    // /// 任务的生命周期
    // pub lifecycle: TaskLifecycle,
    // /// 任务使用的系统调用及调用次数
    // ///
    // /// 在我们的实验中，系统调用号一定小于 `500`，
    // /// 所以直接使用一个长为 `MAX_SYSCALL_NUM = 500` 的数组做桶计数。
    // ///
    // /// 数组索引即为对应系统调用的`syscall_id`，索引处的值即为调用次数
    // pub syscall_times: [u32; config::MAX_SYSCALL_NUM],
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

// /// 任务的生命周期
// /// ```rust
// /// struct TaskLifecycle {
// ///     /// 任务初始化时间，单位为 `ms`
// ///     pub init_time_ms: usize,
// ///     /// 任务首次运行时间，单位为 `ms`
// ///     pub first_run_time_ms: usize,
// ///     /// 任务退出时间，单位为 `ms`
// ///     pub exit_time_ms: usize,
// /// }
// /// ```
// #[derive(Clone, Copy)]
// pub struct TaskLifecycle {
//     /// 任务初始化时间，单位为 `ms`
//     pub init_time_ms: usize,
//     /// 任务首次运行时间，单位为 `ms`
//     pub first_run_time_ms: usize,
//     /// 任务退出时间，单位为 `ms`
//     pub exit_time_ms: usize,
// }
