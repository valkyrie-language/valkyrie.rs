虚表

```csharp
class A {
    field_a
    virtual method_a()
}
class B: A {
    field_b
    override method_a()
}


class A {
    print() { 0 }
}
class B {
    print() { 1 }
}
class C(A, B) {
    run() {
        <Self as B>::print()
    }
}
```



