`while cond {...} otherwise {...}`


```vk
outer: loop {
    if c1 {
        break; // break 'outer
    }
    if c2 {
        continue; // continue 'outer
    }
    if c3 {
        return;
    }
    inner: loop {
        if c4 {
            break 'outer;
        }
        if c5 {
            continue 'outer;
        }
        if c6 {
            return;
        }
        if c7 {
            break; // break 'inner
        }
        if c8 {
            continue; // continue 'inner
        }
        if c9 {
            return;
        }
        "inner-continuation"
    }
    "outer-continuation"
}
"function-continuation"
```



```vk
// promotion local variable here!!
let outer_continue = || {
    if c1 {
        outer_break()
    }
    if c2 {
        outer_continue();
    }
    if c3 {
        return;
    }
    inner: loop {
        if c4 {
            outer_break()
        }
        if c5 {
            outer_continue()
        }
        if c6 {
            return;
        }
        if c7 {
            break; // break 'inner
        }
        if c8 {
            continue; // continue 'inner
        }
        if c9 {
            return;
        }
        "inner-continuation"
    }
    "outer-continuation"
}
let outer_break = || {
    "function-continuation"
}
```



```vk
// promotion local variable here!!
let outer_continue = || {
    if c1 {
        outer_break()
    }
    if c2 {
        outer_continue();
    }
    if c3 {
        return;
    }
    inner_continue()
}
let outer_break = || {
    "function-continuation"
}
let inner_continue = || {
    if c4 {
        outer_break()
    }
    if c5 {
        outer_continue()
    }
    if c6 {
        return;
    }
    if c7 {
        inner_break()
    }
    if c8 {
        inner_continue()
    }
    if c9 {
        return;
    }
}
let inner_break = || {
    "outer-continuation"
}
```