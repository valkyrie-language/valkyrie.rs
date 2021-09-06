


## Inline ASM

```vk
wasm micro add(lhs: i32, rhs: i32) -> i32 {
    """
    local.get $lhs
    local.get $rhs
    i32.add
    """
}
wasm micro double(p: i32) -> i32 {
    add(p, p)
}
```

```wasm
(func add
    (param $p i32)
    (result i32)

    local.get $lhs
    local.get $rhs
    i32.add

)

(func $double
    (param $p i32)
    (result i32)
    local.get $p
    local.get $p
    call $add
)
```

