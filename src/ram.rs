use crate::arch;
use crate::util;

type RamBuf = util::Array<u8, { arch::RAMSIZE as usize}>;

pub struct Ram {
    pub mem: RamBuf,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            mem: RamBuf::new(),
        }
    }

    // TODO: handle overflow

    pub fn write_u8(&mut self, addr: u32, value: u8) {
        self.mem[addr] = value;
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        self.mem[addr]
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        u16::from_be_bytes([self.mem[addr], self.mem[addr + 1]])
    }

    pub fn write_u16(&mut self, addr: u32, v: u16) {
        self.mem[addr] = ((v & 0xff00) >> 8) as u8;
        self.mem[addr + 1] = (v & 0xff) as u8;
    }

    pub fn load_block_u16(&mut self, addr: u32, buf: &[u16]) {
        let mut addr = addr;
        for op in buf {
            self.write_u16(addr, *op);
            addr += 2;
        }
    }

    pub fn load_block_u8(&mut self, addr: u32, buf: &[u8]) -> u32 {
        let mut addr = addr;
        for op in buf {
            self.write_u8(addr, *op);
            addr += 1;
        }
        addr
    }
}

#[cfg(test)]
mod tests {
    use crate::ram::Ram;

    #[test]
    fn clear_when_created() {
        use crate::arch;
        let ram = Ram::new();

        for i in 0..arch::RAMSIZE {
            assert_eq!(ram.read_u8(i), 0);
        }
    }

    #[test]
    fn write_u8_read_u8() {
        let mut ram = Ram::new();

        ram.write_u8(0x0, 0x00);
        ram.write_u8(0x1, 0x10);

        assert_eq!(ram.read_u8(0x0), 0x00);
        assert_eq!(ram.read_u8(0x1), 0x10);

        ram.write_u8(0xff0, 0x00);
        ram.write_u8(0xff1, 0x10);

        assert_eq!(ram.read_u8(0xff0), 0x00);
        assert_eq!(ram.read_u8(0xff1), 0x10);
    }

    #[test]
    fn write_u16_read_u16() {
        let mut ram = Ram::new();

        ram.write_u16(0x0, 0x1122);
        ram.write_u16(0x2, 0x3344);

        assert_eq!(ram.read_u16(0x0), 0x1122);
        assert_eq!(ram.read_u16(0x2), 0x3344);
    }

    #[test]
    fn write_u16_read_u8() {
        let mut ram = Ram::new();

        ram.write_u16(0x0, 0x1122);
        assert_eq!(ram.read_u8(0x0), 0x11);
        assert_eq!(ram.read_u8(0x1), 0x22);

        ram.write_u16(0x2, 0x3344);
        assert_eq!(ram.read_u8(0x2), 0x33);
        assert_eq!(ram.read_u8(0x3), 0x44);
    }

    #[test]
    fn load_block_u16() {
        let mut ram = Ram::new();

        let data = [0x1122u16];
        ram.load_block_u16(0, &data);
        assert_eq!(ram.read_u16(0), 0x1122u16);

        let data = [0x1122u16, 0x3344u16, 0x5566u16];
        let mut addr = 0x200;
        ram.load_block_u16(addr, &data);
        for bb in data {
            assert_eq!(ram.read_u16(addr), bb);
            addr += 2;
        }
    }

    #[test]
    fn load_block_u8() {
        let mut ram = Ram::new();

        let data = [0x12];
        ram.load_block_u8(0, &data);
        assert_eq!(ram.read_u8(0), 0x12);

        let data = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let mut addr = 0x200;
        ram.load_block_u8(addr, &data);
        for bb in data {
            assert_eq!(ram.read_u8(addr), bb);
            addr += 1;
        }
    }
}
