pub trait Command {
    type Buffer: AsMut<[u8]>;
    type Response;

    fn number(&self) -> u8;
    fn encode(self) -> Self::Buffer;
    fn decode(&Self::Buffer) -> Self::Response;
}

macro_rules! simple_command {
    ($name: ident, $number: tt) => (
        pub struct $name;

        impl Command for $name {
            type Buffer = [u8; 0];
            type Response = ();

            fn number(&self) -> u8 {
                $number
            }

            fn encode(self) -> Self::Buffer {
                []
            }

            fn decode(_buffer: &Self::Buffer) -> Self::Response {
                ()
            }
        }
    )
}

pub struct MemoryWrite<'a>(pub &'a mut [u8]);

impl<'a> Command for MemoryWrite<'a> {
    type Buffer = &'a mut [u8];
    type Response = ();

    fn number(&self) -> u8 {
        0x2C
    }

    fn encode(self) -> Self::Buffer {
        self.0
    }

    fn decode(_buffer: &Self::Buffer) -> Self::Response {
        ()
    }
}

#[derive(Debug, Clone)]
pub struct DisplayIdentification {
    module_manufacturer: u8,
    module_version: u8,
    module_id: u8,
}

pub struct ReadDisplayIdentification;

impl Command for ReadDisplayIdentification {
    type Buffer = [u8; 4];
    type Response = DisplayIdentification;

    fn number(&self) -> u8 {
        0x04
    }

    fn encode(self) -> Self::Buffer {
        [0; 4]
    }

    fn decode(buffer: &Self::Buffer) -> Self::Response {
        DisplayIdentification {
            module_manufacturer: buffer[1],
            module_version: buffer[2],
            module_id: buffer[3],
        }
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
    type Response = ();

    fn number(&self) -> u8 {
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

    fn decode(buffer: &Self::Buffer) -> Self::Response {
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PixelFormat {
    Bpp16 = 0b101,
    Bpp18 = 0b110,
}

pub struct InterfacePixelFormat {
    pub cpu_format: PixelFormat,
    pub rgb_format: PixelFormat,
}

impl Command for InterfacePixelFormat {
    type Buffer = [u8; 1];
    type Response = ();

    fn number(&self) -> u8 {
        0x3A
    }

    fn encode(self) -> Self::Buffer {
        [((self.cpu_format as u8) << 4) |
         (self.cpu_format as u8)]
    }

    fn decode(buffer: &Self::Buffer) -> Self::Response {
    }
}

