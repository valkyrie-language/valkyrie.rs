`for ... in ... if ... #label {...}`

## Simple For Loop

```valkyrie
for i in j if c1 {
    if c2 {
        continue
    }
    else if c3 {
        break
    }
    else {
        return
    }
}
"residual expression"
```


```v
j = j.into_iterator()
let looper = {
    
    if c2 {
        looper()
    }
    else if c3 {
        break()
    }
    else {
        return
    }
}
let break = {
    "other"
}
```


```v
j = j.into_iterator()
let looper = {
    
    if c2 {
        looper()
    }
    else if c3 {
        break()
    }
    else {
        return
    }
}
let break = {
    "other"
}
```

```v

loop switch label {
᳀function.1.start:
    j = j.into_iterator()
᳀for.1.start:
    let next = j.next();


}

loop {
    ᳀for.1.start
    let next = j.next();

    if next != null && c1 {
        ᳀if.1.head
        if c2 {
            ⤮for.1.start
        }
        else if c3 {
            ⤮for.1.tail
        }
        else {
            ⤮function.1.return
        }
    }
    else {
        break
    }
    ᳀for.1.end
}
⤮for.1.tail

᳀function.1.return
```


## Nested For Loop

```vk
for i in j #outer {
    for x in y #inner {
        if c1 {
            continue #outer
        }
        if c2 {
            continue #inner
        }
    }
}
```


