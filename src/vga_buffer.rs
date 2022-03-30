#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colors {
    // Because of the repr(u8) attribute each enum variant is stored as an u8
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// To ensure that the ColorCode has the exact same data layout as
// an u8, we use the repr(transparent) attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Colors, background: Colors) -> ColorCode {
        ColorCode(( background as u8 ) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use core::fmt;
use core::fmt::Write;
use volatile::Volatile;

// Since the field ordering in default structs is undefined in Rust,
// we need the repr(C) attribute. It guarantees that the struct’s
// fields are laid out exactly like in a C struct and thus guarantees
// the correct field ordering.
#[repr(transparent)]
struct Buffer {
    // Volatile guarantees that the compiler will never optimize away
    // writes to the buffer.
    // https://en.wikipedia.org/wiki/Volatile_(computer_programming)
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// To actually write to screen, we now create a writer type:
// The writer will always write to the last line and shift lines up when
// a line is full (or on \n). The column_position field keeps track of the
// current position in the last row. The current foreground and background
// colors are specified by color_code and a reference to the VGA buffer is
// stored in buffer.
//
// Note that we need an explicit lifetime here to tell the
// compiler how long the reference is valid. The 'static lifetime specifies
// that the reference is valid for the whole program run time
// (which is true for the VGA text buffer).
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH { // we reached end of the screen
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1; // move the column position a step to the right
            }
        }
    }

    fn new_line(&mut self) {
        // We iterate over all screen characters and move each character one row up.
       for row in 1..BUFFER_HEIGHT {
           for col in 0..BUFFER_WIDTH {
               let character = self.buffer.chars[row][col].read();
               self.buffer.chars[row - 1][col].write(character);
           }
       }
       self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    // clear_row clears a row by overwriting all of its characters with a space character.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ', // space
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    // To print whole strings, we can convert them to bytes and print them one-by-one:
    // The VGA text buffer only supports ASCII and the additional bytes of code page 437.
    // Rust strings are UTF-8 by default, so they might contain bytes that are not supported
    // by the VGA text buffer. We use a match to differentiate printable ASCII bytes
    // For unprintable bytes, we print a ■ character, which has the hex code 0xfe on the VGA hardware.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use lazy_static::lazy_static;
use spin::Mutex;

// With lazy_static, we can define our static WRITER without problems.
lazy_static! {
    // To provide a global writer that can be used as an interface from other modules
    // without carrying a Writer instance around, we try to create a static WRITER.
    // It first creates a new Writer that points to the VGA buffer at 0xb8000.
    // The syntax for this might seem a bit strange: First, we cast the integer
    // 0xb8000 as an mutable raw pointer. Then we convert it to a mutable
    // reference by dereferencing it (through *) and immediately borrowing it again
    // (through &mut). This conversion requires an unsafe block, since the compiler
    // can’t guarantee that the raw pointer is valid.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Colors::White, Colors::LightBlue),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    // unwrap panics if an error occurs. This isn’t a problem in our case,
    // since writes to the VGA buffer never fails. we returned OK() in write_str.
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many_input() {
    for _ in 0..200 {
        println!("test_println_many_input output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Both operations are unsafe, because writing to an I/O port";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}

// TODO: Tests to be written
// - a function that tests that no panic occurs when printing very long lines and that
// they’re wrapped correctly.
//- a function for testing that newlines, non-printable characters, and non-unicode
// characters are handled correctly.
