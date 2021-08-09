(component $root
    (core module $MockMemory
        (func $realloc (export "realloc") (param i32 i32 i32 i32) (result i32)
            (i32.const 0)
        )
        (memory $memory (export "memory") 15)
    )
    (core instance $memory (instantiate $MockMemory))
    (import "wasi:io/streams" (instance $wasi:io/streams
        (export $std::io::InputStream "input-stream" (type (sub resource)))
        (export $std::io::OutputStream "output-stream" (type (sub resource)))
    ))
    (alias export $wasi:io/streams "input-stream" (type $std::io::InputStream))
    (alias export $wasi:io/streams "output-stream" (type $std::io::OutputStream))
    (import "wasi:io/error" (instance $wasi:io/error
        (export $std::io::IoError "error" (type (sub resource)))
    ))
    (alias export $wasi:io/error "error" (type $std::io::IoError))
    (import "wasi:cli/stderr" (instance $wasi:cli/stderr
        (export "get-stderr" (func
        ))
    ))
    (alias export $wasi:cli/stderr "get-stderr" (func $std::io::standard_error))
    (import "wasi:cli/stdin" (instance $wasi:cli/stdin
        (export "get-stdin" (func
        ))
    ))
    (alias export $wasi:cli/stdin "get-stdin" (func $std::io::standard_input))
    (import "wasi:cli/stdout" (instance $wasi:cli/stdout
        (export "get-stdout" (func
        ))
    ))
    (alias export $wasi:cli/stdout "get-stdout" (func $std::io::standard_output))
    (import "wasi:random/insecure" (instance $wasi:random/insecure
        (export "get-insecure-random-bytes" (func
        ))
    ))
    (alias export $wasi:random/insecure "get-insecure-random-bytes" (func $std::random::fast_random_seed))
    (import "wasi:random/random" (instance $wasi:random/random
        (export "get-random-u64" (func
        ))
    ))
    (alias export $wasi:random/random "get-random-u64" (func $std::random::safe_random_seed))
    (import "wasi:clock/monotonic-clock" (instance $wasi:clock/monotonic-clock
        (export "now" (func
        ))
    ))
    (alias export $wasi:clock/monotonic-clock "now" (func $std::time::now))
    (import "unstable:debugger/print" (instance $unstable:debugger/print
        (export "print-bool" (func
        ))
        (export "print-char" (func
        ))
        (export "print-i64" (func
        ))
        (export "print-i64" (func
        ))
        (export "print-i64" (func
        ))
        (export "print-i64" (func
        ))
        (export "print-u16" (func
        ))
        (export "print-u32" (func
        ))
        (export "print-i64" (func
        ))
        (export "print-u8" (func
        ))
    ))
    (alias export $unstable:debugger/print "print-bool" (func $std::time::print_bool))
    (alias export $unstable:debugger/print "print-char" (func $std::time::print_char))
    (alias export $unstable:debugger/print "print-i64" (func $std::time::print_i16))
    (alias export $unstable:debugger/print "print-i64" (func $std::time::print_i32))
    (alias export $unstable:debugger/print "print-i64" (func $std::time::print_i64))
    (alias export $unstable:debugger/print "print-i64" (func $std::time::print_i8))
    (alias export $unstable:debugger/print "print-u16" (func $std::time::print_u16))
    (alias export $unstable:debugger/print "print-u32" (func $std::time::print_u32))
    (alias export $unstable:debugger/print "print-i64" (func $std::time::print_u64))
    (alias export $unstable:debugger/print "print-u8" (func $std::time::print_u8))
    (core func $std::io::standard_error (canon lower
        (func $wasi:cli/stderr "get-stderr")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::io::standard_input (canon lower
        (func $wasi:cli/stdin "get-stdin")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::io::standard_output (canon lower
        (func $wasi:cli/stdout "get-stdout")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::random::fast_random_seed (canon lower
        (func $wasi:random/insecure "get-insecure-random-bytes")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::random::safe_random_seed (canon lower
        (func $wasi:random/random "get-random-u64")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::now (canon lower
        (func $wasi:clock/monotonic-clock "now")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_bool (canon lower
        (func $unstable:debugger/print "print-bool")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_char (canon lower
        (func $unstable:debugger/print "print-char")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_i16 (canon lower
        (func $unstable:debugger/print "print-i64")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_i32 (canon lower
        (func $unstable:debugger/print "print-i64")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_i64 (canon lower
        (func $unstable:debugger/print "print-i64")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_i8 (canon lower
        (func $unstable:debugger/print "print-i64")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_u16 (canon lower
        (func $unstable:debugger/print "print-u16")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_u32 (canon lower
        (func $unstable:debugger/print "print-u32")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_u64 (canon lower
        (func $unstable:debugger/print "print-i64")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core func $std::time::print_u8 (canon lower
        (func $unstable:debugger/print "print-u8")
        (memory $memory "memory")(realloc (func $memory "realloc"))
        string-encoding=utf8
    ))
    (core module $Main
        
        
        
        
        (import "wasi:cli/stderr" "get-stderr" (func $std::io::standard_error))
        
        (import "wasi:cli/stdin" "get-stdin" (func $std::io::standard_input))
        
        (import "wasi:cli/stdout" "get-stdout" (func $std::io::standard_output))
        
        (import "wasi:random/insecure" "get-insecure-random-bytes" (func $std::random::fast_random_seed))
        
        (import "wasi:random/random" "get-random-u64" (func $std::random::safe_random_seed))
        
        (import "wasi:clock/monotonic-clock" "now" (func $std::time::now))
        
        (import "unstable:debugger/print" "print-bool" (func $std::time::print_bool))
        (import "unstable:debugger/print" "print-char" (func $std::time::print_char))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i16))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i32))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i64))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i8))
        (import "unstable:debugger/print" "print-u16" (func $std::time::print_u16))
        (import "unstable:debugger/print" "print-u32" (func $std::time::print_u32))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_u64))
        (import "unstable:debugger/print" "print-u8" (func $std::time::print_u8))
    )
    (core instance $main (instantiate $Main
        (with "wasi:io/streams" (instance
        ))
        (with "wasi:io/error" (instance
        ))
        (with "wasi:cli/stderr" (instance
            (export "get-stderr" (func $std::io::standard_error))
        ))
        (with "wasi:cli/stdin" (instance
            (export "get-stdin" (func $std::io::standard_input))
        ))
        (with "wasi:cli/stdout" (instance
            (export "get-stdout" (func $std::io::standard_output))
        ))
        (with "wasi:random/insecure" (instance
            (export "get-insecure-random-bytes" (func $std::random::fast_random_seed))
        ))
        (with "wasi:random/random" (instance
            (export "get-random-u64" (func $std::random::safe_random_seed))
        ))
        (with "wasi:clock/monotonic-clock" (instance
            (export "now" (func $std::time::now))
        ))
        (with "unstable:debugger/print" (instance
            (export "print-bool" (func $std::time::print_bool))
            (export "print-char" (func $std::time::print_char))
            (export "print-i64" (func $std::time::print_i16))
            (export "print-i64" (func $std::time::print_i32))
            (export "print-i64" (func $std::time::print_i64))
            (export "print-i64" (func $std::time::print_i8))
            (export "print-u16" (func $std::time::print_u16))
            (export "print-u32" (func $std::time::print_u32))
            (export "print-i64" (func $std::time::print_u64))
            (export "print-u8" (func $std::time::print_u8))
        ))
    ))
)