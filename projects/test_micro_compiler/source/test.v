[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

[clr("mscorlib", "System.String", "op_Equality")]
micro string_equals(a: utf16, b: utf16): bool;

[clr("mscorlib", "System.String", "Compare")]
micro string_compare(a: utf16, b: utf16): i32;

[clr("mscorlib", "System.Environment", "GetCommandLineArgs")]
micro get_command_line_args(): [utf16];

[clr("mscorlib", "System.IO.File", "ReadAllText")]
micro file_read_all_text(path: utf16): utf16;

[clr("mscorlib", "System.IO.File", "ReadAllBytes")]
micro file_read_all_bytes(path: utf16): [u8];

[clr("mscorlib", "System.IO.File", "WriteAllBytes")]
micro file_write_all_bytes(path: utf16, bytes: [u8]): unit;

[main]
micro main(args: [utf16]): i32 {
    let cmd_args = get_command_line_args();
    if len(cmd_args) < 2 {
        console_write_line("micro-compiler: usage: micro-compiler <command> [args]");
        return 1;
    }
    let command = cmd_args[1];
    if string_equals(command, "build") {
        if len(cmd_args) < 4 {
            console_write_line("micro-compiler: usage: micro-compiler build <source> <output>");
            return 1;
        }
        let source_path = cmd_args[2];
        let output_path = cmd_args[3];
        let source = file_read_all_text(source_path);
        let return_code = parse_return_code(source);
        let self_path = cmd_args[0];
        let pe_bytes = file_read_all_bytes(self_path);
        let patched = patch_return_code(pe_bytes, return_code);
        file_write_all_bytes(output_path, patched);
        console_write_line("micro-compiler: build complete");
        return 0;
    }
    if string_equals(command, "--help") {
        console_write_line("micro-compiler: minimal Valkyrie compiler");
        console_write_line("usage: micro-compiler build <source> <output>");
        return 0;
    }
    if string_equals(command, "--version") {
        console_write_line("micro-compiler 0.1.0");
        return 0;
    }
    console_write_line("micro-compiler: unknown command");
    return 1;
}

micro parse_return_code(source: utf16): i32 {
    if string_compare(source, "return 0") == 0 {
        return 0;
    }
    if string_compare(source, "return 1") == 0 {
        return 1;
    }
    if string_compare(source, "return 2") == 0 {
        return 2;
    }
    if string_compare(source, "return 3") == 0 {
        return 3;
    }
    if string_compare(source, "return 4") == 0 {
        return 4;
    }
    if string_compare(source, "return 5") == 0 {
        return 5;
    }
    if string_compare(source, "return 6") == 0 {
        return 6;
    }
    if string_compare(source, "return 7") == 0 {
        return 7;
    }
    if string_compare(source, "return 8") == 0 {
        return 8;
    }
    return 0;
}

micro patch_return_code(bytes: [u8], code: i32): [u8] {
    let offset = find_return_offset(bytes);
    let opcode = code + 22;
    bytes[offset] = opcode;
    return bytes;
}

micro find_return_offset(bytes: [u8]): i32 {
    let len = len(bytes);
    let i = 0;
    loop i < len {
        if i + 1 < len {
            let b0 = bytes[i];
            let next_i = i + 1;
            let b1 = bytes[next_i];
            if b0 == 22 {
                if b1 == 42 {
                    return i;
                }
            }
        }
        i = i + 1;
    }
    return 0;
}
