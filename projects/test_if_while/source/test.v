[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16);

[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line_int(value: i32);

[main]
micro main(): i32 {
    console_write_line("Starting computation...");
    let result = compute(10);
    console_write_line_int(result);
    return 0;
}

micro compute(n: i32) -> i32 {
    let sum = 0;
    loop i in array(1, 2, 3, 4, 5, 6, 7, 8, 9, 10) {
        if i > n {
            return sum;
        }
        sum = sum + i;
    }
    return sum;
}
