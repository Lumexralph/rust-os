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

// Since the field ordering in default structs is undefined in Rust,
// we need the repr(C) attribute. It guarantees that the struct’s
// fields are laid out exactly like in a C struct and thus guarantees
// the correct field ordering.
#[repr(transparent)]
struct Buffer {
    char: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
                self.buffer.char[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1; // move the column position a step to the right
            }
        }
    }

    fn new_line(&mut self) {
        // TODO:
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

// temporary function to print stuff.
pub fn print_something() {
    // It first creates a new Writer that points to the VGA buffer at 0xb8000.
    // The syntax for this might seem a bit strange: First, we cast the integer
    // 0xb8000 as an mutable raw pointer. Then we convert it to a mutable
    // reference by dereferencing it (through *) and immediately borrowing it again
    // (through &mut). This conversion requires an unsafe block, since the compiler
    // can’t guarantee that the raw pointer is valid.
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Colors::Cyan, Colors::LightGreen),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'W');
    writer.write_string("elcome ");
    writer.write_string("to LùmëxOS");
    writer.write_string("😎");
}
