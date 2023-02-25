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

|Operation|Value 1    |Value 2     |Value 3|Description        |
|---------|-----------|------------|-------|-------------------|
|mov      |target[r]  |source[r,v] |       |move value         |
|add      |source[r]  |value[r,v]  |       |add                |
|sub      |source[r]  |value[r,v]  |       |subtract           |
|div      |source[r]  |value[r,v]  |       |multiply           |
|mul      |source[r]  |value[r,v]  |       |divide             |
|push     |source[r]  |-           |       |push stack         |
|load     |register[r]|address[r,v]|       |load from memory   |
|store    |register[r]|address[r,v]|       |store to memory    |
|pop      |source[r]  |-           |       |pop stack          |
|in       |device[r,v]|data[r,v]   |       |input data         |
|out      |device[r,v]|data[r,v]   |       |output data        |
|cmp      |val1[r,v]  |val2[r,v]   |       |compare            |
|ret      |-          |-           |       |return             |
|jmp      |addr[r,v]  |-           |       |jump to            |
|je       |addr[r,v]  |-           |       |jump if equal      |
|jne      |addr[r,v]  |-           |       |jump if not equal  |
|jg       |addr[r,v]  |-           |       |jump if greater    |
|jl       |addr[r,v]  |-           |       |jump if less       |
|jng      |addr[r,v]  |-           |       |jump if not greater|
|jnl      |addr[r,v]  |-           |       |jump if not less   |
|hal      |-          |-           |       |halt               |
