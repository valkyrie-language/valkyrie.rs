(component $root(core module $MockMemory
        (func $realloc (export "realloc") (param i32 i32 i32 i32) (result i32)
            (i32.const 0)
        )
        (memory $memory (export "memory") 15)
    )
    (core instance $memory (instantiate $MockMemory))
    (import "wasi:cli/terminal-input" (instance $wasi:cli/terminal-input
        (export $std::cli::TerminalInput "terminal-input" (type (sub resource)))
    ))
    (alias export $wasi:cli/terminal-input "terminal-input" (type $std::cli::TerminalInput))
    (import "wasi:cli/terminal-output" (instance $wasi:cli/terminal-output
        (export $std::cli::TerminalOutput "terminal-output" (type (sub resource)))
    ))
    (alias export $wasi:cli/terminal-output "terminal-output" (type $std::cli::TerminalOutput))
    (import "wasi:filesystem/types" (instance $wasi:filesystem/types
        (export $std::fs::Descriptor "descriptor" (type (sub resource)))
    ))
    (alias export $wasi:filesystem/types "descriptor" (type $std::fs::Descriptor))
    (import "wasi:io/error" (instance $wasi:io/error
        (export $std::io::IoError "error" (type (sub resource)))
    ))
    (alias export $wasi:io/error "error" (type $std::io::IoError))
    (import "wasi:random/insecure" (instance $wasi:random/insecure
        (export "get-insecure-random-bytes" (func
        ))
    ))
    (alias export $wasi:random/insecure "get-insecure-random-bytes" (func $std::rand::fast_random_seed))
    (import "wasi:random/random" (instance $wasi:random/random
        (export "get-random-u64" (func
        ))
    ))
    (alias export $wasi:random/random "get-random-u64" (func $std::rand::safe_random_seed))
    (import "unstable:debugger/print" (instance $unstable:debugger/print
        (export "print-bool" (func
            (param "value" bool)
        ))
        (export "print-char" (func
            (param "value" char)
        ))
        (export "print-i64" (func
            (param "value" s16)
        ))
        (export "print-i64" (func
            (param "value" s32)
        ))
        (export "print-i64" (func
            (param "value" s64)
        ))
        (export "print-i64" (func
            (param "value" s8)
        ))
        (export "print-u16" (func
            (param "value" u16)
        ))
        (export "print-u32" (func
            (param "value" u32)
        ))
        (export "print-i64" (func
            (param "value" u64)
        ))
        (export "print-u8" (func
            (param "value" u8)
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
    (type $std::fs::DescriptorFlags (flags
        "read" ;; 0
        "write" ;; 1
        "file-integrity-sync" ;; 2
        "data-integrity-sync" ;; 3
        "requested-write-sync" ;; 4
        "mutate-directory" ;; 5
    ))
    (type $std::fs::DescriptorType (enum
        "unknown" ;; 0
        "block-device" ;; 1
        "character-device" ;; 2
        "directory" ;; 3
        "fifo" ;; 4
        "symbolic-link" ;; 5
        "regular-file" ;; 6
        "socket" ;; 7
    ))
    (type $std::fs::OpenFlags (flags
        "create" ;; 0
        "directory" ;; 1
        "exclusive" ;; 2
        "truncate" ;; 3
    ))
    (type $std::fs::PathFlags (flags
        "symlink-follow" ;; 0
    ))
    (type $std::io::Endian (enum
        "big" ;; 0
        "little" ;; 1
    ))
    ;; variant std∷io∷StreamError
    (type $std::io::StreamError (variant
        ;; LastOperationFailed
        (case "last-operation-failed")
        ;; Closed
        (case "closed")
    ))
    (type $std::meth::Comparison (enum
        "incomparable" ;; 0
        "lesser" ;; 1
        "equal" ;; 2
        "greater" ;; 3
    ))
    
    
    (core func $std::rand::fast_random_seed (canon lower
        (func $wasi:random/insecure "get-insecure-random-bytes")
    ))
    (core func $std::rand::safe_random_seed (canon lower
        (func $wasi:random/random "get-random-u64")
    ))
    (core func $std::time::print_bool (canon lower
        (func $unstable:debugger/print "print-bool")
    ))
    (core func $std::time::print_char (canon lower
        (func $unstable:debugger/print "print-char")
    ))
    (core func $std::time::print_i16 (canon lower
        (func $unstable:debugger/print "print-i64")
    ))
    (core func $std::time::print_i32 (canon lower
        (func $unstable:debugger/print "print-i64")
    ))
    (core func $std::time::print_i64 (canon lower
        (func $unstable:debugger/print "print-i64")
    ))
    (core func $std::time::print_i8 (canon lower
        (func $unstable:debugger/print "print-i64")
    ))
    (core func $std::time::print_u16 (canon lower
        (func $unstable:debugger/print "print-u16")
    ))
    (core func $std::time::print_u32 (canon lower
        (func $unstable:debugger/print "print-u32")
    ))
    (core func $std::time::print_u64 (canon lower
        (func $unstable:debugger/print "print-i64")
    ))
    (core func $std::time::print_u8 (canon lower
        (func $unstable:debugger/print "print-u8")
    ))
    (core module $Main
        (import "wasi:random/insecure" "get-insecure-random-bytes" (func $std::rand::fast_random_seed
        ))
        (import "wasi:random/random" "get-random-u64" (func $std::rand::safe_random_seed
        ))
        (import "unstable:debugger/print" "print-bool" (func $std::time::print_bool
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-char" (func $std::time::print_char
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i16
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i32
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i64
            (param $value i64)
        ))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_i8
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-u16" (func $std::time::print_u16
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-u32" (func $std::time::print_u32
            (param $value i32)
        ))
        (import "unstable:debugger/print" "print-i64" (func $std::time::print_u64
            (param $value i64)
        ))
        (import "unstable:debugger/print" "print-u8" (func $std::time::print_u8
            (param $value i32)
        ))
        (func $std::time::let_us_random
        )
        (func $std::time::test
        )
    )
    (core instance $main (instantiate $Main
        (with "wasi:cli/terminal-input" (instance
        ))
        (with "wasi:cli/terminal-output" (instance
        ))
        (with "wasi:filesystem/types" (instance
        ))
        (with "wasi:io/error" (instance
        ))
        (with "wasi:random/insecure" (instance
            (export "get-insecure-random-bytes" (func $std::rand::fast_random_seed))
        ))
        (with "wasi:random/random" (instance
            (export "get-random-u64" (func $std::rand::safe_random_seed))
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