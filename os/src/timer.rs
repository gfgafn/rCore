//! RISC-V timer-related functionality

use riscv::register;

use crate::{config, sbi};

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

/// read the `mtime` register
pub fn get_time() -> usize {
    register::time::read()
}

/// get current time in milliseconds
pub fn get_time_ms() -> usize {
    register::time::read() / (config::CLOCK_FREQ / MSEC_PER_SEC)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    sbi::set_timer(get_time() + config::CLOCK_FREQ / TICKS_PER_SEC);
}
