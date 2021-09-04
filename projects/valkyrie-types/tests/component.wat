(component $root(core module $MockMemory
        (func $realloc (export "realloc") (param i32 i32 i32 i32) (result i32)
            (i32.const 0)
        )
        (memory $memory (export "memory") 15)
    )
    (core instance $memory (instantiate $MockMemory))
    
    
    (core module $Main
        (func $let_us_random
        )
        (func $test
        )
    )
    (core instance $main (instantiate $Main
    ))
)