[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

[clr("mscorlib", "System.String", "op_Equality")]
micro string_equals(a: utf16, b: utf16): bool;

[clr("mscorlib", "System.Environment", "GetCommandLineArgs")]
micro get_command_line_args(): [utf16];

[clr("mscorlib", "System.IO.File", "Copy")]
micro file_copy(source: utf16, dest: utf16): unit;

[clr("mscorlib", "System.IO.Path", "GetFileName")]
micro path_get_filename(path: utf16): utf16;

[clr("mscorlib", "System.IO.Path", "Combine")]
micro path_combine(a: utf16, b: utf16): utf16;

micro main(args: [utf16]): i32 {
    if len(args) > 0 {
        if string_equals(args[0], "build") {
            let cmd_args = get_command_line_args();
            let self_path = cmd_args[0];
            let output_dir = args[3];
            let self_filename = path_get_filename(self_path);
            let output_path = path_combine(output_dir, self_filename);
            file_copy(self_path, output_path);
            return 0;
        }
        if string_equals(args[0], "--help") {
            console_write_line("legion - Valkyrie compiler toolchain");
            console_write_line("Usage: legion <command> [options]");
            console_write_line("Commands:");
            console_write_line("  build   Build a project");
            return 0;
        }
        if string_equals(args[0], "--version") {
            console_write_line("legion 0.1.0");
            return 0;
        }
    }
    console_write_line("legion 0.1.0");
    return 0;
}

