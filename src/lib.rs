#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Like the main.rs, the lib.rs is a special file that is automatically recognized by cargo.
// The library is a separate compilation unit, so we need to specify the #![no_std]
// attribute again.
//
//  using the cfg_attr crate attribute, we conditionally enable the no_main attribute
// in this case.
// The library is usable like a normal external crate. It is called like our crate,
// which is rust_os in our case.

use core::panic::PanicInfo;

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    // We use u32 because we specified the iosize of the isa-debug-exit device as 4 bytes.
    // Both operations are unsafe, because writing to an I/O port can generally result in
    // arbitrary behavior.
    unsafe {
        // 0xf4 is the port address/iobase of isa-debug-exit device.
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


pub trait Testable {
    fn run(&self) -> ();
}

// implement this trait for all types T that implement the Fn() trait.
impl<T> Testable for T
    where T: Fn() {
    fn run(&self) -> () {
        // We implement the run function by first printing the function name using
        // the any::type_name function.
        serial_print!("{}...\t", core::any::type_name::<T>());
        self(); // invoke the test function
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

// Panic handler in test mode.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop { }
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();

    loop { }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// Initializing the IDT.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    //  initialize the 8259 PIC. It is unsafe because it can cause undefined
    // behavior if the PIC is misconfigured.
    unsafe { interrupts::PICS.lock().initialize() };

    // The interrupts::enable function of the x86_64 crate executes the special
    // sti instruction (“set interrupts”) to enable external interrupts.
    x86_64::instructions::interrupts::enable();
}
