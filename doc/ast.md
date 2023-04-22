# Abstract Syntax Tree
## AST in Anicat
Here I take some examples to show how AST in Anicat looks like.
### Variable declaration
code:
```
var i: u32;
```

AST:
```
 VAR_DECL
 /    \
ID   TYPE
|     |
i    u32
```

### Variable evaluation & calculation
code:
```
a = b + c;
```
AST:
```
  SET_VAR
 /     \
ID    ADD
|     /  \
a    ID  ID
     |   |
     b   c
```
or the simple version:
```
   =
  / \
 a   +
    / \
   b   c
```

### Function defination
code:
```
func foo(var a) -> u32 {
   // do something
}
```

AST:
```
    FUNC_DEF -------------\
   /   |   \               |
  /    |    \              |
ID   PARAMS  CODE_BLOCK  TYPE
 |      |                  |
foo     VAR                u8
       /   \
      ID  TYPE
      |      |
      a      u8
```
### Return
code:
```
return value;
```

AST:
```
RETURN
  |
value
```
