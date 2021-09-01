`until condition {...}`, `until not pattern = condition {...}`

```vk
until condition() {
    do()
}
```

```vk
loop ^label {
    if condition() {
        do()
    }
    break ^label
}
```

```vk
until not Integer = condition() {
    a.do()
}
```

```vk
loop ^label {
    if condition() is not Integer {
        a.do()
    }
    break ^label
}
```