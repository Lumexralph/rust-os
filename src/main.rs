#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

// function to handle panic, `!` means a function
// that does not return control to its caller.
// it just doesn't return. A function that returns
// nothing gives an empty tuple ().
//
// The PanicInfo parameter contains the file and line
// where the panic happened and the optional panic message.
// The function should never return, so it is marked as a diverging
// function by returning the “never” type !.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default.
    println!("Welcome to LumexOS {}\
         Current year - {}", "😎", 2022);

    // initialize the IDT to be used by the CPU.
    rust_os::init();

    // invoke a breakpoint exception.
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
        test_main();

    println!("It did not crash!");

    loop { }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
