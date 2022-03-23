#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

// All integration tests are their own executables and completely separate from our main.rs.
// This means that each test needs to define its own entry point function and
// crate attributes (no_std, no_main, test_runner, etc.) again.

#[no_mangle]// don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop { }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
