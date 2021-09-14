## Tests

```vk
pub fn check_raw(list: &[u8]) {
    let mut iter = list.iter();
    let mut next = iter.next();
    while next.is_some() {
        @1
        let i = next.unwrap();
        if i % 4 == 0 {
            goto @1
        }
        else if i & 2 == 0 {
            goto @2
        }
        else {
            goto @3
        }
    }
    @2
    println!("Even!")
    @3
}
```



# nullable

```vk
# A | throw
a?
a.match {
    case Some(a) => a
    case None => raise NPE
}
a.match {
    case Success(s) => a
    case Failure(e) => raise e
}


# B?
a?.b
a.and_then(a => a.c)


# B | throw
a?.b?



# C | throw
a?.b?.c

a.and_then(a => ErrorB::from(a.b))
// or explicit throw
a!.b?.c


```


````vk
a ?? b ?? c

a ?= b ?= c
````