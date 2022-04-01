use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breaking_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
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

// One difference to the breakpoint handler is that the double fault handler is diverging.
// The reason is that the x86_64 architecture does not permit returning from a double
// fault exception, so, we don't return to the caller from this handler.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION DOUBLE FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception_handler () {
    // invokes the int3 function to trigger a breakpoint exception.
    // By checking that the execution continues afterwards,
    // we verify that our breakpoint handler is working correctly.
    x86_64::instructions::interrupts::int3();
}