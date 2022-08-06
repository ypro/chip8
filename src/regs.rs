use crate::arch;
use crate::util;

type VxRegs = util::Array<u8, {arch::NVREGS as usize}>;

pub struct RegMap {
    pub vx: VxRegs,
    pub dt: u8,
    pub st: u8,
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
}

impl RegMap {
    pub fn new() -> Self {
        RegMap {
            vx: VxRegs::new(),
            dt: 0,
            st: 0,
            i: 0,
            pc: 0,
            sp: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arch;
    use super::RegMap;

    #[test]
    fn new() {
        let regs = RegMap::new();
        for i in 0..arch::NVREGS as usize{
            assert_eq!(regs.vx[i], 0);
        }
        assert_eq!(regs.dt, 0);
        assert_eq!(regs.st, 0);
        assert_eq!(regs.i, 0);
        assert_eq!(regs.pc, 0);
        assert_eq!(regs.sp, 0);
    }
}
