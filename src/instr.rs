pub struct Instr {
    pub opcode: u16,
    pub c: u8,
    pub x: u8,
    pub y: u8,
    pub n: u8,
    pub nn: u8,
    pub nnn: u16,
}

impl Instr {
    pub fn new(opcode: u16) -> Self {
        Instr {
            opcode,
            c: ((opcode & 0xf000) >> 12) as u8,
            x: ((opcode & 0x0f00) >> 8) as u8,
            y: ((opcode & 0x00f0) >> 4) as u8,
            n: (opcode & 0x000f) as u8,
            nn: (opcode & 0x00ff) as u8,
            nnn: (opcode & 0x0fff) as u16,
        }
    }
}
