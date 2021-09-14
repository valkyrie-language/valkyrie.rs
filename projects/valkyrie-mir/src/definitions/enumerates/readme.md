```scala
class C(B1, B2) { }

structure C(B1, B2) {
    _b1: B1,
    _b2: B2,
}
imply C: Cast(B1) {
    cast(self): B1 {
        self._b1;
    }
}
imply C: Cast(B2) {
    cast(self): B2 {
        self._b2;
    }
}
```

```vk
class A {
    @get
    _field_a: i32;
    
    set field_a() {
        self._field_a = value;
    }
    
    set field_b(mut self, value: i32) {
        self._field_b = value;
    }
}

effect AskName(Integer) -> String;
 
def ask_name(user): Unit / AskName {
    var name: String? = user.name;
    if name == null {
        name = raise AskName(user.id);
    }
    print("你的名字是：{name}")
}

try {
    ask_name({ id: 24, name: "是学生" });
    ask_name({ id: 10 });
}
catch AskName(id) {
    if id > 10 {
        return 0
    }
    else {
        resume "无名氏";
    }
}
return -1



class Point {
    x: int,
    y: int,
}

refine interface Positive {
    get positive(self): bool;
}

refine interface Negative {
    get negative(self): bool;
}

refine interfae Mutable {
    get mutable(self): bool;
    set mutable(self, value: bool);
}


refine interface Scalable {
    get scalable(self): bool;
    set scalable(self, value: bool);
}

refine interface Modifiable: Scalable {
    final get modifiable(self): bool {
        if self.scalable {
            return true;
        }
        my_modifiable
    }
    final set modifiable(self, value: bool) {
        my_modifiable = value;
    }
    
    private my_modifiable: bool
}
```

```vk
let a = 1;

fn test() {
    let a = 2;
    let b = 1;
    {
        b = 2;
        a = 3;
        // do things with effect
    }
    let a = 4;
    // do things with effect
}




```

```vk
effect DivideZeroError(numerator: Integer): Integer;

def lower(a: Integer, b:Integer): Integer / DivideZeroError {
    if b != 0 {
        a / b
    }
    else {
        raise DivideZeroError(a)
    }
}

def middle(): Integer / DivideZeroError {
    lower(1, 0)
}

def higher(): Integer {
    try {
        middle()
    }
    catch DivideZeroError(e) {
        if a >= 0 {
            print(e)
            resume 0
        }
        else {
            return -1
        }
    }
}
```

```vk
micro DivideZeroError(numerator: Integer): Integer {
    print(numerator)
    return 0
}

def lower(a: Integer, b:Integer, e: DivideZeroError): Integer {
    if b != 0 {
        a / b
    }
    else {
        if a >= 0 {
            print(e)
            return 0
        }
        else {
            return(higher)
        }
    }
}

def middle(e: DivideZeroError): Integer {
    lower(1, 0)
}

def higher(return): Integer {
    let mut error = null;
    middle(error)
    if a >= 0 {
        print(e)
        resume 0
    }
    else {
        return()
    }
}
```

```vk
def select_positive(list: List<Integer>) -> Iterator<Integer> / FixZero {
    for i in list {
        if i < 0 {
            contniue
        }
        if i == 0 {
            yield raise FixZero()
        }
        yield i
    }
    return 999;
}

def run_select(input: List<Integer>) -> List<Integer> {
    var zeros = 0;
    var terms = [];
    try {
        for i in select_positive(input) {
            terms.append(i)
        }
    }
    .catch {
        case FixZero():
            zeros += 1;
            if zeros >= 3 {
                print("Catch { zeros.length }")
                return terms
            }
            resume zeros
    }
    print("Return { zeros.length }")
    return terms
}

run_select([2, 0, -3, 0, 5, 0, -7])
```

Catch 3
[2, 1, 2, 5]

```vk
def select_positive(list: List<Integer>) -> Iterator<Integer> / FixZero {
    for i in list {
        if i < 0 {
            contniue
        }
        if i == 0 {
            yield raise FixZero()
        }
        yield i
    }
    return 999;
}
```

if condA {
actionA
}
actionB

function id(x: Any): Any {
x
}
function cps_id(x: Any, ret: Continuation<Any>) {
ret(x)
}

```
function fact(n: Integer): Integer {
    if n == 0 {
        1
    } else {
        n * fact(n-1)
    }
}

function cps_fact(n: Integer, ret: Continuation<Integer>) {
    if n == 0 {
        ret(1)
    } else {
        cps_fact(n - 1, t0 => { ret(n * t0) })
    }
}
```

```vk
function iter(list: List<Integer>): Integer {
    for n in list {
        if n < 0 {
            continue
        } else if n == 0 {
            break
        } else {
           return n
        }
    }
    return -1
}


function cps_iter(list: List<Integer>, ret: Continuation<Integer>) {
    let cps_for = (index: Integer, br: Continuation) => {
        if (index >= list.length) {
            br()
        } else {
            let n = list[index]
            if (n < 0) {
                cps_for(index + 1) 
            } else if n == 0 {
                br()
            } else {
                ret(n)
            }
        }
    }
    loop(0, () => { ret(-1) })
}
```


```vk
function iter(list: List<Integer>): Iterator<Integer> {
    print(list)
    for n in list {
        if n < 0 {
            continue
        } else if n == 0 {
            break
        } else {
            yield n
        }
    }
    return -1
}


let mut n;
let state = 0;
let ret = 0;

print(list)
loop {
    @1
    n = list.next();
    if n == null {
        @goto 2
    }
    if n < 0 {
        @goto 1
    } else if n == 0 {
        @goto 2
    } else {
        ret = n;
        state = 1
        @break
    }
}
@2
ret = -1
@unreachable


struct iter {
    __ret: Integer?,
    __state: Integer = 0,
    n: Integer?,
    n_1: Integer,
}

fsm iter.next(iter.state) {
    case 0:
        print(list)
        @jump 1
    case 1: 
        n = list.next();
        @jump 2 if n == null
        n_1 = n.unwrap()
        @jump 1 if n < 0
        @jump 2 if n == 0
        __ret = n_1;
        __state = 1
        @break
    case 2:
        __ret = -1
        state = -1
        @break
}

if iter.curry_finish() {
    iter.next()
}



```



```vk
function iter(list: List<Integer>): Integer {
    print(n)
    @loop.head
    loop {
        @loop.start
        if n < 0 {
            goto loop.start
        } else if n == 0 {
            goto loop.end
        } else {
            ret = n
            goto block.end
            yield n
        }
        @block.end
        @loop.end
        goto loop.start
    }
    @loop.tail
    return -1
}


function cps_iter(list: List<Integer>) {
    .ret: Continuation<Integer>
    .next: self.cps_loop.next()
    .cps_loop.next = {
        if (n < 0) {
            cps_loop.next()
        } else if n == 0 {
            cps_loop.br()
        } else {
            cps_iter.next = cps_loop()
            cps_iter.ret(n)
        }
    }
    .cps_loop.br = { cps_iter.next() }
}

print(cps_iter.next())

```
