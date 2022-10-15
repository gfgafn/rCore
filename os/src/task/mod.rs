//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

pub use self::context::TaskContext;
pub use self::task::{TaskControlBlock, TaskStatus};

use lazy_static::*;

use crate::config::{APP_SIZE_LIMIT, MAX_APP_NUM, MAX_SYSCALL_NUM, USER_STACK_SIZE};
use crate::loader::{get_base_i, get_current_user_stack_bottom, get_num_app, init_app_cx};
use crate::timer;

use self::task::TaskLifecycle;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: crate::sync::UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            lifecycle: TaskLifecycle {init_time_ms: 0, first_run_time_ms: 0, exit_time_ms: 0},
            syscall_times:[0; MAX_SYSCALL_NUM]
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.lifecycle = TaskLifecycle {init_time_ms: timer::get_time_ms(), first_run_time_ms: 0, exit_time_ms: 0};
            task.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                crate::sync::UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.lifecycle.first_run_time_ms = timer::get_time_ms();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            self::switch::__switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
        inner.tasks[current].lifecycle.exit_time_ms = timer::get_time_ms();
    }

    /// Find next task to run and return app id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            if 0 != inner.tasks[next].lifecycle.first_run_time_ms {
                inner.tasks[next].lifecycle.first_run_time_ms = timer::get_time_ms();
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            core::mem::drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                self::switch::__switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            println!("All applications completed!");

            #[cfg(feature = "board_qemu")]
            use crate::board::QEMUExit;
            #[cfg(feature = "board_qemu")]
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
    }

    /// 检查一段内存范围是否在 `current_task` 的内存空间内
    fn check_address_within_current(&self, buf_addr: usize, len: usize) -> bool {
        let inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        let current_user_stack_bottom = get_current_user_stack_bottom(current_task);
        let current_user_stack_top = current_user_stack_bottom - USER_STACK_SIZE;
        let current_task_base_address = get_base_i(current_task);

        if current_user_stack_top <= buf_addr && buf_addr + len <= current_user_stack_bottom {
            return true;
        }
        if current_task_base_address <= buf_addr
            && buf_addr + len <= current_task_base_address + APP_SIZE_LIMIT
        {
            return true;
        }

        false
    }

    fn update_current_syscall_times(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let cur_task = inner.current_task;
        inner.tasks[cur_task].syscall_times[syscall_id] += 1;
    }

    fn currrent_control_block(&self) -> TaskControlBlock {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task]
    }
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// 对用户程序要输出的数据进行检查，使其仅能输出位于程序本身内存空间内的数据
pub fn check_sys_write_buffer(buf_addr: usize, len: usize) -> bool {
    TASK_MANAGER.check_address_within_current(buf_addr, len)
}

/// 更新当前运行的任务的系统调用次数
pub fn update_current_syscall_times(syscall_id: usize) {
    TASK_MANAGER.update_current_syscall_times(syscall_id)
}

pub fn get_current_control_block() -> TaskControlBlock {
    TASK_MANAGER.currrent_control_block()
}
