[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

[main]
micro main(): i32 {
    let greeting: utf16 = "Hello from Valkyrie CLR!";
    console_write_line(greeting);
    return 0;
}
