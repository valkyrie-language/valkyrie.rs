(component $root(core module $MockMemory
        (func $realloc (export "realloc") (param i32 i32 i32 i32) (result i32)
            (i32.const 0)
        )
        (memory $memory (export "memory") 15)
    )
    (core instance $memory (instantiate $MockMemory))
    (import "v:math/legacy" (instance $v:math/legacy
        (export "exp-f64" (func
            (param "value" float64)
        ))
        (export "sin-f64" (func
            (param "value" float64)
        ))
        (export "cos-f64" (func
            (param "value" float64)
        ))
        (export "cos-pi-f64" (func
            (param "value" float64)
        ))
        (export "acos-f64" (func
            (param "value" float64)
        ))
        (export "acos-f64" (func
            (param "value" float64)
        ))
        (export "abs-f64" (func
            (param "value" float64)
        ))
        (export "ceil-f64" (func
            (param "value" float64)
        ))
        (export "floor-f64" (func
            (param "value" float64)
        ))
        (export "sqrt-f64" (func
            (param "value" float64)
        ))
        (export "cbrt-f64" (func
            (param "value" float64)
        ))
    ))
    (alias export $v:math/legacy "exp-f64" (func $std::number::exp_f64))
    (alias export $v:math/legacy "sin-f64" (func $std::number::sin_f64))
    (alias export $v:math/legacy "cos-f64" (func $std::number::cos_f64))
    (alias export $v:math/legacy "cos-pi-f64" (func $std::number::cos_pi_f64))
    (alias export $v:math/legacy "acos-f64" (func $std::number::arc_cos_f64))
    (alias export $v:math/legacy "acos-f64" (func $std::number::arc_cosh_f64))
    (alias export $v:math/legacy "abs-f64" (func $std::number::abs_f64))
    (alias export $v:math/legacy "ceil-f64" (func $std::number::ceil_f64))
    (alias export $v:math/legacy "floor-f64" (func $std::number::floor_f64))
    (alias export $v:math/legacy "sqrt-f64" (func $std::number::sqrt_f64))
    (alias export $v:math/legacy "cbrt-f64" (func $std::number::cbrt_f64))
    (import "v:legacy/number" (instance $v:legacy/number
        (export $std::number::Integer "big-integer" (type (sub resource)))
    ))
    (alias export $v:legacy/number "big-integer" (type $std::number::Integer))
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
    (import "wasi:sockets/network" (instance $wasi:sockets/network
        (export $std::random::Network "network" (type (sub resource)))
    ))
    (alias export $wasi:sockets/network "network" (type $std::random::Network))
    (import "v:legacy/text" (instance $v:legacy/text
        (export $std::text::Utf16Text "utf16" (type (sub resource)))
    ))
    (alias export $v:legacy/text "utf16" (type $std::text::Utf16Text))
    (import "webidl:dom/dom" (instance $webidl:dom/dom
        (export $dom::Console "console" (type (sub resource)))
        (export $dom::DomElement "element" (type (sub resource)))
        (export $dom::HtmlElement "html-element" (type (sub resource)))
    ))
    (alias export $webidl:dom/dom "console" (type $dom::Console))
    (alias export $webidl:dom/dom "element" (type $dom::DomElement))
    (alias export $webidl:dom/dom "html-element" (type $dom::HtmlElement))
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
    
    
    (core func $std::number::exp_f64 (canon lower
        (func $v:math/legacy "exp-f64")
    ))
    (core func $std::number::sin_f64 (canon lower
        (func $v:math/legacy "sin-f64")
    ))
    (core func $std::number::cos_f64 (canon lower
        (func $v:math/legacy "cos-f64")
    ))
    (core func $std::number::cos_pi_f64 (canon lower
        (func $v:math/legacy "cos-pi-f64")
    ))
    (core func $std::number::arc_cos_f64 (canon lower
        (func $v:math/legacy "acos-f64")
    ))
    (core func $std::number::arc_cosh_f64 (canon lower
        (func $v:math/legacy "acos-f64")
    ))
    (core func $std::number::abs_f64 (canon lower
        (func $v:math/legacy "abs-f64")
    ))
    (core func $std::number::ceil_f64 (canon lower
        (func $v:math/legacy "ceil-f64")
    ))
    (core func $std::number::floor_f64 (canon lower
        (func $v:math/legacy "floor-f64")
    ))
    (core func $std::number::sqrt_f64 (canon lower
        (func $v:math/legacy "sqrt-f64")
    ))
    (core func $std::number::cbrt_f64 (canon lower
        (func $v:math/legacy "cbrt-f64")
    ))
    (core func $std::random::random_seed_fast (canon lower
        (func $wasi:random/insecure "get-insecure-random-u64")
    ))
    (core func $std::random::random_seed_safe (canon lower
        (func $wasi:random/random "get-random-u64")
    ))
    (core module $Main
        (import "v:math/legacy" "exp-f64" (func $std::number::exp_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "sin-f64" (func $std::number::sin_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "cos-f64" (func $std::number::cos_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "cos-pi-f64" (func $std::number::cos_pi_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "acos-f64" (func $std::number::arc_cos_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "acos-f64" (func $std::number::arc_cosh_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "abs-f64" (func $std::number::abs_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "ceil-f64" (func $std::number::ceil_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "floor-f64" (func $std::number::floor_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "sqrt-f64" (func $std::number::sqrt_f64
            (param $value f64)
        ))
        (import "v:math/legacy" "cbrt-f64" (func $std::number::cbrt_f64
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
        (with "v:math/legacy" (instance
            (export "exp-f64" (func $std::number::exp_f64))
            (export "sin-f64" (func $std::number::sin_f64))
            (export "cos-f64" (func $std::number::cos_f64))
            (export "cos-pi-f64" (func $std::number::cos_pi_f64))
            (export "acos-f64" (func $std::number::arc_cos_f64))
            (export "acos-f64" (func $std::number::arc_cosh_f64))
            (export "abs-f64" (func $std::number::abs_f64))
            (export "ceil-f64" (func $std::number::ceil_f64))
            (export "floor-f64" (func $std::number::floor_f64))
            (export "sqrt-f64" (func $std::number::sqrt_f64))
            (export "cbrt-f64" (func $std::number::cbrt_f64))
        ))
        (with "v:legacy/number" (instance
        ))
        (with "wasi:random/insecure" (instance
            (export "get-insecure-random-u64" (func $std::random::random_seed_fast))
        ))
        (with "wasi:random/random" (instance
            (export "get-random-u64" (func $std::random::random_seed_safe))
        ))
        (with "wasi:sockets/network" (instance
        ))
        (with "v:legacy/text" (instance
        ))
        (with "webidl:dom/dom" (instance
        ))
    ))
)