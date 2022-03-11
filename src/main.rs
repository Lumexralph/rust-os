#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;


// function to handle panic, `!` means a function
// that does not return control to its caller.
// it just doesn't return. A function that returns
// nothing gives an empty tuple ().
//
// The PanicInfo parameter contains the file and line
// where the panic happened and the optional panic message.
// The function should never return, so it is marked as a diverging
// function by returning the “never” type !.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {

    }
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    loop {

    }
}
