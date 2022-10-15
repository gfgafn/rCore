#![no_std]
#![no_main]

extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let current_timer = user_lib::get_time();
    let wait_for = current_timer + 3000;
    while user_lib::get_time() < wait_for {
        user_lib::yield_();
    }
    user_lib::println!("Test sleep OK!");
    0
}
