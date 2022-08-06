use crate::arch;
use crate::util;

// Each pixel is stored as 1 or 0 value.
// Waste of memory, but OK to start with.
pub type Frame = util::Array<util::Array<u32, {arch::DISPLAY_WIDTH as usize}>, {arch::DISPLAY_HEIGHT as usize}>;

pub struct Framebuffer {
    frame: Frame,
}

impl Framebuffer {
    pub fn new() -> Self {
        Framebuffer {
            frame: Frame::new(),
        }
    }

    pub fn clear(&mut self) {
        self.frame.clear();
    }

    pub fn get_frame(&self) -> &Frame {
        &self.frame
    }

    #[cfg(test)]
    fn fill_frame_u8(&mut self, v: u8) {
        for i in 0..arch::DISPLAY_HEIGHT {
            for j in 0..arch::DISPLAY_WIDTH {
                let shift = 7 - (j % 8);
                let mask = 1 << shift;
                let set: bool = (v & mask) != 0;

                self.frame[i][j] = if set { 1 } else { 0 };
            }
        }
    }

    pub fn draw_sprite(&mut self, sprite: &[u8], start_x: u32, start_y: u32, colisions: &mut bool) {
        *colisions = false;

        // Start position wraps.
        let start_x = start_x % arch::DISPLAY_WIDTH;
        let start_y = start_y % arch::DISPLAY_HEIGHT;

        for (n, s) in sprite.iter().enumerate() {
            let frame_y = start_y + n as u32;

            // Drawing should be clipped.
            if frame_y >= arch::DISPLAY_HEIGHT {
                break;
            }

            for x in 0..8 {
                let frame_x = start_x + x;

                // Drawing should be clipped.
                if frame_x >= arch::DISPLAY_WIDTH {
                    break;
                }

                let bit_mask = 1u8 << (7 - x);
                let flip_bit: bool = s & bit_mask != 0;

                if flip_bit {
                    let frame_bit = self.frame[frame_y][frame_x];
                    *colisions = frame_bit == 1;

                    self.frame[frame_y][frame_x] = 1 - frame_bit;
                }
            }
        }
    }

    #[cfg(test)]
    pub fn print_screen(&self) {
        let out: &Frame = self.get_frame();
        print!("[");
        for row in out.iter() {
            for col in row.iter() {
                if *col == 1 {
                    print!("*");
                } else {
                    print!(" ");
                }
            }
            println!("]");
            print!("[");
        }
        println!("]");
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;

    fn match_screen(d: &Framebuffer, pixel: u32) -> bool {
        for row in d.frame.iter() {
            for cell in row.iter() {
                if *cell != pixel {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn new() {
        let d = Framebuffer::new();
        assert!(match_screen(&d, 0x00));
    }

    // Sprite 3x8
    //    01234567
    // 0  ***  ***  E7
    // 1  * * * *   AA
    // 2  *      *  81

    const SPRITE_3X8: [u8; 3] = [0xE7, 0xAA, 0x81];

    #[test]
    fn draw_sprite_1() {
        let mut d = Framebuffer::new();
        let mut c = false;

        d.draw_sprite(&SPRITE_3X8, 0, 0, &mut c);
        println!("draw_sprite");
        d.print_screen();
        assert_eq!(c, false);
    }

    #[test]
    fn draw_sprite_2() {
        use crate::arch;
        let mut d = Framebuffer::new();
        let mut c = false;

        d.draw_sprite(&SPRITE_3X8, arch::DISPLAY_WIDTH-1, 0, &mut c);
        println!("draw_sprite");
        d.print_screen();
        assert_eq!(c, false);
    }

    #[test]
    fn draw_sprite_3() {
        use crate::arch;
        let mut d = Framebuffer::new();
        let mut c = false;

        d.draw_sprite(&SPRITE_3X8, arch::DISPLAY_WIDTH-1, arch::DISPLAY_HEIGHT-2, &mut c);
        println!("draw_sprite");
        d.print_screen();
        assert_eq!(c, false);
    }

    #[test]
    fn draw_sprite_4() {
        let mut d = Framebuffer::new();
        d.fill_frame_u8(0xff);
        let mut c = false;

        d.draw_sprite(&SPRITE_3X8, 0, 0, &mut c);
        println!("draw_sprite");
        d.print_screen();
        assert_eq!(c, true);
    }
}
