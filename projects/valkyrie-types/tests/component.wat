(component $root(core module $MockMemory
        (func $realloc (export "realloc") (param i32 i32 i32 i32) (result i32)
            (i32.const 0)
        )
        (memory $memory (export "memory") 15)
    )
    (core instance $memory (instantiate $MockMemory))
    (import "wasi:sockets/network" (instance $wasi:sockets/network
        (export $std::random::Network "network" (type (sub resource)))
    ))
    (alias export $wasi:sockets/network "network" (type $std::random::Network))
    (import "wasi:random/insecure" (instance $wasi:random/insecure
        (export "get-insecure-random-u64" (func
        ))
    ))
    (alias export $wasi:random/insecure "get-insecure-random-u64" (func $std::random::random_seed_fast))
    (import "wasi:random/random" (instance $wasi:random/random
        (export "get-random-u64" (func
        ))
    ))
    (alias export $wasi:random/random "get-random-u64" (func $std::random::random_seed_safe))
    
    
    (core func $std::random::random_seed_fast (canon lower
        (func $wasi:random/insecure "get-insecure-random-u64")
    ))
    (core func $std::random::random_seed_safe (canon lower
        (func $wasi:random/random "get-random-u64")
    ))
    (core module $Main
        (import "wasi:random/insecure" "get-insecure-random-u64" (func $std::random::random_seed_fast
        ))
        (import "wasi:random/random" "get-random-u64" (func $std::random::random_seed_safe
        ))
        (func $std::text::let_us_random
        )
        (func $std::text::test
        )
    )
    (core instance $main (instantiate $Main
        (with "wasi:sockets/network" (instance
        ))
        (with "wasi:random/insecure" (instance
            (export "get-insecure-random-u64" (func $std::random::random_seed_fast))
        ))
        (with "wasi:random/random" (instance
            (export "get-random-u64" (func $std::random::random_seed_safe))
        ))
    ))
)