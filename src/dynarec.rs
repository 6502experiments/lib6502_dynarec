use std::collections::HashMap;

use crate::CPU;

type JitFunction = Box<dyn Fn(&mut CPU) -> u16>;

pub struct Dynarec {
    compiled_blocks: HashMap<u16, JitFunction>,
}

impl Dynarec {
    pub fn new() -> Self {
        Dynarec {
            compiled_blocks: HashMap::new(),
        }
    }

    pub fn translate_block(&mut self, cpu: &mut CPU, start_pc: u16) -> &JitFunction {
        self.compiled_blocks.entry(start_pc).or_insert_with(|| {
            let mut pc = start_pc;
            let mut instructions = vec![];
    
            println!("Compiling block at pc: {:#X}", pc);
    
            while pc < 0xFFFF {
                let opcode = cpu.memory[pc as usize];
                pc += 1;
    
                println!("Compiling instruction: {:#X}", opcode);
    
                match opcode {
                    0xA9 => { // LDA Immediate
                        let value = cpu.memory[pc as usize];
                        pc += 1;
    
                        instructions.push(move |cpu: &mut CPU| {
                            println!("Executing instruction: LDA {:#X}", value);
                            cpu.a = value;
                            cpu.update_zero_and_negative_flags(cpu.a);
                            cpu.pc += 2; // Ensure PC correctly advances
                        });
                    }
                    0x00 => break, // BRK (stop execution)
                    _ => panic!("Unknown opcode in dynarec: {:#X}", opcode),
                }
            }
    
            println!("Returning compiled block at pc: {:#X}", start_pc);
    
            Box::new(move |cpu: &mut CPU| -> u16 {
                for inst in &instructions {
                    inst(cpu);
                }
                cpu.pc // Ensure PC is correctly updated
            })
        });
    
        self.compiled_blocks.get(&start_pc).unwrap()
    }    
}

#[cfg(test)]
mod tests {
    use crate::{CPU, Dynarec};

    #[test]
    fn test_lda_immediate_dynarec() {
        let mut cpu = CPU::new();
        let mut dynarec = Dynarec::new();

        // Load program at address 0x8000
        cpu.memory[0x8000] = 0xA9; // LDA #$42 (Immediate mode)
        cpu.memory[0x8001] = 0x42; // Load value 0x42 into A

        // Run the dynarec translation
        let jit_fn = dynarec.translate_block(&mut cpu, 0x8000);
        cpu.pc = 0x8000; // Reset PC
        cpu.pc = jit_fn(&mut cpu); // Execute compiled block

        // Assert the CPU state after execution
        assert_eq!(cpu.a, 0x42, "Accumulator should contain 0x42");
        assert_eq!(cpu.pc, 0x8002, "Program Counter should advance by 2");
        assert_eq!(cpu.status & 0b0000_0010, 0, "Zero flag should not be set");
        assert_eq!(cpu.status & 0b1000_0000, 0, "Negative flag should not be set");
    }

    #[test]
    fn test_lda_immediate_zero_flag() {
        let mut cpu = CPU::new();
        let mut dynarec = Dynarec::new();

        // Load program at address 0x8000
        cpu.memory[0x8000] = 0xA9; // LDA #$00
        cpu.memory[0x8001] = 0x00; // Load 0x00 into A

        // Run the dynarec translation
        let jit_fn = dynarec.translate_block(&mut cpu, 0x8000);
        cpu.pc = 0x8000;
        cpu.pc = jit_fn(&mut cpu);

        // Assert CPU state after execution
        assert_eq!(cpu.a, 0x00, "Accumulator should be 0x00");
        assert_eq!(cpu.pc, 0x8002, "Program Counter should advance by 2");
        assert_eq!(cpu.status & 0b0000_0010, 0b0000_0010, "Zero flag should be set");
    }

    #[test]
    fn test_lda_immediate_negative_flag() {
        let mut cpu = CPU::new();
        let mut dynarec = Dynarec::new();

        // Load program at address 0x8000
        cpu.memory[0x8000] = 0xA9; // LDA #$80
        cpu.memory[0x8001] = 0x80; // Load 0x80 into A (negative bit set)

        // Run the dynarec translation
        let jit_fn = dynarec.translate_block(&mut cpu, 0x8000);
        cpu.pc = 0x8000;
        cpu.pc = jit_fn(&mut cpu);

        // Assert CPU state after execution
        assert_eq!(cpu.a, 0x80, "Accumulator should be 0x80");
        assert_eq!(cpu.pc, 0x8002, "Program Counter should advance by 2");
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000, "Negative flag should be set");
    }
}
