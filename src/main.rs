use std::{u8, usize};

use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
struct PS {
    c: bool, //Carry Flag
    z: bool, //Zero Flag
    i: bool, //Interupt Disable
    d: bool, //Decimal Mode
    b: bool, //Break Command
    u: B1,   //Unused
    v: bool, //Overflow Flag
    n: bool, //Negative Flag
}

struct MEM {
    data: [u8; 65536]
}

impl MEM {
    fn initialize(&mut self) {
        self.data = [0; 65536];
    }

    fn write_word(&mut self, value: &u16, addr: &u16, cycles: &mut u32) {
        self.data[*addr as usize] = *value as u8;
        self.data[*addr as usize + 1] = (*value >> 8) as u8;
        *cycles -= 2;
    }
}

struct CPU {
    pc: u16, //Program Counter
    sp: u8,  //Stack Poniter
    a: u8,   //Accumulator
    x: u8,   //Register X
    y: u8,   //Register Y
    ps: PS,  //Processor Status
}

impl CPU {
    // Upcodes
    const INS_LDA_IM: u8 = 0xA9;
    const INS_LDA_ZP: u8 = 0xA5;
    const INS_LDA_ZX: u8 = 0xB5;
    const INS_LDA_AB: u8 = 0xAD;
    const INS_LDA_AX: u8 = 0xBD;
    const INS_LDA_AY: u8 = 0xB9;
    const INS_LDA_IX: u8 = 0xA1;
    const INS_LDA_IY: u8 = 0xB1;
    const INS_JSR: u8 = 0x20;

    fn reset(&mut self, mem: &mut MEM){
        self.pc = 0xFFFC;
        self.sp = 0xFE;
        self.ps = PS::new();
        self.a = 0;
        self.x = 0;
        self.y = 0;
        mem.initialize();
    }

    fn fetch_byte(&mut self, cycles: &mut u32, memory: &MEM) -> u8 {
        let data = memory.data[self.pc as usize];
        
        self.pc += 1;
        *cycles -= 1;
        return data;
    }

    fn fetch_word(&mut self, cycles: &mut u32, memory: &MEM) -> u16 {
        // 6502 is little endian
        let mut data = memory.data[self.pc as usize] as u16;        
        self.pc += 1;

        data |= (memory.data[self.pc as usize] as u16) << 8;        
        self.pc += 1;
        
        *cycles -= 2;

        return data;
    }

    fn read_byte(&mut self, cycles: &mut u32, memory: &MEM, addr: u16) -> u8 {
        *cycles -= 1;
        memory.data[addr as usize]
    }

    fn lda_set_status(&mut self) {
        self.ps.set_z(self.a == 0);
        self.ps.set_n((self.a & 0b1000000) > 0);
    }

    fn execute(&mut self, mut cycles: u32, memory: &mut MEM) {
        while cycles > 0 {
            let instruction = self.fetch_byte(&mut cycles, memory);

            match instruction {
                CPU::INS_LDA_IM => {
                    let value: u8 = self.fetch_byte(&mut cycles, memory);
                    self.a = value;
                    self.lda_set_status();
                },
                CPU::INS_LDA_ZP => {
                    let zero_page_addr = self.fetch_byte(&mut cycles, memory) as u16;
                    self.a = self.read_byte(&mut cycles, memory,zero_page_addr);                    
                    self.lda_set_status();
                },
                CPU::INS_LDA_ZX => {
                    let zero_page_addr = (self.fetch_byte(&mut cycles, memory) + self.x) as u16;
                    cycles -= 1;
                    self.a = self.read_byte(&mut cycles, memory, zero_page_addr);                    
                    self.lda_set_status();
                },
                CPU::INS_JSR => {
                    let sub_addr = self.fetch_word(&mut cycles, memory);
                    memory.write_word(&(self.pc - 1),&(self.sp as u16), &mut cycles);
                    self.pc = sub_addr;
                    self.sp += 1;
                    cycles -= 1;
                }
                _ => print!("Instruction not handled {0}", instruction),
            };
        }
    }
}

fn main() {
    let mut mem: MEM = MEM {
        data: [0; 65536],
    };
    
    let mut cpu: CPU = CPU {
        pc: 0,
        sp: 0,
        a: 0,
        x: 0,
        y: 0,
        ps: PS::new(),
    };

    cpu.reset(&mut mem);

    mem.data[0xFFFC] = CPU::INS_JSR;
    mem.data[0xFFFD] = 0x42;
    mem.data[0xFFFE] = 0x42;
    mem.data[0x4242] = CPU::INS_LDA_IM;
    mem.data[0x4243] = 0x84;
    
    cpu.execute(8, &mut mem);
}