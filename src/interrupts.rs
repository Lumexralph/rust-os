use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breaking_handler);
        idt
    };
}


pub fn init_idt() {

    // In order that the CPU uses our new interrupt descriptor table,
    // we need to load it using the lidt instruction.
    IDT.load();
}

extern "x86-interrupt" fn breaking_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception_handler () {
    // invokes the int3 function to trigger a breakpoint exception.
    // By checking that the execution continues afterwards,
    // we verify that our breakpoint handler is working correctly.
    x86_64::instructions::interrupts::int3();
}