use super::assembly::AssemblyValue;
use super::vram::VRAM;
use std::cell::RefCell;
use std::io::{Read, Write};

const VM_STACK_SIZE: usize = 8 * 1024 * 1024;

/// op codes (0x01 - 0x14)
pub const VM_OP_MOV: u8 = 0x01;
pub const VM_OP_IN: u8 = 0x02;
pub const VM_OP_OUT: u8 = 0x03;
pub const VM_OP_JMP: u8 = 0x04;
pub const VM_OP_CMP: u8 = 0x05;
pub const VM_OP_ADD: u8 = 0x06;
pub const VM_OP_SUB: u8 = 0x07;
pub const VM_OP_MUL: u8 = 0x08;
pub const VM_OP_DIV: u8 = 0x09;
pub const VM_OP_PUSH: u8 = 0x0a;
pub const VM_OP_POP: u8 = 0x0b;
pub const VM_OP_CALL: u8 = 0x0c;
pub const VM_OP_RET: u8 = 0x0d;
pub const VM_OP_JE: u8 = 0x0e;
pub const VM_OP_JNE: u8 = 0x0f;
pub const VM_OP_JG: u8 = 0x10;
pub const VM_OP_JL: u8 = 0x11;
pub const VM_OP_JNG: u8 = 0x12;
pub const VM_OP_JNL: u8 = 0x13;
pub const VM_OP_LOAD8: u8 = 0x14;
pub const VM_OP_LOAD16: u8 = 0x15;
pub const VM_OP_LOAD32: u8 = 0x16;
pub const VM_OP_LOAD64: u8 = 0x17;
pub const VM_OP_STORE8: u8 = 0x18;
pub const VM_OP_STORE16: u8 = 0x19;
pub const VM_OP_STORE32: u8 = 0x1a;
pub const VM_OP_STORE64: u8 = 0x1b;
pub const VM_OP_HAL: u8 = 0x1f;

pub const VM_REG_C0: u8 = 0x20;
pub const VM_REG_C1: u8 = 0x21;
pub const VM_REG_C2: u8 = 0x22;
pub const VM_REG_C3: u8 = 0x23;
pub const VM_REG_SP: u8 = 0x24;
pub const VM_REG_IP: u8 = 0x25;
pub const VM_REG_AR: u8 = 0x26; // Adress Register

/**
 * value types (0x20 - 0x32)
 * This takes 3-bit
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
 * `type`:  
 * * `VM_TYPE_MEM`: size of memory
 * * `VM_TYPE_RMEM`: register
 * * `VM_TYPE_VAL`: size of value
 * * `VM_TYPE_REG`: register type
 *
 * `value`:  
 * * `VM_TYPE_MEM`: memory address
 * * `VM_TYPE_RMEM`: memory address from register
 * * `VM_TYPE_VAL`: value
 * * `VM_TYPE_REG`: register value
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
            let vtype = (opcode >> 3 * (3 - i) & 0b111) as u8;

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
            AssemblyValue::Register(r) => return vm.get_register(r),
            AssemblyValue::Value8(v) => return v as u64,
            AssemblyValue::Value16(v) => return v as u64,
            AssemblyValue::Value32(v) => return v as u64,
            AssemblyValue::Value64(v) => return v,
        }
    }
}

#[derive(Clone)]
pub struct VM {
    pub c0: u64,
    pub c1: u64,
    pub c2: u64,
    pub c3: u64,
    pub ip: u64,
    pub sp: u64,
    pub ar: u64,

    /* flags */
    pub zf: bool,
    pub cf: bool,
    pub ram: VRAM,
    pub code: Box<RefCell<Vec<u8>>>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            c0: 0,
            c1: 0,
            c2: 0,
            c3: 0,
            ip: 0,
            ar: 0,
            zf: false,
            cf: false,
            sp: VM_STACK_SIZE as u64,
            ram: VRAM::new(4 * 1024 * 1024 * 1024),
            code: Box::new(RefCell::new(Vec::new())),
        }
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
            VM_REG_C0 => return self.c0,
            VM_REG_C1 => return self.c1,
            VM_REG_C2 => return self.c2,
            VM_REG_C3 => return self.c3,
            VM_REG_SP => return self.sp,
            VM_REG_IP => return self.ip,
            VM_REG_AR => return self.ar,
            _ => return 0,
        }
    }
    /// run VM
    pub fn run(&mut self) {
        loop {
            let opcode = OPcode::from(self);
            println!("{:?}", &opcode);

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
                        for i in 0..2 {
                            data[6 + i] = dump_data[i];
                        }
                    }
                    VM_OP_LOAD32 => {
                        let dump_data = self.ram.dump(address, 4);
                        for i in 0..4 {
                            data[4 + i] = dump_data[i];
                        }
                    }
                    VM_OP_LOAD64 => {
                        let dump_data = self.ram.dump(address, 8);
                        for i in 0..8 {
                            data[i] = dump_data[i];
                        }
                    }
                    _ => {}
                }
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    match register {
                        VM_REG_C0 => self.c0 = u64::from_be_bytes(data),
                        VM_REG_C1 => self.c1 = u64::from_be_bytes(data),
                        VM_REG_C2 => self.c2 = u64::from_be_bytes(data),
                        VM_REG_C3 => self.c3 = u64::from_be_bytes(data),
                        _ => {}
                    }
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
                    VM_OP_LOAD8 => {
                        self.ram.load(address, 1, &value.to_be_bytes()[7..]);
                    }
                    VM_OP_LOAD16 => {
                        self.ram.load(address, 1, &value.to_be_bytes()[6..]);
                    }
                    VM_OP_LOAD32 => {
                        self.ram.load(address, 1, &value.to_be_bytes()[4..]);
                    }
                    VM_OP_LOAD64 => {
                        self.ram.load(address, 1, &value.to_be_bytes());
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
            /* add source, target */
            if opcode.op == VM_OP_ADD {
                let source = opcode.get_value(0, self);
                let target = opcode.get_value(1, self);
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, source + target);
                }
            }
            /* sub source, target */
            if opcode.op == VM_OP_SUB {
                let source = opcode.get_value(0, self);
                let target = opcode.get_value(1, self);
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, source - target);
                }
            }
            /* mul source, target */
            if opcode.op == VM_OP_MUL {
                let source = opcode.get_value(0, self);
                let target = opcode.get_value(1, self);
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, source * target);
                }
            }
            /* div source, target */
            if opcode.op == VM_OP_DIV {
                let source = opcode.get_value(0, self);
                let target = opcode.get_value(1, self);
                if let AssemblyValue::Register(register) = opcode.values[0] {
                    self.set_register(register, source / target);
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
            /* cmp val1, val2 */
            if opcode.op == VM_OP_CMP {
                if opcode.get_value(0, self) == opcode.get_value(1, self) {
                    self.zf = true;
                    self.cf = false;
                } else if opcode.get_value(0, self) > opcode.get_value(1, self) {
                    self.zf = false;
                    self.cf = false;
                } else {
                    self.zf = false;
                    self.cf = true;
                }
            }
            /* je addr */
            if opcode.op == VM_OP_JE {
                if self.zf {
                    self.ip = opcode.get_value(0, self);
                }
            }
            /* jne addr */
            if opcode.op == VM_OP_JNE {
                if !self.zf {
                    self.ip = opcode.get_value(0, self);
                }
            }
            /* jg addr */
            if opcode.op == VM_OP_JG {
                if !self.zf && !self.cf {
                    self.ip = opcode.get_value(0, self);
                }
            }
            /* jl addr */
            if opcode.op == VM_OP_JL {
                if !self.zf && self.cf {
                    self.ip = opcode.get_value(0, self);
                }
            }
            /* jng addr */
            if opcode.op == VM_OP_JNG {
                if self.zf || self.zf && self.cf {
                    self.ip = opcode.get_value(0, self);
                }
            }
            /* jnl addr */
            if opcode.op == VM_OP_JNL {
                if self.zf || !self.zf && self.cf {
                    self.ip = opcode.get_value(0, self);
                }
            }
            /* jmp addr */
            if opcode.op == VM_OP_JMP {
                self.ip = opcode.get_value(0, self);
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
                if opcode.get_value(0, self) as u8 == VM_DEV_STDIN {
                    let mut buf = [0];
                    std::io::stdin().read(&mut buf).unwrap();
                    //data.set_value(buf[0] as u64, self);
                }
            }
            /* out port data */
            if opcode.op == VM_OP_OUT {
                if opcode.get_value(0, self) as u8 == VM_DEV_STDOUT {
                    let buf = [opcode.get_value(1, self) as u8];
                    std::io::stdout().write(&buf).unwrap();
                }
                if opcode.get_value(0, self) as u8 == VM_DEV_STDERR {
                    let buf = [opcode.get_value(1, self) as u8];
                    std::io::stderr().write(&buf).unwrap();
                }
            }
            /* hal */
            if opcode.op == VM_OP_HAL {
                return;
            }
        }
    }
    /// update VM opcode
    pub fn update_code(&mut self, code: &[u8]) {
        self.code = Box::new(RefCell::new(Vec::from(code)));
    }
}
