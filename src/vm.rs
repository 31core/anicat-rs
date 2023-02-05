use super::vram::VRAM;
use std::cell::RefCell;

const VM_STACK_SIZE: usize = 8 * 1024 * 1024;

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
pub const VM_OP_JG: u8 = 0x0f;
pub const VM_OP_JL: u8 = 0x20;
pub const VM_OP_HAL: u8 = 0x1f;

pub const VM_REG_C0: u8 = 0x20;
pub const VM_REG_SP: u8 = 0x23;
pub const VM_REG_IP: u8 = 0x24;
pub const VM_TYPE_REG: u8 = 0x25;
pub const VM_TYPE_VAL8: u8 = 0x26;
pub const VM_TYPE_VAL16: u8 = 0x27;
pub const VM_TYPE_VAL32: u8 = 0x28;
pub const VM_TYPE_VAL64: u8 = 0x29;
pub const VM_TYPE_MEM8: u8 = 0x2a;
pub const VM_TYPE_MEM16: u8 = 0x2b;
pub const VM_TYPE_MEM32: u8 = 0x2c;
pub const VM_TYPE_MEM64: u8 = 0x2d;

pub const VM_DEV_STDIN: u8 = 0;
pub const VM_DEV_STDOUT: u8 = 1;
pub const VM_DEV_STDERR: u8 = 2;

#[derive(Clone)]
struct Param {
    r#type: u8,
    value: u64,
}

impl Param {
    fn from(vm: &mut VM) -> Self {
        let code = vm.code.borrow_mut();
        let param_type = code[vm.ip as usize];
        vm.ip += 1;

        let mut value: u64 = 0;
        match param_type {
            VM_TYPE_VAL8 => {
                value = code[vm.ip as usize] as u64;
                vm.ip += 1;
            }
            VM_TYPE_VAL16 => {
                value = u16::from_be_bytes(
                    code[vm.ip as usize..vm.ip as usize + 2].try_into().unwrap(),
                ) as u64;
                vm.ip += 2;
            }
            VM_TYPE_VAL32 => {
                value = u32::from_be_bytes(
                    code[vm.ip as usize..vm.ip as usize + 4].try_into().unwrap(),
                ) as u64;
                vm.ip += 4;
            }
            VM_TYPE_VAL64 => {
                value = u64::from_be_bytes(
                    code[vm.ip as usize..vm.ip as usize + 8].try_into().unwrap(),
                );
                vm.ip += 8;
            }
            _ => {}
        }

        if param_type == VM_TYPE_MEM8
            || param_type == VM_TYPE_MEM16
            || param_type == VM_TYPE_MEM32
            || param_type == VM_TYPE_MEM64
        {
            value =
                u64::from_be_bytes(code[vm.ip as usize..vm.ip as usize + 8].try_into().unwrap());
            vm.ip += 8;
        }

        Param {
            r#type: param_type,
            value,
        }
    }
    fn get_value(&self, vm: &VM) -> u64 {
        if self.r#type == VM_TYPE_VAL8
            || self.r#type == VM_TYPE_VAL16
            || self.r#type == VM_TYPE_VAL32
            || self.r#type == VM_TYPE_VAL64
        {
            return self.value;
        }
        match self.r#type {
            VM_REG_C0 => return vm.c0,
            VM_REG_IP => return vm.ip,
            VM_REG_SP => return vm.sp,
            _ => {}
        }
        /* get value from vram */
        if self.r#type == VM_TYPE_MEM8 {
            return u8::from_be_bytes(vm.ram.dump(self.value, 1).try_into().unwrap()) as u64;
        } else if self.r#type == VM_TYPE_MEM16 {
            return u16::from_be_bytes(vm.ram.dump(self.value, 2).try_into().unwrap()) as u64;
        } else if self.r#type == VM_TYPE_MEM32 {
            return u32::from_be_bytes(vm.ram.dump(self.value, 4).try_into().unwrap()) as u64;
        } else if self.r#type == VM_TYPE_MEM64 {
            return u64::from_be_bytes(vm.ram.dump(self.value, 8).try_into().unwrap()) as u64;
        }
        0
    }
    fn set_value(&mut self, value: u64, vm: &mut VM) {
        /* copy to register */
        match self.r#type {
            VM_REG_C0 => vm.c0 = value,
            VM_REG_IP => vm.ip = value,
            VM_REG_SP => vm.sp = value,

            _ => {}
        }
        /* copy to vram */
        if self.r#type == VM_TYPE_MEM8 {
            /* addr = self.value */
            vm.ram.load(self.value, 1, &(value as u8).to_be_bytes());
        } else if self.r#type == VM_TYPE_MEM16 {
            vm.ram.load(self.value, 2, &(value as u16).to_be_bytes());
        } else if self.r#type == VM_TYPE_MEM32 {
            return vm.ram.load(self.value, 4, &(value as u32).to_be_bytes());
        } else if self.r#type == VM_TYPE_MEM64 {
            return vm.ram.load(self.value, 8, &value.to_be_bytes());
        }
    }
}

#[derive(Clone)]
pub struct VM {
    pub c0: u64,
    pub ip: u64,
    pub sp: u64,
    pub ram: VRAM,
    pub code: Box<RefCell<Vec<u8>>>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            c0: 0,
            ip: 0,
            sp: VM_STACK_SIZE as u64,
            ram: VRAM::new(4 * 1024 * 1024 * 1024),
            code: Box::new(RefCell::new(Vec::new())),
        }
    }
    /// run VM
    pub fn run(&mut self) {
        loop {
            let op = self.code.borrow()[self.ip as usize];
            self.ip += 1;
            /* mov target, source */
            if op == VM_OP_MOV {
                let mut target = Param::from(self);
                let source = Param::from(self);

                target.set_value(source.get_value(self), self);
            }
            /* add source, target */
            if op == VM_OP_ADD {
                let mut source = Param::from(self);
                let target = Param::from(self);

                let mut source_val = source.get_value(self);
                let target_val = target.get_value(self);
                source_val += target_val;

                source.set_value(source_val, self);
            }
            /* sub source, target */
            if op == VM_OP_SUB {
                let mut source = Param::from(self);
                let target = Param::from(self);

                let mut source_val = source.get_value(self);
                let target_val = target.get_value(self);
                source_val -= target_val;

                source.set_value(source_val, self);
            }
            /* mul source, target */
            if op == VM_OP_MUL {
                let mut source = Param::from(self);
                let target = Param::from(self);

                let mut source_val = source.get_value(self);
                let target_val = target.get_value(self);
                source_val *= target_val;

                source.set_value(source_val, self);
            }
            /* div source, target */
            if op == VM_OP_DIV {
                let mut source = Param::from(self);
                let target = Param::from(self);

                let mut source_val = source.get_value(self);
                let target_val = target.get_value(self);
                source_val /= target_val;

                source.set_value(source_val, self);
            }
            /* push register */
            if op == VM_OP_PUSH {
                let register = Param::from(self);

                self.ram
                    .load(self.sp, 8, &register.get_value(self).to_be_bytes());

                self.sp -= 8;
            }
            /* pop register */
            if op == VM_OP_POP {
                let mut register = Param::from(self);

                let reg_val = self.ram.dump(self.sp, 8);
                register.set_value(u64::from_be_bytes(reg_val.try_into().unwrap()), self);

                self.sp -= 8;
            }
            /* jmp addr */
            if op == VM_OP_JMP {
                let addr = Param::from(self);
                self.ip = addr.get_value(self);
            }
            /* hal */
            if op == VM_OP_HAL {
                return;
            }
        }
    }
    /// update VM opcode
    pub fn update_code(&mut self, code: &[u8]) {
        self.code = Box::new(RefCell::new(Vec::from(code)));
    }
}
