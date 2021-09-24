## Tests

```vks
class Animal;
class Dog(Animal) { name: String }
class Cat(Animal) { name: String }
class Fish(Animal)
def sound(animal: Animal) -> String {
    let name = ""
    if animal is Dog {
        name = animal.name
        println("Woof!")
    } else if animal is Cat {
        name = animal.name
        println("Meow!")
    }
    else {
        println("What?")
    }
    return name
}
```


```scala
class SoundEval1 {
    name: string
}

def sound(animal: Animal) {
    let ctx = new SoundEval1()
    if animal is Dog {
        ctx.name = animal.name
        println("Woof!")
    } else if animal is Cat {
        println("Meow!")
    }
    let name = ctx.name
}
```



```scala
class SoundEval1 {
    name: string
}
impl DynamicCast for SoundEval1 {
    fn downcast_dog(&mut self, dog: &Dog) {
        self.name = animal.name
        println("Woof!")
    }

    fn downcast_cat(&mut self, cat: &Cat) {
        println("Meow!")
    }
}
def sound(animal: Animal) {
    let mut ctx = new SoundEval1()
    animal.downcast(&mut ctx);
    let name = ctx.name
}
```