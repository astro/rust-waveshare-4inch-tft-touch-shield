use super::{
    channels,
    command::Command,
};

pub const XY_READS: usize = 4;

pub fn read_commands() -> ReadCommands {
    ReadCommands {
        n: 0,
    }
}

pub struct ReadCommands {
    n: usize,
}

impl Iterator for ReadCommands {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.n += 1;

        let channel =
            if self.n <= 2 * XY_READS {
                if self.n & 1 != 0 {
                    channels::X
                } else {
                    channels::Y
                }
            } else if self.n == 2 * XY_READS + 1 {
                channels::Z1
            } else if self.n == 2 * XY_READS + 2 {
                channels::Z2
            } else {
                return None;
            };
        Some(Command {
            channel,
            mode: false,
            ser_dfr: self.n > 2 * XY_READS,
            pd1: self.n <= 8,
            pd0: self.n <= 8,
        })
    }
}
