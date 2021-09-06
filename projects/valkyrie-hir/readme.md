我想在 wasm 中做类似于以下 Java 代码的事情。

```java
abstract class Animal {}
interface Say {
    void say();
}
interface Swim {
    boolean swim();
}
class Cat extends Animal implements Say {
    @Override
    public void say() {
        System.out.println("Meow!");
    }
}
class Dog extends Animal implements Say, Swim {
    @Override
    public void say() {
        System.out.println("Woof!");
    }
    @Override
    public boolean swim() {
        return true;
    }
}
class Fish extends Animal implements Swim {
    @Override
    public boolean swim() {
        return true;
    }
}
public class Main {
    public static void listen(Animal animal) {
        if (animal instanceof Say) {
            ((Say) animal).say();
        }
    }
    public static void main(String[] args) {
        Animal cat = new Cat();
        Animal dog = new Dog();
        Animal fish = new Fish();

        listen(cat);
        listen(dog);
        listen(fish);
    }
}
```

我这里参考了虚拟表定义，并添加了接口表

基类和接口定义如下：

```wat
;; class Animal
(type $Animal (struct (ref $AnimalVTable) (ref $AnimalITables)))
(type $AnimalVTable (struct))
(type $AnimalITables (struct))

;; interface Say
(type $Say (struct (ref null) (ref $SayITables)))
(type $SayITables (struct (ref $SayVTable)))
(type $SayVTable (struct (ref $say_func)))
(type $say_func (func (param (ref $Say))))

;; interface Swim
(type $Swim (struct (ref null) (ref $SwimITables)))
(type $SwimITables (struct (ref $SwimVTable)))
(type $SwimVTable (struct (ref $swim_func)))
(type $swim_func (func (param (ref $Swim))))
```

然后我定义了子类：

```wat
;; Cat
(type $Cat (struct (ref $CatVTable) (ref $CatITables)))
(type $CatVTable (struct))
(type $CatITables (struct (ref $SayVTable)))

;; Dog
(type $Dog (struct (ref $DogVTable) (ref $DogITables)))
(type $DogVTable (struct))
(type $DogITables (struct (ref $SayVTable) (ref $SwimVTable)))
```

但我发现不能简单地将子类对象提升为接口对象。

每个接口对象都需要先将自己的虚拟表对象放入 itable 最上方。

因此在这个结构体布局中，`Dog`可以转换为`Say`，但不能转换为`Swim`。

```java
Animal dog = new Dog(); // ok
((Say) dog).say() // Invoke 1, 0
((Swim) dog).swim() // Invoke index not right
```

这意味着无法实现`Dog -> Animal -> Swim` 的类型转换链。

我应该如何改进我的编译器来解决这个问题？