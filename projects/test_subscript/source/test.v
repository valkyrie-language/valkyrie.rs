[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

[clr("mscorlib", "System.String", "op_Equality")]
micro string_equals(a: utf16, b: utf16): bool;

[main]
micro main(args: [utf16]): i32 {
    if len(args) > 0 {
        let first = args[0];
        if string_equals(first, "--help") {
            console_write_line("legion - Valkyrie compiler toolchain");
            console_write_line("Usage: legion <command> [options]");
            console_write_line("Commands:");
            console_write_line("  build   Build a project");
            console_write_line("  run     Run a project");
            console_write_line("  spy     Analyze binaries");
            return 0;
        }
        if string_equals(first, "--version") {
            console_write_line("legion 0.1.0");
            return 0;
        }
    }
    console_write_line("legion 0.1.0");
    console_write_line("Use --help for usage information.");
    return 0;
}
