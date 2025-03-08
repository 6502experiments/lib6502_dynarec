use dynarec::Dynarec;

mod dynarec;

#[derive(Debug, Clone)]
pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,
    pub memory: [u8; 0x10000],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0x8000,
            status: 0x24,
            memory: [0; 0x10000],
        }
    }

    pub fn run(&mut self, dynarec: &mut Dynarec) {
        loop {
            let block = dynarec.translate_block(self, self.pc);
            self.pc = block(self);
        }
    }

    fn execute_instruction(&mut self) {
        let opcode = self.memory[self.pc as usize];
        self.pc += 1;

        match opcode {
            0xA9 => {
                let value = self.memory[self.pc as usize];
                self.pc += 1;
                self.a = value;
                self.update_zero_and_negative_flags(self.a);
            }
            _ => {
                println!("Unsupported opcode: {:#02x}", opcode);
            }
        }
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        if value == 0 {
            self.status |= 0b0000_0010; // Set Zero flag
        } else {
            self.status &= !0b0000_0010; // Clear Zero flag
        }

        if value & 0b1000_0000 != 0 {
            self.status |= 0b1000_0000; // Set Negative flag
        } else {
            self.status &= !0b1000_0000; // Clear Negative flag
        }
    }
}