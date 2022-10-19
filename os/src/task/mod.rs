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
use self::task::{TaskControlBlock, TaskStatus};

use lazy_static::*;

use crate::loader::{get_app_data, get_num_app};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use ::alloc::vec::Vec;

// use self::task::TaskLifecycle;

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
    inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        // let mut tasks = [TaskControlBlock {
        //     task_cx: TaskContext::zero_init(),
        //     task_status: TaskStatus::UnInit,
        //     lifecycle: TaskLifecycle {init_time_ms: 0, first_run_time_ms: 0, exit_time_ms: 0},
        //     syscall_times:[0; MAX_SYSCALL_NUM]
        // }; MAX_APP_NUM];
        // for (i, task) in tasks.iter_mut().enumerate() {
        //     task.task_cx = TaskContext::goto_restore(init_app_cx(i));
        //     task.lifecycle = TaskLifecycle {init_time_ms: timer::get_time_ms(), first_run_time_ms: 0, exit_time_ms: 0};
        //     task.task_status = TaskStatus::Ready;
        // }
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
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
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        // task0.lifecycle.first_run_time_ms = timer::get_time_ms();
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
        // inner.tasks[current].lifecycle.exit_time_ms = timer::get_time_ms();
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            // if 0 != inner.tasks[next].lifecycle.first_run_time_ms {
            //     inner.tasks[next].lifecycle.first_run_time_ms = timer::get_time_ms();
            // }
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

    // /// 检查一段内存范围是否在 `current_task` 的内存空间内
    // fn check_address_within_current(&self, buf_addr: usize, len: usize) -> bool {
    //     let inner = self.inner.exclusive_access();
    //     let current_task = inner.current_task;
    //     let current_user_stack_bottom = get_current_user_stack_bottom(current_task);
    //     let current_user_stack_top = current_user_stack_bottom - USER_STACK_SIZE;
    //     let current_task_base_address = get_base_i(current_task);

    //     if current_user_stack_top <= buf_addr && buf_addr + len <= current_user_stack_bottom {
    //         return true;
    //     }
    //     if current_task_base_address <= buf_addr
    //         && buf_addr + len <= current_task_base_address + APP_SIZE_LIMIT
    //     {
    //         return true;
    //     }

    //     false
    // }

    // fn update_current_syscall_times(&self, syscall_id: usize) {
    //     let mut inner = self.inner.exclusive_access();
    //     let cur_task = inner.current_task;
    //     inner.tasks[cur_task].syscall_times[syscall_id] += 1;
    // }

    // fn currrent_control_block(&self) -> TaskControlBlock {
    //     let inner = self.inner.exclusive_access();
    //     inner.tasks[inner.current_task]
    // }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}
