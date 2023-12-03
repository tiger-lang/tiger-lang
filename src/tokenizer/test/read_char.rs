// From Wikipedia:
// First code point 	Last code point 	Byte 1 	    Byte 2 	    Byte 3 	    Byte 4
// U+0000 	            U+007F 	            0xxxxxxx
// U+0080 	            U+07FF 	            110xxxxx 	10xxxxxx
// U+0800 	            U+FFFF 	            1110xxxx 	10xxxxxx 	10xxxxxx
// U+10000 	            U+10FFFF    	    11110xxx 	10xxxxxx 	10xxxxxx 	10xxxxxx

use std::u8;

use crate::tokenizer::read_char;

struct AllUTF8CharReader {
    pos: usize,
}

impl AllUTF8CharReader {
    // This will return all UTF-8 encoded Unicode codepoints in sequence.
    // The invalid range of 0xD800-0xDFFF will all be replaced by the next valid character instead (0xE000)
    fn next_u8(&mut self) -> Option<u8> {
        let mut res = None;

        if self.pos <= 0x7f {
            // Single byte characters
            res = Some(self.pos as u8);
        }

        if res == None {
            let codepoint = 0x80 + (self.pos - 0x80) / 2;
            if codepoint <= 0x7ff {
                // 2-byte characters
                let b1 = 0b11000000 | ((codepoint >> 6) as u8 & 0b00011111);
                let b2 = 0b10000000 | ((codepoint >> 0) as u8 & 0b00111111);
                res = match self.pos % 2 {
                    0 => Some(b1),
                    _ => Some(b2),
                }
            }
        }

        if res == None {
            let mut codepoint = (self.pos - (0x80 + 0x780 * 2)) / 3 + 0x800;
            // 0xd800 - 0xdfff are not valid unicode values, they are surrogate halves used by UTF-16
            if codepoint >= 0xd800 && codepoint <= 0xdfff {
                codepoint = 0xe000;
            }
            if codepoint <= 0xffff {
                if codepoint < 0xd800 && codepoint < 0xdfff {}

                // 3-byte characters
                let b1 = 0b11100000 | ((codepoint >> 12) as u8 & 0b00001111);
                let b2 = 0b10000000 | ((codepoint >> 06) as u8 & 0b00111111);
                let b3 = 0b10000000 | ((codepoint >> 00) as u8 & 0b00111111);
                // pos-2 because the offset at which the 3-byte codepoint begins into the stream is 2 % 3.
                res = match (self.pos - 2) % 3 {
                    0 => Some(b1),
                    1 => Some(b2),
                    _ => Some(b3),
                }
            }
        }

        if res == None {
            let codepoint = (self.pos - (0x80 + 0x780 * 2 + 0xf800 * 3)) / 4 + 0x10000;

            if codepoint > 0x10ffff {
                return None; // All valid codepoints have been read
            }

            let b1 = 0b11110000 | ((codepoint >> 18) as u8 & 0b00000111);
            let b2 = 0b10000000 | ((codepoint >> 12) as u8 & 0b00111111);
            let b3 = 0b10000000 | ((codepoint >> 06) as u8 & 0b00111111);
            let b4 = 0b10000000 | ((codepoint >> 00) as u8 & 0b00111111);

            res = match self.pos % 4 {
                0 => Some(b1),
                1 => Some(b2),
                2 => Some(b3),
                _ => Some(b4),
            }
        }

        self.pos += 1;
        return res;
    }
}

impl std::io::Read for AllUTF8CharReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for i in 0..buf.len() {
            match self.next_u8() {
                Some(c) => buf[i] = c,
                None => return Ok(i),
            }
        }
        return Ok(buf.len());
    }
}

#[test]
fn read_char_test() {
    let mut r = AllUTF8CharReader { pos: 0 };

    for codepoint in 0..0x11000 {
        //println!("attempting to read codepoint {:x}", codepoint);
        let actual_char = read_char(&mut r);
        match char::from_u32(codepoint) {
            Some(c) => {
                let actual_char = actual_char.unwrap();
                let actual_char = actual_char.unwrap();
                assert_eq!(actual_char as u32, c as u32);
            }
            None => match actual_char {
                Some(Ok(c)) => {
                    if c as u32 != 0xe000 {
                        panic!("codepoint {:x} does not return a valid character, but result from stream produced {:x}", codepoint, c as u32)
                    }
                }
                None => (),
                Some(Err(_)) => (),
            },
        }
    }
}
