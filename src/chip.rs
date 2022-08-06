use log::{trace, info};

use crate::arch;
use crate::ram::Ram;
use crate::regs::RegMap;
use crate::instr::Instr;
use crate::framebuffer::Framebuffer;
use crate::framebuffer::Frame;
use crate::util;
use crate::profile::Profile;

type Stack = util::Array<u16, {arch::STACKSIZE as usize}>;
type Keys = [bool; 16];
type Sprite = [u8; 5];
type SpriteAddrs = util::Array<u16, {arch::NSPRITES as usize}>;

pub struct Chip {
    ram: Ram,
    sprite_addr: SpriteAddrs,
    regs: RegMap,
    stack: Stack,
    keys: Keys,
    framebuffer: Framebuffer,
    rnd: oorandom::Rand32,
    profile: Profile,
}

macro_rules! trace_instr {
    ($self:ident, $fmt: expr $(, $($arg:tt)* )? ) =>
    {
        trace!(concat!("[PC:0x{:04x}] ", $fmt), $self.regs.pc - 2, $( $( $arg )* )? );
    };
}

impl Chip {
    pub fn new(profile: Profile) -> Chip {
        // Generate RND seed.
        let mut seed_bytes: [u8; 8] = [0u8; 8];
        getrandom::getrandom(&mut seed_bytes).unwrap();
        let seed: u64 = u64::from_le_bytes(seed_bytes);

        Chip::new_seed(seed, profile)
    }

    pub fn new_seed(seed: u64, profile: Profile) -> Chip {
        let sprites: [Sprite; 0x10] = [
            // 0x0
            [0b01100000,
             0b10010000,
             0b10010000,
             0b10010000,
             0b01100000],
            // 0x1
            [0b00100000,
             0b01100000,
             0b00100000,
             0b00100000,
             0b01110000],
            // 0x2
            [0b11110000,
             0b00010000,
             0b11110000,
             0b10000000,
             0b11110000],
            // 0x3
            [0b11100000,
             0b00010000,
             0b11100000,
             0b00010000,
             0b11100000],
            // 0x4
            [0b10010000,
             0b10010000,
             0b11110000,
             0b00010000,
             0b00010000],
            // 0x5
            [0b11110000,
             0b10000000,
             0b11100000,
             0b00010000,
             0b11100000],
            // 0x6
            [0b01110000,
             0b10000000,
             0b11100000,
             0b10010000,
             0b11100000],
            // 0x7
            [0b11110000,
             0b00010000,
             0b00100000,
             0b01000000,
             0b01000000],
            // 0x8
            [0b01100000,
             0b10010000,
             0b01100000,
             0b10010000,
             0b01100000],
            // 0x9
            [0b01100000,
             0b10010000,
             0b01110000,
             0b00010000,
             0b11100000],
            // 0xA
            [0b01100000,
             0b10010000,
             0b11110000,
             0b10010000,
             0b10010000],
            // 0xB
            [0b11100000,
             0b10010000,
             0b11100000,
             0b10010000,
             0b11100000],
            // 0xC
            [0b01110000,
             0b10000000,
             0b10000000,
             0b10000000,
             0b01110000],
            // 0xD
            [0b11100000,
             0b10010000,
             0b10010000,
             0b10010000,
             0b11100000],
            // 0xE
            [0b11110000,
             0b10000000,
             0b11110000,
             0b10000000,
             0b11110000],
            // 0xF
            [0b11110000,
             0b10000000,
             0b11110000,
             0b10000000,
             0b10000000],
        ];

        let mut ram = Ram::new();
        let mut sprite_addr = SpriteAddrs::new();
        let mut addr: u32 = 0x0;

        for (i, s) in sprites.iter().enumerate() {
            sprite_addr[i] = addr as u16;
            addr = ram.load_block_u8(addr, s);
        }

        Chip {
            ram,
            sprite_addr,
            regs: RegMap::new(),
            stack: Stack::new(),
            keys: [false; 16],
            framebuffer: Framebuffer::new(),
            rnd: oorandom::Rand32::new(seed),
            profile,
        }
    }

    pub fn key_press(&mut self, key: u8) {
        self.keys[key as usize] = true;
    }

    pub fn key_unpress(&mut self, key: u8) {
        self.keys[key as usize] = false;
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.regs.pc = pc;
    }

    pub fn cycle(&mut self) {
        let code = self.ram.read_u16(self.regs.pc as u32);
        let instr = Instr::new(code);

        // PC points to the next instruction to execute.
        self.regs.pc += 2;

        match instr {
            Instr { opcode: 0x00E0, .. } => {
                // CLS - Clear framebuffer
                trace_instr!(self, "CLS");
                self.framebuffer.clear();
            },

            Instr { opcode: 0x00EE, .. } => {
                // RET - Return from a subroutine.
                trace_instr!(self, "RET");
                self.regs.sp -= 1;
                self.regs.pc = self.stack[self.regs.sp];
            },

            Instr { c: 0x1, nnn, .. } => {
                // JP addr
                trace_instr!(self, "JP {:#x}", nnn);
                self.regs.pc = nnn;
            },

            Instr { c: 0x2, nnn, .. } => {
                // CALL addr.
                trace_instr!(self, "CALL {:#x}", nnn);
                self.stack[self.regs.sp] = self.regs.pc;
                self.regs.sp += 1;
                self.regs.pc = nnn;
            },

            Instr { c: 0x3, x, nn, .. } => {
                // SE Vx, nn
                trace_instr!(self, "SE V{:X}, {:#x}", x, nn);
                if self.regs.vx[x] == nn {
                    self.regs.pc += 2;
                }
            },

            Instr { c: 0x4, x, nn, .. } => {
                // SNE Vx, nn
                trace_instr!(self, "SNE V{:X}, {:#x}", x, nn);
                if self.regs.vx[x] != nn {
                    self.regs.pc += 2;
                }
            },

            Instr { c: 0x5, x, y, n:0x0, .. } => {
                // SE Vx, Vy
                trace_instr!(self, "SE V{:X}, V{:X}", x, y);
                if self.regs.vx[x] == self.regs.vx[y] {
                    self.regs.pc += 2;
                }
            },

            Instr { c: 0x6, x, nn, .. } => {
                // LD Vx, nn
                trace_instr!(self, "LD V{:X}, {:#x}", x, nn);
                self.regs.vx[x] = nn;
            },

            Instr { c: 0x7, x, nn, .. } => {
                // ADD Vx, nn
                trace_instr!(self, "ADD V{:X}, {:#x}", x, nn);
                (self.regs.vx[x], _) = self.regs.vx[x].overflowing_add(nn);
            },

            Instr { c: 0x8, x, y, n: 0x0, .. } => {
                // LD Vx, Vy
                trace_instr!(self, "LD V{:X}, V{:X}", x, y);
                self.regs.vx[x] = self.regs.vx[y];
            },

            Instr { c: 0x8, x, y, n: 0x1, .. } => {
                // OR Vx, Vy
                trace_instr!(self, "OR V{:X}, V{:X}", x, y);
                self.regs.vx[x] |= self.regs.vx[y];
            },

            Instr { c: 0x8, x, y, n: 0x2, .. } => {
                // AND Vx, Vy
                trace_instr!(self, "AND V{:X}, V{:X}", x, y);
                self.regs.vx[x] &= self.regs.vx[y];
            },

            Instr { c: 0x8, x, y, n: 0x3, .. } => {
                // XOR Vx, Vy
                trace_instr!(self, "XOR V{:X}, V{:X}", x, y);
                self.regs.vx[x] ^= self.regs.vx[y];
            },

            Instr { c: 0x8, x, y, n: 0x4, .. } => {
                // ADD Vx, Vy
                trace_instr!(self, "ADD V{:X}, V{:X}", x, y);
                let overflow: bool;
                (self.regs.vx[x], overflow) = self.regs.vx[x].overflowing_add(self.regs.vx[y]);
                // VF := overflow
                self.regs.vx[0xf_u8] = if overflow { 1 } else { 0 };
            },

            Instr { c: 0x8, x, y, n: 0x5, .. } => {
                // SUB Vx, Vy
                trace_instr!(self, "SUB V{:X}, V{:X}", x, y);
                let overflow: bool;
                (self.regs.vx[x], overflow) = self.regs.vx[x].overflowing_sub(self.regs.vx[y]);
                // VF := not overflow
                self.regs.vx[0xf_u8] = if overflow { 0 } else { 1 };
            },

            Instr { c: 0x8, x, y, n: 0x6, .. } => {
                // SHR Vx, Vy. Ambiguous.
                trace_instr!(self, "SHR V{:X}, V{:X}", x, y);
                if self.profile.op_8xy6_use_vy {
                    self.regs.vx[x] = self.regs.vx[y];
                }
                self.regs.vx[0xf_u8] = self.regs.vx[x] & 0x01_u8;
                self.regs.vx[x] >>= 1;
            },

            Instr { c: 0x8, x, y, n: 0x7, .. } => {
                // SUBN Vx, Vy
                trace_instr!(self, "SUBN V{:X}, V{:X}", x, y);
                let overflow: bool;
                (self.regs.vx[x], overflow) = self.regs.vx[y].overflowing_sub(self.regs.vx[x]);
                // VF := not overflow
                self.regs.vx[0xf_u8] = if overflow { 0 } else { 1 };
            },

            Instr { c: 0x8, x, y, n: 0xE, .. } => {
                // SHL Vx, Vy
                trace_instr!(self, "SHL V{:X}, V{:X}", x, y);
                if self.profile.op_8xye_use_vy {
                    self.regs.vx[x] = self.regs.vx[y];
                }
                self.regs.vx[0xf_u8] = if self.regs.vx[x] & 0x80_u8 != 0 { 1 } else { 0 };
                self.regs.vx[x] <<= 1;
            },

            Instr { c: 0x9, x, y, n: 0x0, .. } => {
                // SNE Vx, Vy
                trace_instr!(self, "SNE V{:X}, V{:X}", x, y);
                if self.regs.vx[x] != self.regs.vx[y] {
                    self.regs.pc += 2;
                }
            },

            Instr { c: 0xA, nnn, .. } => {
                // LD I, nnn
                trace_instr!(self, "LD I, {:#x}", nnn);
                self.regs.i = nnn;
            },

            Instr { c: 0xB, nnn, .. } => {
                // JP V0, nnn
                trace_instr!(self, "JP V0, {:#x}", nnn);
                self.regs.pc = self.regs.vx[0] as u16 + nnn;
            },

            Instr { c: 0xC, x, nn, .. } => {
                // RND Vx, nn
                trace_instr!(self, "RND V{:X}, {:#x}", x, nn);
                let rnd: u8 = self.rnd.rand_range(0..0x100) as u8;
                self.regs.vx[x] = rnd & nn;
            },

            Instr { c: 0xD, x, y, n, .. } => {
                // DRW Vx, Vy, n
                trace_instr!(self, "DRW V{:X}, V{:X}, {:#x}", x, y, n);

                let addr_start = self.regs.i as usize;
                let addr_end = addr_start + (n as usize);
                let sprites = &self.ram.mem[addr_start..addr_end];

                let mut colisions: bool = false;

                let start_x = self.regs.vx[x] as u32;
                let start_y = self.regs.vx[y] as u32;

                self.framebuffer.draw_sprite(sprites, start_x, start_y, &mut colisions);

                self.regs.vx[0xF] = if colisions { 1u8 } else { 0u8 };
            },

            Instr { c: 0xE, x, nn: 0x9E, .. } => {
                // SKP Vx
                trace_instr!(self, "SKP V{:X}", x);
                if self.is_key_pressed(self.regs.vx[x]) {
                    self.regs.pc += 2;
                }
            },

            Instr { c: 0xE, x, nn: 0xA1, .. } => {
                // SKPN Vx
                trace_instr!(self, "SKPN V{:X}", x);
                if !self.is_key_pressed(self.regs.vx[x]) {
                    self.regs.pc += 2;
                }
            },

            Instr { c: 0xF, x, nn: 0x07, .. } => {
                // LD Vx, DT
                trace_instr!(self, "LD V{:X}, DT", x);
                self.regs.vx[x] = self.regs.dt;
                info!("DT={}", self.regs.dt);
            },

            Instr { c: 0xF, x, nn: 0x0A, .. } => {
                // LD Vx, K
                trace_instr!(self, "LD V{:X}, K", x);
                match self.keys.iter().position(|&pressed| { pressed }) {
                    Some(i) => self.regs.vx[x] = i as u8,
                    None => self.regs.pc -= 2,
                }
            },

            Instr { c: 0xF, x, nn: 0x15, .. } => {
                // LD DT, Vx
                trace_instr!(self, "LD DT, V{:X}", x);
                self.regs.dt = self.regs.vx[x];
                info!("DT={}", self.regs.dt);
            },

            Instr { c: 0xF, x, nn: 0x18, .. } => {
                // LD ST, Vx
                trace_instr!(self, "LD ST, V{:X}", x);
                self.regs.st = self.regs.vx[x];
            },

            Instr { c: 0xF, x, nn: 0x1E, .. } => {
                // ADD I, Vx
                trace_instr!(self, "ADD I, V{:X}", x);
                self.regs.i += self.regs.vx[x] as u16;
            },

            Instr { c: 0xF, x, nn: 0x29, .. } => {
                // LD F, Vx
                trace_instr!(self, "LD F, V{:X}", x);
                self.regs.i = self.sprite_addr[self.regs.vx[x]];
            },

            Instr { c: 0xF, x, nn: 0x33, .. } => {
                // LD B, Vx
                trace_instr!(self, "LD B, V{:X}", x);
                let mut bcd = [0u8; 3];
                bcd[2] = self.regs.vx[x] % 10;
                bcd[1] = (self.regs.vx[x] / 10) % 10;
                bcd[0] = self.regs.vx[x] / 100;

                self.ram.load_block_u8(self.regs.i as u32, &bcd);
            },

            Instr { c: 0xF, x, nn: 0x55, .. } => {
                // LD [I], Vx
                trace_instr!(self, "LD [I], V{:X}", x);
                for i in 0..=x {
                    let addr: u32 = self.regs.i as u32 + i as u32;
                    self.ram.write_u8(addr, self.regs.vx[i]);
                }
                if self.profile.op_fx55_store_i {
                    self.regs.i += x as u16 + 1;
                }
            },

            Instr { c: 0xF, x, nn: 0x65, .. } => {
                // LD Vx, [I]
                trace_instr!(self, "LD V{:X}, [I]", x);
                for i in 0..=x {
                    let addr: u32 = self.regs.i as u32 + i as u32;
                    self.regs.vx[i] = self.ram.read_u8(addr);
                }
                if self.profile.op_fx65_store_i {
                    self.regs.i += x as u16 + 1;
                }
            },

            _ => panic!("Unknown opcode: {:#x}", instr.opcode),
        }
    }

    pub fn cycle_timers(&mut self) {
        if self.regs.dt > 0 {
            self.regs.dt -= 1;
        }

        if self.regs.st > 0 {
            self.regs.st -= 1;
        }
        info!("cycle_timers, dt={}, st={}", self.regs.dt, self.regs.st);
    }

    pub fn is_sound_on(&self) -> bool {
        self.regs.st > 0
    }

    pub fn load_rom(&mut self, rom: &[u8], start: u32) {
        let mut code = Vec::<u16>::new();
        for i in 0..rom.len()/2 {
            let op: u16 = u16::from_be_bytes([rom[2*i], rom[2*i+1]]);
            code.push(op);
        }
        self.ram.load_block_u16(start, code.as_slice());
    }

    pub fn get_frame(&self) -> &Frame {
        self.framebuffer.get_frame()
    }
}

#[cfg(test)]
mod tests {
    use super::Chip;
    use super::Sprite;
    use super::Profile;

    fn run_code(chip: &mut Chip, code: &[u16]) {
        chip.ram.load_block_u16(0x200, &code);
        chip.set_pc(0x200);
        for _ in code {
            chip.cycle();
        }
    }

    #[test]
    fn new() {
        let _ = Chip::new(Profile::original());
    }

    #[test]
    fn set_pc() {
        let mut chip = Chip::new(Profile::original());
        chip.set_pc(0x200);
        assert_eq!(chip.regs.pc, 0x200);
    }

    #[test]
    fn one_instr_pc() {
        let mut chip = Chip::new(Profile::original());
        let code  = [ 0x00E0_u16 ];
        chip.ram.load_block_u16(0x200, &code);
        chip.set_pc(0x200);
        assert_eq!(chip.regs.pc, 0x200);
        chip.cycle();
        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn few_instr_pc() {
        let mut chip = Chip::new(Profile::original());
        let code  = [ 0x00E0_u16, 0x00E0_u16 ];
        chip.ram.load_block_u16(0x200, &code);
        chip.set_pc(0x200);
        assert_eq!(chip.regs.pc, 0x200);
        chip.cycle();
        assert_eq!(chip.regs.pc, 0x202);
        chip.cycle();
        assert_eq!(chip.regs.pc, 0x204);

        chip.set_pc(0x200);
        assert_eq!(chip.regs.pc, 0x200);
        for _ in code {
            chip.cycle();
        }
        assert_eq!(chip.regs.pc, (0x200 + code.len() * 2) as u16);

    }

    #[test]
    fn ret_0() {
        let mut chip = Chip::new(Profile::original());

        const START_PC: u16 = 0x210;
        const START_SP: u8 = 3;
        chip.regs.sp = START_SP;
        chip.stack[chip.regs.sp] = START_PC;
        chip.regs.sp += 1;

        run_code(&mut chip, &[0x00EE_u16]); // RET

        assert_eq!(chip.regs.sp, START_SP);
        assert_eq!(chip.regs.pc, START_PC);
    }

    #[test]
    fn jp_nnn_0() {
        let mut chip = Chip::new(Profile::original());

        run_code(&mut chip, &[0x1320_u16]); // JP 0x320

        assert_eq!(chip.regs.pc, 0x320);
    }

    #[test]
    fn ld_vx_nn_0() {
        let mut chip = Chip::new(Profile::original());

        run_code(&mut chip, &[
            0x6222_u16, // LD V2, 0x22
            0x6015_u16, // LD V0, 0x15
            0x6FFF_u16, // LD VF, 0xFF
        ]);

        assert_eq!(chip.regs.vx[2], 0x22_u8);
        assert_eq!(chip.regs.vx[0], 0x15_u8);
        assert_eq!(chip.regs.vx[0xF], 0xFF_u8);
    }

    #[test]
    fn se_vx_nn_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x3222_u16]); // SE V2, 0x22

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn se_vx_nn_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[4] = 0x33_u8;
        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x3433_u16]); // SE V4, 0x33

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn se_vx_nn_2() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[4] = 0x33_u8;
        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x3422_u16]); // SE V4, 0x22

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn sne_vx_nn_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x4222_u16]); // SNE V2, 0x22

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn sne_vx_nn_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[4] = 0x33_u8;
        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x4433_u16]); // SNE V4, 0x33

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn sne_vx_nn_2() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[4] = 0x33_u8;
        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x4422_u16]); // SNE V4, 0x22

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn se_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x22_u8;
        chip.regs.vx[3] = 0x22_u8;

        run_code(&mut chip, &[0x5230_u16]); // SE V2, V3

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn se_vx_vy_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x22_u8;
        chip.regs.vx[3] = 0x23_u8;

        run_code(&mut chip, &[0x5230_u16]); // SE V2, V3

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn ld_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        run_code(&mut chip, &[
            0x6222_u16, // LD V2, 0x22  V2:=0x22
            0x6323_u16, // LD V3, 0x23  V3:=0x23
            0x8230_u16, // LD V2, V3    V2:=V3(0x23)
                        // V3 must be 0x23
        ]);

        assert_eq!(chip.regs.vx[3], 0x23_u8);
    }

    #[test]
    fn or_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x64_u8;
        chip.regs.vx[3] = 0xA9_u8;

        run_code(&mut chip, &[0x8231_u16]); // OR V2, V3    V2:=b'11101101 -> 0xED

        assert_eq!(chip.regs.vx[2], 0xED_u8);
    }

    #[test]
    fn and_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xCC_u8; // b'11001100
        chip.regs.vx[3] = 0xAA_u8; // b'10101010
                                   // b'10001000

        run_code(&mut chip, &[0x8232_u16]); // AND V2, V3

        assert_eq!(chip.regs.vx[2], 0x88_u8);
    }

    #[test]
    fn xor_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xCC_u8; // b'11001100
        chip.regs.vx[3] = 0xAA_u8; // b'10101010
                                   // b'01100110

        run_code(&mut chip, &[0x8233_u16]); // XOR V2, V3

        assert_eq!(chip.regs.vx[2], 0x66_u8);
    }

    #[test]
    fn add_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xFE_u8;
        chip.regs.vx[3] = 0x01_u8;

        run_code(&mut chip, &[0x8234_u16]); // ADD V2, V3

        assert_eq!(chip.regs.vx[2], 0xFF_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn add_vx_vy_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xFF_u8;
        chip.regs.vx[3] = 0x01_u8;

        run_code(&mut chip, &[0x8234_u16]); // ADD V2, V3

        assert_eq!(chip.regs.vx[2], 0x00_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }

    #[test]
    fn sub_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x10_u8;
        chip.regs.vx[3] = 0x04_u8;

        run_code(&mut chip, &[0x8235_u16]); // SUB V2, V3

        assert_eq!(chip.regs.vx[2], 0x0C_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }

    #[test]
    fn sub_vx_vy_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x10_u8;
        chip.regs.vx[3] = 0x55_u8;

        run_code(&mut chip, &[0x8235_u16]); // SUB V2, V3

        assert_eq!(chip.regs.vx[2], 0xBB_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn shr_vx_vy_0_orig() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x00_u8;
        chip.regs.vx[3] = 0x55_u8;

        run_code(&mut chip, &[0x8236_u16]); // SHR V2, V3

        assert_eq!(chip.regs.vx[2], 0x2A_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }

    #[test]
    fn shr_vx_vy_1_orig() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x00_u8;
        chip.regs.vx[3] = 0x54_u8;

        run_code(&mut chip, &[0x8236_u16]); // SHR V2, V3

        assert_eq!(chip.regs.vx[2], 0x2A_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn shr_vx_vy_0_modern() {
        let mut chip = Chip::new(Profile::modern());

        chip.regs.vx[2] = 0x55_u8;

        run_code(&mut chip, &[0x8236_u16]); // SHR V2, V3

        assert_eq!(chip.regs.vx[2], 0x2A_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }

    #[test]
    fn shr_vx_vy_1_modern() {
        let mut chip = Chip::new(Profile::modern());

        chip.regs.vx[2] = 0x54_u8;

        run_code(&mut chip, &[0x8236_u16]); // SHR V2, V3

        assert_eq!(chip.regs.vx[2], 0x2A_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn subn_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x04_u8;
        chip.regs.vx[3] = 0x10_u8;

        run_code(&mut chip, &[0x8237_u16]); // SUBN V2, V3

        assert_eq!(chip.regs.vx[2], 0x0C_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }

    #[test]
    fn subn_vx_vy_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x55_u8;
        chip.regs.vx[3] = 0x10_u8;

        run_code(&mut chip, &[0x8237_u16]); // SUBN V2, V3

        assert_eq!(chip.regs.vx[2], 0xBB_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn shl_vx_vy_0_orig() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x00_u8;
        chip.regs.vx[3] = 0x55_u8;

        run_code(&mut chip, &[0x823E_u16]); // SHL V2, V3

        assert_eq!(chip.regs.vx[2], 0xAA_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn shl_vx_vy_1_orig() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x00_u8;
        chip.regs.vx[3] = 0xD5_u8;

        run_code(&mut chip, &[0x823E_u16]); // SHL V2, V3

        assert_eq!(chip.regs.vx[2], 0xAA_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }

    #[test]
    fn shl_vx_vy_0_modern() {
        let mut chip = Chip::new(Profile::modern());

        chip.regs.vx[2] = 0x55_u8;

        run_code(&mut chip, &[0x823E_u16]); // SHL V2, V3

        assert_eq!(chip.regs.vx[2], 0xAA_u8);
        assert_eq!(chip.regs.vx[0xf], 0_u8);
    }

    #[test]
    fn shl_vx_vy_1_modern() {
        let mut chip = Chip::new(Profile::modern());

        chip.regs.vx[2] = 0xD5_u8;

        run_code(&mut chip, &[0x823E_u16]); // SHL V2, V3

        assert_eq!(chip.regs.vx[2], 0xAA_u8);
        assert_eq!(chip.regs.vx[0xf], 1_u8);
    }


    #[test]
    fn sne_vx_vy_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x00_u8;
        chip.regs.vx[3] = 0xD5_u8;

        run_code(&mut chip, &[0x9230_u16]); // SNE V2, V3

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn sne_vx_vy_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xD5_u8;
        chip.regs.vx[3] = 0xD5_u8;

        run_code(&mut chip, &[0x9230_u16]); // SNE V2, V3

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn ld_i_nnn_0() {
        let mut chip = Chip::new(Profile::original());

        run_code(&mut chip, &[0xA222_u16]); // LD I, 0x222

        assert_eq!(chip.regs.i, 0x222);
    }

    #[test]
    fn ld_i_nnn_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x222;

        run_code(&mut chip, &[0xA001_u16]); // LD I, 0x001

        assert_eq!(chip.regs.i, 0x001);
    }

    #[test]
    fn jp_v0_nnn_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[0] = 0x20_u8;

        run_code(&mut chip, &[0xB300_u16]); // JP V0, 0x300

        assert_eq!(chip.regs.pc, 0x320);
    }

    #[test]
    fn rnd_vx_nn_0() {
        let mut chip = Chip::new_seed(0x0102030405060708, Profile::original());

        run_code(&mut chip, &[0xC255_u16]); // RND V2, 0x55

        assert_eq!(chip.regs.vx[2], 0x10u8);
    }

    #[test]
    fn rnd_vx_nn_1() {
        let mut chip = Chip::new_seed(0x123456789abcdef0, Profile::original());

        run_code(&mut chip, &[0xC255_u16]); // RND V2, 0x55

        assert_eq!(chip.regs.vx[2], 0x00u8);
    }

    #[test]
    fn rnd_vx_nn_2() {
        let mut chip = Chip::new_seed(0xa3b400323f8bcf31, Profile::original());

        run_code(&mut chip, &[0xC255_u16]); // RND V2, 0x55

        assert_eq!(chip.regs.vx[2], 0x04u8);
    }

    #[test]
    fn rnd_vx_nn_3() {
        let mut chip = Chip::new_seed(0x0ba443debe98f002, Profile::original());

        run_code(&mut chip, &[0xC255_u16]); // RND V2, 0x55

        assert_eq!(chip.regs.vx[2], 0x01u8);
    }

    #[test]
    fn skp_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        run_code(&mut chip, &[0xE79E_u16]); // SKP V7

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn skp_vx_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        chip.key_press(2);
        chip.key_unpress(2);

        run_code(&mut chip, &[0xE79E_u16]); // SKP V7

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn skp_vx_2() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        chip.key_press(2);

        run_code(&mut chip, &[0xE79E_u16]); // SKP V7

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn skp_vx_3() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        chip.key_press(2);
        chip.key_unpress(2);
        chip.key_press(2);

        run_code(&mut chip, &[0xE79E_u16]); // SKP V7

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn skpn_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        run_code(&mut chip, &[0xE7A1_u16]); // SKPN V7

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn skpn_vx_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        chip.key_press(2);
        chip.key_unpress(2);

        run_code(&mut chip, &[0xE7A1_u16]); // SKPN V7

        assert_eq!(chip.regs.pc, 0x204);
    }

    #[test]
    fn skpn_vx_2() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        chip.key_press(2);

        run_code(&mut chip, &[0xE7A1_u16]); // SKPN V7

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn skpn_vx_3() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[7] = 0x02_u8;

        chip.key_press(2);
        chip.key_unpress(2);
        chip.key_press(2);

        run_code(&mut chip, &[0xE7A1_u16]); // SKPN V7

        assert_eq!(chip.regs.pc, 0x202);
    }

    #[test]
    fn ld_vx_dt_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.dt = 0x20;

        run_code(&mut chip, &[0xF207_u16]); // LD V2, DT

        assert_eq!(chip.regs.vx[2], 0x20u8);
    }

    #[test]
    fn ld_vx_k_0() {
        let mut chip = Chip::new(Profile::original());

        let code = [
            0xF20A_u16, // LD V2, K
        ];
        chip.ram.load_block_u16(0x200, &code);
        chip.set_pc(0x200);
        for _ in 0..5 {
            chip.cycle();
        }

        assert_eq!(chip.regs.pc, 0x200);
    }

    #[test]
    fn ld_vx_k_1() {
        let mut chip = Chip::new(Profile::original());

        chip.key_press(0xA);
        run_code(&mut chip, &[0xF20A_u16]); // LD V2, K

        assert_eq!(chip.regs.pc, 0x202);
        assert_eq!(chip.regs.vx[2], 0xA_u8);
    }

    #[test]
    fn ld_dt_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x20u8;

        run_code(&mut chip, &[0xF215_u16]); // LD DT, V2

        assert_eq!(chip.regs.dt, 0x20u8);
    }

    #[test]
    fn ld_st_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x20u8;

        run_code(&mut chip, &[0xF218_u16]); // LD ST, V2

        assert_eq!(chip.regs.st, 0x20u8);
    }

    #[test]
    fn add_i_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x1A30u16;
        chip.regs.vx[2] = 0x22u8;

        run_code(&mut chip, &[0xF21E_u16]); // ADD I, V2

        assert_eq!(chip.regs.i, 0x1A52u16);
        assert_eq!(chip.regs.vx[2], 0x22u8);
    }

    #[test]
    fn add_i_vx_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x1AF0u16;
        chip.regs.vx[2] = 0x22u8;

        run_code(&mut chip, &[0xF21E_u16]); // ADD I, V2

        assert_eq!(chip.regs.i, 0x1B12u16);
        assert_eq!(chip.regs.vx[2], 0x22u8);
    }

    #[test]
    fn ld_f_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x03u8;

        run_code(&mut chip, &[0xF229_u16]); // LD F, V2

        assert_eq!(chip.regs.i, chip.sprite_addr[0x03]);

        let mut s:Sprite = [0;5];
        for i in 0usize..5usize {
            let addr: u32 = (chip.regs.i + i as u16) as u32;
            s[i] = chip.ram.read_u8(addr);
        }

        let expected: Sprite = [
            // 0x3
            0b11100000,
            0b00010000,
            0b11100000,
            0b00010000,
            0b11100000
        ];
        assert_eq!(s, expected);
    }

    #[test]
    fn ld_b_vx_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x300;
        chip.regs.vx[2] = 123u8;

        run_code(&mut chip, &[0xF233_u16]); // LD B, V2

        assert_eq!(chip.ram.mem[0x300], 1u8);
        assert_eq!(chip.ram.mem[0x301], 2u8);
        assert_eq!(chip.ram.mem[0x302], 3u8);
    }

    #[test]
    fn ld_b_vx_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x300;
        chip.regs.vx[2] = 12u8;

        run_code(&mut chip, &[0xF233_u16]); // LD B, V2

        assert_eq!(chip.ram.mem[0x300], 0u8);
        assert_eq!(chip.ram.mem[0x301], 1u8);
        assert_eq!(chip.ram.mem[0x302], 2u8);
    }

    #[test]
    fn ld_b_vx_2() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x300;
        chip.regs.vx[2] = 1u8;

        run_code(&mut chip, &[0xF233_u16]); // LD B, V2

        assert_eq!(chip.ram.mem[0x300], 0u8);
        assert_eq!(chip.ram.mem[0x301], 0u8);
        assert_eq!(chip.ram.mem[0x302], 1u8);
    }

    #[test]
    fn ld_b_vx_3() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.i = 0x300;
        chip.regs.vx[2] = 0u8;

        run_code(&mut chip, &[0xF233_u16]); // LD B, V2

        assert_eq!(chip.ram.mem[0x300], 0u8);
        assert_eq!(chip.ram.mem[0x301], 0u8);
        assert_eq!(chip.ram.mem[0x302], 0u8);
    }

    fn ld_i_vx_init_regs(c: &mut Chip) {
        use crate::arch::NVREGS;
        for i in 0..NVREGS {
            c.regs.vx[i as usize] = (i+1) as u8;
        }
    }

    fn ld_i_vx_check_mem(c: &Chip, i_start: u16, idx: u32) {
        use crate::arch::NVREGS;

        // Registers up to (including) V[idx] copied into the memory from location I.
        for i in 0..=idx {
            let addr: u32 = i_start as u32 + i as u32;
            assert_eq!(c.ram.read_u8(addr), c.regs.vx[i]);
        }

        // Registers from V[idx + 1] are not copied into the memory.
        for i in idx+1..NVREGS {
            let addr: u32 = i_start as u32 + i as u32;
            assert_eq!(c.ram.read_u8(addr), 0);
        }
    }

    fn ld_i_vx_test(i: u32, store_i: bool) {
        let profile = if store_i { Profile::original() } else { Profile::modern() };
        let mut chip = Chip::new(profile);

        ld_i_vx_init_regs(&mut chip);

        const I: u16 = 0x300;
        chip.regs.i = I;

        let mut op: u16 = 0xF055_u16;
        op = op | (i << 8) as u16;

        run_code(&mut chip, &[op]); // LD [I], V2

        ld_i_vx_check_mem(&chip, I, i);
        if store_i {
            assert_eq!(chip.regs.i, I + i as u16 + 1);
        } else {
            assert_eq!(chip.regs.i, I);
        }
    }

    #[test]
    fn ld_i_vx_0_orig() {
        ld_i_vx_test(0, true);
    }

    #[test]
    fn ld_i_vx_1_orig() {
        ld_i_vx_test(2, true);
    }

    #[test]
    fn ld_i_vx_2_orig() {
        ld_i_vx_test(7, true);
    }

    #[test]
    fn ld_i_vx_3_orig() {
        ld_i_vx_test(0xf, true);
    }

    #[test]
    fn ld_i_vx_0_modern() {
        ld_i_vx_test(0, false);
    }

    #[test]
    fn ld_i_vx_1_modern() {
        ld_i_vx_test(2, false);
    }

    #[test]
    fn ld_i_vx_2_modern() {
        ld_i_vx_test(7, false);
    }

    #[test]
    fn ld_i_vx_3_modern() {
        ld_i_vx_test(0xf, false);
    }

    fn ld_vx_i_init_regs(c: &mut Chip) {
        use crate::arch::NVREGS;
        for i in 0..NVREGS {
            c.regs.vx[i] = 0xFFu8;
        }
    }

    fn ld_vx_i_init_mem(c: &mut Chip) {
        use crate::arch::NVREGS;
        for i in 0..NVREGS {
            let addr: u32 = c.regs.i as u32 + i as u32;
            c.ram.write_u8(addr, (i+1) as u8);
        }
    }

    fn ld_vx_i_check_regs(c: &Chip, i_start: u16, idx: u32) {
        use crate::arch::NVREGS;

        // Registers up to (including) V[idx] read from the memory from location I.
        for i in 0..=idx {
            let addr: u32 = i_start as u32 + i as u32;
            assert_eq!(c.ram.read_u8(addr), c.regs.vx[i]);
        }

        // Registers from V[idx + 1] are not read into the memory.
        for i in idx+1..NVREGS {
            assert_eq!(c.regs.vx[i], 0xFFu8);
        }
    }

    fn ld_vx_i_test(i: u32, store_i: bool) {
        let profile = if store_i { Profile::original() } else { Profile::modern() };
        let mut chip = Chip::new(profile);

        const I: u16 = 0x300;
        chip.regs.i = I;

        ld_vx_i_init_regs(&mut chip);
        ld_vx_i_init_mem(&mut chip);

        let mut op: u16 = 0xF065_u16;
        op = op | (i << 8) as u16;

        run_code(&mut chip, &[op]); // LD [I], V2

        ld_vx_i_check_regs(&chip, I, i);
        if store_i {
            assert_eq!(chip.regs.i, I + i as u16 + 1);
        } else {
            assert_eq!(chip.regs.i, I);
        }
    }

    #[test]
    fn ld_vx_i_0_orig() {
        ld_vx_i_test(0, true);
    }

    #[test]
    fn ld_vx_i_1_orig() {
        ld_vx_i_test(3, true);
    }


    #[test]
    fn ld_vx_i_2_orig() {
        ld_vx_i_test(7, true);
    }

    #[test]
    fn ld_vx_i_3_orig() {
        ld_vx_i_test(0xF, true);
    }

    #[test]
    fn ld_vx_i_0_modern() {
        ld_vx_i_test(0, false);
    }

    #[test]
    fn ld_vx_i_1_modern() {
        ld_vx_i_test(3, false);
    }

    #[test]
    fn ld_vx_i_2_modern() {
        ld_vx_i_test(7, false);
    }

    #[test]
    fn ld_vx_i_3_modern() {
        ld_vx_i_test(0xF, false);
    }

    #[test]
    fn add_vx_nn_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0x22_u8;

        run_code(&mut chip, &[0x7215_u16]);

        assert_eq!(chip.regs.vx[2], 0x37_u8);
    }

    #[test]
    fn add_vx_nn_1() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xFE_u8;

        run_code(&mut chip, &[0x7202_u16]);

        assert_eq!(chip.regs.vx[2], 0x00_u8);
    }

    #[test]
    fn add_vx_nn_2() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.vx[2] = 0xFF_u8;

        run_code(&mut chip, &[0x7215_u16]);

        assert_eq!(chip.regs.vx[2], 0x14_u8);
    }

    #[test]
    fn cycle_timers_0() {
        let mut chip = Chip::new(Profile::original());

        chip.regs.dt = 3;
        chip.regs.st = 2;

        chip.cycle_timers();

        assert_eq!(chip.regs.dt, 2);
        assert_eq!(chip.regs.st, 1);

        chip.cycle_timers();

        assert_eq!(chip.regs.dt, 1);
        assert_eq!(chip.regs.st, 0);

        chip.cycle_timers();

        assert_eq!(chip.regs.dt, 0);
        assert_eq!(chip.regs.st, 0);

        chip.cycle_timers();

        assert_eq!(chip.regs.dt, 0);
        assert_eq!(chip.regs.st, 0);
    }
}
