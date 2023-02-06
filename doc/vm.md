# Virtual Machine
## Format
`[operation] [value type] [value/register] ...`

For example:
 * operation: `VM_OP_`
 * register: `VM_REG_`
 * value type: `VM_TYPE_`

## OP code
Accepted value types:
|Type|Meaning |
|----|--------|
|r   |register|
|v   |value   |
|m   |memory  |

|Operation|Value 1      |Value 2      |Description|
|---------|-------------|-------------|-----------|
|mov      |target[r,v,m]|source[r,v,m]|move value |
|add      |source[r,v,m]|value[r,v,m] |add        |
|sub      |source[r,v,m]|value[r,v,m] |subtract   |
|div      |source[r,v,m]|value[r,v,m] |multiply   |
|mul      |source[r,v,m]|value[r,v,m] |divide     |
|push     |source[r]    |-            |push stack |
|pop      |source[r]    |-            |pop stack  |
|in       |device[r,v,m]|data[r,v,m]  |input data |
|out      |device[r,v,m]|data[r,v,m]  |output data|
|hal      |-            |-            |halt       |
