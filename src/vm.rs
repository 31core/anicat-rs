use crate::assembly::AssemblyValue;
use crate::vram::Vram;
use std::cell::RefCell;
use std::io::{Read, Write};

pub const VM_STACK_SIZE: usize = 8 * 1024 * 1024;

pub const VM_BITS: usize = 64;
pub const VM_USIZE: usize = VM_BITS / 8;

// op codes (0x01 - 0x22)
pub const VM_OP_MOV: u8 = 0x01;
pub const VM_OP_IN: u8 = 0x02;
pub const VM_OP_OUT: u8 = 0x03;
pub const VM_OP_JMP: u8 = 0x04;
pub const VM_OP_ADD: u8 = 0x06;
pub const VM_OP_SUB: u8 = 0x07;
pub const VM_OP_MUL: u8 = 0x08;
pub const VM_OP_DIV: u8 = 0x09;
pub const VM_OP_PUSH: u8 = 0x0a;
pub const VM_OP_POP: u8 = 0x0b;
pub const VM_OP_CALL: u8 = 0x0c;
pub const VM_OP_RET: u8 = 0x0d;
pub const VM_OP_LOAD8: u8 = 0x0e;
pub const VM_OP_LOAD16: u8 = 0x0f;
pub const VM_OP_LOAD32: u8 = 0x10;
pub const VM_OP_LOAD64: u8 = 0x11;
pub const VM_OP_STORE8: u8 = 0x12;
pub const VM_OP_STORE16: u8 = 0x13;
pub const VM_OP_STORE32: u8 = 0x14;
pub const VM_OP_STORE64: u8 = 0x15;
pub const VM_OP_MOD: u8 = 0x16;
pub const VM_OP_SHL: u8 = 0x17;
pub const VM_OP_SHR: u8 = 0x18;
pub const VM_OP_AND: u8 = 0x19;
pub const VM_OP_OR: u8 = 0x1a;
pub const VM_OP_XOR: u8 = 0x1b;
pub const VM_OP_NOT: u8 = 0x1c;
pub const VM_OP_TESTEQ: u8 = 0x1d;
pub const VM_OP_TESTNEQ: u8 = 0x1e;
pub const VM_OP_TESTGT: u8 = 0x1f;
pub const VM_OP_TESTLT: u8 = 0x20;
pub const VM_OP_TESTGE: u8 = 0x21;
pub const VM_OP_TESTLE: u8 = 0x22;
pub const VM_OP_JE: u8 = 0x23;
pub const VM_OP_JNE: u8 = 0x24;
pub const VM_OP_HAL: u8 = 0x25;

pub const VM_REG_C0: u8 = 0x20;
pub const VM_REG_C1: u8 = 0x21;
pub const VM_REG_C2: u8 = 0x22;
pub const VM_REG_C3: u8 = 0x23;
pub const VM_REG_SP: u8 = 0x24;
pub const VM_REG_IP: u8 = 0x25;
pub const VM_REG_AR: u8 = 0x26;

/*
value types (0x20 - 0x32)
This takes 3-bit
*/
pub const VM_TYPE_VAL8: u8 = 1;
pub const VM_TYPE_VAL16: u8 = 2;
pub const VM_TYPE_VAL32: u8 = 3;
pub const VM_TYPE_VAL64: u8 = 4;
pub const VM_TYPE_REG: u8 = 5;

pub const VM_DEV_STDIN: u8 = 0;
pub const VM_DEV_STDOUT: u8 = 1;
pub const VM_DEV_STDERR: u8 = 2;
/**
 * Parse type from bytes slice.  
 */
#[derive(Clone, Debug)]
struct OPcode {
    op: u8,
    values: Vec<AssemblyValue>,
}

impl OPcode {
    fn from(vm: &mut VM) -> Self {
        let code = vm.code.borrow_mut();
        let opcode =
            u16::from_be_bytes(code[vm.ip as usize..vm.ip as usize + 2].try_into().unwrap());
        let op = (opcode >> 9) as u8;
        let mut values = Vec::new();

        vm.ip += 2;

        for i in 1..=3 {
            let vtype = (opcode >> (3 * (3 - i)) & 0b111) as u8;

            match vtype {
                VM_TYPE_VAL8 => {
                    let value = code[vm.ip as usize];
                    values.push(AssemblyValue::Value8(value));
                    vm.ip += 1;
                }
                VM_TYPE_VAL16 => {
                    let value = u16::from_be_bytes(
                        code[vm.ip as usize..vm.ip as usize + 2].try_into().unwrap(),
                    );
                    values.push(AssemblyValue::Value16(value));
                    vm.ip += 2;
                }
                VM_TYPE_VAL32 => {
                    let value = u32::from_be_bytes(
                        code[vm.ip as usize..vm.ip as usize + 4].try_into().unwrap(),
                    );
                    values.push(AssemblyValue::Value32(value));
                    vm.ip += 4;
                }
                VM_TYPE_VAL64 => {
                    let value = u64::from_be_bytes(
                        code[vm.ip as usize..vm.ip as usize + 8].try_into().unwrap(),
                    );
                    values.push(AssemblyValue::Value64(value));
                    vm.ip += 8;
                }
                VM_TYPE_REG => {
                    let value = code[vm.ip as usize];
                    values.push(AssemblyValue::Register(value));
                    vm.ip += 1;
                }
                _ => {}
            }
        }

        OPcode { op, values }
    }
    fn get_value(&self, index: usize, vm: &VM) -> u64 {
        match self.values[index] {
            AssemblyValue::Register(r) => vm.get_register(r),
            AssemblyValue::Value8(v) => v as u64,
            AssemblyValue::Value16(v) => v as u64,
            AssemblyValue::Value32(v) => v as u64,
            AssemblyValue::Value64(v) => v,
        }
    }
}

#[derive(Clone, Default)]
pub struct VM {
    pub c0: u64,
    pub c1: u64,
    pub c2: u64,
    pub c3: u64,
    /** Instruction Pointer */
    pub ip: u64,
    /** Stack Pointer */
    pub sp: u64,
    /** Address Register */
    pub ar: u64,

    /* flags */
    pub zf: bool,
    pub cf: bool,
    pub ram: Vram,
    pub code: Box<RefCell<Vec<u8>>>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            sp: VM_STACK_SIZE as u64,
            ram: Vram::new(4 * 1024 * 1024 * 1024),
            code: Box::new(RefCell::new(Vec::new())),
            ..Default::default()
        }
    }
    pub fn set_entry_point(&mut self, addr: u64) {
        self.ip = addr;
    }
    fn set_register(&mut self, register: u8, value: u64) {
        match register {
            VM_REG_C0 => self.c0 = value,
            VM_REG_C1 => self.c1 = value,
            VM_REG_C2 => self.c2 = value,
            VM_REG_C3 => self.c3 = value,
            VM_REG_SP => self.sp = value,
            VM_REG_IP => self.ip = value,
            VM_REG_AR => self.ar = value,
            _ => {}
        }
    }
    fn get_register(&self, register: u8) -> u64 {
        match register {
            VM_REG_C0 => self.c0,
            VM_REG_C1 => self.c1,
            VM_REG_C2 => self.c2,
            VM_REG_C3 => self.c3,
            VM_REG_SP => self.sp,
            VM_REG_IP => self.ip,
            VM_REG_AR => self.ar,
            _ => 0,
        }
    }
    /** run VM */
    pub fn run(&mut self) {
        loop {
            let opcode = OPcode::from(self);

            /* load register, address */
            if opcode.op == VM_OP_LOAD8
                || opcode.op == VM_OP_LOAD16
                || opcode.op == VM_OP_LOAD32
                || opcode.op == VM_OP_LOAD64
            {
                let address = opcode.get_value(1, self);
                let mut data = [0; 8];
                match opcode.op {
                    VM_OP_LOAD8 => {
                        let dump_data = self.ram.dump(address, 1);
                        data[7] = dump_data[0];
                    }
                    VM_OP_LOAD16 => {
                        let dump_data = self.ram.dump(address, 2);
                        data[6..].copy_from_slice(dump_data)
                    }
                    VM_OP_LOAD32 => {
                        let dump_data = self.ram.dump(address, 4);
                        data[4..].copy_from_slice(dump_data)
                    }
                    VM_OP_LOAD64 => {
                        let dump_data = self.ram.dump(address, 8);
                        data.copy_from_slice(dump_data)
                    }
                    _ => {}
                }
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, u64::from_be_bytes(data));
                }
            }
            /* store register, address */
            if opcode.op == VM_OP_STORE8
                || opcode.op == VM_OP_STORE16
                || opcode.op == VM_OP_STORE32
                || opcode.op == VM_OP_STORE64
            {
                let value = opcode.get_value(0, self);
                let address = opcode.get_value(1, self);
                match opcode.op {
                    VM_OP_STORE8 => {
                        self.ram.load(address, 1, &value.to_be_bytes()[7..]);
                    }
                    VM_OP_STORE16 => {
                        self.ram.load(address, 2, &value.to_be_bytes()[6..]);
                    }
                    VM_OP_STORE32 => {
                        self.ram.load(address, 4, &value.to_be_bytes()[4..]);
                    }
                    VM_OP_STORE64 => {
                        self.ram.load(address, 8, &value.to_be_bytes());
                    }
                    _ => {}
                }
            }
            /* mov target, source */
            if opcode.op == VM_OP_MOV {
                let source = opcode.get_value(1, self);

                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, source);
                }
            }
            /* binary operations
             *[op] source, target */
            if opcode.op == VM_OP_ADD
                || opcode.op == VM_OP_SUB
                || opcode.op == VM_OP_MUL
                || opcode.op == VM_OP_DIV
                || opcode.op == VM_OP_MOD
                || opcode.op == VM_OP_AND
                || opcode.op == VM_OP_OR
                || opcode.op == VM_OP_XOR
                || opcode.op == VM_OP_SHL
                || opcode.op == VM_OP_SHR
            {
                let source = opcode.get_value(0, self);
                let target = opcode.get_value(1, self);

                if let AssemblyValue::Register(register) = opcode.values[0] {
                    match opcode.op {
                        VM_OP_ADD => self.set_register(register, source + target),
                        VM_OP_SUB => self.set_register(register, source - target),
                        VM_OP_MUL => self.set_register(register, source * target),
                        VM_OP_DIV => self.set_register(register, source / target),
                        VM_OP_MOD => self.set_register(register, source % target),
                        VM_OP_AND => self.set_register(register, source & target),
                        VM_OP_OR => self.set_register(register, source | target),
                        VM_OP_XOR => self.set_register(register, source ^ target),
                        VM_OP_SHL => self.set_register(register, source << target),
                        VM_OP_SHR => self.set_register(register, source >> target),
                        _ => {}
                    }
                }
            }
            /* push register */
            if opcode.op == VM_OP_PUSH {
                self.sp -= 8;
                self.ram
                    .load(self.sp, 8, &opcode.get_value(0, self).to_be_bytes());
            }
            /* pop register */
            if opcode.op == VM_OP_POP {
                let reg_val = self.ram.dump(self.sp, 8);

                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, u64::from_be_bytes(reg_val.try_into().unwrap()));
                }

                self.sp += 8;
            }
            /* TEST commands */
            /* test[?] result, source, target */
            if opcode.op == VM_OP_TESTEQ
                || opcode.op == VM_OP_TESTNEQ
                || opcode.op == VM_OP_TESTGT
                || opcode.op == VM_OP_TESTLT
                || opcode.op == VM_OP_TESTGE
                || opcode.op == VM_OP_TESTLE
            {
                let source = opcode.get_value(1, self);
                let target = opcode.get_value(2, self);

                let result = match opcode.op {
                    VM_OP_TESTEQ => source == target,
                    VM_OP_TESTNEQ => source != target,
                    VM_OP_TESTGT => source > target,
                    VM_OP_TESTLT => source < target,
                    VM_OP_TESTGE => source >= target,
                    VM_OP_TESTLE => source <= target,
                    _ => false,
                };

                if let AssemblyValue::Register(register) = opcode.values[0] {
                    if result {
                        self.set_register(register, 1);
                    } else {
                        self.set_register(register, 0);
                    }
                }
            }
            /* jmp addr */
            if opcode.op == VM_OP_JMP {
                self.ip = opcode.get_value(0, self);
            }
            /* je source, addr */
            if opcode.op == VM_OP_JE && opcode.get_value(0, self) == 1 {
                self.ip = opcode.get_value(1, self);
            }
            /* jne source, addr */
            if opcode.op == VM_OP_JNE && opcode.get_value(0, self) == 0 {
                self.ip = opcode.get_value(1, self);
            }
            /* call addr */
            if opcode.op == VM_OP_CALL {
                /* push IP */
                self.sp -= 8;
                self.ram.load(self.sp, 8, &self.ip.to_be_bytes());
                self.ip = opcode.get_value(0, self);
            }
            /* ret */
            if opcode.op == VM_OP_RET {
                /* pop IP */
                self.ip = u64::from_be_bytes(self.ram.dump(self.sp, 8).try_into().unwrap());
                self.sp += 8;
            }
            /* in port data */
            if opcode.op == VM_OP_IN {
                let dev = opcode.get_value(0, self) as u8;
                #[allow(clippy::single_match)]
                match dev {
                    VM_DEV_STDIN => {
                        let mut buf = [0];
                        std::io::stdin().read_exact(&mut buf).unwrap();
                    }
                    _ => {}
                }
            }
            /* out port data */
            if opcode.op == VM_OP_OUT {
                let dev = opcode.get_value(0, self) as u8;
                match dev {
                    VM_DEV_STDOUT => {
                        let buf = [opcode.get_value(1, self) as u8];
                        std::io::stdout().write_all(&buf).unwrap();
                    }
                    VM_DEV_STDERR => {
                        let buf = [opcode.get_value(1, self) as u8];
                        std::io::stderr().write_all(&buf).unwrap();
                    }
                    _ => {}
                }
            }
            /* not reg */
            if opcode.op == VM_OP_NOT {
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, !opcode.get_value(0, self));
                }
            }
            /* hal */
            if opcode.op == VM_OP_HAL {
                return;
            }
        }
    }
    /** update VM opcode */
    pub fn update_code(&mut self, code: &[u8]) {
        self.code = Box::new(RefCell::new(Vec::from(code)));
    }
}
