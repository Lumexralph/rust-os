use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use spin::Mutex;
use x86_64::instructions::port::Port;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breaking_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
                .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
                .set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

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

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// By wrapping the ChainedPics struct in a Mutex we are able to get safe
// mutable access (through the lock method).
// The ChainedPics::new function is unsafe because wrong offsets could cause undefined behavior.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new( unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) } );

// The enum is a C-like enum so that we can directly specify the index for each variant.
// The repr(u8) attribute specifies that each variant is represented as an u8.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // 32
    Keyboard, // 33 (previous value in enum + 1)
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    // The notify_end_of_interrupt figures out whether the primary or secondary PIC
    // sent the interrupt and then uses the command and data ports to send an EOI signal
    // to respective controllers. If the secondary PIC sent the interrupt both PICs need
    // to be notified because the secondary PIC is connected to an input line of the primary PIC.
    //
    // We need to be careful to use the correct interrupt vector number, otherwise we could
    // accidentally delete an important unsent interrupt or cause our system to hang.
    // This is the reason that the function is unsafe.
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()) }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // the keyboard controller won’t send another interrupt until we have read the
    // so-called scancode of the pressed key.
    // We use the Port type of the x86_64 crate to read a byte from the keyboard’s data port.
    // This byte is called the scancode and is a number that represents the key press/release.
    use x86_64::instructions::port::Port;
    use spin::Mutex;
    use pc_keyboard::{ layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1 };

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error code: {:?}", error_code);
    println!("Stack frame: {:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception_handler () {
    // invokes the int3 function to trigger a breakpoint exception.
    // By checking that the execution continues afterwards,
    // we verify that our breakpoint handler is working correctly.
    x86_64::instructions::interrupts::int3();
}