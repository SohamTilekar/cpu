use crate::cpu::registers::Registers;
use log::{debug, error, info, trace};
mod registers;

pub enum CPUError {
    Ok,
    PcOutOfBounds,
    Halt,
}

pub struct CPU {
    pub registers: Registers,
    pub ram: Box<[u32]>, // Allocating on the heap
    pub log: bool,
    pub clock: u64,
    pub clock_speed: f32,
    pub l_executed_t: std::time::Instant,
    pub hz: f64,
    pub run_fast: bool,
    pub last_update_time: std::time::Instant,
    pub opcode: u32,
    pub ir: u32,
    pub dr: usize,
    pub sr2: usize,
    pub sr1: usize,
    pub immediate: u32,
    pub recent_memory_accesses: (u32, u32),
}

impl CPU {
    pub fn new(initial_ram_content: Vec<u32>, log: bool, clock_speed: f32) -> CPU {
        if log {
            info!("Initializing CPU");
            debug!("Initialing 64MB RAM");
        }
        let mut ram: Vec<u32> = Vec::with_capacity(16777216);
        if log {
            trace!("RAM Filled with random values");
        }
        // let mut value: u32 = random(); // Seed value
        for i in 0..16777216 {
            // Simple LCG formula: value = (value * 1664525 + 1013904223) % 2^32
            if i < initial_ram_content.len() {
                ram.push(initial_ram_content[i]);
                continue;
            }
            let value = rand::random();
            // value = value.wrapping_mul(1664525).wrapping_add(1013904223);
            // value = value ^ (value >> 16);
            // value = value ^ (value >> 8);
            // value = value ^ (value >> 4);
            // value = value ^ (value >> 2);
            // value = value ^ (value >> 1);
            ram.push(value);
        }
        if log {
            for i in 0..10 {
                trace!("RAM[{}] = {}", i, ram[i]);
            }
        }
        return CPU {
            registers: Registers::new(),
            ram: ram.into_boxed_slice(),
            log,
            clock: 0,
            clock_speed,
            l_executed_t: std::time::Instant::now(),
            hz: 0.0,
            run_fast: false,
            last_update_time: std::time::Instant::now(),
            opcode: 0,
            ir: 0,
            dr: 0,
            sr2: 0,
            sr1: 0,
            immediate: 0,
            recent_memory_accesses: (0, 0),
        };
    }
    pub fn reset(&mut self) {
        self.registers = Registers::new();
    }
    pub fn restart(&mut self) {
        let run_fast = self.run_fast;
        *self = CPU::new(Vec::new(), self.log, self.clock_speed);
        self.run_fast = run_fast;
    }
    fn set_ram(&mut self, address: usize, value: u32) {
        self.recent_memory_accesses = (address as u32, value);
        self.ram[address & 0xFFFFFF] = value;
    }
    fn get_ram(&mut self, address: usize) -> u32 {
        self.recent_memory_accesses = (address as u32, self.ram[address & 0xFFFFFF]);
        return self.ram[address & 0xFFFFFF];
    }
    pub fn execute_instruction(&mut self, interrupt: bool, _interrupt_number: u8) -> CPUError {
        self.l_executed_t = std::time::Instant::now();
        if interrupt {}
        // Fetch instruction from memory
        if self.registers.pc >= self.ram.len() {
            println!("PC out of bounds");
            if self.log {
                error!("PC out of bounds");
            }
            return CPUError::PcOutOfBounds;
        }
        let instr = self.get_ram(self.registers.pc);
        if self.log {
            trace!("PC: {}, Instruction: {}", self.registers.pc, instr);
        }
        self.registers.pc += 1;
        self.clock += 2;
        // Decode & Execute instruction
        self.ir = instr;
        let opcode = instr >> 28 & 0x0F;
        self.opcode = opcode;
        let dr: usize = ((instr >> 16) & 0x0F).try_into().unwrap();
        self.dr = dr;
        let sr2: usize = ((instr >> 20) & 0x0F).try_into().unwrap();
        self.sr2 = sr2;
        let sr1: usize = ((instr >> 24) & 0x0F).try_into().unwrap();
        self.sr1 = sr1;
        let immediate: u32 = instr & 0xFFFFFF;
        self.immediate = immediate;
        match opcode {
            0 => {
                // NOP
                if self.log {
                    trace!("NOP");
                }
                self.clock += 1;
                return CPUError::Ok;
            }
            1 => {
                // ALU Calculate
                let alu_op = (instr >> 12) & 0x0F;
                if self.log {
                    trace!("ALU Opcode: {}", alu_op);
                }
                match alu_op {
                    0 => {
                        // ADD
                        self.registers[dr] = self.registers[sr1].wrapping_add(self.registers[sr2]);
                        self.registers.carry_f = self.registers[dr] < self.registers[sr1]
                            || self.registers[dr] < self.registers[sr2];
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("ADD: R{} = R{} + R{}", dr, sr1, sr2);
                            trace!(
                                "Flags: Carry = {}, Zero = {}",
                                self.registers.carry_f,
                                self.registers.zero_f
                            );
                        }
                    }
                    1 => {
                        // SUB
                        self.registers[dr] = self.registers[sr1].wrapping_sub(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("SUB: R{} = R{} - R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    2 => {
                        // MUL
                        self.registers[dr] = self.registers[sr1].wrapping_mul(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("MUL: R{} = R{} * R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    3 => {
                        // Nig
                        self.registers[dr] = self.registers[sr1].wrapping_neg();
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("Nig: R{} = R{}", dr, sr1);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    4 => {
                        // And
                        self.registers[dr] = self.registers[sr1] & self.registers[sr2];
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("AND: R{} = R{} & R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    5 => {
                        // OR
                        self.registers[dr] = self.registers[sr1] | self.registers[sr2];
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("OR: R{} = R{} | R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    6 => {
                        // Not
                        self.registers[dr] = !self.registers[sr1];
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("NOT: R{} = !R{}", dr, sr1);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    7 => {
                        // NAnd
                        self.registers[dr] = !(self.registers[sr1] & self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("NAND: R{} = !(R{} & R{})", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    8 => {
                        // XOr
                        self.registers[dr] = self.registers[sr1] ^ self.registers[sr2];
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("XOR: R{} = R{} ^ R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    9 => {
                        // XNor
                        self.registers[dr] = !(self.registers[sr1] ^ self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("XNOR: R{} = !(R{} ^ R{})", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    10 => {
                        // SFT Log Left
                        self.registers[dr] = self.registers[sr1].wrapping_shl(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("SFT_LOG_LEFT: R{} = R{} << R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    11 => {
                        // SFT Log Right
                        self.registers[dr] = self.registers[sr1].wrapping_shr(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("SFT_LOG_RIGHT: R{} = R{} >> R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    12 => {
                        // ArtM Right
                        self.registers[dr] = self.registers[sr1].wrapping_shr(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("ARTM_RIGHT: R{} = R{} >> R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    13 => {
                        // Rotate Left
                        self.registers[dr] = self.registers[sr1].rotate_left(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("ROTATE_LEFT: R{} = R{} rotate_left R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    14 => {
                        // Rotate Right
                        self.registers[dr] = self.registers[sr1].rotate_right(self.registers[sr2]);
                        self.registers.zero_f = self.registers[dr] == 0;
                        if self.log {
                            trace!("ROTATE_RIGHT: R{} = R{} rotate_right R{}", dr, sr1, sr2);
                            trace!("Flags: Zero = {}", self.registers.zero_f);
                        }
                    }
                    15 => {
                        // Default 0
                        self.registers[dr] = 0;
                        self.registers.zero_f = true;
                        if self.log {
                            trace!("DEFAULT: R{} = 0", dr);
                            trace!("Flags: Zero = 1");
                        }
                    }
                    _ => {
                        println!("Invalid ALU opcode: {}", alu_op);
                        if self.log {
                            error!("Invalid ALU opcode: {}", alu_op);
                        }
                        std::process::exit(1);
                    }
                }
                self.clock += 1;
                return CPUError::Ok;
            }
            2 => {
                // ALU Compare
                let alu_op = (instr >> 12) & 0x0F;
                match alu_op {
                    0 => {
                        // Greater Than
                        self.registers.comp_f = self.registers[sr1] > self.registers[sr2];
                        if self.log {
                            trace!("ALU_COMPARE: R{} > R{}", sr1, sr2);
                            trace!("Flags: Comp = {}", self.registers.comp_f);
                        }
                    }
                    1 => {
                        // Equal To
                        self.registers.comp_f = self.registers[sr1] == self.registers[sr2];
                        if self.log {
                            trace!("ALU_COMPARE: R{} == R{}", sr1, sr2);
                            trace!("Flags: Comp = {}", self.registers.comp_f);
                        }
                    }
                    2 => {
                        // Less Than
                        self.registers.comp_f = self.registers[sr1] < self.registers[sr2];
                        if self.log {
                            trace!("ALU_COMPARE: R{} < R{}", sr1, sr2);
                            trace!("Flags: Comp = {}", self.registers.comp_f);
                        }
                    }
                    3 => {
                        // Greater Than or Equal To
                        self.registers.comp_f = self.registers[sr1] >= self.registers[sr2];
                        if self.log {
                            trace!("ALU_COMPARE: R{} >= R{}", sr1, sr2);
                            trace!("Flags: Comp = {}", self.registers.comp_f);
                        }
                    }
                    4 => {
                        // Less Than or Equal To
                        self.registers.comp_f = self.registers[sr1] <= self.registers[sr2];
                        if self.log {
                            trace!("ALU_COMPARE: R{} <= R{}", sr1, sr2);
                            trace!("Flags: Comp = {}", self.registers.comp_f);
                        }
                    }
                    5..16 => {
                        // Default False
                        self.registers.comp_f = false;
                        if self.log {
                            trace!("ALU_COMPARE: Default False");
                            trace!("Flags: Comp = false");
                        }
                    }
                    _ => {
                        println!("Invalid ALU opcode: {}", alu_op);
                        std::process::exit(1);
                    }
                }
                self.clock += 1;
                return CPUError::Ok;
            }
            3 => {
                // Jump
                if instr >> 27 & 0x01 == 1 {
                    // Jump Immediate
                    let jmp_if = (instr >> 24) & 0x07;
                    if self.log {
                        trace!("Jump Immediate Opcode: {}", jmp_if);
                    }
                    match jmp_if {
                        0 => {
                            // Jump if Any
                            self.registers.pc = immediate as usize;
                            if self.log {
                                trace!("Jumping to Immediate: {}", immediate);
                            }
                        }
                        1 => {
                            // Jump if Carry
                            if self.registers.carry_f {
                                self.registers.pc = immediate as usize;
                                if self.log {
                                    trace!("Jumping to Immediate: {}", immediate);
                                }
                            }
                        }
                        2 => {
                            // Jump if Not Carry
                            if !self.registers.carry_f {
                                self.registers.pc = immediate as usize;
                                if self.log {
                                    trace!("Jumping to Immediate: {}", immediate);
                                }
                            }
                        }
                        3 => {
                            // Jump if Comp
                            if self.registers.comp_f {
                                self.registers.pc = immediate as usize;
                                if self.log {
                                    trace!("Jumping to Immediate: {}", immediate);
                                }
                            }
                        }
                        4 => {
                            // Jump if Not Comp
                            if !self.registers.comp_f {
                                self.registers.pc = immediate as usize;
                                if self.log {
                                    trace!("Jumping to Immediate: {}", immediate);
                                }
                            }
                        }
                        5 => {
                            // Jump if Zero
                            if self.registers.zero_f {
                                self.registers.pc = immediate as usize;
                                if self.log {
                                    trace!("Jumping to Immediate: {}", immediate);
                                }
                            }
                        }
                        6 => {
                            // Jump if Not Zero
                            if !self.registers.zero_f {
                                self.registers.pc = immediate as usize;
                                if self.log {
                                    trace!("Jumping to Immediate: {}", immediate);
                                }
                            }
                        }
                        7 => {
                            // Default Jump
                            self.registers.pc = immediate as usize;
                            if self.log {
                                trace!("Jumping to Immediate: {}", immediate);
                            }
                        }
                        _ => {
                            println!("Invalid Jump Immediate opcode: {}", jmp_if);
                            if self.log {
                                error!("Invalid Jump Immediate opcode: {}", jmp_if);
                            }
                            std::process::exit(1);
                        }
                    };
                    self.clock += 1;
                    return CPUError::Ok;
                } else {
                    // Jump Register
                    let jmp_if = (instr >> 24) & 0x07;
                    if self.log {
                        trace!("Jump Register Opcode: {}", jmp_if);
                    }
                    match jmp_if {
                        0 => {
                            // Jump if Any
                            self.registers.pc = sr2 & 0xFFFFFF as usize;
                            if self.log {
                                trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                            }
                        }
                        1 => {
                            // Jump if Carry
                            if self.registers.carry_f {
                                self.registers.pc = sr2 & 0xFFFFFF as usize;
                                if self.log {
                                    trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                                }
                            }
                        }
                        2 => {
                            // Jump if Not Carry
                            if !self.registers.carry_f {
                                self.registers.pc = sr2 & 0xFFFFFF as usize;
                                if self.log {
                                    trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                                }
                            }
                        }
                        3 => {
                            // Jump if Comp
                            if self.registers.comp_f {
                                self.registers.pc = sr2 & 0xFFFFFF as usize;
                                if self.log {
                                    trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                                }
                            }
                        }
                        4 => {
                            // Jump if Not Comp
                            if !self.registers.comp_f {
                                self.registers.pc = sr2 & 0xFFFFFF as usize;
                                if self.log {
                                    trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                                }
                            }
                        }
                        5 => {
                            // Jump if Zero
                            if self.registers.zero_f {
                                self.registers.pc = sr2 & 0xFFFFFF as usize;
                                if self.log {
                                    trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                                }
                            }
                        }
                        6 => {
                            // Jump if Not Zero
                            if !self.registers.zero_f {
                                self.registers.pc = sr2 & 0xFFFFFF as usize;
                                if self.log {
                                    trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                                }
                            }
                        }
                        7 => {
                            // Default Jump
                            self.registers.pc = sr2 & 0xFFFFFF as usize;
                            if self.log {
                                trace!("Jumping to Register: {}", sr2 & 0xFFFFFF);
                            }
                        }
                        _ => {
                            println!("Invalid Jump opcode: {}", jmp_if);
                            if self.log {
                                error!("Invalid Jump opcode: {}", jmp_if);
                            }
                            std::process::exit(1);
                        }
                    }
                    self.clock += 1;
                    return CPUError::Ok;
                }
            }
            4 => {
                // Load Full-bit
                self.registers[dr] = self.get_ram(immediate as usize);
                self.clock += 2;
                if self.log {
                    trace!(
                        "Load Full-bit: Register[{}] = RAM[{}]",
                        dr,
                        immediate as usize
                    );
                }
                return CPUError::Ok;
            }
            5 => {
                // Load From Reg
                self.clock += 2;
                self.registers[dr] = self.get_ram(self.registers[sr2] as usize);
                if self.log {
                    trace!("Load From Reg: Register[{}] = Register[{}]", dr, sr2);
                }
                return CPUError::Ok;
            }
            6 => {
                // Load Immediate
                self.clock += 1;
                self.registers[dr] = immediate;
                if self.log {
                    trace!("Load Immediate: Register[{}] = {}", dr, immediate);
                }
                return CPUError::Ok;
            }
            7 => {
                // Store
                self.set_ram(immediate as usize, self.registers[sr1]);
                self.clock += 1;
                if self.log {
                    trace!("Store: RAM[{}] = Register[{}]", immediate as usize, sr1);
                }
                return CPUError::Ok;
            }
            8 => {
                // Store To
                self.set_ram(
                    (self.registers[sr2] & 0xFFFFFF) as usize,
                    self.registers[sr1],
                );
                self.clock += 1;
                if self.log {
                    trace!(
                        "Store To: RAM[{}] = Register[{}]",
                        self.registers[sr2] & 0xFFFFFF,
                        sr1
                    );
                }
                return CPUError::Ok;
            }
            9 => {
                // Mov
                self.registers[dr] = self.registers[sr1];
                self.clock += 1;
                if self.log {
                    trace!("Mov: Register[{}] = Register[{}]", dr, sr1);
                }
                return CPUError::Ok;
            }
            10 | 11 => {
                return CPUError::Ok;
            }
            12 => {
                // Stack Operation
                let stack_op = (instr >> 22) & 0x03;
                if self.log {
                    trace!("Stack Operation Opcode: {}", stack_op);
                }
                match stack_op {
                    0 => {
                        // Push
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        self.set_ram((self.registers.sp & 0xFFFFFF) as usize, self.registers[sr1]);
                        if self.log {
                            trace!(
                                "Push: RAM[{}] = Register[{}]",
                                self.registers.sp & 0xFFFFFF,
                                sr1
                            );
                        }
                        self.clock += 1;
                        return CPUError::Ok;
                    }
                    1 => {
                        // Pop
                        self.registers[dr] = self.get_ram((self.registers.sp & 0xFFFFFF) as usize);
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if self.log {
                            trace!(
                                "Pop: Register[{}] = RAM[{}]",
                                dr,
                                (self.registers.sp.wrapping_sub(1) & 0xFFFFFF) as usize
                            );
                        }
                        self.clock += 1;
                        return CPUError::Ok;
                    }
                    2 => {
                        // Top
                        self.registers[dr] = self.get_ram(self.registers.sp as usize);
                        if self.log {
                            trace!(
                                "Top: Register[{}] = RAM[{}]",
                                dr,
                                self.registers.sp & 0xFFFFFF
                            );
                        }
                        self.clock += 1;
                        return CPUError::Ok;
                    }
                    3 => {
                        // Default 0
                        self.registers[dr] = 0;
                        if self.log {
                            trace!("Default 0: Register[{}] = 0", dr);
                        }
                        self.clock += 1;
                        return CPUError::Ok;
                    }
                    _ => {
                        println!("Invalid Stack opcode: {}", stack_op);
                        if self.log {
                            error!("Invalid Stack opcode: {}", stack_op);
                        }
                        std::process::exit(1);
                    }
                }
            }
            13 => {
                // Function
                let ret = instr >> 25 & 0x01;
                if self.log {
                    trace!(
                        "Function Operation: {}",
                        if ret == 1 { "Return" } else { "Call" }
                    );
                }
                if ret == 1 {
                    // Return
                    self.registers.pc =
                        self.get_ram((self.registers.sp & 0xFFFFFF) as usize) as usize;
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    let flags = self.get_ram((self.registers.sp & 0xFFFFFF) as usize);
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    self.registers.carry_f = flags & 0x01 == 1;
                    self.registers.zero_f = flags >> 1 & 0x01 == 1;
                    self.registers.comp_f = flags >> 2 & 0x01 == 1;
                    if self.log {
                        trace!("Return: PC = {}, Flags = 0x{:X}", self.registers.pc, flags);
                    }
                    self.clock += 2;
                    return CPUError::Ok;
                } else {
                    // Call
                    self.registers.sp = self.registers.sp.wrapping_sub(1);
                    self.set_ram(
                        (self.registers.sp & 0xFFFFFF) as usize,
                        (self.registers.pc & 0xFFFFFF) as u32,
                    );
                    self.registers.pc = immediate as usize;
                    self.registers.sp = self.registers.sp.wrapping_sub(1);
                    let flags = self.registers.carry_f as u32
                        | (self.registers.zero_f as u32) << 1
                        | (self.registers.comp_f as u32) << 2;
                    self.set_ram((self.registers.sp & 0xFFFFFF) as usize, flags);
                    if self.log {
                        trace!("Call: PC = {}, Flags = 0x{:X}", immediate, flags);
                    }
                    self.clock += 3;
                    return CPUError::Ok;
                }
            }
            14 => {
                // System
                let do_reti = instr >> 27 & 0x01;
                if do_reti == 1 {
                    // Return from System Call
                    let flags = self.get_ram((self.registers.sp & 0xFFFFFF) as usize);
                    self.registers.carry_f = flags & 0x01 == 1;
                    self.registers.zero_f = flags >> 1 & 0x01 == 1;
                    self.registers.comp_f = flags >> 2 & 0x01 == 1;
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    self.registers.privilege = false;
                    self.registers.pc = self.registers.reti as usize;
                    if self.log {
                        trace!(
                            "Return from System Call: PC = {}, Flags = 0x{:X}",
                            self.registers.pc,
                            flags
                        );
                    }
                    self.clock += 3;
                    return CPUError::Ok;
                } else {
                    // System Call
                    self.registers.reti = self.registers.pc as u32;
                    let flags = self.registers.carry_f as u32
                        | (self.registers.zero_f as u32) << 1
                        | (self.registers.comp_f as u32) << 2;
                    self.registers.sp = self.registers.sp.wrapping_sub(1);
                    self.set_ram((self.registers.sp & 0xFFFFFF) as usize, flags);
                    self.registers.privilege = true;
                    self.registers.pc = (self.get_ram(0x40) & 0xFFFFFF) as usize;
                    self.set_ram(0x41, immediate);
                    if self.log {
                        trace!(
                            "System Call: PC = {}, Flags = 0x{:X}",
                            self.registers.pc,
                            flags
                        );
                    }
                    self.clock += 3;
                    return CPUError::Ok;
                }
            }
            15 => {
                // Halt
                if self.log {
                    trace!("Halting CPU");
                }
                self.clock += 1;
                return CPUError::Halt;
            }
            _ => {
                println!("Invalid opcode");
                if self.log {
                    error!("Invalid opcode");
                }
                std::process::exit(1);
            }
        }
    }
}
