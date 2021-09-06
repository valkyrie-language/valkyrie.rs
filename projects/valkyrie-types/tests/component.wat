(component $root(core module $MockMemory
        (func $realloc (export "realloc") (param i32 i32 i32 i32) (result i32)
            (i32.const 0)
        )
        (memory $memory (export "memory") 15)
    )
    (core instance $memory (instantiate $MockMemory))
    (import "v:legacy/number" (instance $v:legacy/number
        (export $std::number::Integer "big-integer" (type (sub resource)))
    ))
    (alias export $v:legacy/number "big-integer" (type $std::number::Integer))
    (import "v:math/legacy" (instance $v:math/legacy
        (export "cos-f64" (func
            (param $"value" float64)
        ))
        (export "exp-f64" (func
            (param $"value" float64)
        ))
        (export "exp-i64" (func
            (param $"value" s64)
        ))
        (export "log-i64" (func
            (param $"value" s64)
        ))
        (export "sin-f64" (func
            (param $"value" float64)
        ))
    ))
    (alias export $v:math/legacy "cos-f64" (func $std::number::cos_f64))
    (alias export $v:math/legacy "exp-f64" (func $std::number::exp_f64))
    (alias export $v:math/legacy "exp-i64" (func $std::number::exp_i64))
    (alias export $v:math/legacy "log-i64" (func $std::number::log_i64))
    (alias export $v:math/legacy "sin-f64" (func $std::number::sin_f64))
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
    (import "v:legacy/text" (instance $v:legacy/text
        (export $std::text::Utf16Text "utf16" (type (sub resource)))
    ))
    (alias export $v:legacy/text "utf16" (type $std::text::Utf16Text))
    (type $std::number::Float32 (record
        (field "value" bool)
    ))
    (type $std::number::Float64 (record
        (field "value" bool)
    ))
    (type $std::number::Integer32 (record
        (field "value" bool)
    ))
    (type $std::number::Integer64 (record
        (field "value" bool)
    ))
    (type $std::text::Unicode (record
        (field "value" bool)
    ))
    
    
    (core func $std::number::cos_f64 (canon lower
        (func $v:math/legacy "cos-f64")
    ))
    (core func $std::number::exp_f64 (canon lower
        (func $v:math/legacy "exp-f64")
    ))
    (core func $std::number::exp_i64 (canon lower
        (func $v:math/legacy "exp-i64")
    ))
    (core func $std::number::log_i64 (canon lower
        (func $v:math/legacy "log-i64")
    ))
    (core func $std::number::sin_f64 (canon lower
        (func $v:math/legacy "sin-f64")
    ))
    (core func $std::random::random_seed_fast (canon lower
        (func $wasi:random/insecure "get-insecure-random-u64")
    ))
    (core func $std::random::random_seed_safe (canon lower
        (func $wasi:random/random "get-random-u64")
    ))
    (core module $Main
        (import "v:math/legacy" "cos-f64" (func $std::number::cos_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "exp-f64" (func $std::number::exp_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "exp-i64" (func $std::number::exp_i64
            (param $value i64)
        ))
        (import "v:math/legacy" "log-i64" (func $std::number::log_i64
            (param $value i64)
        ))
        (import "v:math/legacy" "sin-f64" (func $std::number::sin_f64
            (param $value f64)
        ))
        (import "wasi:random/insecure" "get-insecure-random-u64" (func $std::random::random_seed_fast
        ))
        (import "wasi:random/random" "get-random-u64" (func $std::random::random_seed_safe
        ))
        (type $std::number::Float32 (struct
            (field "value" i32)
        ))
        (type $std::number::Float64 (struct
            (field "value" i32)
        ))
        (type $std::number::Integer32 (struct
            (field "value" i32)
        ))
        (type $std::number::Integer64 (struct
            (field "value" i32)
        ))
        (type $std::text::Unicode (struct
            (field "value" i32)
        ))
        (func $std::text::let_us_random
        )
        (func $std::text::test
        )
    )
    (core instance $main (instantiate $Main
        (with "v:legacy/number" (instance
        ))
        (with "v:math/legacy" (instance
            (export "cos-f64" (func $std::number::cos_f64))
            (export "exp-f64" (func $std::number::exp_f64))
            (export "exp-i64" (func $std::number::exp_i64))
            (export "log-i64" (func $std::number::log_i64))
            (export "sin-f64" (func $std::number::sin_f64))
        ))
        (with "wasi:sockets/network" (instance
        ))
        (with "wasi:random/insecure" (instance
            (export "get-insecure-random-u64" (func $std::random::random_seed_fast))
        ))
        (with "wasi:random/random" (instance
            (export "get-random-u64" (func $std::random::random_seed_safe))
        ))
        (with "v:legacy/text" (instance
        ))
    ))
)