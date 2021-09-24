//! Performance benchmarks for Valkyrie parser

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nyar_core::*;
use valkyrie_parser::*;

/// Sample Valkyrie code for benchmarking
const SIMPLE_EXPRESSION: &str = "1 + 2 * 3";

const VARIABLE_DECLARATION: &str = r#"
let x = 42;
let mut y = "hello";
let z = x + y.length();
"#;

const FUNCTION_DEFINITION: &str = r#"
fn fibonacci(n) {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
"#;

const COMPLEX_PROGRAM: &str = r#"
fn main() {
    let numbers = [1, 2, 3, 4, 5];
    let mut sum = 0;
    
    for num in numbers {
        if num % 2 == 0 {
            sum = sum + num;
        }
    }
    
    let result = calculate_result(sum);
    return result;
}

fn calculate_result(value) {
    let multiplier = 2.5;
    let base = 10;
    
    if value > base {
        return value * multiplier;
    } else {
        return value + base;
    }
}

fn factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}
"#;

/// Generate a large program for stress testing
fn generate_large_program(size: usize) -> String {
    let mut program = String::new();

    for i in 0..size {
        program.push_str(&format!(
            r#"
fn function_{i}() {{
    let var_{i} = {i};
    let result_{i} = var_{i} * 2 + 1;
    return result_{i};
}}
"#
        ));
    }

    program
}

/// Benchmark tokenization performance
fn bench_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");

    let tokenizer = ValkyrieTokenizer::new();
    let file_id = FileId::new(0);

    group.bench_function("simple_expression", |b| b.iter(|| tokenizer.tokenize(black_box(SIMPLE_EXPRESSION), file_id)));

    group.bench_function("variable_declaration", |b| b.iter(|| tokenizer.tokenize(black_box(VARIABLE_DECLARATION), file_id)));

    group.bench_function("function_definition", |b| b.iter(|| tokenizer.tokenize(black_box(FUNCTION_DEFINITION), file_id)));

    group.bench_function("complex_program", |b| b.iter(|| tokenizer.tokenize(black_box(COMPLEX_PROGRAM), file_id)));

    group.finish();
}

/// Benchmark parsing performance
fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    let parser = ValkyrieParser::new();
    let file_id = FileId::new(0);

    group.bench_function("simple_expression", |b| b.iter(|| parser.parse(black_box(SIMPLE_EXPRESSION), file_id)));

    group.bench_function("variable_declaration", |b| b.iter(|| parser.parse(black_box(VARIABLE_DECLARATION), file_id)));

    group.bench_function("function_definition", |b| b.iter(|| parser.parse(black_box(FUNCTION_DEFINITION), file_id)));

    group.bench_function("complex_program", |b| b.iter(|| parser.parse(black_box(COMPLEX_PROGRAM), file_id)));

    group.finish();
}

/// Benchmark parsing with different configurations
fn bench_parser_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_configurations");

    let file_id = FileId::new(0);

    // Standard parser
    let standard_parser = ValkyrieParser::new();
    group.bench_function("standard", |b| b.iter(|| standard_parser.parse(black_box(COMPLEX_PROGRAM), file_id)));

    // Parser without error recovery
    let no_recovery_parser = ValkyrieParser::new().with_error_recovery(false);
    group.bench_function("no_error_recovery", |b| b.iter(|| no_recovery_parser.parse(black_box(COMPLEX_PROGRAM), file_id)));

    // Parser with incremental parsing
    let incremental_parser = ValkyrieParser::new().with_incremental(true);
    group.bench_function("incremental", |b| b.iter(|| incremental_parser.parse(black_box(COMPLEX_PROGRAM), file_id)));

    group.finish();
}

/// Benchmark parsing scalability
fn bench_parsing_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_scalability");

    let parser = ValkyrieParser::new();
    let file_id = FileId::new(0);

    for size in [10, 50, 100, 200, 500].iter() {
        let program = generate_large_program(*size);

        group.bench_with_input(BenchmarkId::new("functions", size), size, |b, _| {
            b.iter(|| parser.parse(black_box(&program), file_id))
        });
    }

    group.finish();
}

/// Benchmark error recovery performance
fn bench_error_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery");

    let parser_with_recovery = ValkyrieParser::new().with_error_recovery(true);
    let parser_without_recovery = ValkyrieParser::new().with_error_recovery(false);
    let file_id = FileId::new(0);

    // Program with syntax errors
    let invalid_program = r#"
fn broken_function( {
    let x = ;
    if condition
        return x +;
    }
}
"#;

    group.bench_function("with_recovery", |b| {
        b.iter(|| {
            let _ = parser_with_recovery.parse(black_box(invalid_program), file_id);
        })
    });

    group.bench_function("without_recovery", |b| {
        b.iter(|| {
            let _ = parser_without_recovery.parse(black_box(invalid_program), file_id);
        })
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let parser = ValkyrieParser::new();
    let file_id = FileId::new(0);

    // Test repeated parsing to check for memory leaks
    group.bench_function("repeated_parsing", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = parser.parse(black_box(COMPLEX_PROGRAM), file_id);
            }
        })
    });

    // Test parsing many small programs
    group.bench_function("many_small_programs", |b| {
        b.iter(|| {
            for i in 0..100 {
                let program = format!("let x_{} = {};", i, i);
                let _ = parser.parse(black_box(&program), file_id);
            }
        })
    });

    group.finish();
}

/// Benchmark string parsing convenience function
fn bench_string_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_parsing");

    group.bench_function("parse_string_simple", |b| b.iter(|| parse_string(black_box(SIMPLE_EXPRESSION))));

    group.bench_function("parse_string_complex", |b| b.iter(|| parse_string(black_box(COMPLEX_PROGRAM))));

    group.finish();
}

criterion_group!(
    benches,
    bench_tokenization,
    bench_parsing,
    bench_parser_configurations,
    bench_parsing_scalability,
    bench_error_recovery,
    bench_memory_usage,
    bench_string_parsing
);

criterion_main!(benches);
