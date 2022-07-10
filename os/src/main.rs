#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;

mod lang_items;
mod sbi;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn _start();
    fn boot_stack();
    fn boot_stack_top();
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
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
    panic!("Shutdown machine!");
}

fn clear_bss() {
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) })
}
