#[derive(Debug, Clone)]
pub struct Command {
    /// 0…7
    pub channel: u8,
    /// 12 bits (false), 8 bits (true)
    pub mode: bool,
    /// Single-Ended (true), Differential Reference (false)
    pub ser_dfr: bool,
    /// Power reference
    pub pd1: bool,
    /// Power ADC
    pub pd0: bool,
}

impl Into<u8> for Command {
    fn into(self: Command) -> u8 {
        #[inline(always)]
        fn flag(b: bool, shift: u8) -> u8 {
            let x = if b { 1 } else { 0 };
            x << shift
        }

        0x80 |
        (self.channel << 4) |
        flag(self.mode, 3) |
        flag(self.ser_dfr, 2) |
        flag(self.pd1, 1) |
        flag(self.pd0, 0)
    }
}
