#![no_std]
#![no_main]
//! - `#![feature(panic_info_message)]`  
//!   panic! 时，获取其中的信息并打印
#![feature(panic_info_message)]

#[macro_use]
mod console;

mod batch;
mod lang_items;
mod logging;
mod sbi;
mod sync;
mod syscall;
mod trap;

use core::arch::global_asm;
use log::{debug, error, info, trace, warn};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn _start();
    fn boot_stack();
    fn boot_stack_top();
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    trap::init();
    batch::init();

    info!("load range: _start = {:#x}", _start as usize);
    info!(
        "boot_stack [{:#x}, {:#x}]",
        boot_stack as usize, boot_stack_top as usize
    );
    info!(".text [{:#x}, {:#x}]", stext as usize, etext as usize);
    info!(".rodata [{:#x}, {:#x}]", srodata as usize, erodata as usize);
    info!(".data [{:#x}, {:#x}]", sdata as usize, edata as usize);
    error!("Hello, world!");
    warn!("This is warnning message!");
    trace!("This is trace message!");
    debug!("This is debug message!");
    batch::run_next_app();
    // panic!("Shutdown machine!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}
