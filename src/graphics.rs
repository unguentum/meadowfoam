use core::fmt;

pub const FONT_WIDTH: usize = 5;
pub const FONT_HEIGHT: usize = 7;

pub const FONT_DATA: [[u32; 7]; 94] = [
    [4, 4, 4, 0, 4, 0, 0],
    [10, 10, 0, 0, 0, 0, 0],
    [10, 31, 10, 31, 10, 0, 0],
    [14, 20, 14, 5, 14, 0, 0],
    [25, 18, 4, 9, 19, 0, 0],
    [14, 24, 21, 18, 13, 0, 0],
    [4, 4, 0, 0, 0, 0, 0],
    [4, 8, 8, 8, 4, 0, 0],
    [4, 2, 2, 2, 4, 0, 0],
    [10, 4, 10, 0, 0, 0, 0],
    [0, 4, 14, 4, 0, 0, 0],
    [0, 0, 0, 4, 4, 0, 0],
    [0, 0, 14, 0, 0, 0, 0],
    [0, 0, 0, 0, 4, 0, 0],
    [2, 4, 4, 4, 8, 0, 0],
    [4, 10, 10, 10, 4, 0, 0],
    [4, 12, 4, 4, 14, 0, 0],
    [4, 10, 2, 4, 14, 0, 0],
    [12, 2, 12, 2, 12, 0, 0],
    [2, 6, 10, 14, 2, 0, 0],
    [14, 8, 12, 2, 12, 0, 0],
    [4, 8, 12, 10, 4, 0, 0],
    [14, 2, 4, 8, 8, 0, 0],
    [4, 10, 4, 10, 14, 0, 0],
    [6, 10, 6, 2, 12, 0, 0],
    [0, 0, 4, 0, 4, 0, 0],
    [0, 4, 0, 4, 4, 0, 0],
    [0, 6, 8, 6, 0, 0, 0],
    [0, 14, 0, 14, 0, 0, 0],
    [0, 12, 2, 12, 0, 0, 0],
    [12, 10, 2, 4, 0, 4, 0],
    [14, 17, 21, 27, 20, 17, 14],
    [4, 10, 14, 10, 10, 0, 0],
    [12, 10, 14, 10, 14, 0, 0],
    [6, 8, 8, 8, 6, 0, 0],
    [12, 10, 10, 10, 12, 0, 0],
    [14, 8, 12, 8, 14, 0, 0],
    [14, 8, 12, 8, 8, 0, 0],
    [14, 16, 22, 18, 14, 0, 0],
    [10, 10, 14, 10, 10, 0, 0],
    [4, 4, 4, 4, 4, 0, 0],
    [4, 4, 4, 4, 20, 12, 0],
    [10, 10, 12, 10, 10, 0, 0],
    [8, 8, 8, 8, 14, 0, 0],
    [17, 27, 21, 17, 17, 0, 0],
    [17, 25, 21, 19, 17, 0, 0],
    [14, 17, 17, 17, 14, 0, 0],
    [12, 10, 14, 8, 8, 0, 0],
    [14, 17, 21, 19, 13, 0, 0],
    [12, 10, 12, 10, 10, 0, 0],
    [6, 8, 12, 2, 12, 0, 0],
    [14, 4, 4, 4, 4, 0, 0],
    [10, 10, 10, 10, 4, 0, 0],
    [17, 10, 10, 4, 4, 0, 0],
    [17, 17, 17, 21, 10, 0, 0],
    [17, 10, 4, 10, 17, 0, 0],
    [10, 4, 4, 4, 4, 0, 0],
    [31, 2, 4, 8, 31, 0, 0],
    [14, 8, 8, 8, 14, 0, 0],
    [8, 4, 4, 4, 2, 0, 0],
    [14, 2, 2, 2, 14, 0, 0],
    [4, 10, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 14, 0, 0],
    [4, 2, 0, 0, 0, 0, 0],
    [0, 12, 6, 10, 14, 0, 0],
    [8, 8, 12, 10, 12, 0, 0],
    [0, 0, 6, 8, 6, 0, 0],
    [2, 2, 6, 10, 6, 0, 0],
    [4, 10, 14, 8, 6, 0, 0],
    [6, 4, 14, 4, 4, 0, 0],
    [0, 0, 6, 10, 6, 2, 6],
    [8, 8, 12, 10, 10, 0, 0],
    [0, 4, 0, 4, 4, 0, 0],
    [4, 0, 4, 4, 4, 20, 8],
    [8, 8, 10, 12, 10, 0, 0],
    [4, 4, 4, 4, 6, 0, 0],
    [0, 0, 26, 21, 21, 0, 0],
    [0, 0, 12, 10, 10, 0, 0],
    [0, 0, 4, 10, 12, 0, 0],
    [0, 0, 12, 10, 12, 8, 8],
    [0, 0, 6, 10, 6, 2, 2],
    [0, 0, 12, 10, 8, 0, 0],
    [6, 8, 4, 2, 12, 0, 0],
    [4, 14, 4, 4, 6, 0, 0],
    [0, 0, 10, 10, 6, 0, 0],
    [0, 0, 10, 10, 4, 0, 0],
    [0, 0, 21, 21, 10, 0, 0],
    [0, 0, 10, 4, 10, 0, 0],
    [0, 0, 10, 10, 4, 4, 8],
    [0, 14, 2, 4, 14, 0, 0],
    [6, 4, 12, 4, 6, 0, 0],
    [4, 4, 4, 4, 4, 0, 0],
    [12, 4, 6, 4, 12, 0, 0],
    [0, 0, 24, 21, 3, 0, 0],
];

pub static mut SCREEN_WRITER: ScreenWriter = ScreenWriter::default();

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => {
		unsafe { SCREEN_WRITER.write_fmt(format_args!($($arg)*)).unwrap() }
	};
}

pub struct ScreenWriter {
    cursor_x: usize,
    cursor_y: usize,
    frame_buffer: *mut u32,
    pixel_width: usize,
    pixel_height: usize,
}

impl fmt::Write for ScreenWriter {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for c in string.chars() {
            self.output_char(c);
        }
        Ok(())
    }
}

impl ScreenWriter {
    fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if (self.cursor_y + 1) * FONT_HEIGHT > self.pixel_height {
			self.cursor_y = 0;

			for n in 0..self.pixel_width*self.pixel_height {
				unsafe {
					*self.frame_buffer.add(n) = 0;
				}
			}
        }
    }
    pub const fn default() -> ScreenWriter {
        ScreenWriter {
            cursor_x: 0,
            cursor_y: 0,
            frame_buffer: core::ptr::null_mut(),
            pixel_width: 0,
            pixel_height: 0,
        }
    }
    pub fn init(&mut self, frame_buffer: *mut u32, pixel_width: usize, pixel_height: usize) {
        self.frame_buffer = frame_buffer;
        self.pixel_width = pixel_width;
        self.pixel_height = pixel_height;
    }
    fn advance_cursor(&mut self) {
        self.cursor_x += 1;
        if (self.cursor_x + 1) * FONT_WIDTH > self.pixel_width {
            self.new_line();
        }
    }
    fn draw_ascii(&mut self, px: usize, py: usize, c: u8) {
        for dy in 0..FONT_HEIGHT {
            for dx in 0..FONT_WIDTH {
                let color =
                    if (FONT_DATA[(c - b'!') as usize][dy] & (1 << (FONT_WIDTH - 1 - dx))) != 0 {
                        0xFFFFFF
                    } else {
                        0
                    };
                let x = px + dx;
                let y = py + dy;
				if x + y * self.pixel_width < self.pixel_width * self.pixel_height {
	                unsafe {
    	                *self.frame_buffer.add(x + y * self.pixel_width) = color;
        	        }
    	        }
            }
        }
    }

    fn output_char(&mut self, c: char) {
        if !c.is_ascii() {
            return;
        }
        let c = c as u8;
        match c {
            b'!'..b'~' => {
                self.draw_ascii(self.cursor_x * FONT_WIDTH, self.cursor_y * FONT_HEIGHT, c);
                self.advance_cursor();
            }
            b'\n' => self.new_line(),
            b' ' => self.advance_cursor(),
            _ => (),
        }
    }
}
