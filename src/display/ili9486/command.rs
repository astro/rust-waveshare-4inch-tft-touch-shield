pub trait Command {
    type Buffer: AsRef<[u8]>;

    fn number() -> u8;
    fn encode(self) -> Self::Buffer;
}

macro_rules! simple_command {
    ($name: ident, $number: tt) => (
        pub struct $name;

        impl Command for $name {
            type Buffer = [u8; 0];

            fn number() -> u8 {
                $number
            }

            fn encode(self) -> Self::Buffer {
                []
            }
        }
    )
}

pub struct MemoryWrite<'a>(pub &'a mut [u8]);

impl<'a> Command for MemoryWrite<'a> {
    type Buffer = &'a [u8];

    fn number() -> u8 {
        0x2C
    }

    fn encode(self) -> Self::Buffer {
        self.0
    }
}

simple_command!(SleepIn, 0x10);
simple_command!(SleepOut, 0x11);
simple_command!(InversionOn, 0x21);
simple_command!(InversionOff, 0x28);
simple_command!(DisplayOn, 0x29);

pub struct MemoryAccessControl {
    pub row_addr_order: bool,
    pub col_addr_order: bool,
    pub row_col_exchange: bool,
    pub vert_refresh_order: bool,
    pub horiz_refresh_order: bool,
    pub rgb_to_bgr: bool,
}

impl Command for MemoryAccessControl {
    type Buffer = [u8; 1];

    fn number() -> u8 {
        0x36
    }

    fn encode(self) -> Self::Buffer {
        fn bit_if(condition: bool, bit: u8) -> u8 {
            if condition {
                1 << bit
            } else {
                0
            }
        }
        let my = bit_if(self.row_addr_order, 7);
        let mx = bit_if(self.col_addr_order, 6);
        let mv = bit_if(self.row_col_exchange, 5);
        let ml = bit_if(self.vert_refresh_order, 4);
        let bgr = bit_if(self.rgb_to_bgr, 3);
        let mh = bit_if(self.horiz_refresh_order, 2);
        [my | mx | mv | ml | bgr | mh]
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PixelFormat {
    Bpp16 = 0b101,
    Bpp18 = 0b110,
}

impl From<u8> for PixelFormat {
    fn from(x: u8) -> Self {
        match x {
            0b101 => PixelFormat::Bpp16,
            0b110 => PixelFormat::Bpp18,
            _ => panic!("Unknown pixel format {:02X}", x),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InterfacePixelFormat {
    pub cpu_format: PixelFormat,
    pub rgb_format: PixelFormat,
}

impl Command for InterfacePixelFormat {
    type Buffer = [u8; 1];

    fn number() -> u8 {
        0x3A
    }

    fn encode(self) -> Self::Buffer {
        [((self.cpu_format as u8) << 4) |
         (self.rgb_format as u8)]
    }
}

pub struct ReadInterfacePixelFormat;

impl Command for ReadInterfacePixelFormat {
    type Buffer = [u8; 2];

    fn number() -> u8 {
        0x0C
    }

    fn encode(self) -> Self::Buffer {
        [0, 0]
    }
}

