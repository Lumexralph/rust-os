#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::{hlt_loop, memory, println};
use bootloader::{BootInfo, entry_point};
use x86_64::{
    structures::{
        paging::{ Translate, Page },
    },
    VirtAddr
};

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
    hlt_loop();
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default.
    println!("Welcome to LumexOS {}\
         Current year - {}", "😎", 2022);

    // initialize the IDT to be used by the CPU.
    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootFrameAllocator::init(&boot_info.memory_map)
    };

    // map an unused page.
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping.
    // Then we convert the page to a raw pointer and write a value to offset 400.
    // We don’t write to the start of the page because the top line of the VGA buffer
    // is directly shifted off the screen by the next println. We write the value
    // 0x_f021_f077_f065_f04e, which represents the string “New!” on white background.
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe  { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0.
        boot_info.physical_memory_offset
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        // We need to import the Translate trait in order to use the translate_addr method it provides.
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
