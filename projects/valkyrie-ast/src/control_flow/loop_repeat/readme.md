`for ... in ... if ... #label {...}`

## Simple For Loop

```valkyrie
loop {
    if c1 {
        continue
    }
    if c2 {
        break
    }
    if c3 {
        return
    }
    if c4 {
        yield break
    }
    if c5 {
        yield return
    }
    "loop other"
}
"loop continuation"
```


```valkyrie
let loop_continue = { 
    if c1 {
        loop_continue()
    }
    if c2 {
        loop_break()
    }
    if c3 {
        return
    }
    if c4 {
        loop_break()
    }
    if c5 {
        yield return
    }
}
let loop_break = {
    "loop continuation"
}
let yield_break = {

}
let yield_return = {
    "loop other"
    loop_continue()
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


