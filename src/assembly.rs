use super::vm::*;

#[derive(Clone, Debug)]
pub enum AssemblyValue {
    Register(u8),
    Value8(u8),
    Value16(u16),
    Value32(u32),
    Value64(u64),
}

/**
 * Compile byte code for virtual machine
*/
pub fn assemblize(ins: u8, val: &[AssemblyValue]) -> Vec<u8> {
    let mut byte_cdde = Vec::new();
    let mut opcode: u16 = 0;
    let mut data = Vec::new();
    opcode |= (ins as u16) << 9;
    for i in 0..val.len() {
        match val[i] {
            AssemblyValue::Value8(v) => {
                opcode |= (VM_TYPE_VAL8 as u16) << (3 * (2 - i));
                data.push(v);
            }
            AssemblyValue::Value16(v) => {
                opcode |= (VM_TYPE_VAL16 as u16) << (3 * (2 - i));
                data.extend(v.to_be_bytes());
            }
            AssemblyValue::Value32(v) => {
                opcode |= (VM_TYPE_VAL32 as u16) << (3 * (2 - i));
                data.extend(v.to_be_bytes());
            }
            AssemblyValue::Value64(v) => {
                opcode |= (VM_TYPE_VAL64 as u16) << (3 * (2 - i));
                data.extend(v.to_be_bytes());
            }
            AssemblyValue::Register(r) => {
                opcode |= (VM_TYPE_REG as u16) << (3 * (2 - i));
                data.push(r);
            }
        }
    }
    byte_cdde.extend(opcode.to_be_bytes());
    byte_cdde.extend(&data);
    byte_cdde
}
