//! Types related to task management

use crate::config;

use super::TaskContext;

/// ```rust
/// pub struct TaskControlBlock {
///     pub task_status: TaskStatus,
///     pub task_cx: TaskContext,
///     pub lifecycle: TaskLifecycle,
///     pub syscall_times: [u32; config::MAX_SYSCALL_NUM],
/// }
/// ```
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    /// 任务的生命周期
    pub lifecycle: TaskLifecycle,
    /// 任务使用的系统调用及调用次数
    ///
    /// 在我们的实验中，系统调用号一定小于 `500`，
    /// 所以直接使用一个长为 `MAX_SYSCALL_NUM = 500` 的数组做桶计数。
    ///
    /// 数组索引即为对应系统调用的`syscall_id`，索引处的值即为调用次数
    pub syscall_times: [u32; config::MAX_SYSCALL_NUM],
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

/// 任务的生命周期
/// ```rust
/// struct TaskLifecycle {
///     /// 任务初始化时间，单位为 `ms`
///     pub init_time_ms: usize,
///     /// 任务首次运行时间，单位为 `ms`
///     pub first_run_time_ms: usize,
///     /// 任务退出时间，单位为 `ms`
///     pub exit_time_ms: usize,
/// }
/// ```
#[derive(Clone, Copy)]
pub struct TaskLifecycle {
    /// 任务初始化时间，单位为 `ms`
    pub init_time_ms: usize,
    /// 任务首次运行时间，单位为 `ms`
    pub first_run_time_ms: usize,
    /// 任务退出时间，单位为 `ms`
    pub exit_time_ms: usize,
}
