[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line_i32(value: i32): unit;

[clr("mscorlib", "System.String", "Compare")]
micro string_compare(a: utf16, b: utf16): i32;

[main]
micro main(args: [utf16]): i32 {
    let result = string_compare("ABC", "ABC");
    console_write_line_i32(result);
    return 0;
}
