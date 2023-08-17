# Virtual Machine

## Definitions
`src/vm.rs`:
 * operation: `VM_OP_`
 * register: `VM_REG_`
 * value type: `VM_TYPE_`
## Format
`[opcode] [value/register] ...`

## OP code
OPcode has is a uint 16 type.

* instruction takes 7 bits, from 0 to 127.
* type1, 2, 3 take 3 bits, from 0 to 7.
```
  instruction   type1 type2 type3
/             \/     \/   \/    \
----------------------------------
0             7     10    13    15
----------------------------------
```
## Instructions
Accepted value types:
|Type|Meaning |
|----|--------|
|r   |register|
|v   |value   |

|Operation|Value 1    |Value 2     |Value 3  |Description        |
|---------|-----------|------------|---------|-------------------|
|mov      |target[r]  |source[r,v] |-        |move value         |
|add      |source[r]  |value[r,v]  |-        |add                |
|sub      |source[r]  |value[r,v]  |-        |subtract           |
|mul      |source[r]  |value[r,v]  |-        |divide             |
|div      |source[r]  |value[r,v]  |-        |multiply           |
|mod      |source[r]  |value[r,v]  |-        |modular            |
|shl      |source[r]  |value[r,v]  |-        |shift logic left   |
|shr      |source[r]  |value[r,v]  |-        |shift logic right  |
|push     |source[r]  |-           |-        |push stack         |
|load     |register[r]|address[r,v]|-        |load from memory   |
|store    |register[r]|address[r,v]|-        |store to memory    |
|pop      |source[r]  |-           |-        |pop stack          |
|in       |device[r,v]|data[r,v]   |-        |input data         |
|out      |device[r,v]|data[r,v]   |-        |output data        |
|ret      |-          |-           |-        |return             |
|testeq   |result[r,v]|val1[r,v]   |val2[r,v]|test if equal      |
|testneq  |val1[r,v]  |val2[r,v]   |val2[r,v]|test if not equal  |
|testgt   |val1[r,v]  |val2[r,v]   |val2[r,v]|test if gt         |
|testlt   |val1[r,v]  |val2[r,v]   |val2[r,v]|test if lt         |
|testge   |val1[r,v]  |val2[r,v]   |val2[r,v]|test if gt & equal |
|testle   |val1[r,v]  |val2[r,v]   |-        |test if gt & equal |
|jmp      |addr[r,v]  |-           |-        |jump to            |
|je       |addr[r,v]  |-           |-        |jump if equal      |
|jne      |addr[r,v]  |-           |-        |jump if not equal  |
|jng      |addr[r,v]  |-           |-        |jump if not greater|
|jnl      |addr[r,v]  |-           |-        |jump if not less   |
|hal      |-          |-           |-        |halt               |

### mov
`mov target, source`

* target: The register to restore the value.
* source: The register or memory address of the original value.

### add
`add source, value`

* source: The register of the first operand, and where to restore the result.
* source: The register or constant of the second operand.

## Registers
|Register|Description                       |
|--------|----------------------------------|
|IP      |Pointer of current executing code.|
|SP      |Stack pointer.                    |
|AR      |Store pre-operated memory address.|
