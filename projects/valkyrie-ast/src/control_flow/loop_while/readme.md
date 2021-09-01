`while condition {...}`, `while let pattern = condition {...}`

```vk
while condition() {
    do()
}
```

```vk
loop ^label {
    if condition() {
        break ^label
    }
    do()
}
```

```vk
while let Some(a) = condition() {
    a.do()
}
```

```vk
loop ^label {
    match condition() {
        case Some(a): a.do(),
        case _      : break ^label,
    }
}
``` {...}`

```vk
while condition() {
    do()
}
```

```vk
loop ^label {
    if condition() {
        break ^label
    }
    do()
}
```

```vk
while let Some(a) = condition() {
    a.do()
}
```

```vk
loop ^label {
    match condition() {
        case Some(a): a.do(),
        case _      : break ^label,
    }
}
```